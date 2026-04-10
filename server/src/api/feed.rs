use axum::Json;
use axum::extract::Extension;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::FeedItem;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;

use super::AppState;

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
pub async fn get_feed<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<Vec<FeedItemResponse>>, AppError> {
    let user_tags = state.db.get_user_tags(user.id).await?;
    if user_tags.is_empty() {
        // 태그 없으면 빈 피드 반환 (에러 아님)
        return Ok(Json(vec![]));
    }

    let all_tags = state.db.list_tags().await?;
    let mut items: Vec<FeedItem> = vec![];

    for user_tag in &user_tags {
        let tag_name = all_tags
            .iter()
            .find(|t| t.id == user_tag.tag_id)
            .map(|t| t.name.as_str())
            .unwrap_or("unknown");

        let query = format!("{tag_name} latest news");

        let search_result = state.search_chain.search(&query, 5).await;
        let (results, source) = match search_result {
            Ok(pair) => pair,
            Err(e) => {
                tracing::warn!(tag = tag_name, error = %e, "search failed for tag, skipping");
                continue;
            }
        };

        // tag가 실제로 존재하는지 검증 (orphaned tag_id 방어)
        let tag_id = if all_tags.iter().any(|t| t.id == user_tag.tag_id) {
            Some(user_tag.tag_id)
        } else {
            tracing::warn!(
                tag_id = %user_tag.tag_id,
                "orphaned user_tag: tag not found in all_tags, skipping tag_id"
            );
            None
        };

        for sr in results {
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
                tag_id,
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
fn is_homepage_url(raw_url: &str) -> bool {
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
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::search_chain::SearchFallbackChain;
    use crate::middleware::auth::AuthUser;
    use axum::Router;
    use axum::routing::get;
    use axum_test::TestServer;
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
        assert_eq!(normalize_url("https://example.com/news/"), "example.com/news");
        assert_eq!(normalize_url("http://www.example.com/news"), "example.com/news");
        assert_eq!(normalize_url("https://www.example.com/news/"), "example.com/news");
        assert_eq!(normalize_url("HTTPS://Example.com/News"), "example.com/news");
        // http vs https → 같은 키
        assert_eq!(normalize_url("http://example.com/a"), normalize_url("https://example.com/a"));
        // www vs non-www → 같은 키
        assert_eq!(normalize_url("https://www.example.com/a"), normalize_url("https://example.com/a"));
    }

    #[test]
    fn test_is_homepage_url() {
        assert!(is_homepage_url("https://example.com/"));
        assert!(is_homepage_url("https://example.com"));
        assert!(is_homepage_url("https://example.com/blog/"));
        assert!(!is_homepage_url("https://example.com/news/some-article"));
        assert!(!is_homepage_url("https://example.com/2024/01/my-post"));
    }
}
