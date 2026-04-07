use axum::Json;
use axum::extract::Extension;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::Article;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;
use crate::services::{collect_service, summary_service};

use super::AppState;

/// 클라이언트 노출용 Article DTO.
/// 내부 필드(`content`, `llm_model`, `prompt_tokens`, `completion_tokens`)는 제외한다.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArticleResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tag_id: Option<Uuid>,
    pub title: String,
    pub title_ko: Option<String>,
    pub url: String,
    pub snippet: Option<String>,
    pub source: String,
    pub search_query: Option<String>,
    pub summary: Option<String>,
    pub insight: Option<String>,
    pub summarized_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<Article> for ArticleResponse {
    fn from(a: Article) -> Self {
        Self {
            id: a.id,
            user_id: a.user_id,
            tag_id: a.tag_id,
            title: a.title,
            title_ko: a.title_ko,
            url: a.url,
            snippet: a.snippet,
            source: a.source,
            search_query: a.search_query,
            summary: a.summary,
            insight: a.insight,
            summarized_at: a.summarized_at,
            published_at: a.published_at,
            created_at: a.created_at,
        }
    }
}

pub async fn collect_articles<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    let count = collect_service::collect_for_user(
        &state.db,
        state.search_chain.as_ref(),
        state.crawl.as_ref(),
        user.id,
    )
    .await?;
    Ok(Json(serde_json::json!({ "collected": count })))
}

#[derive(Debug, Deserialize)]
pub struct ListArticlesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    /// 문자열로 받아서 내부에서 UUID 파싱 (파싱 실패 시 400)
    pub tag_id: Option<String>,
}

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 100;

pub async fn list_articles<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    query: axum::extract::Query<ListArticlesQuery>,
) -> Result<Json<Vec<ArticleResponse>>, AppError> {
    let limit_raw = query.limit.unwrap_or(DEFAULT_LIMIT);
    if limit_raw < 1 {
        return Err(AppError::BadRequest("limit must be >= 1".to_string()));
    }
    let limit = limit_raw.min(MAX_LIMIT);

    let offset = query.offset.unwrap_or(0);
    if offset < 0 {
        return Err(AppError::BadRequest("offset must be >= 0".to_string()));
    }

    let tag_id = match query.tag_id.as_deref() {
        Some(s) => Some(
            Uuid::parse_str(s).map_err(|_| AppError::BadRequest("invalid tag_id".to_string()))?,
        ),
        None => None,
    };

    let articles = state
        .db
        .get_user_articles(user.id, limit, offset, tag_id)
        .await?;
    Ok(Json(
        articles.into_iter().map(ArticleResponse::from).collect(),
    ))
}

