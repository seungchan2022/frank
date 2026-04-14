use axum::Json;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::{QuizWrongAnswer, SaveWrongAnswerParams};
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;

use super::AppState;

/// POST /me/quiz/wrong-answers 요청 바디.
#[derive(Debug, Deserialize)]
pub struct SaveWrongAnswerRequest {
    pub article_url: String,
    pub article_title: String,
    pub question: String,
    pub options: Vec<String>,
    pub correct_index: i32,
    pub user_index: i32,
    pub explanation: Option<String>,
}

/// POST /me/quiz/wrong-answers
/// 오답 1건 저장 (중복 시 덮어쓰기) → 201 + QuizWrongAnswer JSON.
pub async fn save_wrong_answer<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<SaveWrongAnswerRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.article_url.trim().is_empty() {
        return Err(AppError::BadRequest(
            "article_url must not be empty".to_string(),
        ));
    }
    if body.question.trim().is_empty() {
        return Err(AppError::BadRequest(
            "question must not be empty".to_string(),
        ));
    }
    if body.options.is_empty() {
        return Err(AppError::BadRequest(
            "options must not be empty".to_string(),
        ));
    }

    let params = SaveWrongAnswerParams {
        article_url: body.article_url.trim().to_string(),
        article_title: body.article_title.trim().to_string(),
        question: body.question.trim().to_string(),
        options: body.options,
        correct_index: body.correct_index,
        user_index: body.user_index,
        explanation: body.explanation,
    };

    let record = state.quiz_wrong_answers.save(user.id, params).await?;

    Ok((StatusCode::CREATED, Json(record)))
}

/// GET /me/quiz/wrong-answers
/// 오답 목록 조회 → 200 + Vec<WrongAnswerResponse> (created_at DESC).
/// options를 Vec<String>으로 변환하여 클라이언트에 전달.
pub async fn list_wrong_answers<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<Vec<WrongAnswerResponse>>, AppError> {
    let records = state.quiz_wrong_answers.list(user.id).await?;
    let response: Result<Vec<WrongAnswerResponse>, AppError> = records
        .into_iter()
        .map(WrongAnswerResponse::try_from)
        .collect();
    Ok(Json(response?))
}

