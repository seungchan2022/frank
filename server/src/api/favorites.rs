use axum::Json;
use axum::extract::{Extension, Query};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::Favorite;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;

use super::AppState;

/// POST /me/favorites 요청 바디.
/// id, user_id, created_at은 서버가 채우므로 요청에서 제외.
#[derive(Debug, Deserialize)]
pub struct AddFavoriteRequest {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub source: String,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub tag_id: Option<Uuid>,
    pub summary: Option<String>,
    pub insight: Option<String>,
}

/// DELETE /me/favorites?url=... 쿼리 파라미터.
#[derive(Debug, Deserialize)]
pub struct DeleteFavoriteQuery {
    pub url: String,
}

/// POST /me/favorites
/// 즐겨찾기 추가 → 201 + Favorite JSON.
/// 중복 시 409 Conflict.
pub async fn add_favorite<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<AddFavoriteRequest>,
) -> Result<(StatusCode, Json<Favorite>), AppError> {
    if body.url.trim().is_empty() {
        return Err(AppError::BadRequest("url is required".to_string()));
    }

    // DTO → Favorite (id/user_id/created_at은 어댑터에서 채움)
    let item = Favorite {
        id: Uuid::nil(), // 어댑터에서 새 UUID 발급
        user_id: user.id,
        title: body.title,
        url: body.url,
        snippet: body.snippet,
        source: body.source,
        published_at: body.published_at,
        tag_id: body.tag_id,
        summary: body.summary,
        insight: body.insight,
        liked_at: None,
        created_at: None,
    };

    let favorite = state.favorites.add_favorite(user.id, &item).await?;
    Ok((StatusCode::CREATED, Json(favorite)))
}

/// DELETE /me/favorites?url=<encoded>
/// 즐겨찾기 삭제 → 204.
/// 없는 URL도 204 (no-op). 빈 url은 400.
pub async fn delete_favorite<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<DeleteFavoriteQuery>,
) -> Result<StatusCode, AppError> {
    if params.url.trim().is_empty() {
        return Err(AppError::BadRequest(
            "url query parameter is required".to_string(),
        ));
    }

    state
        .favorites
        .delete_favorite(user.id, &params.url)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /me/favorites
/// 즐겨찾기 목록 조회 → 200 + Vec<Favorite> (created_at DESC).
pub async fn list_favorites<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<Vec<Favorite>>, AppError> {
    let favorites = state.favorites.list_favorites(user.id).await?;
    Ok(Json(favorites))
}

#[cfg(test)]
mod tests {
    use super::*;
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
    use axum::routing::{delete, get, post};
    use axum_test::TestServer;
    use std::sync::Arc;

    fn make_app(favorites: FakeFavoritesAdapter, user_id: Uuid) -> Router {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            vec![],
            false,
        ))]);
        let state = super::super::AppState {
            db: FakeDbAdapter::new(),
            search_chain: Arc::new(chain) as Arc<dyn SearchChainPort>,
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(favorites),
        };

        Router::new()
            .route("/me/favorites", post(add_favorite::<FakeDbAdapter>))
            .route("/me/favorites", delete(delete_favorite::<FakeDbAdapter>))
            .route("/me/favorites", get(list_favorites::<FakeDbAdapter>))
            .layer(Extension(state))
            .layer(Extension(AuthUser { id: user_id }))
    }

    fn valid_body() -> serde_json::Value {
        serde_json::json!({
            "title": "테스트 기사",
            "url": "https://example.com/article",
            "snippet": null,
            "source": "test",
            "published_at": null,
            "tag_id": null,
            "summary": null,
            "insight": null
        })
    }

    #[tokio::test]
    async fn post_favorite_returns_201() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeFavoritesAdapter::new(), user_id));

        let resp = server.post("/me/favorites").json(&valid_body()).await;

        resp.assert_status(StatusCode::CREATED);
        let body: serde_json::Value = resp.json();
        assert_eq!(body["url"], "https://example.com/article");
        assert_eq!(body["user_id"], user_id.to_string());
    }

    #[tokio::test]
    async fn post_favorite_duplicate_returns_409() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeFavoritesAdapter::new(), user_id));

        server.post("/me/favorites").json(&valid_body()).await;
        let resp = server.post("/me/favorites").json(&valid_body()).await;

        resp.assert_status(StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn post_favorite_empty_url_returns_400() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeFavoritesAdapter::new(), user_id));

        let body = serde_json::json!({
            "title": "테스트",
            "url": "",
            "source": "test",
            "snippet": null,
            "published_at": null,
            "tag_id": null,
            "summary": null,
            "insight": null
        });
        let resp = server.post("/me/favorites").json(&body).await;

        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn delete_favorite_returns_204() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeFavoritesAdapter::new(), user_id));

        server.post("/me/favorites").json(&valid_body()).await;
        let resp = server
            .delete("/me/favorites")
            .add_query_param("url", "https://example.com/article")
            .await;

        resp.assert_status(StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn delete_nonexistent_returns_204() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeFavoritesAdapter::new(), user_id));

        let resp = server
            .delete("/me/favorites")
            .add_query_param("url", "https://nonexistent.com")
            .await;

        resp.assert_status(StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn delete_empty_url_returns_400() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeFavoritesAdapter::new(), user_id));

        let resp = server
            .delete("/me/favorites")
            .add_query_param("url", "")
            .await;

        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn get_favorites_returns_200_with_list() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeFavoritesAdapter::new(), user_id));

        // 2개 추가
        server.post("/me/favorites").json(&valid_body()).await;
        let body2 = serde_json::json!({
            "title": "두 번째 기사",
            "url": "https://example.com/second",
            "snippet": null,
            "source": "test",
            "published_at": null,
            "tag_id": null,
            "summary": null,
            "insight": null
        });
        server.post("/me/favorites").json(&body2).await;

        let resp = server.get("/me/favorites").await;

        resp.assert_status_ok();
        let list: Vec<serde_json::Value> = resp.json();
        assert_eq!(list.len(), 2);
        // DESC 정렬: 두 번째 기사가 먼저
        assert_eq!(list[0]["url"], "https://example.com/second");
    }
}
