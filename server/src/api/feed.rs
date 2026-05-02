use axum::Json;
use axum::extract::{Extension, Query};
use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::FeedItem;
use crate::domain::ports::{DbPort, MONTHLY_CALL_LIMIT};
use crate::middleware::auth::AuthUser;

use super::AppState;

/// 피드 TTL: 완전 성공 5분
const FEED_CACHE_TTL: Duration = Duration::from_secs(300);
/// 피드 TTL: 부분 실패(일부 태그 검색 오류) 1분 (이 값이 항상 우선)
const FEED_CACHE_TTL_PARTIAL: Duration = Duration::from_secs(60);
/// MVP15 M2 S3: 80% 도달 시 TTL 30분 (캐시 강화로 추가 호출 억제)
const FEED_CACHE_TTL_QUOTA_WARN: Duration = Duration::from_secs(1800);
/// 사용률 80% 임계.
const QUOTA_WARN_PCT: i32 = 80;
/// 사용률 100% 임계 (안내 응답 트리거).
const QUOTA_BLOCK_PCT: i32 = 100;
// 분모(무료 한도)는 domain/ports.rs::MONTHLY_CALL_LIMIT SSOT 사용.
/// MVP15 M2 S2 통합: 카운터 PK로 사용하는 엔진 식별자 목록.
/// `FeedItem.source` 와 동일해야 한다 (advisor 지적: PK 일관성).
const ENGINE_IDS: &[&str] = &["tavily", "exa", "firecrawl"];

/// 캐시 키 생성: 특정 태그 → `"{user_id}:{tag_id}"`, 전체 피드 → `"{user_id}:{sorted_tag_ids}"`
fn make_cache_key(user_id: Uuid, tag_ids: &[Uuid]) -> String {
    if tag_ids.is_empty() {
        return format!("{user_id}:all");
    }
    let mut sorted: Vec<String> = tag_ids.iter().map(|id| id.to_string()).collect();
    sorted.sort();
    format!("{user_id}:{}", sorted.join(","))
}

/// `Cache-Control: no-cache` 헤더가 있으면 true 반환.
fn is_no_cache(headers: &HeaderMap) -> bool {
    headers
        .get("cache-control")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("no-cache"))
        .unwrap_or(false)
}

/// 피드 한 번 요청에 가져올 최대 기사 수
const MAX_FEED_LIMIT: u32 = 50;

/// GET /me/feed 쿼리 파라미터
#[derive(Debug, Deserialize, Default)]
pub struct FeedQuery {
    /// 특정 태그만 필터링. 없으면 전체 태그 검색 (하위 호환)
    pub tag_id: Option<Uuid>,
    /// 한 번에 가져올 최대 기사 수. 없으면 전체 반환 (하위 호환). 최대 50.
    pub limit: Option<u32>,
    /// 건너뛸 기사 수 (0-based). 기본값 0.
    pub offset: Option<u32>,
}

/// 전체 피드 items에서 limit/offset 슬라이싱을 적용한다.
/// limit이 없으면 전체 반환 (하위 호환).
fn apply_pagination(
    items: Vec<FeedItem>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Vec<FeedItem> {
    let offset = offset.unwrap_or(0) as usize;
    match limit {
        None => items,
        Some(lim) => items.into_iter().skip(offset).take(lim as usize).collect(),
    }
}

/// 클라이언트 노출용 FeedItem DTO.
/// ephemeral — DB에 저장되지 않음. id 없음.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedItemResponse {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub source: String,
    pub published_at: Option<DateTime<Utc>>,
    pub tag_id: Option<Uuid>,
    /// MVP6 M1: 썸네일 이미지 URL (없으면 null)
    pub image_url: Option<String>,
}

impl From<FeedItem> for FeedItemResponse {
    fn from(item: FeedItem) -> Self {
        Self {
            title: item.title,
            url: item.url,
            snippet: item.snippet,
            source: item.source,
            published_at: item.published_at,
            tag_id: item.tag_id,
            image_url: item.image_url,
        }
    }
}

/// MVP15 M2 S5: 셋 다 한도 시 사용자 안내 페이로드.
/// `notice = None`이면 정상, `Some`이면 빈 결과 + 안내.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedNotice {
    /// 한국어 안내 메시지
    pub message: String,
    /// 회복 시각 (가장 빠른 reset_at). NULL 엔진(Exa)은 제외하고 min.
    /// 셋 다 NULL이면 None.
    pub recovery_at: Option<DateTime<Utc>>,
}

/// MVP15 M2 S5: 항상-envelope 응답.
/// `notice`는 평소 None, 한도 도달로 빈 결과일 때만 Some.
/// (M3에서 클라이언트가 함께 전환)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedResponse {
    pub items: Vec<FeedItemResponse>,
    pub notice: Option<FeedNotice>,
}

/// 셋 다 한도 안내 메시지 빌드. 순수 함수.
pub fn build_quota_notice_message() -> String {
    "오늘은 모든 검색 엔진의 무료 한도가 소진되어 새 기사를 가져올 수 없습니다. 캐시된 기사만 보일 수 있습니다.".to_string()
}

/// 회복 시각 선택: NULL 엔진(Exa) 제외하고 가장 빠른 reset_at.
/// 셋 다 NULL이면 None.
/// 순수 함수 — 표 기반 테스트 용이.
pub fn select_recovery_at(snapshots: &[(String, Option<DateTime<Utc>>)]) -> Option<DateTime<Utc>> {
    snapshots.iter().filter_map(|(_, r)| *r).min()
}

/// 사용률 계산. (calls / MONTHLY_CALL_LIMIT) * 100.
/// 100 초과는 100으로 clamp (UI 안정성).
pub fn usage_pct(calls: i32) -> i32 {
    let pct = ((calls as f64 / MONTHLY_CALL_LIMIT as f64) * 100.0).floor() as i32;
    pct.clamp(0, 100)
}

/// MVP15 M2 S3: 캐시 TTL 결정 함수. 순수 함수 — 표 기반 테스트 용이.
///
/// 우선순위 (advisor 지적):
/// 1. 부분 실패 → 1분 (불완전 결과 조기 만료, 항상 우선)
/// 2. 사용률 ≥ 80% → 30분 (캐시 강화로 추가 호출 억제)
/// 3. 그 외 → 5분 (기본)
pub fn decide_cache_ttl(search_failed_count: usize, max_usage_pct: i32) -> Duration {
    if search_failed_count > 0 {
        return FEED_CACHE_TTL_PARTIAL;
    }
    if max_usage_pct >= QUOTA_WARN_PCT {
        return FEED_CACHE_TTL_QUOTA_WARN;
    }
    FEED_CACHE_TTL
}

/// 모든 엔진 사용률이 100%인지 — 셋 다 차단 판정.
pub fn all_engines_blocked(snapshots: &[(String, i32, Option<DateTime<Utc>>)]) -> bool {
    !snapshots.is_empty()
        && snapshots
            .iter()
            .all(|(_, calls, _)| usage_pct(*calls) >= QUOTA_BLOCK_PCT)
}

