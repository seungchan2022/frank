use axum::Json;
use axum::extract::{Extension, Query};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::FeedItem;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;

use super::AppState;

/// GET /me/feed 쿼리 파라미터
#[derive(Debug, Deserialize, Default)]
pub struct FeedQuery {
    /// 특정 태그만 필터링. 없으면 전체 태그 검색 (하위 호환)
    pub tag_id: Option<Uuid>,
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

/// GET /me/feed
/// 사용자 태그를 기반으로 검색 API를 직접 호출해 피드를 반환한다.
/// DB에 저장하지 않음 — 매 호출마다 검색 API를 새로 호출.
/// 모든 태그 검색을 `futures::future::join_all`로 병렬 실행한다.
/// `tag_id` 쿼리 파라미터가 있으면 해당 태그만 검색 (하위 호환).
pub async fn get_feed<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Query(feed_query): Query<FeedQuery>,
) -> Result<Json<Vec<FeedItemResponse>>, AppError> {
    let user_tags = state.db.get_user_tags(user.id).await?;
    if user_tags.is_empty() {
        // 태그 없으면 빈 피드 반환 (에러 아님)
        return Ok(Json(vec![]));
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
        return Ok(Json(vec![]));
    }

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

    // join_all로 모든 태그 검색을 동시에 실행
    let chain = std::sync::Arc::clone(&state.search_chain);
    let futures = jobs.into_iter().map(|(tag_id, tag_name, search_query)| {
        let chain = std::sync::Arc::clone(&chain);
        async move {
            let result = chain.search(&search_query, 5).await;
            (tag_id, tag_name, result)
        }
    });
    let results = futures::future::join_all(futures).await;

    let mut items: Vec<FeedItem> = vec![];

    for (tag_id, tag_name, search_result) in results {
        let (search_items, source) = match search_result {
            Ok(pair) => pair,
            Err(e) => {
                tracing::warn!(tag = %tag_name, error = %e, "search failed for tag, skipping");
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
            // 홈페이지/목록 URL 필터링
            if is_homepage_url(&sr.url) {
                tracing::debug!(url = %sr.url, "skipping homepage/listing URL");
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

    Ok(Json(
        items.into_iter().map(FeedItemResponse::from).collect(),
    ))
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

/// 홈페이지/목록 URL을 판별한다.
/// path 세그먼트가 1개 이하면 개별 기사가 아닌 것으로 간주.
pub(super) fn is_homepage_url(raw_url: &str) -> bool {
    let path = raw_url
        .find("://")
        .and_then(|scheme_end| {
            let after_scheme = &raw_url[scheme_end + 3..];
            after_scheme
                .find('/')
                .map(|slash_pos| &after_scheme[slash_pos..])
        })
        .unwrap_or("/");
    let path = path.trim_end_matches('/');
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    segments.len() <= 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Profile, SearchResult};
    use crate::domain::ports::SearchChainPort;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_favorites::FakeFavoritesAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_quiz_wrong_answers::FakeQuizWrongAnswerAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;
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
        }
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
        let items: Vec<FeedItemResponse> = resp.json();
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
        let items: Vec<FeedItemResponse> = resp.json();
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
        let items: Vec<FeedItemResponse> = resp.json();
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
        let items: Vec<FeedItemResponse> = resp.json();
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
        let items: Vec<FeedItemResponse> = resp.json();
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
        };
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/feed").await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json();
        assert!(items.is_empty());
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
        let items: Vec<FeedItemResponse> = resp.json();
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
        let items: Vec<FeedItemResponse> = resp.json();
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
        let items: Vec<FeedItemResponse> = resp.json();
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
        let items: Vec<FeedItemResponse> = resp.json();
        assert!(items.is_empty(), "미구독 tag_id → 빈 결과");
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
        let items: Vec<FeedItemResponse> = resp.json();
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
        let items: Vec<FeedItemResponse> = resp.json();
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
        let items: Vec<FeedItemResponse> = resp.json();
        assert_eq!(
            items.len(),
            1,
            "tag_id 지정 시 키워드 boost 없는 기본 쿼리로 검색돼야 한다"
        );
        assert_eq!(items[0].url, "https://example.com/news/ai");
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
        let items: Vec<FeedItemResponse> = resp.json();
        // A 실패는 skip, B 성공 결과만
        assert_eq!(items.len(), 1, "실패한 태그는 skip, 성공한 태그 결과만");
        assert_eq!(items[0].url, "https://example.com/news/web-only");
    }
}
