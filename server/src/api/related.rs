use axum::Json;
use axum::extract::{Extension, Query};
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;

use super::AppState;
use super::feed::{FeedItemResponse, is_homepage_url};

/// GET /me/articles/related 쿼리 파라미터
#[derive(Debug, Deserialize)]
pub struct RelatedQuery {
    pub title: String,
    pub snippet: Option<String>,
}

/// GET /me/articles/related
/// title + snippet + 사용자 top 키워드로 연관 기사를 검색해 반환한다.
/// tag_id는 항상 None — 태그 연관 없이 내용 기반 연관 기사만.
pub async fn get_related<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<RelatedQuery>,
) -> Result<Json<Vec<FeedItemResponse>>, AppError> {
    // 좋아요 3회 이상일 때만 키워드 개인화 적용 (feed.rs와 동일 기준)
    // like_count 조회 실패 시 0으로 폴백
    let like_count = state.db.get_like_count(user.id).await.unwrap_or(0);
    // feed.rs의 3개와 달리 5개 사용: 태그 쿼리가 없어 키워드 기반 연관도가 높을수록 정확도 향상
    let top_keywords = if like_count >= 3 {
        state
            .db
            .get_top_keywords(user.id, 5)
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };

    // 검색 쿼리 조합: title + (snippet) + (top_keywords join)
    let mut search_query = query.title.clone();
    if let Some(ref snippet) = query.snippet
        && !snippet.is_empty()
    {
        search_query.push(' ');
        search_query.push_str(snippet);
    }
    if !top_keywords.is_empty() {
        search_query.push(' ');
        search_query.push_str(&top_keywords.join(" "));
    }

    // 검색 실패 시 빈 배열 반환 (연관 기사는 보조 기능 — 500 전파 금지)
    let (search_items, source) = match state.search_chain.search(&search_query, 5).await {
        Ok(result) => result,
        Err(e) => {
            tracing::warn!(error = %e, "related search failed, returning empty");
            return Ok(Json(vec![]));
        }
    };

    let items: Vec<FeedItemResponse> = search_items
        .into_iter()
        .filter(|sr| !is_homepage_url(&sr.url)) // feed.rs와 동일하게 홈페이지 URL 제거
        .map(|sr| FeedItemResponse {
            title: sr.title,
            url: sr.url,
            snippet: sr.snippet,
            source: source.clone(),
            published_at: sr
                .published_at
                .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok()),
            tag_id: None, // 연관 기사는 항상 None
            image_url: sr.image_url,
        })
        .collect();

    Ok(Json(items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::SearchResult;
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
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

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
        }
    }

    fn make_app(state: super::super::AppState<FakeDbAdapter>, user_id: Uuid) -> Router {
        Router::new()
            .route("/me/articles/related", get(get_related::<FakeDbAdapter>))
            .layer(Extension(state))
            .layer(Extension(AuthUser { id: user_id }))
    }

    /// 테스트 1: title + top_keywords로 검색 쿼리가 조합됨
    #[tokio::test]
    async fn get_related_builds_query_from_title_and_keywords() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();

        // like_count >= 3 충족 (개인화 활성화 조건)
        // profile 없이도 fake_db의 get_like_count는 0 반환하므로 increment만 호출
        db.seed_profile(crate::domain::models::Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });
        for _ in 0..3 {
            db.increment_like_count(user_id).await.unwrap();
        }

        // 키워드 사전 세팅
        db.increment_keyword_weights(user_id, vec!["Rust".to_string(), "async".to_string()])
            .await
            .unwrap();

        let top_keywords = db.get_top_keywords(user_id, 5).await.unwrap();
        let expected_query = format!("Concurrency article {}", top_keywords.join(" "));

        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(
            expected_query,
            Ok(vec![SearchResult {
                title: "Related Article".to_string(),
                url: "https://example.com/news/related".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );

        let state = make_test_state_with_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .get("/me/articles/related")
            .add_query_param("title", "Concurrency article")
            .await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json();
        assert_eq!(items.len(), 1, "title + keywords 조합 쿼리로 결과 반환");
        assert_eq!(items[0].title, "Related Article");
    }

    /// 테스트 2: keywords 없을 때 title만으로 검색
    #[tokio::test]
    async fn get_related_uses_title_only_when_no_keywords() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        // 키워드 없음

        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(
            "SwiftUI tutorial".to_string(),
            Ok(vec![SearchResult {
                title: "SwiftUI Guide".to_string(),
                url: "https://example.com/news/swiftui-guide".to_string(),
                snippet: None,
                published_at: None,
                image_url: None,
            }]),
        );

        let state = make_test_state_with_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .get("/me/articles/related")
            .add_query_param("title", "SwiftUI tutorial")
            .await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json();
        assert_eq!(items.len(), 1, "keywords 없으면 title만으로 검색");
        assert_eq!(items[0].title, "SwiftUI Guide");
    }

    /// 테스트 3: snippet 포함 시 쿼리에 반영
    #[tokio::test]
    async fn get_related_includes_snippet_in_query() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        // 키워드 없음 — snippet만 추가

        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(
            "iOS development Swift 5.9 new features".to_string(),
            Ok(vec![SearchResult {
                title: "Swift 5.9 Deep Dive".to_string(),
                url: "https://example.com/news/swift-59-deep-dive".to_string(),
                snippet: Some("Swift 5.9 features".to_string()),
                published_at: None,
                image_url: None,
            }]),
        );

        let state = make_test_state_with_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .get("/me/articles/related")
            .add_query_param("title", "iOS development")
            .add_query_param("snippet", "Swift 5.9 new features")
            .await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json();
        assert_eq!(items.len(), 1, "snippet이 쿼리에 포함돼야 결과 반환");
        assert_eq!(items[0].title, "Swift 5.9 Deep Dive");
    }

    /// 테스트 4: 결과의 tag_id는 항상 None
    #[tokio::test]
    async fn get_related_returns_items_with_no_tag_id() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();

        let mut query_map: HashMap<String, Result<Vec<SearchResult>, String>> = HashMap::new();
        query_map.insert(
            "machine learning".to_string(),
            Ok(vec![
                SearchResult {
                    title: "ML Article 1".to_string(),
                    url: "https://example.com/news/ml-1".to_string(),
                    snippet: None,
                    published_at: None,
                    image_url: None,
                },
                SearchResult {
                    title: "ML Article 2".to_string(),
                    url: "https://example.com/news/ml-2".to_string(),
                    snippet: None,
                    published_at: None,
                    image_url: None,
                },
            ]),
        );

        let state = make_test_state_with_query_map(db, query_map);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .get("/me/articles/related")
            .add_query_param("title", "machine learning")
            .await;
        resp.assert_status_ok();
        let items: Vec<FeedItemResponse> = resp.json();
        assert_eq!(items.len(), 2);
        for item in &items {
            assert!(item.tag_id.is_none(), "연관 기사의 tag_id는 항상 None");
        }
    }
}
