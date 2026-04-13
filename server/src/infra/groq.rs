use std::future::Future;
use std::pin::Pin;

use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::{LlmResponse, LlmSummary, QuizConcept, QuizQuestion, QuizResult};
use crate::domain::ports::LlmPort;
use crate::infra::http_utils::{RetryConfig, read_body_limited, send_with_retry};

const GROQ_MODEL: &str = "llama-3.3-70b-versatile";
const MAX_KEYWORD_LEN: usize = 100;

const SYSTEM_PROMPT: &str = r#"You are a news analyst for a Korean-speaking audience. Given an article title and content, provide a JSON response with exactly three fields:
1. "title_ko": 한국어로 번역한 기사 제목 (원문의 핵심을 살리되 한국 독자가 바로 이해할 수 있는 자연스러운 표현)
2. "summary": 기사 핵심 내용을 한국어로 쉽게 풀어서 설명 (전문 용어는 괄호 안에 원문 병기, 3-5문장). Use limited Markdown in the string value: **bold** for key terms, *italic* for emphasis, - for bullet lists, blank lines between paragraphs.
3. "insight": 이 기사가 왜 중요한지, 독자에게 어떤 의미인지 한국어로 분석 (2-3문장). Use limited Markdown in the string value: **bold** for key terms, *italic* for emphasis.

Respond ONLY with valid JSON. Do NOT use Markdown outside of the string values (no code blocks, no headings, no tables)."#;

#[derive(Debug, Clone)]
pub struct GroqAdapter {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    model: Option<String>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: Option<i32>,
    completion_tokens: Option<i32>,
}

impl GroqAdapter {
    pub fn new(api_key: &str) -> Self {
        Self::with_base_url(api_key, "https://api.groq.com/openai/v1")
    }

    pub fn with_base_url(api_key: &str, base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("Failed to build HTTP client"),
            api_key: api_key.to_string(),
            model: GROQ_MODEL.to_string(),
            base_url: base_url.to_string(),
        }
    }

    /// Groq Chat Completions API 공통 호출 helper.
    /// body를 POST하고 응답 텍스트를 반환한다. 상태코드 에러도 여기서 처리.
    async fn call_chat_api(
        &self,
        body: serde_json::Value,
        ctx: &'static str,
    ) -> Result<String, AppError> {
        let config = RetryConfig::for_llm();
        let api_key = self.api_key.clone();
        let url = format!("{}/chat/completions", self.base_url);

        let resp = send_with_retry(
            || {
                let url = url.clone();
                let body = body.clone();
                let api_key = api_key.clone();
                let client = self.client.clone();
                async move {
                    client
                        .post(&url)
                        .header("Authorization", format!("Bearer {api_key}"))
                        .header("Content-Type", "application/json")
                        .json(&body)
                }
            },
            &config,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Groq {ctx} request failed: {e}")))?;

        let status = resp.status();
        let body_text = read_body_limited(resp, config.max_response_size)
            .await
            .map_err(|e| AppError::Internal(format!("Groq {ctx} read failed: {e}")))?;

        if !status.is_success() {
            return Err(AppError::Internal(format!(
                "Groq API error [{ctx}] ({status}): {body_text}"
            )));
        }

        Ok(body_text)
    }

    /// ChatResponse 파싱 공통 helper
    fn parse_chat_response(body_text: &str, ctx: &'static str) -> Result<ChatResponse, AppError> {
        serde_json::from_str(body_text)
            .map_err(|e| AppError::Internal(format!("Groq {ctx} parse failed: {e}")))
    }

    /// chat_resp 파싱 + choices[0].message.content 추출 공통 helper
    fn extract_content(body_text: &str, ctx: &'static str) -> Result<String, AppError> {
        let chat_resp = Self::parse_chat_response(body_text, ctx)?;
        chat_resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| AppError::Internal(format!("Groq {ctx} returned no choices")))
    }
}

