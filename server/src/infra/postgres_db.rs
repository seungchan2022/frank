use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::{Article, Profile, Tag, UserTag};
use crate::domain::ports::DbPort;

/// articles 테이블의 `SELECT` 컬럼 목록. `ArticleRow::FromRow`와 순서가 일치해야 한다.
const ARTICLE_COLUMNS: &str = "id, user_id, tag_id, title, url, snippet, source, search_query, summary, insight, summarized_at, published_at, created_at, title_ko, content, llm_model, prompt_tokens, completion_tokens";

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

#[derive(sqlx::FromRow)]
struct ArticleRow {
    id: Uuid,
    user_id: Uuid,
    tag_id: Option<Uuid>,
    title: String,
    url: String,
    snippet: Option<String>,
    source: String,
    search_query: Option<String>,
    summary: Option<String>,
    insight: Option<String>,
    summarized_at: Option<DateTime<Utc>>,
    published_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    title_ko: Option<String>,
    content: Option<String>,
    llm_model: Option<String>,
    prompt_tokens: Option<i32>,
    completion_tokens: Option<i32>,
}

impl From<ArticleRow> for Article {
    fn from(r: ArticleRow) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            tag_id: r.tag_id,
            title: r.title,
            url: r.url,
            snippet: r.snippet,
            source: r.source,
            search_query: r.search_query,
            summary: r.summary,
            insight: r.insight,
            summarized_at: r.summarized_at,
            published_at: r.published_at,
            created_at: r.created_at,
            title_ko: r.title_ko,
            content: r.content,
            llm_model: r.llm_model,
            prompt_tokens: r.prompt_tokens,
            completion_tokens: r.completion_tokens,
        }
    }
}

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

        // 동적 SET 절 빌드 — sqlx는 dynamic query에 약하므로 분기 처리
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

    async fn save_articles(&self, articles: Vec<Article>) -> Result<usize, AppError> {
        if articles.is_empty() {
            return Ok(0);
        }

        let count = articles.len();

        for article in &articles {
            sqlx::query(
                "INSERT INTO articles (id, user_id, tag_id, title, url, snippet, source, search_query, published_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 ON CONFLICT (user_id, url) DO NOTHING",
            )
            .bind(article.id)
            .bind(article.user_id)
            .bind(article.tag_id)
            .bind(&article.title)
            .bind(&article.url)
            .bind(&article.snippet)
            .bind(&article.source)
            .bind(&article.search_query)
            .bind(article.published_at)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("DB insert failed: {e}")))?;
        }

        Ok(count)
    }

    async fn get_user_articles(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
        tag_id: Option<Uuid>,
    ) -> Result<Vec<Article>, AppError> {
        // tag_id 필터 유무에 따라 쿼리 분기 (sqlx는 동적 WHERE를 직접 지원하지 않음)
        let rows = if let Some(tid) = tag_id {
            let sql = format!(
                "SELECT {ARTICLE_COLUMNS} FROM articles \
                 WHERE user_id = $1 AND tag_id = $2 \
                 AND tag_id IN (SELECT tag_id FROM user_tags WHERE user_id = $1) \
                 ORDER BY created_at DESC \
                 LIMIT $3 OFFSET $4"
            );
            sqlx::query_as::<_, ArticleRow>(&sql)
                .bind(user_id)
                .bind(tid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
        } else {
            let sql = format!(
                "SELECT {ARTICLE_COLUMNS} FROM articles \
                 WHERE user_id = $1 \
                 AND tag_id IN (SELECT tag_id FROM user_tags WHERE user_id = $1) \
                 ORDER BY created_at DESC \
                 LIMIT $2 OFFSET $3"
            );
            sqlx::query_as::<_, ArticleRow>(&sql)
                .bind(user_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
        }
        .map_err(|e| AppError::Internal(format!("DB query failed: {e}")))?;
        Ok(rows.into_iter().map(Article::from).collect())
    }

    async fn get_user_article_by_id(
        &self,
        user_id: Uuid,
        article_id: Uuid,
    ) -> Result<Option<Article>, AppError> {
        let sql = format!(
            "SELECT {ARTICLE_COLUMNS} FROM articles \
             WHERE id = $1 AND user_id = $2 \
             AND tag_id IN (SELECT tag_id FROM user_tags WHERE user_id = $2)"
        );
        let row = sqlx::query_as::<_, ArticleRow>(&sql)
            .bind(article_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("DB query failed: {e}")))?;
        Ok(row.map(Article::from))
    }

    async fn update_article_summary(
        &self,
        article_id: Uuid,
        summary: &str,
        insight: &str,
        title_ko: &str,
        llm_model: &str,
        prompt_tokens: i32,
        completion_tokens: i32,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            "UPDATE articles SET summary = $1, insight = $2, title_ko = $3, llm_model = $4, prompt_tokens = $5, completion_tokens = $6, summarized_at = now() WHERE id = $7",
        )
        .bind(summary)
        .bind(insight)
        .bind(title_ko)
        .bind(llm_model)
        .bind(prompt_tokens)
        .bind(completion_tokens)
        .bind(article_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("DB update failed: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Article not found".to_string()));
        }
        Ok(())
    }

    async fn update_article_content(
        &self,
        article_id: Uuid,
        content: &str,
    ) -> Result<(), AppError> {
        let result = sqlx::query("UPDATE articles SET content = $1 WHERE id = $2")
            .bind(content)
            .bind(article_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("DB update failed: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Article not found".to_string()));
        }
        Ok(())
    }
}
