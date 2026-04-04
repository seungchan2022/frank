use reqwest::Client;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::domain::error::AppError;
use crate::domain::models::{Article, Profile, Tag, UserTag};
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

    async fn save_articles(
        &self,
        articles: Vec<Article>,
        auth_token: &str,
    ) -> Result<usize, AppError> {
        if articles.is_empty() {
            return Ok(0);
        }

        let rows: Vec<serde_json::Value> = articles
            .iter()
            .map(|a| {
                serde_json::json!({
                    "user_id": a.user_id,
                    "tag_id": a.tag_id,
                    "title": a.title,
                    "url": a.url,
                    "snippet": a.snippet,
                    "source": a.source,
                    "search_query": a.search_query,
                    "published_at": a.published_at,
                })
            })
            .collect();

        let count = rows.len();

        let resp = self
            .client
            .post(format!("{}/articles?on_conflict=user_id,url", self.base_url))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {auth_token}"))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal,resolution=ignore-duplicates")
            .json(&rows)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("DB insert failed: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("DB insert failed: {body}")));
        }

        Ok(count)
    }

    async fn get_user_articles(
        &self,
        user_id: Uuid,
        limit: i64,
        auth_token: &str,
    ) -> Result<Vec<Article>, AppError> {
        let resp = self
            .rest_request(
                &format!(
                    "/articles?user_id=eq.{user_id}&select=id,user_id,tag_id,title,url,snippet,source,search_query,summary,insight,summarized_at,published_at,created_at&order=created_at.desc&limit={limit}"
                ),
                auth_token,
            )
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("DB request failed: {e}")))?;

        resp.json()
            .await
            .map_err(|e| AppError::Internal(format!("DB parse failed: {e}")))
    }

    async fn update_article_summary(
        &self,
        article_id: Uuid,
        summary: &str,
        insight: &str,
        auth_token: &str,
    ) -> Result<(), AppError> {
        let resp = self
            .client
            .patch(format!("{}/articles?id=eq.{article_id}", self.base_url))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {auth_token}"))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&serde_json::json!({
                "summary": summary,
                "insight": insight,
                "summarized_at": chrono::Utc::now().to_rfc3339(),
            }))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("DB request failed: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "DB update summary failed: {body}"
            )));
        }
        Ok(())
    }
}
