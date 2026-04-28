use std::future::Future;
use std::pin::Pin;

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::{QuizWrongAnswer, SaveWrongAnswerParams};
use crate::domain::ports::QuizWrongAnswerPort;

/// PostgreSQL 기반 QuizWrongAnswerAdapter.
/// quiz_wrong_answers 테이블에 sqlx로 직접 접근.
#[derive(Debug, Clone)]
pub struct PostgresQuizWrongAnswerAdapter {
    pool: PgPool,
}

impl PostgresQuizWrongAnswerAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl QuizWrongAnswerPort for PostgresQuizWrongAnswerAdapter {
    fn save<'a>(
        &'a self,
        user_id: Uuid,
        params: SaveWrongAnswerParams,
    ) -> Pin<Box<dyn Future<Output = Result<QuizWrongAnswer, AppError>> + Send + 'a>> {
        Box::pin(async move {
            let options_value = serde_json::to_value(&params.options)
                .map_err(|e| AppError::Internal(format!("options serialize failed: {e}")))?;

            // MVP13 M1: tag_id 포함 INSERT + ON CONFLICT DO UPDATE에 tag_id 명시
            sqlx::query_as::<_, QuizWrongAnswer>(
                r#"INSERT INTO quiz_wrong_answers
                   (user_id, article_url, article_title, question, options, correct_index, user_index, explanation, tag_id)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                   ON CONFLICT (user_id, article_url, question)
                   DO UPDATE SET
                     correct_index = EXCLUDED.correct_index,
                     user_index    = EXCLUDED.user_index,
                     explanation   = EXCLUDED.explanation,
                     options       = EXCLUDED.options,
                     tag_id        = EXCLUDED.tag_id
                   RETURNING *"#,
            )
            .bind(user_id)
            .bind(&params.article_url)
            .bind(&params.article_title)
            .bind(&params.question)
            .bind(&options_value)
            .bind(params.correct_index)
            .bind(params.user_index)
            .bind(&params.explanation)
            .bind(params.tag_id) // MVP13 M1
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("quiz_wrong_answers insert failed: {e}")))
        })
    }

    fn list<'a>(
        &'a self,
        user_id: Uuid,
        tag_id: Option<Uuid>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<QuizWrongAnswer>, AppError>> + Send + 'a>> {
        Box::pin(async move {
            // MVP13 M1: tag_id 필터 지원
            // tag_id = Some → 해당 태그만 반환 (NULL 행 제외)
            // tag_id = None → 전체 반환
            match tag_id {
                Some(tid) => sqlx::query_as::<_, QuizWrongAnswer>(
                    "SELECT * FROM quiz_wrong_answers WHERE user_id = $1 AND tag_id = $2 ORDER BY created_at DESC",
                )
                .bind(user_id)
                .bind(tid)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| AppError::Internal(format!("quiz_wrong_answers list failed: {e}"))),

                None => sqlx::query_as::<_, QuizWrongAnswer>(
                    "SELECT * FROM quiz_wrong_answers WHERE user_id = $1 ORDER BY created_at DESC",
                )
                .bind(user_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| AppError::Internal(format!("quiz_wrong_answers list failed: {e}"))),
            }
        })
    }

    fn delete<'a>(
        &'a self,
        user_id: Uuid,
        id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            sqlx::query("DELETE FROM quiz_wrong_answers WHERE id = $1 AND user_id = $2")
                .bind(id)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    AppError::Internal(format!("quiz_wrong_answers delete failed: {e}"))
                })?;

            Ok(())
        })
    }
}
