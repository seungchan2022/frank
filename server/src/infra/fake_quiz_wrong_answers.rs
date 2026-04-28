use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::{QuizWrongAnswer, SaveWrongAnswerParams};
use crate::domain::ports::QuizWrongAnswerPort;

/// (user_id, article_url, question) → id 중복 키 인덱스.
type ConflictIndex = Arc<Mutex<HashMap<(Uuid, String, String), Uuid>>>;

/// 테스트용 인메모리 QuizWrongAnswerAdapter.
/// 키: id → QuizWrongAnswer.
/// 삽입 순서 추적을 위해 별도 Vec 보관.
#[derive(Debug, Clone)]
pub struct FakeQuizWrongAnswerAdapter {
    /// id → QuizWrongAnswer
    store: Arc<Mutex<HashMap<Uuid, QuizWrongAnswer>>>,
    /// (user_id, article_url, question) → id (중복 키 → 덮어쓰기)
    index: ConflictIndex,
    /// 삽입 순서 (DESC 정렬 모사)
    order: Arc<Mutex<Vec<Uuid>>>,
    should_fail: bool,
}

impl FakeQuizWrongAnswerAdapter {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            index: Arc::new(Mutex::new(HashMap::new())),
            order: Arc::new(Mutex::new(Vec::new())),
            should_fail: false,
        }
    }

    pub fn failing() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            index: Arc::new(Mutex::new(HashMap::new())),
            order: Arc::new(Mutex::new(Vec::new())),
            should_fail: true,
        }
    }
}

impl Default for FakeQuizWrongAnswerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl QuizWrongAnswerPort for FakeQuizWrongAnswerAdapter {
    fn save<'a>(
        &'a self,
        user_id: Uuid,
        params: SaveWrongAnswerParams,
    ) -> Pin<Box<dyn Future<Output = Result<QuizWrongAnswer, AppError>> + Send + 'a>> {
        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal(
                    "Fake quiz wrong answer failure".to_string(),
                ));
            }

            let options_value = serde_json::to_value(&params.options)
                .map_err(|e| AppError::Internal(format!("options serialize failed: {e}")))?;

            let conflict_key = (user_id, params.article_url.clone(), params.question.clone());

            let mut store = self.store.lock().unwrap();
            let mut index = self.index.lock().unwrap();
            let mut order = self.order.lock().unwrap();

            if let Some(existing_id) = index.get(&conflict_key).copied() {
                // 덮어쓰기 (ON CONFLICT DO UPDATE)
                let entry = store.get_mut(&existing_id).unwrap();
                entry.correct_index = params.correct_index;
                entry.user_index = params.user_index;
                entry.explanation = params.explanation;
                entry.options = options_value;
                entry.tag_id = params.tag_id; // MVP13 M1
                Ok(entry.clone())
            } else {
                let id = Uuid::new_v4();
                let record = QuizWrongAnswer {
                    id,
                    user_id,
                    article_url: params.article_url,
                    article_title: params.article_title,
                    question: params.question,
                    options: options_value,
                    correct_index: params.correct_index,
                    user_index: params.user_index,
                    explanation: params.explanation,
                    created_at: Utc::now(),
                    tag_id: params.tag_id, // MVP13 M1
                };

                index.insert(conflict_key, id);
                order.push(id);
                store.insert(id, record.clone());
                Ok(record)
            }
        })
    }

    fn list<'a>(
        &'a self,
        user_id: Uuid,
        tag_id: Option<Uuid>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<QuizWrongAnswer>, AppError>> + Send + 'a>> {
        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal(
                    "Fake quiz wrong answer failure".to_string(),
                ));
            }

            let store = self.store.lock().unwrap();
            let order = self.order.lock().unwrap();

            // 삽입 역순 (created_at DESC 모사), 본인 데이터만
            // MVP13 M1: tag_id 필터 — Some이면 해당 태그만 (NULL 행 제외), None이면 전체
            let mut items: Vec<QuizWrongAnswer> = order
                .iter()
                .rev()
                .filter_map(|id| store.get(id))
                .filter(|r| r.user_id == user_id)
                .filter(|r| match tag_id {
                    Some(tid) => r.tag_id == Some(tid),
                    None => true,
                })
                .cloned()
                .collect();
            // created_at DESC 정렬 (역순 삽입 순서로 충분하지만 명시적 정렬)
            items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            Ok(items)
        })
    }

    fn delete<'a>(
        &'a self,
        user_id: Uuid,
        id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal(
                    "Fake quiz wrong answer failure".to_string(),
                ));
            }

            let mut store = self.store.lock().unwrap();
            let mut index = self.index.lock().unwrap();
            let mut order = self.order.lock().unwrap();

            // 본인 데이터만 삭제 (타인 데이터는 no-op)
            if store.get(&id).is_some_and(|r| r.user_id != user_id) {
                return Ok(());
            }

            if let Some(record) = store.remove(&id) {
                let conflict_key = (record.user_id, record.article_url, record.question);
                index.remove(&conflict_key);
                order.retain(|oid| *oid != id);
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_params(article_url: &str, question: &str) -> SaveWrongAnswerParams {
        SaveWrongAnswerParams {
            article_url: article_url.to_string(),
            article_title: "테스트 기사".to_string(),
            question: question.to_string(),
            options: vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
            ],
            correct_index: 0,
            user_index: 1,
            explanation: Some("해설".to_string()),
            tag_id: None,
        }
    }

    fn make_params_with_tag(
        article_url: &str,
        question: &str,
        tag_id: Uuid,
    ) -> SaveWrongAnswerParams {
        SaveWrongAnswerParams {
            tag_id: Some(tag_id),
            ..make_params(article_url, question)
        }
    }

    #[tokio::test]
    async fn save_returns_wrong_answer_with_id() {
        let adapter = FakeQuizWrongAnswerAdapter::new();
        let user_id = Uuid::new_v4();

        let result = adapter
            .save(user_id, make_params("https://example.com", "질문1"))
            .await
            .unwrap();

        assert_eq!(result.user_id, user_id);
        assert_eq!(result.article_url, "https://example.com");
        assert_eq!(result.question, "질문1");
        assert_eq!(result.correct_index, 0);
        assert_eq!(result.user_index, 1);
        assert_eq!(result.tag_id, None);
    }

    #[tokio::test]
    async fn save_with_tag_id_stores_tag() {
        let adapter = FakeQuizWrongAnswerAdapter::new();
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        let result = adapter
            .save(
                user_id,
                make_params_with_tag("https://example.com", "질문1", tag_id),
            )
            .await
            .unwrap();

        assert_eq!(result.tag_id, Some(tag_id));
    }

    #[tokio::test]
    async fn save_duplicate_overwrites_including_tag_id() {
        let adapter = FakeQuizWrongAnswerAdapter::new();
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        let first = adapter
            .save(user_id, make_params("https://example.com", "질문1"))
            .await
            .unwrap();

        let params2 = make_params_with_tag("https://example.com", "질문1", tag_id);
        let second = adapter.save(user_id, params2).await.unwrap();

        // 같은 id (덮어쓰기)
        assert_eq!(first.id, second.id);
        assert_eq!(second.tag_id, Some(tag_id));

        // list도 1개만
        let list = adapter.list(user_id, None).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn list_returns_own_items_desc_order() {
        let adapter = FakeQuizWrongAnswerAdapter::new();
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();

        adapter
            .save(user_id, make_params("https://a.com", "Q1"))
            .await
            .unwrap();
        adapter
            .save(user_id, make_params("https://b.com", "Q2"))
            .await
            .unwrap();
        adapter
            .save(other_id, make_params("https://c.com", "Q3"))
            .await
            .unwrap();

        let list = adapter.list(user_id, None).await.unwrap();
        assert_eq!(list.len(), 2);
        // 타인 데이터 미포함
        assert!(list.iter().all(|r| r.user_id == user_id));
    }

    #[tokio::test]
    async fn list_with_tag_id_filters_correctly() {
        let adapter = FakeQuizWrongAnswerAdapter::new();
        let user_id = Uuid::new_v4();
        let tag_a = Uuid::new_v4();
        let tag_b = Uuid::new_v4();

        adapter
            .save(user_id, make_params_with_tag("https://a.com", "Q1", tag_a))
            .await
            .unwrap();
        adapter
            .save(user_id, make_params_with_tag("https://b.com", "Q2", tag_b))
            .await
            .unwrap();
        // tag_id 없는 오답
        adapter
            .save(user_id, make_params("https://c.com", "Q3"))
            .await
            .unwrap();

        // tag_a만 필터
        let list_a = adapter.list(user_id, Some(tag_a)).await.unwrap();
        assert_eq!(list_a.len(), 1);
        assert_eq!(list_a[0].tag_id, Some(tag_a));

        // tag_b만 필터
        let list_b = adapter.list(user_id, Some(tag_b)).await.unwrap();
        assert_eq!(list_b.len(), 1);
        assert_eq!(list_b[0].tag_id, Some(tag_b));

        // 전체 조회 (None)
        let list_all = adapter.list(user_id, None).await.unwrap();
        assert_eq!(list_all.len(), 3);
    }

    #[tokio::test]
    async fn list_with_same_tag_id_returns_all_matching_rows() {
        let adapter = FakeQuizWrongAnswerAdapter::new();
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        // 동일 tag_id로 오답 2개 저장 (다른 question → 중복 키 아님)
        adapter
            .save(user_id, make_params_with_tag("https://a.com", "Q1", tag_id))
            .await
            .unwrap();
        adapter
            .save(user_id, make_params_with_tag("https://b.com", "Q2", tag_id))
            .await
            .unwrap();

        let list = adapter.list(user_id, Some(tag_id)).await.unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.iter().all(|r| r.tag_id == Some(tag_id)));
    }

    #[tokio::test]
    async fn list_with_tag_id_excludes_null_rows() {
        let adapter = FakeQuizWrongAnswerAdapter::new();
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        // tag_id 없는 오답만 저장
        adapter
            .save(user_id, make_params("https://a.com", "Q1"))
            .await
            .unwrap();

        // tag_id로 필터하면 빈 결과 (NULL 행 제외)
        let list = adapter.list(user_id, Some(tag_id)).await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn delete_removes_own_item() {
        let adapter = FakeQuizWrongAnswerAdapter::new();
        let user_id = Uuid::new_v4();

        let record = adapter
            .save(user_id, make_params("https://example.com", "Q1"))
            .await
            .unwrap();

        adapter.delete(user_id, record.id).await.unwrap();

        let list = adapter.list(user_id, None).await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn delete_nonexistent_returns_ok() {
        let adapter = FakeQuizWrongAnswerAdapter::new();
        let user_id = Uuid::new_v4();

        let result = adapter.delete(user_id, Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_other_users_item_is_noop() {
        let adapter = FakeQuizWrongAnswerAdapter::new();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        let record = adapter
            .save(user1, make_params("https://example.com", "Q1"))
            .await
            .unwrap();

        // user2가 user1의 오답 삭제 시도
        adapter.delete(user2, record.id).await.unwrap();

        // user1의 데이터는 그대로
        let list = adapter.list(user1, None).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn failing_save_returns_error() {
        let adapter = FakeQuizWrongAnswerAdapter::failing();
        let user_id = Uuid::new_v4();

        let result = adapter
            .save(user_id, make_params("https://example.com", "Q1"))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn failing_list_returns_error() {
        let adapter = FakeQuizWrongAnswerAdapter::failing();
        let result = adapter.list(Uuid::new_v4(), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn failing_delete_returns_error() {
        let adapter = FakeQuizWrongAnswerAdapter::failing();
        let result = adapter.delete(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(result.is_err());
    }
}
