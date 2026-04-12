use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::{Profile, Tag, UserTag};
use crate::domain::ports::DbPort;

// --- infra-only row structs (sqlx::FromRow는 여기서만 사용) ---

#[derive(sqlx::FromRow)]
struct ProfileRow {
    id: Uuid,
    display_name: Option<String>,
    onboarding_completed: bool,
}

impl From<ProfileRow> for Profile {
    fn from(r: ProfileRow) -> Self {
        Self {
            id: r.id,
            display_name: r.display_name,
            onboarding_completed: r.onboarding_completed,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TagRow {
    id: Uuid,
    name: String,
    category: Option<String>,
}

impl From<TagRow> for Tag {
    fn from(r: TagRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            category: r.category,
        }
    }
}

#[derive(sqlx::FromRow)]
struct UserTagRow {
    user_id: Uuid,
    tag_id: Uuid,
}

impl From<UserTagRow> for UserTag {
    fn from(r: UserTagRow) -> Self {
        Self {
            user_id: r.user_id,
            tag_id: r.tag_id,
        }
    }
}

// MVP5 M3에서 favorites 엔드포인트 구현 시 FavoriteRow 추가 예정

// ---

#[derive(Debug, Clone)]
pub struct PostgresDbAdapter {
    pool: PgPool,
}

impl PostgresDbAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl DbPort for PostgresDbAdapter {
    async fn get_profile(&self, user_id: Uuid) -> Result<Profile, AppError> {
        sqlx::query_as::<_, ProfileRow>(
            "SELECT id, display_name, onboarding_completed FROM profiles WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("DB query failed: {e}")))?
        .map(Profile::from)
        .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))
    }

    async fn update_profile_onboarding(
        &self,
        user_id: Uuid,
        completed: bool,
    ) -> Result<(), AppError> {
        // profiles row가 없는 경우(트리거 누락 등)를 대비해 UPSERT 처리
        sqlx::query(
            "INSERT INTO profiles (id, onboarding_completed)
             VALUES ($1, $2)
             ON CONFLICT (id) DO UPDATE SET onboarding_completed = EXCLUDED.onboarding_completed",
        )
        .bind(user_id)
        .bind(completed)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("DB upsert failed: {e}")))?;
        Ok(())
    }

    async fn update_profile(
        &self,
        user_id: Uuid,
        onboarding_completed: Option<bool>,
        display_name: Option<String>,
    ) -> Result<Profile, AppError> {
        // 두 필드 모두 None이면 현재 프로필을 반환 (no-op)
        if onboarding_completed.is_none() && display_name.is_none() {
            return self.get_profile(user_id).await;
        }

        // 동적 SET 절 빌드
        let row = match (onboarding_completed, display_name) {
            (Some(oc), Some(dn)) => {
                sqlx::query_as::<_, ProfileRow>(
                    "UPDATE profiles SET onboarding_completed = $1, display_name = $2
                 WHERE id = $3
                 RETURNING id, display_name, onboarding_completed",
                )
                .bind(oc)
                .bind(dn)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await
            }
            (Some(oc), None) => {
                sqlx::query_as::<_, ProfileRow>(
                    "UPDATE profiles SET onboarding_completed = $1
                 WHERE id = $2
                 RETURNING id, display_name, onboarding_completed",
                )
                .bind(oc)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await
            }
            (None, Some(dn)) => {
                sqlx::query_as::<_, ProfileRow>(
                    "UPDATE profiles SET display_name = $1
                 WHERE id = $2
                 RETURNING id, display_name, onboarding_completed",
                )
                .bind(dn)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await
            }
            (None, None) => unreachable!(),
        }
        .map_err(|e| AppError::Internal(format!("DB update failed: {e}")))?;

        row.map(Profile::from)
            .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))
    }

    async fn list_tags(&self) -> Result<Vec<Tag>, AppError> {
        let rows = sqlx::query_as::<_, TagRow>(
            "SELECT id, name, category FROM tags ORDER BY category, name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("DB query failed: {e}")))?;
        Ok(rows.into_iter().map(Tag::from).collect())
    }

    async fn get_user_tags(&self, user_id: Uuid) -> Result<Vec<UserTag>, AppError> {
        let rows = sqlx::query_as::<_, UserTagRow>(
            "SELECT user_id, tag_id FROM user_tags WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("DB query failed: {e}")))?;
        Ok(rows.into_iter().map(UserTag::from).collect())
    }

    async fn set_user_tags(&self, user_id: Uuid, tag_ids: Vec<Uuid>) -> Result<(), AppError> {
        // 트랜잭션으로 원자성 확보 (DELETE + INSERT)
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(format!("DB transaction begin failed: {e}")))?;

        // 기존 태그 삭제
        sqlx::query("DELETE FROM user_tags WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(format!("DB delete failed: {e}")))?;

        // 새 태그 삽입
        for tag_id in &tag_ids {
            sqlx::query("INSERT INTO user_tags (user_id, tag_id) VALUES ($1, $2)")
                .bind(user_id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Internal(format!("DB insert failed: {e}")))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(format!("DB transaction commit failed: {e}")))?;

        Ok(())
    }

    async fn increment_keyword_weights(
        &self,
        user_id: Uuid,
        keywords: Vec<String>,
    ) -> Result<(), AppError> {
        for keyword in &keywords {
            sqlx::query(
                "INSERT INTO user_keyword_weights (user_id, keyword, weight, updated_at)
                 VALUES ($1, $2, 1, NOW())
                 ON CONFLICT (user_id, keyword) DO UPDATE
                   SET weight = user_keyword_weights.weight + 1, updated_at = NOW()",
            )
            .bind(user_id)
            .bind(keyword)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("DB keyword weight upsert failed: {e}")))?;
        }
        Ok(())
    }

    async fn get_top_keywords(&self, user_id: Uuid, limit: u32) -> Result<Vec<String>, AppError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT keyword FROM user_keyword_weights
             WHERE user_id = $1
             ORDER BY weight DESC, updated_at DESC, keyword ASC
             LIMIT $2",
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("DB get_top_keywords failed: {e}")))?;

        Ok(rows.into_iter().map(|(k,)| k).collect())
    }

    async fn increment_like_count(&self, user_id: Uuid) -> Result<i32, AppError> {
        let row: Option<(i32,)> = sqlx::query_as(
            "UPDATE profiles SET like_count = like_count + 1 WHERE id = $1 RETURNING like_count",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("DB increment_like_count failed: {e}")))?;

        row.map(|(c,)| c)
            .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))
    }

    async fn get_like_count(&self, user_id: Uuid) -> Result<i32, AppError> {
        let row: Option<(i32,)> = sqlx::query_as("SELECT like_count FROM profiles WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("DB get_like_count failed: {e}")))?;

        Ok(row.map(|(c,)| c).unwrap_or(0))
    }
}
