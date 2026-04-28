use axum::Json;
use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::{QuizWrongAnswer, SaveWrongAnswerParams};
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;

use super::AppState;

/// `Option<&str>` → `Option<Uuid>` 파싱 헬퍼 (tag_id 전용).
/// - `None` 또는 빈 문자열(`""`) → `None` (미전달과 동일 처리)
/// - 공백 포함 등 UUID로 파싱 불가 → `AppError::BadRequest`
/// - 유효한 UUID 문자열 → `Some(Uuid)`
fn parse_optional_tag_id(s: Option<&str>) -> Result<Option<Uuid>, AppError> {
    match s {
        Some(v) if !v.is_empty() => v
            .parse::<Uuid>()
            .map(Some)
            .map_err(|_| AppError::BadRequest("tag_id: invalid UUID format".to_string())),
        _ => Ok(None),
    }
}

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
    /// MVP13 M1: 오답이 발생한 퀴즈의 태그 ID.
    /// String으로 수신 후 핸들러에서 수동 파싱 (잘못된 형식 → 400 보장).
    pub tag_id: Option<String>,
}

/// GET /me/quiz/wrong-answers 쿼리 파라미터.
/// MVP13 M1: ?tag_id= 필터 파라미터 추가.
#[derive(Debug, Deserialize)]
pub struct WrongAnswersQuery {
    /// 태그 ID 필터. 없으면 전체 반환.
    /// String으로 수신 후 핸들러에서 수동 파싱 (잘못된 형식 → 400 보장).
    pub tag_id: Option<String>,
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

    // MVP13 M1: tag_id 문자열 → Uuid 파싱. 잘못된 형식 → 400
    let tag_id = parse_optional_tag_id(body.tag_id.as_deref())?;

    let params = SaveWrongAnswerParams {
        article_url: body.article_url.trim().to_string(),
        article_title: body.article_title.trim().to_string(),
        question: body.question.trim().to_string(),
        options: body.options,
        correct_index: body.correct_index,
        user_index: body.user_index,
        explanation: body.explanation,
        tag_id,
    };

    let record = state.quiz_wrong_answers.save(user.id, params).await?;

    Ok((StatusCode::CREATED, Json(record)))
}

