use uuid::Uuid;

use super::error::AppError;
use super::models::{Profile, Tag, UserTag};

/// Supabase DB 접근 포트 (REST API)
pub trait DbPort: Send + Sync {
    fn get_profile(
        &self,
        user_id: Uuid,
        auth_token: &str,
    ) -> impl std::future::Future<Output = Result<Profile, AppError>> + Send;

    fn update_profile_onboarding(
        &self,
        user_id: Uuid,
        completed: bool,
        auth_token: &str,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    fn list_tags(
        &self,
        auth_token: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Tag>, AppError>> + Send;

    fn get_user_tags(
        &self,
        user_id: Uuid,
        auth_token: &str,
    ) -> impl std::future::Future<Output = Result<Vec<UserTag>, AppError>> + Send;

    fn set_user_tags(
        &self,
        user_id: Uuid,
        tag_ids: Vec<Uuid>,
        auth_token: &str,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}