/// GET /me/feed
/// 사용자 태그를 기반으로 검색 API를 직접 호출해 피드를 반환한다.
/// DB에 저장하지 않음 — 캐시 HIT 시 즉시 반환, MISS 시 검색 API 호출.
/// `Cache-Control: no-cache` 헤더 수신 시 캐시 우회 → 항상 검색 API 호출.
/// 모든 태그 검색을 `futures::future::join_all`로 병렬 실행한다.
/// `tag_id` 쿼리 파라미터가 있으면 해당 태그만 검색 (하위 호환).
///
/// MVP15 M2 S5: 응답은 항상 `FeedResponse` envelope.
/// 평소 `notice = None`, 셋 다 한도 시 `notice = Some({ message, recovery_at })`.
pub async fn get_feed<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Query(feed_query): Query<FeedQuery>,
    headers: HeaderMap,
) -> Result<Json<FeedResponse>, AppError> {
    // limit 상한 검증
    if feed_query.limit.is_some_and(|l| l > MAX_FEED_LIMIT) {
        return Err(AppError::BadRequest(format!(
            "limit must be ≤ {MAX_FEED_LIMIT}"
        )));
    }

    let user_tags = state.db.get_user_tags(user.id).await?;
    if user_tags.is_empty() {
        // 태그 없으면 빈 피드 반환 (에러 아님)
        return Ok(Json(FeedResponse {
            items: vec![],
            notice: None,
        }));
    }

    let all_tags = state.db.list_tags().await?;

    // tag_id → name 맵: 잡 생성 + orphan 검증 양쪽에서 재사용
    let tag_name_map: std::collections::HashMap<uuid::Uuid, &str> =
        all_tags.iter().map(|t| (t.id, t.name.as_str())).collect();

    // tag_id 파라미터가 있으면 해당 태그만 필터 (사용자 구독 태그 중)
    let filtered_tags: Vec<_> = if let Some(filter_tag_id) = feed_query.tag_id {
        user_tags
            .iter()
            .filter(|ut| ut.tag_id == filter_tag_id)
            .collect()
    } else {
        user_tags.iter().collect()
    };

    if filtered_tags.is_empty() {
        // 해당 tag_id가 사용자 구독 태그가 아니면 빈 결과 (403 아님)
        return Ok(Json(FeedResponse {
            items: vec![],
            notice: None,
        }));
    }

    // ── 피드 캐시 조회 ──────────────────────────────────────────────────────
    let no_cache = is_no_cache(&headers);
    let cache_tag_ids: Vec<Uuid> = filtered_tags.iter().map(|ut| ut.tag_id).collect();
    let cache_key = make_cache_key(user.id, &cache_tag_ids);

    if !no_cache {
        if let Some(cached_items) = state.feed_cache.get(&cache_key) {
            tracing::info!(cache_key = %cache_key, "feed cache HIT — skipping search API");
            let paginated = apply_pagination(cached_items, feed_query.limit, feed_query.offset);
            return Ok(Json(FeedResponse {
                items: paginated.into_iter().map(FeedItemResponse::from).collect(),
                notice: None,
            }));
        }
        tracing::info!(cache_key = %cache_key, "feed cache MISS — calling search API");
    } else {
        tracing::info!(cache_key = %cache_key, "feed cache BYPASS (no-cache header)");
    }
    // ────────────────────────────────────────────────────────────────────────

    // MVP7 M3 ST-1: 좋아요 3회 이상일 때만 키워드 개인화 적용.
    // MVP9 M2 수정: 태그별 키워드 분리 — 각 태그 검색 쿼리에 해당 태그 키워드만 붙임.
    // 전체 피드(tag_id 없음)에서도 cross-tag 오염 방지: AI 키워드가 모바일 기사에 붙지 않도록.
    // like_count 조회 실패 시 0으로 폴백 (개인화 비활성화)
    let use_personalization = if feed_query.tag_id.is_none() {
        let like_count = state.db.get_like_count(user.id).await.unwrap_or(0);
        like_count >= 3
    } else {
        false
    };

    // 개인화 활성화 시 태그별 키워드 미리 조회 (tag_id → keyword_suffix 맵)
    let mut tag_keyword_map: std::collections::HashMap<uuid::Uuid, String> =
        std::collections::HashMap::new();
    if use_personalization {
        for user_tag in &filtered_tags {
            let keywords = state
                .db
                .get_top_keywords(user.id, vec![user_tag.tag_id], 3)
                .await
                .unwrap_or_default();
            if !keywords.is_empty() {
                tag_keyword_map.insert(user_tag.tag_id, format!(" {}", keywords.join(" ")));
            }
        }
    }

    // (tag_id, tag_name, search_query) tuple로 병렬 검색 잡 표현 — owned String으로 future 캡처
    let jobs: Vec<(uuid::Uuid, String, String)> = filtered_tags
        .iter()
        .map(|user_tag| {
            let tag_name = tag_name_map
                .get(&user_tag.tag_id)
                .copied()
                .unwrap_or("unknown")
                .to_string();
            let suffix = tag_keyword_map
                .get(&user_tag.tag_id)
                .cloned()
                .unwrap_or_default();
            let search_query = format!("{tag_name} latest news{suffix}");
            (user_tag.tag_id, tag_name, search_query)
        })
        .collect();

    // 실제 검색 쿼리 로깅 (태그별 키워드 오염 여부 확인용)
    for (_, _, q) in &jobs {
        tracing::info!(search_query = %q, "feed search query");
    }

    // MVP15 M2 S1: 5 → 20 (limit 확대). 실제 결과 수는 어댑터별 cap에 의해 제한됨.
    // (Tavily=20, Exa=10, Firecrawl=5 — main.rs:build_search_sources)
    let chain = std::sync::Arc::clone(&state.search_chain);
    let futures = jobs.into_iter().map(|(tag_id, tag_name, search_query)| {
        let chain = std::sync::Arc::clone(&chain);
        async move {
            let result = chain.search(&search_query, 20).await;
            (tag_id, tag_name, result)
        }
    });
    let results = futures::future::join_all(futures).await;

    let mut items: Vec<FeedItem> = vec![];
    let mut search_failed_count: usize = 0;

    for (tag_id, tag_name, search_result) in results {
        let (search_items, source) = match search_result {
            Ok(pair) => pair,
            Err(e) => {
                tracing::warn!(tag = %tag_name, error = %e, "search failed for tag, skipping");
                search_failed_count += 1;
                continue;
            }
        };

        // tag가 실제로 존재하는지 검증 (orphaned tag_id 방어)
        let resolved_tag_id = if tag_name_map.contains_key(&tag_id) {
            Some(tag_id)
        } else {
            tracing::warn!(
                tag_id = %tag_id,
                "orphaned user_tag: tag not found in all_tags, skipping tag_id"
            );
            None
        };

        for sr in search_items {
            // listing URL 먼저 체크 — /tag/ 같은 단일 세그먼트 listing이
            // is_homepage_url(segments ≤ 1)에 흡수되어 로그가 틀리게 남는 것을 방지
            if is_listing_url(&sr.url) {
                tracing::debug!(url = %sr.url, "skipping listing URL");
                continue;
            }
            if is_homepage_url(&sr.url) {
                tracing::debug!(url = %sr.url, "skipping homepage URL");
                continue;
            }
            items.push(FeedItem {
                title: sr.title,
                url: sr.url,
                snippet: sr.snippet,
                source: source.clone(),
                published_at: sr
                    .published_at
                    .and_then(|s| s.parse::<DateTime<Utc>>().ok()),
                tag_id: resolved_tag_id,
                image_url: sr.image_url,
            });
        }
    }

    // URL 정규화 기반 중복 제거
    // trailing slash / www. / 스킴 차이 등으로 같은 페이지가 다른 URL로 올 수 있음
    let mut seen_urls = std::collections::HashSet::new();
    items.retain(|item| seen_urls.insert(normalize_url(&item.url)));

    // ── MVP15 M2 S3: 카운터 사용률에 따른 캐시 TTL 결정 ─────────────────────
    // 우선순위 (advisor 지적):
    //   1. 부분 실패 → 1분 (항상 우선)
    //   2. 사용률 ≥ 80% → 30분 (캐시 강화로 추가 호출 억제)
    //   3. 그 외 → 5분 (기본)
    // S5 트리거: 셋 다 100% → notice 응답.
    let mut engine_snapshots: Vec<(String, i32, Option<DateTime<Utc>>)> = vec![];
    for engine in ENGINE_IDS {
        let snap = state.counter.snapshot(engine).await.unwrap_or_else(|e| {
            tracing::warn!(engine = %engine, error = %e, "counter snapshot 실패, 0으로 폴백");
            crate::domain::ports::CounterSnapshot {
                calls: 0,
                reset_at: None,
            }
        });
        engine_snapshots.push((engine.to_string(), snap.calls, snap.reset_at));
    }
    let max_usage = engine_snapshots
        .iter()
        .map(|(_, calls, _)| usage_pct(*calls))
        .max()
        .unwrap_or(0);

    // ── 피드 캐시 저장 ──────────────────────────────────────────────────────
    // no-cache 요청도 결과는 저장 (다음 일반 요청에서 HIT)
    // 완전 실패(items 빈 배열) → 저장 스킵
    if !items.is_empty() {
        let ttl = decide_cache_ttl(search_failed_count, max_usage);
        tracing::info!(
            failed_tags = search_failed_count,
            max_usage_pct = max_usage,
            ttl_secs = ttl.as_secs(),
            "feed cache TTL decided"
        );
        state.feed_cache.set(&cache_key, items.clone(), ttl);
    }
    // ────────────────────────────────────────────────────────────────────────

    // S5: 셋 다 차단 + 결과 비어있음 → 안내 응답
    if items.is_empty() && all_engines_blocked(&engine_snapshots) {
        let recovery_at = select_recovery_at(
            &engine_snapshots
                .iter()
                .map(|(e, _, r)| (e.clone(), *r))
                .collect::<Vec<_>>(),
        );
        return Ok(Json(FeedResponse {
            items: vec![],
            notice: Some(FeedNotice {
                message: build_quota_notice_message(),
                recovery_at,
            }),
        }));
    }

    let paginated = apply_pagination(items, feed_query.limit, feed_query.offset);
    Ok(Json(FeedResponse {
        items: paginated.into_iter().map(FeedItemResponse::from).collect(),
        notice: None,
    }))
}