/// GET /me/quiz/wrong-answers
/// 오답 목록 조회 → 200 + Vec<WrongAnswerResponse> (created_at DESC).
/// MVP13 M1: ?tag_id={uuid} 필터 지원. 잘못된 UUID 형식 → 400.
pub async fn list_wrong_answers<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<WrongAnswersQuery>,
) -> Result<Json<Vec<WrongAnswerResponse>>, AppError> {
    // MVP13 M1: ?tag_id= 파라미터 파싱 + 형식 검증
    let tag_id_filter = parse_optional_tag_id(query.tag_id.as_deref())?;

    let records = state
        .quiz_wrong_answers
        .list(user.id, tag_id_filter)
        .await?;
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
    /// MVP13 M1: 오답이 발생한 퀴즈의 태그 ID
    pub tag_id: Option<Uuid>,
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
            tag_id: r.tag_id, // MVP13 M1
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

    // ─── 기존 회귀 테스트 (G-01, G-02) ─────────────────────────────────────

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
        // MVP13 M1: tag_id 없으면 null
        assert!(body["tag_id"].is_null());
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

    // ─── MVP13 M1 신규 테스트 ───────────────────────────────────────────────

    /// T-01: tag_id 포함 오답 저장 후 응답에 tag_id 포함됨
    #[tokio::test]
    async fn post_wrong_answer_with_tag_id_stored_correctly() {
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        let mut body = valid_body();
        body["tag_id"] = serde_json::json!(tag_id.to_string());

        let resp = server.post("/me/quiz/wrong-answers").json(&body).await;

        resp.assert_status(StatusCode::CREATED);
        let result: serde_json::Value = resp.json();
        assert_eq!(result["tag_id"].as_str().unwrap(), tag_id.to_string());
    }

    /// R-02: POST body tag_id 잘못된 UUID 형식 → 400
    #[tokio::test]
    async fn post_wrong_answer_invalid_tag_id_returns_400() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        let mut body = valid_body();
        body["tag_id"] = serde_json::json!("not-a-valid-uuid");

        let resp = server.post("/me/quiz/wrong-answers").json(&body).await;

        resp.assert_status_bad_request();
    }

    /// T-02: ?tag_id= 필터 조회
    #[tokio::test]
    async fn get_wrong_answers_with_tag_id_filter() {
        let user_id = Uuid::new_v4();
        let tag_a = Uuid::new_v4();
        let tag_b = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        // tag_a 오답 저장
        let mut body_a = valid_body();
        body_a["tag_id"] = serde_json::json!(tag_a.to_string());
        server.post("/me/quiz/wrong-answers").json(&body_a).await;

        // tag_b 오답 저장
        let mut body_b = valid_body();
        body_b["question"] = serde_json::json!("두 번째 질문");
        body_b["tag_id"] = serde_json::json!(tag_b.to_string());
        server.post("/me/quiz/wrong-answers").json(&body_b).await;

        // tag_a만 필터
        let resp = server
            .get(&format!("/me/quiz/wrong-answers?tag_id={tag_a}"))
            .await;
        resp.assert_status_ok();
        let list: Vec<serde_json::Value> = resp.json();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0]["tag_id"].as_str().unwrap(), tag_a.to_string());
    }

    /// E-01: ?tag_id= 없을 때 전체 오답 반환
    #[tokio::test]
    async fn get_wrong_answers_without_filter_returns_all() {
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        // tag 있는 오답
        let mut body_with_tag = valid_body();
        body_with_tag["tag_id"] = serde_json::json!(tag_id.to_string());
        server
            .post("/me/quiz/wrong-answers")
            .json(&body_with_tag)
            .await;

        // tag 없는 오답
        let mut body_no_tag = valid_body();
        body_no_tag["question"] = serde_json::json!("tag 없는 질문");
        server
            .post("/me/quiz/wrong-answers")
            .json(&body_no_tag)
            .await;

        // 전체 조회 (필터 없음)
        let resp = server.get("/me/quiz/wrong-answers").await;
        resp.assert_status_ok();
        let list: Vec<serde_json::Value> = resp.json();
        assert_eq!(list.len(), 2);
    }

    /// E-03: ?tag_id= 필터 시 tag_id=NULL 행 제외
    #[tokio::test]
    async fn get_wrong_answers_with_tag_filter_excludes_null_tag_rows() {
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        // tag_id 없는 오답만 저장
        server
            .post("/me/quiz/wrong-answers")
            .json(&valid_body())
            .await;

        // tag_id로 필터하면 빈 결과
        let resp = server
            .get(&format!("/me/quiz/wrong-answers?tag_id={tag_id}"))
            .await;
        resp.assert_status_ok();
        let list: Vec<serde_json::Value> = resp.json();
        assert!(list.is_empty());
    }

    /// R-01: GET ?tag_id= 잘못된 UUID 형식 → 400
    #[tokio::test]
    async fn get_wrong_answers_invalid_tag_id_returns_400() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        let resp = server
            .get("/me/quiz/wrong-answers?tag_id=not-a-valid-uuid")
            .await;

        resp.assert_status_bad_request();
    }

    /// E-02: tag_id 미전달 시 NULL로 저장
    #[tokio::test]
    async fn post_wrong_answer_without_tag_id_stores_null() {
        let user_id = Uuid::new_v4();
        let server = TestServer::new(make_app(FakeQuizWrongAnswerAdapter::new(), user_id));

        let resp = server
            .post("/me/quiz/wrong-answers")
            .json(&valid_body())
            .await;

        resp.assert_status(StatusCode::CREATED);
        let body: serde_json::Value = resp.json();
        assert!(body["tag_id"].is_null());
    }
}