impl GroqAdapter {
    /// extract_keywords 전용 내부 helper: LLM 호출 + JSON 파싱 + 정규화
    async fn call_extract_keywords(
        &self,
        title: &str,
        snippet: Option<&str>,
    ) -> Result<Vec<String>, AppError> {
        let user_content = match snippet {
            Some(s) => format!(
                "제목: {title}\n스니펫: {s}\n\n핵심 기술 키워드 5개를 JSON으로 반환해줘: {{\"keywords\": [\"키워드1\", \"키워드2\", ...]}}",
            ),
            None => format!(
                "제목: {title}\n\n핵심 기술 키워드 5개를 JSON으로 반환해줘: {{\"keywords\": [\"키워드1\", \"키워드2\", ...]}}",
            ),
        };

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": user_content
                }
            ],
            "temperature": 0.3,
            "max_tokens": 100,
            "response_format": { "type": "json_object" },
        });

        let body_text = self.call_chat_api(body, "keyword extract").await?;
        let content = Self::extract_content(&body_text, "keyword extract")?;

        let parsed: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            AppError::Internal(format!(
                "Keyword extract response is not valid JSON: {e}, content: {content}"
            ))
        })?;

        let keywords = parsed["keywords"]
            .as_array()
            .ok_or_else(|| {
                AppError::Internal("Keyword extract response missing 'keywords' array".to_string())
            })?
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s.len() <= MAX_KEYWORD_LEN)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        Ok(keywords)
    }
}