/// DELETE /me/quiz/wrong-answers/{id}
/// 오답 1건 삭제 → 204 No Content.
/// 존재하지 않는 id여도 204 (no-op).
pub async fn delete_wrong_answer<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.quiz_wrong_answers.delete(user.id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /me/quiz/wrong-answers 응답용 직렬화 구조체 (QuizWrongAnswer 재사용).
/// options를 Vec<String>으로 변환하여 클라이언트에 전달.
#[derive(Debug, Serialize)]
pub struct WrongAnswerResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub article_url: String,
    pub article_title: String,
    pub question: String,
    pub options: Vec<String>,
    pub correct_index: i32,
    pub user_index: i32,
    pub explanation: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<QuizWrongAnswer> for WrongAnswerResponse {
    type Error = AppError;

    fn try_from(r: QuizWrongAnswer) -> Result<Self, Self::Error> {
        let options: Vec<String> = serde_json::from_value(r.options)
            .map_err(|e| AppError::Internal(format!("options deserialize failed: {e}")))?;
        Ok(Self {
            id: r.id,
            user_id: r.user_id,
            article_url: r.article_url,
            article_title: r.article_title,
            question: r.question,
            options,
            correct_index: r.correct_index,
            user_index: r.user_index,
            explanation: r.explanation,
            created_at: r.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_favorites::FakeFavoritesAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_quiz_wrong_answers::FakeQuizWrongAnswerAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::feed_cache::NoopFeedCache;
    use crate::infra::search_chain::SearchFallbackChain;
    use axum::Router;
    use axum::routing::{delete, post};
    use axum_test::TestServer;
    use std::sync::Arc;

    fn make_test_state(qwa: FakeQuizWrongAnswerAdapter) -> AppState<FakeDbAdapter> {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            vec![],
            false,
        ))]);
        AppState {
            db: FakeDbAdapter::new(),
            search_chain: Arc::new(chain),
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(qwa),
            feed_cache: Arc::new(NoopFeedCache),
        }
    }

    fn make_app(qwa: FakeQuizWrongAnswerAdapter, user_id: Uuid) -> Router {
        let state = make_test_state(qwa);
        Router::new()
            .route(
                "/me/quiz/wrong-answers",
                post(save_wrong_answer::<FakeDbAdapter>).get(list_wrong_answers::<FakeDbAdapter>),
            )
            .route(
                "/me/quiz/wrong-answers/{id}",
                delete(delete_wrong_answer::<FakeDbAdapter>),
            )
            .layer(Extension(state))
            .layer(Extension(AuthUser { id: user_id }))
    }

    fn valid_body() -> serde_json::Value {
        serde_json::json!({
            "article_url": "https://example.com/article",
            "article_title": "테스트 기사",
            "question": "이 기사의 핵심 내용은?",
            "options": ["A", "B", "C", "D"],
            "correct_index": 0,
            "user_index": 1,
            "explanation": "정답은 A입니다."
        })
    }

    #[tokio::test]
    async fn post_wrong_answer_returns_201() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        let resp = server
            .post("/me/quiz/wrong-answers")
            .json(&valid_body())
            .await;

        resp.assert_status(StatusCode::CREATED);
        let body: serde_json::Value = resp.json();
        assert_eq!(body["article_url"], "https://example.com/article");
        assert_eq!(body["correct_index"], 0);
        assert_eq!(body["user_index"], 1);
    }

    #[tokio::test]
    async fn post_wrong_answer_empty_url_returns_400() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        let mut body = valid_body();
        body["article_url"] = serde_json::json!("");

        let resp = server.post("/me/quiz/wrong-answers").json(&body).await;

        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn post_wrong_answer_empty_question_returns_400() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        let mut body = valid_body();
        body["question"] = serde_json::json!("");

        let resp = server.post("/me/quiz/wrong-answers").json(&body).await;

        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn post_wrong_answer_empty_options_returns_400() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        let mut body = valid_body();
        body["options"] = serde_json::json!([]);

        let resp = server.post("/me/quiz/wrong-answers").json(&body).await;

        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn get_wrong_answers_returns_200_with_list() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        // 2개 저장
        server
            .post("/me/quiz/wrong-answers")
            .json(&valid_body())
            .await;

        let mut body2 = valid_body();
        body2["question"] = serde_json::json!("두 번째 질문");
        server.post("/me/quiz/wrong-answers").json(&body2).await;

        let resp = server.get("/me/quiz/wrong-answers").await;

        resp.assert_status_ok();
        let list: Vec<serde_json::Value> = resp.json();
        assert_eq!(list.len(), 2);
        // options가 Vec<String>으로 직렬화되는지 확인 (WrongAnswerResponse 변환 검증)
        assert!(list[0]["options"].is_array());
        assert_eq!(list[0]["options"][0].as_str().unwrap(), "A");
    }

    #[tokio::test]
    async fn delete_wrong_answer_returns_204() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        let resp = server
            .post("/me/quiz/wrong-answers")
            .json(&valid_body())
            .await;
        let body: serde_json::Value = resp.json();
        let id = body["id"].as_str().unwrap();

        let del_resp = server.delete(&format!("/me/quiz/wrong-answers/{id}")).await;

        del_resp.assert_status(StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn delete_nonexistent_wrong_answer_returns_204() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        let del_resp = server
            .delete(&format!("/me/quiz/wrong-answers/{}", Uuid::new_v4()))
            .await;

        del_resp.assert_status(StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn duplicate_wrong_answer_overwrites() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        // 같은 url + question 두 번 저장
        server
            .post("/me/quiz/wrong-answers")
            .json(&valid_body())
            .await;
        server
            .post("/me/quiz/wrong-answers")
            .json(&valid_body())
            .await;

        let resp = server.get("/me/quiz/wrong-answers").await;
        let list: Vec<serde_json::Value> = resp.json();
        // 중복은 1개만
        assert_eq!(list.len(), 1);
    }
}
