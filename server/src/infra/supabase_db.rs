use reqwest::Client;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::domain::error::AppError;
use crate::domain::models::{Profile, Tag, UserTag};
use crate::domain::ports::DbPort;

#[derive(Debug, Clone)]
pub struct SupabaseDbAdapter {
    client: Client,
    base_url: String,
    anon_key: String,
}

impl SupabaseDbAdapter {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: format!("{}/rest/v1", config.supabase_url),
            anon_key: config.supabase_anon_key.clone(),
        }
    }

    fn rest_request(&self, path: &str, auth_token: &str) -> reqwest::RequestBuilder {
        self.client
            .get(format!("{}{}", self.base_url, path))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {auth_token}"))
    }
}

impl DbPort for SupabaseDbAdapter {
    async fn get_profile(&self, user_id: Uuid, auth_token: &str) -> Result<Profile, AppError> {
        let resp = self
            .rest_request(
                &format!("/profiles?id=eq.{user_id}&select=id,display_name,onboarding_completed"),
                auth_token,
            )
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("DB request failed: {e}")))?;

        let profiles: Vec<Profile> = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("DB parse failed: {e}")))?;

        profiles
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))
    }

    async fn update_profile_onboarding(
        &self,
        user_id: Uuid,
        completed: bool,
        auth_token: &str,
    ) -> Result<(), AppError> {
        let resp = self
            .client
            .patch(format!("{}/profiles?id=eq.{user_id}", self.base_url))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {auth_token}"))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&serde_json::json!({ "onboarding_completed": completed }))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("DB request failed: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("DB update failed: {body}")));
        }
        Ok(())
    }

    async fn list_tags(&self, auth_token: &str) -> Result<Vec<Tag>, AppError> {
        let resp = self
            .rest_request(
                "/tags?select=id,name,category&order=category,name",
                auth_token,
            )
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("DB request failed: {e}")))?;

        resp.json()
            .await
            .map_err(|e| AppError::Internal(format!("DB parse failed: {e}")))
    }

    async fn get_user_tags(
        &self,
        user_id: Uuid,
        auth_token: &str,
    ) -> Result<Vec<UserTag>, AppError> {
        let resp = self
            .rest_request(
                &format!("/user_tags?user_id=eq.{user_id}&select=user_id,tag_id"),
                auth_token,
            )
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("DB request failed: {e}")))?;

        resp.json()
            .await
            .map_err(|e| AppError::Internal(format!("DB parse failed: {e}")))
    }

    async fn set_user_tags(
        &self,
        user_id: Uuid,
        tag_ids: Vec<Uuid>,
        auth_token: &str,
    ) -> Result<(), AppError> {
        // 기존 태그 삭제
        let del_resp = self
            .client
            .delete(format!("{}/user_tags?user_id=eq.{user_id}", self.base_url))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {auth_token}"))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("DB delete failed: {e}")))?;

        if !del_resp.status().is_success() {
            let body = del_resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("DB delete failed: {body}")));
        }

        // 새 태그 삽입
        if !tag_ids.is_empty() {
            let rows: Vec<serde_json::Value> = tag_ids
                .iter()
                .map(|tid| serde_json::json!({ "user_id": user_id, "tag_id": tid }))
                .collect();

            let ins_resp = self
                .client
                .post(format!("{}/user_tags", self.base_url))
                .header("apikey", &self.anon_key)
                .header("Authorization", format!("Bearer {auth_token}"))
                .header("Content-Type", "application/json")
                .header("Prefer", "return=minimal")
                .json(&rows)
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("DB insert failed: {e}")))?;

            if !ins_resp.status().is_success() {
                let body = ins_resp.text().await.unwrap_or_default();
                return Err(AppError::Internal(format!("DB insert failed: {body}")));
            }
        }

        Ok(())
    }
}