impl LlmPort for GroqAdapter {
    fn summarize(
        &self,
        title: &str,
        content: &str,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse, AppError>> + Send + '_>> {
        let title = title.to_string();
        let content = content.to_string();

        Box::pin(async move {
            let user_message = format!("Title: {title}\nContent: {content}");

            let body = serde_json::json!({
                "model": self.model,
                "messages": [
                    { "role": "system", "content": SYSTEM_PROMPT },
                    { "role": "user", "content": user_message },
                ],
                "temperature": 0.3,
                "max_tokens": 800,
                "response_format": { "type": "json_object" },
            });

            let body_text = self.call_chat_api(body, "summarize").await?;
            let ChatResponse {
                mut choices,
                model: resp_model,
                usage,
            } = Self::parse_chat_response(&body_text, "summarize")?;
            let content = choices
                .drain(..)
                .next()
                .map(|c| c.message.content)
                .ok_or_else(|| {
                    AppError::Internal("Groq summarize returned no choices".to_string())
                })?;

            let parsed: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                AppError::Internal(format!(
                    "LLM response is not valid JSON: {e}, content: {content}"
                ))
            })?;

            let title_ko = parsed["title_ko"]
                .as_str()
                .ok_or_else(|| {
                    AppError::Internal("LLM response missing 'title_ko' field".to_string())
                })?
                .to_string();

            let summary = parsed["summary"]
                .as_str()
                .ok_or_else(|| {
                    AppError::Internal("LLM response missing 'summary' field".to_string())
                })?
                .to_string();

            let insight = parsed["insight"]
                .as_str()
                .ok_or_else(|| {
                    AppError::Internal("LLM response missing 'insight' field".to_string())
                })?
                .to_string();

            let model = resp_model.unwrap_or_else(|| self.model.clone());

            let prompt_tokens = usage.as_ref().and_then(|u| u.prompt_tokens).unwrap_or(0);

            let completion_tokens = usage
                .as_ref()
                .and_then(|u| u.completion_tokens)
                .unwrap_or(0);

            Ok(LlmResponse {
                summary: LlmSummary {
                    title_ko,
                    summary,
                    insight,
                },
                model,
                prompt_tokens,
                completion_tokens,
            })
        })
    }

    fn extract_keywords<'a>(
        &'a self,
        title: &'a str,
        snippet: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, AppError>> + Send + 'a>> {
        Box::pin(self.call_extract_keywords(title, snippet))
    }

    fn generate_quiz<'a>(
        &'a self,
        title: &'a str,
        content: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<QuizResult, AppError>> + Send + 'a>> {
        let title = title.to_string();
        let content = content.to_string();

        Box::pin(async move {
            let user_content = format!(
                "다음 기사에서 아래 두 가지를 JSON으로만 반환해줘 (다른 텍스트 없이):\n\
                1. \"concepts\": 핵심 기술 개념 3~5개. 각 항목은 {{ \"term\": \"용어\", \"explanation\": \"한국어 설명 1~2문장\" }}\n\
                2. \"questions\": 이해도 확인 객관식 문제 3개. 각 항목은 {{ \"question\": \"질문\", \"options\": [\"A\",\"B\",\"C\",\"D\"], \"answer_index\": 0, \"explanation\": \"해설\" }}\n\n\
                제목: {title}\n내용: {content}",
            );

            let body = serde_json::json!({
                "model": self.model,
                "messages": [
                    {
                        "role": "user",
                        "content": user_content
                    }
                ],
                "temperature": 0.3,
                "max_tokens": 1200,
                "response_format": { "type": "json_object" },
            });

            let body_text = self.call_chat_api(body, "quiz").await?;
            let content_str = Self::extract_content(&body_text, "quiz")?;

            let parsed: serde_json::Value = serde_json::from_str(&content_str).map_err(|e| {
                AppError::Internal(format!(
                    "Quiz response is not valid JSON: {e}, content: {content_str}"
                ))
            })?;

            let concepts = parsed["concepts"]
                .as_array()
                .ok_or_else(|| {
                    AppError::Internal("Quiz response missing 'concepts' array".to_string())
                })?
                .iter()
                .filter_map(|v| {
                    let term = v["term"].as_str()?.to_string();
                    let explanation = v["explanation"].as_str()?.to_string();
                    Some(QuizConcept { term, explanation })
                })
                .collect::<Vec<_>>();

            let questions = parsed["questions"]
                .as_array()
                .ok_or_else(|| {
                    AppError::Internal("Quiz response missing 'questions' array".to_string())
                })?
                .iter()
                .filter_map(|v| {
                    let question = v["question"].as_str()?.to_string();
                    let options = v["options"]
                        .as_array()?
                        .iter()
                        .filter_map(|o| o.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>();
                    let answer_index = v["answer_index"].as_u64()? as u8;
                    let explanation = v["explanation"].as_str()?.to_string();
                    Some(QuizQuestion {
                        question,
                        options,
                        answer_index,
                        explanation,
                    })
                })
                .collect::<Vec<_>>();

            Ok(QuizResult {
                concepts,
                questions,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn valid_summarize_response() -> serde_json::Value {
        serde_json::json!({
            "choices": [{
                "message": {
                    "content": "{\"title_ko\": \"테스트 제목\", \"summary\": \"요약입니다.\", \"insight\": \"인사이트입니다.\"}"
                }
            }],
            "model": "llama-3.3-70b-versatile",
            "usage": {"prompt_tokens": 100, "completion_tokens": 50}
        })
    }

    fn valid_quiz_response() -> serde_json::Value {
        serde_json::json!({
            "choices": [{
                "message": {
                    "content": "{\"concepts\": [{\"term\": \"테스트\", \"explanation\": \"설명\"}], \"questions\": [{\"question\": \"질문?\", \"options\": [\"A\",\"B\",\"C\",\"D\"], \"answer_index\": 0, \"explanation\": \"해설\"}]}"
                }
            }],
            "model": "llama-3.3-70b-versatile",
            "usage": {"prompt_tokens": 200, "completion_tokens": 100}
        })
    }

    fn valid_keywords_response() -> serde_json::Value {
        serde_json::json!({
            "choices": [{
                "message": {
                    "content": "{\"keywords\": [\"AI\", \"머신러닝\", \"딥러닝\", \"PyTorch\", \"LLM\"]}"
                }
            }],
            "model": "llama-3.3-70b-versatile",
            "usage": {"prompt_tokens": 50, "completion_tokens": 30}
        })
    }

    // MARK: - summarize 테스트

    #[tokio::test]
    async fn groq_summarize_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_summarize_response()))
            .mount(&mock_server)
            .await;

        let adapter = GroqAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.summarize("Test Title", "Test Content").await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.summary.title_ko, "테스트 제목");
        assert_eq!(resp.summary.summary, "요약입니다.");
        assert_eq!(resp.summary.insight, "인사이트입니다.");
        assert_eq!(resp.model, "llama-3.3-70b-versatile");
        assert_eq!(resp.prompt_tokens, 100);
        assert_eq!(resp.completion_tokens, 50);
    }

    #[tokio::test]
    async fn groq_summarize_uses_bearer_auth() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("Authorization", "Bearer test-api-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_summarize_response()))
            .mount(&mock_server)
            .await;

        let adapter = GroqAdapter::with_base_url("test-api-key", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn groq_summarize_no_reasoning_param() {
        // reasoning 파라미터가 없어야 함을 body로 간접 검증:
        // body_partial_json은 포함된 경우만 검증 가능하므로,
        // reasoning 없이도 성공 응답 처리되는지 확인
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(body_partial_json(serde_json::json!({
                "model": "llama-3.3-70b-versatile",
                "response_format": { "type": "json_object" }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_summarize_response()))
            .mount(&mock_server)
            .await;

        let adapter = GroqAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn groq_summarize_non_2xx_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        let adapter = GroqAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Groq API error"));
    }

    #[tokio::test]
    async fn groq_summarize_invalid_json() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock_server)
            .await;

        let adapter = GroqAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse failed"));
    }

    #[tokio::test]
    async fn groq_summarize_empty_choices() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [],
                "model": "llama-3.3-70b-versatile",
                "usage": null
            })))
            .mount(&mock_server)
            .await;

        let adapter = GroqAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("returned no choices")
        );
    }

    // MARK: - generate_quiz 테스트

    #[tokio::test]
    async fn groq_generate_quiz_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_quiz_response()))
            .mount(&mock_server)
            .await;

        let adapter = GroqAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.generate_quiz("테스트 제목", "테스트 내용").await;

        assert!(result.is_ok());
        let quiz = result.unwrap();
        assert_eq!(quiz.concepts.len(), 1);
        assert_eq!(quiz.concepts[0].term, "테스트");
        assert_eq!(quiz.questions.len(), 1);
        assert_eq!(quiz.questions[0].options.len(), 4);
        assert_eq!(quiz.questions[0].answer_index, 0);
    }

    #[tokio::test]
    async fn groq_generate_quiz_non_2xx_error() {
        let mock_server = MockServer::start().await;

        // 401은 재시도 대상이 아니므로 즉시 에러 반환
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        let adapter = GroqAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.generate_quiz("Title", "Content").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Groq API error"));
    }

    // MARK: - extract_keywords 테스트

    #[tokio::test]
    async fn groq_extract_keywords_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_keywords_response()))
            .mount(&mock_server)
            .await;

        let adapter = GroqAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter
            .extract_keywords("AI 기사 제목", Some("머신러닝 관련 스니펫"))
            .await;

        assert!(result.is_ok());
        let keywords = result.unwrap();
        assert_eq!(keywords.len(), 5);
        assert!(keywords.contains(&"AI".to_string()));
    }

    #[tokio::test]
    async fn groq_extract_keywords_no_snippet() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_keywords_response()))
            .mount(&mock_server)
            .await;

        let adapter = GroqAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.extract_keywords("AI 기사 제목", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn groq_extract_keywords_non_2xx_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        let adapter = GroqAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.extract_keywords("Title", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Groq API error"));
    }

    #[tokio::test]
    async fn groq_retry_on_retryable_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_summarize_response()))
            .mount(&mock_server)
            .await;

        let adapter = GroqAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn groq_network_failure() {
        let adapter = GroqAdapter::with_base_url("test-key", "http://127.0.0.1:1");
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
    }
}
