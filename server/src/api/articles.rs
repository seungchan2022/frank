use axum::Json;
use axum::extract::Extension;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;
use crate::services::{collect_service, summary_service};

use super::AppState;

pub async fn collect_articles<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    let count = collect_service::collect_for_user(&state.db, &state.search_chain, user.id).await?;
    Ok(Json(serde_json::json!({ "collected": count })))
}

#[derive(Debug, Deserialize)]
pub struct ListArticlesQuery {
    pub limit: Option<i64>,
}

pub async fn list_articles<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    query: axum::extract::Query<ListArticlesQuery>,
) -> Result<Json<Vec<crate::domain::models::Article>>, AppError> {
    let limit = query.limit.unwrap_or(50);
    let articles = state.db.get_user_articles(user.id, limit).await?;
    Ok(Json(articles))
}

pub async fn summarize_articles<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    let count = summary_service::summarize_articles(&state.db, state.llm.as_ref(), user.id).await?;
    Ok(Json(serde_json::json!({ "summarized": count })))
}
