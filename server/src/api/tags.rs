use axum::Json;
use axum::extract::Extension;
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::Tag;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;
use crate::services::tag_service;

use super::AppState;

pub async fn list_tags<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<Vec<Tag>>, AppError> {
    let tags = tag_service::list_tags(&state.db, &user.token).await?;
    Ok(Json(tags))
}

pub async fn get_my_tags<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<Vec<Uuid>>, AppError> {
    let tag_ids = tag_service::get_user_tag_ids(&state.db, user.id, &user.token).await?;
    Ok(Json(tag_ids))
}

#[derive(Debug, Deserialize)]
pub struct SaveTagsRequest {
    pub tag_ids: Vec<Uuid>,
}

pub async fn save_my_tags<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<SaveTagsRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    tag_service::save_user_tags(&state.db, user.id, body.tag_ids, &user.token).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn get_my_profile<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<crate::domain::models::Profile>, AppError> {
    let profile = state.db.get_profile(user.id, &user.token).await?;
    Ok(Json(profile))
}