pub async fn get_article<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    axum::extract::Path(article_id): axum::extract::Path<Uuid>,
) -> Result<Json<ArticleResponse>, AppError> {
    let article = state
        .db
        .get_user_article_by_id(user.id, article_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;
    Ok(Json(ArticleResponse::from(article)))
}

pub async fn summarize_articles<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    let count = summary_service::summarize_articles(
        &state.db,
        state.llm.as_ref(),
        state.notifier.as_ref(),
        user.id,
    )
    .await?;
    Ok(Json(serde_json::json!({ "summarized": count })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Profile, SearchResult};
    use crate::domain::ports::SearchChainPort;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::search_chain::SearchFallbackChain;
    use crate::middleware::auth::AuthUser;
    use axum::Router;
    use axum::routing::{get, post};
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
        }
    }

    fn make_app(state: super::super::AppState<FakeDbAdapter>, user_id: Uuid) -> Router {
        Router::new()
            .route("/me/collect", post(collect_articles::<FakeDbAdapter>))
            .route("/me/articles", get(list_articles::<FakeDbAdapter>))
            .route("/me/articles/{id}", get(get_article::<FakeDbAdapter>))
            .route("/me/summarize", post(summarize_articles::<FakeDbAdapter>))
            .layer(Extension(state))
            .layer(Extension(AuthUser { id: user_id }))
    }

    #[tokio::test]
    async fn list_articles_empty() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/articles").await;
        resp.assert_status_ok();
        let articles: Vec<ArticleResponse> = resp.json();
        assert!(articles.is_empty());
    }

    /// 테스트 전용 Article 생성 헬퍼.
    /// 기본값은 전부 None/기본 문자열이며 필요한 필드만 호출부에서 덮어쓴다.
    fn make_article(user_id: Uuid) -> Article {
        Article {
            id: Uuid::new_v4(),
            user_id,
            tag_id: None,
            title: "title".to_string(),
            url: "https://example.com".to_string(),
            snippet: None,
            source: "test".to_string(),
            search_query: None,
            summary: None,
            insight: None,
            summarized_at: None,
            published_at: None,
            created_at: None,
            title_ko: None,
            content: None,
            llm_model: None,
            prompt_tokens: None,
            completion_tokens: None,
        }
    }

    #[test]
    fn article_response_excludes_internal_fields() {
        // ArticleResponse는 content/llm_model/prompt_tokens/completion_tokens를 노출하지 않는다
        let article = Article {
            title_ko: Some("한글".to_string()),
            content: Some("internal-content".to_string()),
            llm_model: Some("internal-model".to_string()),
            prompt_tokens: Some(123),
            completion_tokens: Some(456),
            ..make_article(Uuid::new_v4())
        };
        let dto = ArticleResponse::from(article);
        let json = serde_json::to_value(&dto).unwrap();
        assert!(json.get("content").is_none());
        assert!(json.get("llm_model").is_none());
        assert!(json.get("prompt_tokens").is_none());
        assert!(json.get("completion_tokens").is_none());
        assert_eq!(json["title_ko"], "한글");
    }

    #[tokio::test]
    async fn collect_articles_with_tags() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: Some("Tester".to_string()),
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_ids: Vec<Uuid> = tags.iter().take(1).map(|t| t.id).collect();
        db.set_user_tags(user_id, tag_ids).await.unwrap();

        let results = vec![SearchResult {
            title: "Test Article".to_string(),
            url: "https://example.com/news/test-article".to_string(),
            snippet: Some("test snippet".to_string()),
            published_at: None,
        }];

        let state = make_test_state(db, results);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.post("/me/collect").await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["collected"], 1);
    }

    #[tokio::test]
    async fn summarize_articles_empty() {
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

        let resp = server.post("/me/summarize").await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["summarized"], 0);
    }

    #[tokio::test]
    async fn list_articles_with_limit() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/articles?limit=10").await;
        resp.assert_status_ok();
    }

    fn seed_articles(db: &FakeDbAdapter, user_id: Uuid, count: usize, tag_id: Option<Uuid>) {
        for i in 0..count {
            db.seed_article(Article {
                tag_id,
                title: format!("title-{i}"),
                url: format!("https://example.com/{i}"),
                ..make_article(user_id)
            });
        }
    }

    #[tokio::test]
    async fn list_articles_offset_pagination() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_articles(&db, user_id, 5, None);
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/articles?limit=2&offset=2").await;
        resp.assert_status_ok();
        let articles: Vec<ArticleResponse> = resp.json();
        assert_eq!(articles.len(), 2);
    }

    #[tokio::test]
    async fn list_articles_tag_filter() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        seed_articles(&db, user_id, 3, Some(tag_id));
        seed_articles(&db, user_id, 2, None);
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get(&format!("/me/articles?tag_id={tag_id}")).await;
        resp.assert_status_ok();
        let articles: Vec<ArticleResponse> = resp.json();
        assert_eq!(articles.len(), 3);
        assert!(articles.iter().all(|a| a.tag_id == Some(tag_id)));
    }

    #[tokio::test]
    async fn list_articles_invalid_tag_id_returns_400() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/articles?tag_id=not-a-uuid").await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn list_articles_negative_offset_returns_400() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/articles?offset=-1").await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn list_articles_zero_limit_returns_400() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/articles?limit=0").await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn get_article_returns_own_article() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        db.seed_article(Article {
            id: article_id,
            title: "mine".to_string(),
            url: "https://example.com/mine".to_string(),
            ..make_article(user_id)
        });
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get(&format!("/me/articles/{article_id}")).await;
        resp.assert_status_ok();
        let article: ArticleResponse = resp.json();
        assert_eq!(article.id, article_id);
        assert_eq!(article.title, "mine");
    }

    #[tokio::test]
    async fn get_article_other_users_returns_404() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        db.seed_article(Article {
            id: article_id,
            title: "theirs".to_string(),
            url: "https://example.com/theirs".to_string(),
            ..make_article(other_id)
        });
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get(&format!("/me/articles/{article_id}")).await;
        resp.assert_status_not_found();
    }

    #[tokio::test]
    async fn get_article_missing_returns_404() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .get(&format!("/me/articles/{}", Uuid::new_v4()))
            .await;
        resp.assert_status_not_found();
    }

    #[tokio::test]
    async fn list_articles_oversized_limit_clamped() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_articles(&db, user_id, 150, None);
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/articles?limit=500").await;
        resp.assert_status_ok();
        let articles: Vec<ArticleResponse> = resp.json();
        assert_eq!(articles.len(), MAX_LIMIT as usize);
    }
}