/// URL을 정규화한다 — 중복 제거 키로 사용.
/// - 스킴(http/https) 제거
/// - www. 제거
/// - trailing slash 제거
/// - 소문자로 통일
fn normalize_url(url: &str) -> String {
    let lower = url.to_lowercase();
    let without_scheme = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .unwrap_or(&lower);
    let without_www = without_scheme
        .strip_prefix("www.")
        .unwrap_or(without_scheme);
    without_www.trim_end_matches('/').to_string()
}

/// URL에서 path 세그먼트 목록을 추출한다.
/// - 스킴+도메인 이후의 path만 대상
/// - 소문자 정규화, query string 제거, trailing slash 제거 후 분리
fn url_path_segments(url: &str) -> Vec<String> {
    let path = url
        .find("://")
        .and_then(|scheme_end| {
            let after_scheme = &url[scheme_end + 3..];
            after_scheme
                .find('/')
                .map(|slash_pos| &after_scheme[slash_pos..])
        })
        .unwrap_or("/");
    let path = path.to_lowercase();
    // query string(?), fragment(#) 제거 후 trailing slash 정리
    // str::split은 항상 ≥1 원소를 반환하므로 unwrap() 안전
    let path = path.split('?').next().unwrap();
    let path = path.split('#').next().unwrap().trim_end_matches('/');
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// listing URL 패턴(태그·카테고리·섹션 인덱스 페이지·페이지네이션)을 판별한다.
///
/// Rule 1: 마지막 세그먼트가 listing 키워드 → 차단
///   예) `/tag/` → 차단, `/latest/` → 차단
///
/// Rule 2: 마지막 세그먼트가 순수 정수(페이지 번호)이고 앞에 listing 관련 세그먼트가 있으면 차단
///   예) `/blog/latest/2` → 차단, `/category/news/3` → 차단
///   예) `/2026/04/23` → 통과 (앞 세그먼트 모두 날짜/숫자, listing 키워드 없음)
///
/// Rule 3: 경로 중간에 `topic`/`topics`가 있고 그 바로 뒤 세그먼트가 해시처럼 생겼으면 차단
///   예) `/news/topics/c9qd23k0` → 차단 (c9qd23k0 = 8자 영숫자 혼합 해시)
///   예) `/topic/technology` → 통과 (technology = 의미 있는 슬러그)
pub(super) fn is_listing_url(url: &str) -> bool {
    // 단수·복수 쌍 순서로 나열. Rule 1 판별 대상.
    const LISTING_SEGMENTS: &[&str] = &[
        "tag",
        "tags",
        "category",
        "categories",
        "topic",
        "topics",
        "section",
        "latest",
        "archive",
    ];

    // Rule 2에서 페이지 번호 앞에 있으면 listing으로 판별할 키워드
    const LISTING_PREFIX_SEGMENTS: &[&str] = &[
        "tag",
        "tags",
        "category",
        "categories",
        "topic",
        "topics",
        "section",
        "latest",
        "archive",
        "page",
        "all",
        "recent",
    ];

    let segments = url_path_segments(url);

    // Rule 3: topic/topics 뒤에 해시 세그먼트가 오면 차단
    for i in 0..segments.len().saturating_sub(1) {
        let seg = segments[i].as_str();
        if (seg == "topic" || seg == "topics") && is_hash_segment(&segments[i + 1]) {
            return true;
        }
    }

    match segments.last() {
        None => false,
        Some(last) => {
            // Rule 1: 마지막 세그먼트가 listing 키워드
            if LISTING_SEGMENTS.contains(&last.as_str()) {
                return true;
            }
            // Rule 2: 마지막 세그먼트가 페이지 번호(순수 양의 정수)이고
            //         앞 경로에 listing 관련 세그먼트가 있으면 페이지네이션 listing
            if last.parse::<u32>().is_ok() {
                let preceding = &segments[..segments.len() - 1];
                if preceding
                    .iter()
                    .any(|s| LISTING_PREFIX_SEGMENTS.contains(&s.as_str()))
                {
                    return true;
                }
            }
            false
        }
    }
}

/// 해시/UUID 세그먼트 판별: 8자 이상, 영숫자만, 하이픈 없음, 숫자와 알파벳 혼합.
/// 슬러그(하이픈 포함, 순수 알파벳)와 구분하기 위함.
fn is_hash_segment(s: &str) -> bool {
    if s.len() < 8 || s.contains('-') {
        return false;
    }
    let all_alnum = s.chars().all(|c| c.is_ascii_alphanumeric());
    all_alnum && s.chars().any(|c| c.is_ascii_digit()) && s.chars().any(|c| c.is_ascii_alphabetic())
}

/// 홈페이지/단순 목록 URL을 판별한다.
/// path 세그먼트가 1개 이하면 개별 기사가 아닌 것으로 간주.
/// 주의: `/openai-launches-model` 같은 단일 슬러그 기사도 탈락할 수 있다.
/// 검색 엔진이 반환하는 URL 중 이런 패턴은 드물어 허용 가능한 트레이드오프로 채택.
pub(super) fn is_homepage_url(url: &str) -> bool {
    url_path_segments(url).len() <= 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Profile, SearchResult};
    use crate::domain::ports::FeedCachePort as _;
    use crate::domain::ports::SearchChainPort;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_favorites::FakeFavoritesAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_quiz_wrong_answers::FakeQuizWrongAnswerAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::feed_cache::{InMemoryFeedCache, NoopFeedCache};
    use crate::infra::search_chain::SearchFallbackChain;
    use crate::middleware::auth::AuthUser;
    use axum::Router;
    use axum::routing::get;
    use axum_test::TestServer;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    fn make_test_state(
        db: FakeDbAdapter,
        search_results: Vec<SearchResult>,
    ) -> super::super::AppState<FakeDbAdapter> {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            search_results,
            false,
        ))]);
        super::super::AppState {
            db,
            search_chain: Arc::new(chain) as Arc<dyn SearchChainPort>,
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
            feed_cache: Arc::new(NoopFeedCache),
            counter: Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new()),
        }
    }

    fn make_test_state_with_query_map(
        db: FakeDbAdapter,
        query_map: HashMap<String, Result<Vec<SearchResult>, String>>,
    ) -> super::super::AppState<FakeDbAdapter> {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::with_query_map(
            "test", query_map,
        ))]);
        super::super::AppState {
            db,
            search_chain: Arc::new(chain) as Arc<dyn SearchChainPort>,
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
            feed_cache: Arc::new(NoopFeedCache),
            counter: Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new()),
        }
    }

    fn make_test_state_with_cache(
        db: FakeDbAdapter,
        search_results: Vec<SearchResult>,
    ) -> (
        super::super::AppState<FakeDbAdapter>,
        Arc<InMemoryFeedCache>,
    ) {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            search_results,
            false,
        ))]);
        let cache = Arc::new(InMemoryFeedCache::new(10));
        let state = super::super::AppState {
            db,
            search_chain: Arc::new(chain) as Arc<dyn SearchChainPort>,
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
            feed_cache: Arc::clone(&cache) as Arc<dyn crate::domain::ports::FeedCachePort>,
            counter: Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new()),
        };
        (state, cache)
    }

    fn make_test_state_with_cache_and_query_map(
        db: FakeDbAdapter,
        query_map: HashMap<String, Result<Vec<SearchResult>, String>>,
    ) -> (
        super::super::AppState<FakeDbAdapter>,
        Arc<InMemoryFeedCache>,
    ) {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::with_query_map(
            "test", query_map,
        ))]);
        let cache = Arc::new(InMemoryFeedCache::new(10));
        let state = super::super::AppState {
            db,
            search_chain: Arc::new(chain) as Arc<dyn SearchChainPort>,
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
            feed_cache: Arc::clone(&cache) as Arc<dyn crate::domain::ports::FeedCachePort>,
            counter: Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new()),
        };
        (state, cache)
    }

    fn make_app(state: super::super::AppState<FakeDbAdapter>, user_id: Uuid) -> Router {
        Router::new()
            .route("/me/feed", get(get_feed::<FakeDbAdapter>))
            .layer(Extension(state))
            .layer(Extension(AuthUser { id: user_id }))
    }

    #[tokio::test]
    async fn get_feed_empty_when_no_tags() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn get_feed_returns_search_results() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: Some("Tester".to_string()),
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_id = tags[0].id;
        db.seed_user_tag(user_id, tag_id);

        let results = vec![SearchResult {
            title: "Test Article".to_string(),
            url: "https://example.com/news/test-article".to_string(),
            snippet: Some("test snippet".to_string()),
            published_at: None,
            image_url: None,
        }];

        let state = make_test_state(db, results);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Test Article");
        assert_eq!(items[0].tag_id, Some(tag_id));
    }

    #[tokio::test]
    async fn get_feed_no_db_storage() {
        // 피드는 DB에 저장하지 않음 — 검색 결과만 반환
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_id = tags[0].id;
        db.seed_user_tag(user_id, tag_id);

        let results = vec![SearchResult {
            title: "Article".to_string(),
            url: "https://example.com/news/article".to_string(),
            snippet: None,
            published_at: None,
            image_url: None,
        }];

        let state = make_test_state(db, results);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        // 응답은 있지만 DB 저장 없음 — FeedItemResponse에 id 필드 없음
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 1);
        let json = serde_json::to_value(&items[0]).unwrap();
        assert!(
            json.get("id").is_none(),
            "id 필드는 ephemeral 피드에 없어야 한다"
        );
    }

    #[tokio::test]
    async fn get_feed_deduplicates_by_url() {
        // 여러 태그에서 동일 URL → 중복 제거
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = tags[0].id;
        let tag_b = tags[1].id;
        db.seed_user_tag(user_id, tag_a);
        db.seed_user_tag(user_id, tag_b);

        // 두 태그 모두 동일 URL 반환
        let results = vec![SearchResult {
            title: "Shared Article".to_string(),
            url: "https://example.com/news/shared".to_string(),
            snippet: None,
            published_at: None,
            image_url: None,
        }];

        let state = make_test_state(db, results);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        // 중복 제거 → 1개
        assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    async fn get_feed_skips_homepage_urls() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_id = tags[0].id;
        db.seed_user_tag(user_id, tag_id);

        let results = vec![
            SearchResult {
                title: "Homepage".to_string(),
                url: "https://example.com/".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            },
            SearchResult {
                title: "Real Article".to_string(),
                url: "https://example.com/news/real-article".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            },
        ];

        let state = make_test_state(db, results);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 1);
        assert!(items[0].url.contains("real-article"));
    }

    #[tokio::test]
    async fn get_feed_skips_failed_searches() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_id = tags[0].id;
        db.seed_user_tag(user_id, tag_id);

        // 검색 실패 시 빈 피드 반환 (에러 아님)
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            vec![],
            true, // should_fail = true
        ))]);
        let state = super::super::AppState {
            db,
            search_chain: Arc::new(chain) as Arc<dyn SearchChainPort>,
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
            feed_cache: Arc::new(NoopFeedCache),
            counter: Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new()),
        };
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert!(items.is_empty());
    }

    // ── 캐시 테스트 (TDD: 구현 전 실패해야 함) ──────────────────────────────

    #[tokio::test]
    async fn get_feed_cache_hit_skips_search_api() {
        // 캐시 HIT 시 검색 API 호출 없이 캐시 데이터 반환
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_id = tags[0].id;
        db.seed_user_tag(user_id, tag_id);

        let cached_items = vec![crate::domain::models::FeedItem {
            title: "Cached Article".to_string(),
            url: "https://example.com/news/cached".to_string(),
            snippet: None,
            source: "cache".to_string(),
            published_at: None,
            tag_id: Some(tag_id),
            image_url: None,
        }];

        // 검색 API는 다른 기사를 반환하도록 설정 (캐시가 우선이면 이 결과는 나오지 않아야 함)
        let (state, cache) = make_test_state_with_cache(
            db,
            vec![crate::domain::models::SearchResult {
                title: "Search Article".to_string(),
                url: "https://example.com/news/search".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }],
        );

        // 사용자 태그 ID 정렬 → 캐시 키 생성 (단일 태그)
        let cache_key = format!("{}:{}", user_id, tag_id);
        cache.set(
            &cache_key,
            cached_items,
            std::time::Duration::from_secs(300),
        );

        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].title, "Cached Article",
            "캐시 HIT → 검색 API 호출 없음"
        );
    }

    #[tokio::test]
    async fn get_feed_cache_miss_calls_search_and_stores() {
        // 캐시 MISS 시 검색 API 호출 후 결과를 캐시에 저장
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_id = tags[0].id;
        db.seed_user_tag(user_id, tag_id);

        let (state, cache) = make_test_state_with_cache(
            db,
            vec![crate::domain::models::SearchResult {
                title: "Fresh Article".to_string(),
                url: "https://example.com/news/fresh".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }],
        );

        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        // 첫 요청 → MISS → 검색 API 호출
        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Fresh Article");

        // 두 번째 요청 → HIT → 캐시 데이터 반환 (검색 API 재호출 없음)
        let resp2 = server.get("/me/feed").await;
        resp2.assert_status_ok();
        let items2: Vec<FeedItemResponse> = resp2.json::<FeedResponse>().items;
        assert_eq!(items2.len(), 1, "캐시 MISS 후 저장 → 두 번째 요청은 HIT");

        // 캐시에 데이터가 저장됐는지 직접 확인
        let cache_key = format!("{}:{}", user_id, tag_id);
        assert!(
            cache.get(&cache_key).is_some(),
            "첫 요청 후 캐시에 저장되어야 함"
        );
    }

    #[tokio::test]
    async fn get_feed_no_cache_header_bypasses_cache() {
        // Cache-Control: no-cache 헤더 시 캐시 우회 → 항상 검색 API 호출
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_id = tags[0].id;
        db.seed_user_tag(user_id, tag_id);

        let cached_items = vec![crate::domain::models::FeedItem {
            title: "Stale Cached Article".to_string(),
            url: "https://example.com/news/stale".to_string(),
            snippet: None,
            source: "cache".to_string(),
            published_at: None,
            tag_id: Some(tag_id),
            image_url: None,
        }];

        let (state, cache) = make_test_state_with_cache(
            db,
            vec![crate::domain::models::SearchResult {
                title: "Fresh Article".to_string(),
                url: "https://example.com/news/fresh-no-cache".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }],
        );

        // 캐시에 stale 데이터 미리 저장
        let cache_key = format!("{}:{}", user_id, tag_id);
        cache.set(
            &cache_key,
            cached_items,
            std::time::Duration::from_secs(300),
        );

        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        // no-cache 헤더로 요청 → 캐시 우회 → 검색 API 호출
        let resp = server
            .get("/me/feed")
            .add_header(
                axum::http::HeaderName::from_static("cache-control"),
                axum::http::HeaderValue::from_static("no-cache"),
            )
            .await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].title, "Fresh Article",
            "no-cache → 캐시 우회 → 검색 API 호출"
        );
    }

    #[test]
    fn feed_item_response_has_no_id_field() {
        let item = FeedItemResponse {
            title: "Test".to_string(),
            url: "https://example.com/news/test".to_string(),
            snippet: Some("snippet".to_string()),
            source: "test".to_string(),
            published_at: None,
            tag_id: Some(Uuid::new_v4()),
            image_url: None,
        };
        let json = serde_json::to_value(&item).unwrap();
        assert!(json.get("id").is_none());
        assert!(json.get("title").is_some());
        assert!(json.get("url").is_some());
        assert!(json.get("tag_id").is_some());
    }

    #[test]
    fn test_normalize_url() {
        assert_eq!(
            normalize_url("https://example.com/news/"),
            "example.com/news"
        );
        assert_eq!(
            normalize_url("http://www.example.com/news"),
            "example.com/news"
        );
        assert_eq!(
            normalize_url("https://www.example.com/news/"),
            "example.com/news"
        );
        assert_eq!(
            normalize_url("HTTPS://Example.com/News"),
            "example.com/news"
        );
        // http vs https → 같은 키
        assert_eq!(
            normalize_url("http://example.com/a"),
            normalize_url("https://example.com/a")
        );
        // www vs non-www → 같은 키
        assert_eq!(
            normalize_url("https://www.example.com/a"),
            normalize_url("https://example.com/a")
        );
    }

    #[test]
    fn test_is_homepage_url() {
        assert!(is_homepage_url("https://example.com/"));
        assert!(is_homepage_url("https://example.com"));
        assert!(is_homepage_url("https://example.com/blog/"));
        assert!(!is_homepage_url("https://example.com/news/some-article"));
        assert!(!is_homepage_url("https://example.com/2024/01/my-post"));
    }

    // MARK: - ST-3: is_listing_url 단위 테스트

    #[test]
    fn listing_url_tag_only_filtered() {
        assert!(is_listing_url("https://example.com/tag/"), "/tag/ → 차단");
        assert!(is_listing_url("https://example.com/tags/"), "/tags/ → 차단");
    }

    #[test]
    fn listing_url_category_only_filtered() {
        assert!(
            is_listing_url("https://example.com/category/"),
            "/category/ → 차단"
        );
        assert!(
            is_listing_url("https://example.com/categories/"),
            "/categories/ → 차단"
        );
    }

    #[test]
    fn listing_url_topics_only_filtered() {
        assert!(
            is_listing_url("https://example.com/topics/"),
            "/topics/ → 차단"
        );
        assert!(
            is_listing_url("https://example.com/topic/"),
            "/topic/ → 차단"
        );
        assert!(
            is_listing_url("https://example.com/section/"),
            "/section/ → 차단"
        );
    }

    #[test]
    fn listing_url_real_article_not_filtered() {
        assert!(
            !is_listing_url("https://example.com/news/some-article"),
            "/news/some-article → 통과"
        );
        assert!(
            !is_listing_url("https://example.com/2026/04/my-post"),
            "/2026/04/my-post → 통과"
        );
    }

    #[test]
    fn listing_url_category_with_slug_not_filtered() {
        assert!(
            !is_listing_url("https://example.com/category/tech/article-slug"),
            "/category/tech/article-slug → 통과 (오탐 방지)"
        );
    }

    #[test]
    fn listing_url_tag_with_value_not_filtered() {
        // 수용된 트레이드오프: /tag/ai-news 는 통과 (마지막 세그먼트 = "ai-news", 비-listing)
        assert!(
            !is_listing_url("https://example.com/tag/ai-news"),
            "/tag/ai-news → 통과 (트레이드오프 수용)"
        );
    }

    #[test]
    fn listing_url_case_insensitive() {
        assert!(
            is_listing_url("https://example.com/TAG/"),
            "/TAG/ → 차단 (대소문자 무관)"
        );
        assert!(
            is_listing_url("https://example.com/Category/"),
            "/Category/ → 차단 (대소문자 무관)"
        );
    }

    #[test]
    fn listing_url_with_fragment_filtered() {
        // fragment가 있어도 listing URL은 차단돼야 한다
        assert!(
            is_listing_url("https://example.com/tag#top"),
            "/tag#top → 차단 (fragment 무시)"
        );
        assert!(
            is_listing_url("https://example.com/category#foo"),
            "/category#foo → 차단 (fragment 무시)"
        );
    }

    #[test]
    fn listing_url_with_query_and_fragment_filtered() {
        // query + fragment 동시에 있어도 listing 판별이 동작해야 한다
        assert!(
            is_listing_url("https://example.com/tag?page=2#section"),
            "/tag?page=2#section → 차단"
        );
        assert!(
            !is_listing_url("https://example.com/news/article?ref=home#comments"),
            "/news/article?...#... → 통과"
        );
    }

    // MARK: - Rule 2: 페이지네이션 listing (마지막 세그먼트가 정수 + 앞에 listing 키워드)

    #[test]
    fn listing_url_paginated_latest_filtered() {
        // /blog/latest/2 → Rule 2: last=2(정수), 앞에 "latest" → 차단
        assert!(
            is_listing_url("https://developer.android.com/blog/latest/2?hl=ko"),
            "/blog/latest/2 → 차단 (페이지네이션 listing)"
        );
        assert!(
            is_listing_url("https://example.com/latest/3"),
            "/latest/3 → 차단"
        );
    }

    #[test]
    fn listing_url_paginated_category_filtered() {
        assert!(
            is_listing_url("https://example.com/category/news/2"),
            "/category/news/2 → 차단 (앞에 category)"
        );
        assert!(
            is_listing_url("https://example.com/page/5"),
            "/page/5 → 차단 (앞에 page)"
        );
    }

    #[test]
    fn listing_url_date_path_not_filtered() {
        // 날짜 기반 기사 URL: 앞 세그먼트가 숫자뿐이라 listing_prefix 없음 → 통과
        assert!(
            !is_listing_url("https://example.com/2026/04/23"),
            "/2026/04/23 → 통과 (날짜 경로)"
        );
        assert!(
            !is_listing_url("https://example.com/2026/01/my-article"),
            "/2026/01/my-article → 통과"
        );
    }

    #[test]
    fn listing_url_latest_without_page_filtered() {
        // /latest/ 자체도 Rule 1으로 차단
        assert!(
            is_listing_url("https://example.com/blog/latest"),
            "/blog/latest → 차단 (Rule 1: last=latest)"
        );
        assert!(
            is_listing_url("https://example.com/archive"),
            "/archive → 차단 (Rule 1)"
        );
    }

    // MARK: - Rule 3: topic/topics + 해시 세그먼트 차단 테스트

    #[test]
    fn listing_url_topics_hash_filtered() {
        assert!(
            is_listing_url("https://www.bbc.com/news/topics/c9qd23k0"),
            "BBC /news/topics/c9qd23k0 → 차단 (Rule 3: 해시 세그먼트)"
        );
        assert!(
            is_listing_url("https://example.com/news/topics/abc12345"),
            "/news/topics/abc12345 → 차단 (8자 영숫자 혼합)"
        );
    }

    #[test]
    fn listing_url_topics_slug_not_filtered() {
        assert!(
            !is_listing_url("https://example.com/topic/technology"),
            "/topic/technology → 통과 (슬러그, 순수 알파벳)"
        );
        assert!(
            !is_listing_url("https://example.com/topic/tech/great-article"),
            "/topic/tech/great-article → 통과 (슬러그에 하이픈)"
        );
    }

    // MARK: - ST-M1-3: 피드 페이지네이션 테스트

    #[tokio::test]
    async fn get_feed_pagination_limit_offset() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = &tags[0];
        db.seed_user_tag(user_id, tag_a.id);

        let results: Vec<SearchResult> = (0..5u32)
            .map(|i| SearchResult {
                title: format!("기사 {i}"),
                url: format!("https://example.com/news/article-{i}"),
                snippet: None,
                published_at: None,
                image_url: None,
            })
            .collect();

        let state = make_test_state(db, results);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        // limit=2, offset=0 → 2개
        let resp = server
            .get("/me/feed")
            .add_query_params([("limit", "2"), ("offset", "0")])
            .await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 2, "limit=2 → 2개 반환");

        // limit=2, offset=2 → 2개 (3~4번째)
        let resp = server
            .get("/me/feed")
            .add_query_params([("limit", "2"), ("offset", "2")])
            .await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 2, "limit=2, offset=2 → 2개 반환");

        // limit=2, offset=4 → 1개 (마지막 1개)
        let resp = server
            .get("/me/feed")
            .add_query_params([("limit", "2"), ("offset", "4")])
            .await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 1, "limit=2, offset=4 → 1개 반환");

        // limit 없음 → 전체 5개
        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 5, "limit 없음 → 전체 반환");
    }

    #[tokio::test]
    async fn get_feed_limit_over_max_returns_400() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .get("/me/feed")
            .add_query_params([("limit", "51")])
            .await;
        resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
    }

    /// S1: 태그 2개 — 각 태그 쿼리에 서로 다른 결과 → 두 결과가 모두 합산됨
    #[tokio::test]
    async fn get_feed_parallel_tags_merged() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = &tags[0]; // "AI/ML"
        let tag_b = &tags[1]; // "웹 개발"
        db.seed_user_tag(user_id, tag_a.id);
        db.seed_user_tag(user_id, tag_b.id);

        let query_a = format!("{} latest news", tag_a.name);
        let query_b = format!("{} latest news", tag_b.name);

        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(
            query_a,
            Ok(vec![SearchResult {
                title: "AI Article".to_string(),
                url: "https://example.com/news/ai-article".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );
        query_map.insert(
            query_b,
            Ok(vec![SearchResult {
                title: "Web Article".to_string(),
                url: "https://example.com/news/web-article".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );

        let state = make_test_state_with_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        // 두 태그 결과가 모두 포함 (URL이 달라 dedupe 없음)
        assert_eq!(items.len(), 2, "두 태그의 결과가 모두 합산돼야 한다");
        let urls: Vec<&str> = items.iter().map(|i| i.url.as_str()).collect();
        assert!(urls.contains(&"https://example.com/news/ai-article"));
        assert!(urls.contains(&"https://example.com/news/web-article"));
    }

    /// M3 S0: tag_id 쿼리 파라미터 — 해당 태그 결과만 반환
    #[tokio::test]
    async fn get_feed_with_tag_id_returns_only_that_tag() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = &tags[0]; // "AI/ML"
        let tag_b = &tags[1]; // "웹 개발"
        db.seed_user_tag(user_id, tag_a.id);
        db.seed_user_tag(user_id, tag_b.id);

        let query_a = format!("{} latest news", tag_a.name);
        let query_b = format!("{} latest news", tag_b.name);

        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(
            query_a,
            Ok(vec![SearchResult {
                title: "AI Article".to_string(),
                url: "https://example.com/news/ai-only".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );
        query_map.insert(
            query_b,
            Ok(vec![SearchResult {
                title: "Web Article".to_string(),
                url: "https://example.com/news/web-only".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );

        let state = make_test_state_with_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        // tag_id=tag_a.id 로 요청 → tag_a 결과만
        let resp = server.get(&format!("/me/feed?tag_id={}", tag_a.id)).await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 1, "tag_id 필터 시 해당 태그 결과만 반환");
        assert_eq!(items[0].url, "https://example.com/news/ai-only");
        assert_eq!(items[0].tag_id, Some(tag_a.id));
    }

    /// M3 S0: tag_id 없으면 전체 태그 검색 (하위 호환)
    #[tokio::test]
    async fn get_feed_without_tag_id_returns_all_tags() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = &tags[0];
        let tag_b = &tags[1];
        db.seed_user_tag(user_id, tag_a.id);
        db.seed_user_tag(user_id, tag_b.id);

        let query_a = format!("{} latest news", tag_a.name);
        let query_b = format!("{} latest news", tag_b.name);

        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(
            query_a,
            Ok(vec![SearchResult {
                title: "AI Article".to_string(),
                url: "https://example.com/news/ai-all".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );
        query_map.insert(
            query_b,
            Ok(vec![SearchResult {
                title: "Web Article".to_string(),
                url: "https://example.com/news/web-all".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );

        let state = make_test_state_with_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        // tag_id 없이 요청 → 전체 태그 결과
        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 2, "tag_id 없으면 전체 태그 결과 합산");
    }

    /// M3 S0: 사용자 구독 태그가 아닌 tag_id → 빈 결과 (403 아님)
    #[tokio::test]
    async fn get_feed_with_unsubscribed_tag_id_returns_empty() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = &tags[0];
        // tag_b는 구독하지 않음
        db.seed_user_tag(user_id, tag_a.id);

        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let random_tag_id = Uuid::new_v4();
        let resp = server
            .get(&format!("/me/feed?tag_id={random_tag_id}"))
            .await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert!(items.is_empty(), "미구독 tag_id → 빈 결과");
    }

    /// ST-3: listing URL 필터 — /tag/, /category/ 패턴은 피드에서 제거됨
    #[tokio::test]
    async fn get_feed_skips_listing_urls() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_id = tags[0].id;
        db.seed_user_tag(user_id, tag_id);

        let results = vec![
            SearchResult {
                title: "Tag Listing".to_string(),
                url: "https://example.com/tag/".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            },
            SearchResult {
                title: "Category Listing".to_string(),
                url: "https://example.com/category/".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            },
            SearchResult {
                title: "Real Article".to_string(),
                url: "https://example.com/news/real-article".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            },
        ];

        let state = make_test_state(db, results);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 1, "listing URL은 피드에서 제거돼야 한다");
        assert!(items[0].url.contains("real-article"));
    }

    /// ST-1: top_keywords가 있으면 search_query에 append
    #[tokio::test]
    async fn get_feed_personalizes_query_with_top_keywords() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = &tags[0]; // "AI/ML"
        db.seed_user_tag(user_id, tag_a.id);

        // like_count >= 3 충족 (개인화 활성화 조건)
        for _ in 0..3 {
            db.increment_like_count(user_id).await.unwrap();
        }

        // GPT(weight=2), transformer(weight=1) seed — tag_a.id 기준으로 심어야 함
        db.increment_keyword_weights(
            user_id,
            tag_a.id,
            vec!["GPT".to_string(), "transformer".to_string()],
        )
        .await
        .unwrap();
        db.increment_keyword_weights(user_id, tag_a.id, vec!["GPT".to_string()])
            .await
            .unwrap();

        // top 3 keywords: GPT(2), transformer(1) → suffix = " GPT transformer"
        let personalized_query = format!("{} latest news GPT transformer", tag_a.name);

        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(
            personalized_query,
            Ok(vec![SearchResult {
                title: "Personalized AI Article".to_string(),
                url: "https://example.com/news/personalized-ai".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );

        let state = make_test_state_with_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(
            items.len(),
            1,
            "personalized query로 검색된 결과가 반환돼야 한다"
        );
        assert_eq!(items[0].url, "https://example.com/news/personalized-ai");
    }

    /// ST-1: top_keywords가 비었으면 기존 쿼리 유지
    #[tokio::test]
    async fn get_feed_uses_default_query_when_no_keywords() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = &tags[0]; // "AI/ML"
        db.seed_user_tag(user_id, tag_a.id);

        // keyword 없음 → 기존 쿼리 유지
        let default_query = format!("{} latest news", tag_a.name);

        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(
            default_query,
            Ok(vec![SearchResult {
                title: "Default AI Article".to_string(),
                url: "https://example.com/news/default-ai".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );

        let state = make_test_state_with_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(
            items.len(),
            1,
            "keyword 없으면 기존 쿼리로 검색된 결과가 반환돼야 한다"
        );
        assert_eq!(items[0].url, "https://example.com/news/default-ai");
    }

    /// ST-1 fix: tag_id 지정 시 키워드 boost 미적용 — cross-tag 오염 방지
    #[tokio::test]
    async fn get_feed_with_tag_id_skips_keyword_boost() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = &tags[0]; // "AI/ML"
        db.seed_user_tag(user_id, tag_a.id);

        // like_count >= 3 + 키워드 세팅
        for _ in 0..3 {
            db.increment_like_count(user_id).await.unwrap();
        }
        let kw_tag_id = Uuid::new_v4();
        db.increment_keyword_weights(user_id, kw_tag_id, vec!["Swift".to_string()])
            .await
            .unwrap();

        // tag_id 지정 시 keyword_suffix 없는 기본 쿼리로 검색돼야 한다
        let default_query = format!("{} latest news", tag_a.name);

        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(
            default_query,
            Ok(vec![SearchResult {
                title: "AI Article".to_string(),
                url: "https://example.com/news/ai".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );

        let state = make_test_state_with_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get(&format!("/me/feed?tag_id={}", tag_a.id)).await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(
            items.len(),
            1,
            "tag_id 지정 시 키워드 boost 없는 기본 쿼리로 검색돼야 한다"
        );
        assert_eq!(items[0].url, "https://example.com/news/ai");
    }

    // MARK: - ST-02 BUG-006 F-03: 부분 실패 TTL 분기 테스트

    /// F-02: 완전 성공 시 5분 TTL 저장 확인
    #[tokio::test]
    async fn feed_cache_full_success_uses_5min_ttl() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = &tags[0];
        db.seed_user_tag(user_id, tag_a.id);

        let query_a = format!("{} latest news", tag_a.name);
        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(
            query_a.clone(),
            Ok(vec![SearchResult {
                title: "AI Article".to_string(),
                url: "https://example.com/news/ai".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );

        let (state, cache) = make_test_state_with_cache_and_query_map(db, query_map);
        let cache_key = format!("{}:{}", user_id, tag_a.id);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();

        // 완전 성공 → 남은 TTL이 4분 이상 (5분에서 약간 경과)
        let remaining = cache.remaining_ttl(&cache_key);
        assert!(remaining.is_some(), "완전 성공 → 캐시에 저장되어야 함");
        let secs = remaining.unwrap().as_secs();
        assert!(
            secs > 4 * 60,
            "완전 성공 TTL은 5분(300초)에 가까워야 함, 실제: {secs}초"
        );
    }

    /// F-03: 부분 실패(일부 태그 검색 오류) 시 1분 단축 TTL 저장 확인
    #[tokio::test]
    async fn feed_cache_partial_failure_uses_1min_ttl() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = &tags[0]; // 실패
        let tag_b = &tags[1]; // 성공
        db.seed_user_tag(user_id, tag_a.id);
        db.seed_user_tag(user_id, tag_b.id);

        let query_a = format!("{} latest news", tag_a.name);
        let query_b = format!("{} latest news", tag_b.name);

        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(query_a, Err("search engine error".to_string()));
        query_map.insert(
            query_b,
            Ok(vec![SearchResult {
                title: "Web Article".to_string(),
                url: "https://example.com/news/web".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );

        let mut sorted_ids = [tag_a.id.to_string(), tag_b.id.to_string()];
        sorted_ids.sort();
        let cache_key = format!("{}:{}", user_id, sorted_ids.join(","));

        let (state, cache) = make_test_state_with_cache_and_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert_eq!(items.len(), 1, "부분 실패: 성공한 태그 결과만 반환");

        // 부분 실패 → 남은 TTL이 1분(60초) 이하
        let remaining = cache.remaining_ttl(&cache_key);
        assert!(
            remaining.is_some(),
            "부분 실패도 성공 결과 있으면 캐시에 저장되어야 함"
        );
        let secs = remaining.unwrap().as_secs();
        assert!(
            secs <= 60,
            "부분 실패 TTL은 1분(60초) 이하여야 함, 실제: {secs}초"
        );
    }

    /// F-02: 완전 실패(모든 태그 검색 실패) 시 캐시 저장 안 함
    #[tokio::test]
    async fn feed_cache_complete_failure_not_cached() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = &tags[0];
        db.seed_user_tag(user_id, tag_a.id);

        let query_a = format!("{} latest news", tag_a.name);
        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(query_a, Err("total failure".to_string()));

        let cache_key = format!("{}:{}", user_id, tag_a.id);
        let (state, cache) = make_test_state_with_cache_and_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        assert!(items.is_empty(), "완전 실패 → 빈 결과");

        // 완전 실패 → 캐시에 저장 안 함
        assert!(
            cache.remaining_ttl(&cache_key).is_none(),
            "완전 실패 시 캐시 저장 안 함"
        );
    }

    /// S1: 태그 A 쿼리 실패, 태그 B 쿼리 성공 → B 결과만 반환
    #[tokio::test]
    async fn get_feed_one_tag_fails_other_succeeds() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_a = &tags[0]; // "AI/ML" — 실패
        let tag_b = &tags[1]; // "웹 개발" — 성공
        db.seed_user_tag(user_id, tag_a.id);
        db.seed_user_tag(user_id, tag_b.id);

        let query_a = format!("{} latest news", tag_a.name);
        let query_b = format!("{} latest news", tag_b.name);

        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(query_a, Err("search failed".to_string()));
        query_map.insert(
            query_b,
            Ok(vec![SearchResult {
                title: "Web Article".to_string(),
                url: "https://example.com/news/web-only".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );

        let state = make_test_state_with_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json::<FeedResponse>().items;
        // A 실패는 skip, B 성공 결과만
        assert_eq!(items.len(), 1, "실패한 태그는 skip, 성공한 태그 결과만");
        assert_eq!(items[0].url, "https://example.com/news/web-only");
    }

    // ── MVP15 M2 — S3/S5 + S7 통합 테스트 ──────────────────────────────────

    #[test]
    fn decide_cache_ttl_table() {
        // S3: TTL 결정 우선순위 표 기반 검증
        // 1) 부분 실패 → 1분 (항상 우선)
        assert_eq!(
            decide_cache_ttl(1, 0),
            FEED_CACHE_TTL_PARTIAL,
            "부분 실패 → 1분"
        );
        assert_eq!(
            decide_cache_ttl(1, 95),
            FEED_CACHE_TTL_PARTIAL,
            "부분 실패는 사용률 95%여도 1분 우선"
        );
        // 2) 사용률 ≥ 80% → 30분
        assert_eq!(decide_cache_ttl(0, 80), FEED_CACHE_TTL_QUOTA_WARN);
        assert_eq!(decide_cache_ttl(0, 99), FEED_CACHE_TTL_QUOTA_WARN);
        assert_eq!(decide_cache_ttl(0, 100), FEED_CACHE_TTL_QUOTA_WARN);
        // 3) 그 외 → 5분
        assert_eq!(decide_cache_ttl(0, 0), FEED_CACHE_TTL);
        assert_eq!(decide_cache_ttl(0, 79), FEED_CACHE_TTL);
    }

    #[test]
    fn usage_pct_table() {
        assert_eq!(usage_pct(0), 0);
        assert_eq!(usage_pct(500), 50);
        assert_eq!(usage_pct(800), 80);
        assert_eq!(usage_pct(999), 99);
        assert_eq!(usage_pct(1000), 100);
        // 100 초과 clamp
        assert_eq!(usage_pct(2000), 100);
    }

    #[test]
    fn select_recovery_at_skips_null_engines() {
        let earliest = chrono::Utc::now() + chrono::Duration::days(3);
        let later = chrono::Utc::now() + chrono::Duration::days(20);
        let snaps = vec![
            ("tavily".to_string(), Some(later)),
            ("exa".to_string(), None),
            ("firecrawl".to_string(), Some(earliest)),
        ];
        let r = select_recovery_at(&snaps);
        assert_eq!(r, Some(earliest), "NULL 제외하고 가장 빠른 reset_at");
    }

    #[test]
    fn select_recovery_at_all_null() {
        let snaps = vec![("exa".to_string(), None)];
        assert_eq!(select_recovery_at(&snaps), None);
    }

    #[test]
    fn all_engines_blocked_table() {
        // 셋 다 ≥1000 → blocked
        let blocked = vec![
            ("tavily".to_string(), 1000, None),
            ("exa".to_string(), 1500, None),
            ("firecrawl".to_string(), 1000, None),
        ];
        assert!(all_engines_blocked(&blocked));

        // 하나라도 < 1000 → not blocked
        let partial = vec![
            ("tavily".to_string(), 1000, None),
            ("exa".to_string(), 999, None),
            ("firecrawl".to_string(), 1000, None),
        ];
        assert!(!all_engines_blocked(&partial));

        // 빈 → not blocked
        assert!(!all_engines_blocked(&[]));
    }

    /// S7 helper: seedable counter handle을 분리한 state 빌더.
    fn make_test_state_with_seedable_counter(
        db: FakeDbAdapter,
        search_results: Vec<SearchResult>,
    ) -> (
        super::super::AppState<FakeDbAdapter>,
        Arc<InMemoryFeedCache>,
        Arc<crate::infra::in_memory_counter::InMemoryCounter>,
    ) {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            search_results,
            false,
        ))]);
        let cache = Arc::new(InMemoryFeedCache::new(10));
        let counter = Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new());
        let state = super::super::AppState {
            db,
            search_chain: Arc::new(chain) as Arc<dyn SearchChainPort>,
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
            feed_cache: Arc::clone(&cache) as Arc<dyn crate::domain::ports::FeedCachePort>,
            counter: Arc::clone(&counter) as Arc<dyn crate::domain::ports::CounterPort>,
        };
        (state, cache, counter)
    }

    /// S7: 통합 시나리오 (a) 800 시드 → 30분 TTL
    #[tokio::test]
    async fn s7_seeded_800_yields_30min_ttl() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(crate::domain::models::Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });
        let tags = db.get_tags();
        let tag_id = tags[0].id;
        db.seed_user_tag(user_id, tag_id);

        let (state, cache, counter) = make_test_state_with_seedable_counter(
            db,
            vec![SearchResult {
                title: "ok".into(),
                url: "https://example.com/news/x".into(),
                snippet: None,
                published_at: None,
                image_url: None,
            }],
        );
        // 카운터 800 시드 (tavily — 80% 도달)
        counter.seed(
            "tavily",
            800,
            Some(chrono::Utc::now() + chrono::Duration::days(7)),
        );

        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();

        let cache_key = format!("{}:{}", user_id, tag_id);
        let ttl = cache.remaining_ttl(&cache_key).expect("캐시에 저장돼야 함");
        // 80% 도달 → 30분 TTL (1800초). 살짝 작은 여유 허용.
        assert!(
            ttl >= Duration::from_secs(1700) && ttl <= Duration::from_secs(1800),
            "30분 TTL 기대, 실제 {}초",
            ttl.as_secs()
        );
    }

    /// S7: (c) 셋 다 1000 시드 → notice + recovery_at min
    #[tokio::test]
    async fn s7_all_blocked_returns_notice_with_recovery_at() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(crate::domain::models::Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });
        let tags = db.get_tags();
        let tag_id = tags[0].id;
        db.seed_user_tag(user_id, tag_id);

        // 모든 엔진을 빈 결과로 — items 비어 있고 셋 다 1000 시드면 안내 트리거
        let (state, _cache, counter) = make_test_state_with_seedable_counter(db, vec![]);

        // 모든 엔진 1000 시드. recovery_at은 tavily가 가장 빠르도록.
        let earliest = chrono::Utc::now() + chrono::Duration::days(3);
        let later = chrono::Utc::now() + chrono::Duration::days(20);
        counter.seed("tavily", 1000, Some(earliest));
        counter.seed("exa", 1000, None); // NULL — 무시
        counter.seed("firecrawl", 1000, Some(later));

        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let body: FeedResponse = resp.json();
        assert!(body.items.is_empty(), "셋 다 차단 + 결과 없음 → 빈 items");
        let notice = body.notice.expect("notice가 있어야 함");
        assert!(notice.message.contains("무료 한도"));
        // recovery_at = tavily의 earliest (NULL인 exa 제외)
        assert_eq!(
            notice.recovery_at.map(|t| t.timestamp()),
            Some(earliest.timestamp()),
            "recovery_at = NULL 제외하고 가장 빠른 reset_at"
        );
    }

    /// S7: (b) CountedSearchAdapter wrap 시나리오 — 1000 시드 시 호출 자체 skip + INC 안 됨
    #[tokio::test]
    async fn s7_counted_decorator_skips_at_quota() {
        use crate::domain::ports::{CounterPort, SearchPort};
        use crate::infra::counted_search::CountedSearchAdapter;
        let counter = Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new());
        counter.seed(
            "tavily",
            1000,
            Some(chrono::Utc::now() + chrono::Duration::days(7)),
        );
        let inner = Box::new(FakeSearchAdapter::new(
            "tavily",
            vec![SearchResult {
                title: "should not appear".into(),
                url: "https://example.com/news/x".into(),
                snippet: None,
                published_at: None,
                image_url: None,
            }],
            false,
        ));
        let notifier: Arc<dyn crate::domain::ports::NotificationPort> =
            Arc::new(FakeNotificationAdapter::new());
        let dec = CountedSearchAdapter::new(
            inner,
            Arc::clone(&counter) as Arc<dyn crate::domain::ports::CounterPort>,
            notifier,
        );
        let out = dec.search("query", 20).await.unwrap();
        assert!(out.is_empty(), "1000 시드 → 호출 skip → 빈 결과");
        let snap = counter.snapshot("tavily").await.unwrap();
        assert_eq!(snap.calls, 1000, "skip 시 INC 안 함 (재시작 후에도 보존)");
    }

    /// S7: source_name 위임 검증 — CountedSearchAdapter는 inner 그대로
    #[tokio::test]
    async fn s7_decorator_source_name_delegates() {
        use crate::domain::ports::SearchPort;
        use crate::infra::counted_search::CountedSearchAdapter;
        let counter = Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new());
        let inner = Box::new(FakeSearchAdapter::new("tavily", vec![], false));
        let notifier: Arc<dyn crate::domain::ports::NotificationPort> =
            Arc::new(FakeNotificationAdapter::new());
        let dec = CountedSearchAdapter::new(
            inner,
            counter as Arc<dyn crate::domain::ports::CounterPort>,
            notifier,
        );
        // source_name == engine PK 일관성 (advisor 지적)
        assert_eq!(dec.source_name(), "tavily");
    }

    /// S7: source_name == engine PK 일관성 (advisor 지적)
    #[test]
    fn s7_engine_ids_match_adapter_source_names() {
        // ENGINE_IDS 와 실제 어댑터 source_name() 일치 검증
        use crate::domain::ports::SearchPort;
        let tavily = crate::infra::tavily::TavilyAdapter::new("k");
        let exa = crate::infra::exa::ExaAdapter::new("k");
        let firecrawl = crate::infra::firecrawl::FirecrawlAdapter::new("k");
        assert_eq!(tavily.source_name(), "tavily");
        assert_eq!(exa.source_name(), "exa");
        assert_eq!(firecrawl.source_name(), "firecrawl");
        // ENGINE_IDS와 일치
        assert!(ENGINE_IDS.contains(&"tavily"));
        assert!(ENGINE_IDS.contains(&"exa"));
        assert!(ENGINE_IDS.contains(&"firecrawl"));
    }
}
