use std::future::Future;
use std::pin::Pin;

use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::{LlmResponse, LlmSummary};
use crate::domain::ports::LlmPort;
use crate::infra::http_utils::{RetryConfig, read_body_limited, send_with_retry};

#[derive(Debug, Clone)]
pub struct OpenRouterAdapter {
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

const SYSTEM_PROMPT: &str = r#"You are a news analyst for a Korean-speaking audience. Given an article title and content, provide a JSON response with exactly three fields:
1. "title_ko": 한국어로 번역한 기사 제목 (원문의 핵심을 살리되 한국 독자가 바로 이해할 수 있는 자연스러운 표현)
2. "summary": 기사 핵심 내용을 한국어로 쉽게 풀어서 설명 (전문 용어는 괄호 안에 원문 병기, 3-5문장)
3. "insight": 이 기사가 왜 중요한지, 독자에게 어떤 의미인지 한국어로 분석 (2-3문장)

Respond ONLY with valid JSON, no markdown or extra text."#;

impl OpenRouterAdapter {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self::with_base_url(api_key, model, "https://openrouter.ai")
    }

    pub fn with_base_url(api_key: &str, model: &str, base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("Failed to build HTTP client"),
            api_key: api_key.to_string(),
            model: model.to_string(),
            base_url: base_url.to_string(),
        }
    }
}

impl LlmPort for OpenRouterAdapter {
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
                // 일부 reasoning 모델(MiniMax M2.5 등)은 reasoning을 강제한다.
                // effort:"none"은 400("Reasoning is mandatory")을 유발하므로
                // exclude:true로 모델은 추론하되 응답에서 trace만 제외한다.
                "reasoning": { "exclude": true },
            });

            let config = RetryConfig::for_llm();

            let api_key = self.api_key.clone();
            let url = format!("{}/api/v1/chat/completions", self.base_url);

            let resp = send_with_retry(
                || {
                    let url = url.clone();
                    let body = body.clone();
                    let api_key = api_key.clone();
                    let client = self.client.clone();
                    async move {
                        client
                            .post(&url)
                            .header("Authorization", format!("Bearer {}", api_key))
                            .header("Content-Type", "application/json")
                            .json(&body)
                    }
                },
                &config,
            )
            .await
            .map_err(|e| AppError::Internal(format!("OpenRouter request failed: {e}")))?;

            let status = resp.status();
            let body = read_body_limited(resp, config.max_response_size)
                .await
                .map_err(|e| AppError::Internal(format!("OpenRouter read failed: {e}")))?;

            if !status.is_success() {
                return Err(AppError::Internal(format!(
                    "OpenRouter API error ({status}): {body}"
                )));
            }

            let chat_resp: ChatResponse = serde_json::from_str(&body)
                .map_err(|e| AppError::Internal(format!("OpenRouter parse failed: {e}")))?;

            let content = chat_resp
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .ok_or_else(|| AppError::Internal("OpenRouter returned no choices".to_string()))?;

            // Parse JSON response from LLM
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

            let model = chat_resp.model.unwrap_or_else(|| self.model.clone());

            let prompt_tokens = chat_resp
                .usage
                .as_ref()
                .and_then(|u| u.prompt_tokens)
                .unwrap_or(0);

            let completion_tokens = chat_resp
                .usage
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn valid_llm_response() -> serde_json::Value {
        serde_json::json!({
            "choices": [{
                "message": {
                    "content": "{\"title_ko\": \"테스트 제목\", \"summary\": \"요약입니다.\", \"insight\": \"인사이트입니다.\"}"
                }
            }],
            "model": "test-model",
            "usage": {"prompt_tokens": 100, "completion_tokens": 50}
        })
    }

    #[tokio::test]
    async fn successful_summarize() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_llm_response()))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.summarize("Test Title", "Test Content").await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.summary.title_ko, "테스트 제목");
        assert_eq!(resp.summary.summary, "요약입니다.");
        assert_eq!(resp.summary.insight, "인사이트입니다.");
        assert_eq!(resp.model, "test-model");
        assert_eq!(resp.prompt_tokens, 100);
        assert_eq!(resp.completion_tokens, 50);
    }

    #[tokio::test]
    async fn retry_on_retryable_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_llm_response()))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn size_limit_exceeded() {
        let mock_server = MockServer::start().await;

        let large_body = "x".repeat(2 * 1024 * 1024);
        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&large_body))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn non_2xx_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("OpenRouter API error")
        );
    }

    #[tokio::test]
    async fn invalid_json_from_api() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse failed"));
    }

    #[tokio::test]
    async fn empty_choices_returns_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [],
                "model": "test-model",
                "usage": null
            })))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no choices"));
    }

    #[tokio::test]
    async fn llm_response_not_valid_json() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{"message": {"content": "this is not json"}}],
                "model": "test-model",
                "usage": null
            })))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not valid JSON"));
    }

    #[tokio::test]
    async fn llm_response_missing_title_ko() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{"message": {"content": "{\"summary\": \"s\", \"insight\": \"i\"}"}}],
                "model": "test-model",
                "usage": null
            })))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("title_ko"));
    }

    #[tokio::test]
    async fn llm_response_missing_summary() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{"message": {"content": "{\"title_ko\": \"t\", \"insight\": \"i\"}"}}],
                "model": "test-model",
                "usage": null
            })))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("summary"));
    }

    #[tokio::test]
    async fn llm_response_missing_insight() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{"message": {"content": "{\"title_ko\": \"t\", \"summary\": \"s\"}"}}],
                "model": "test-model",
                "usage": null
            })))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("insight"));
    }

    /// 회귀: MiniMax M2.5 등 reasoning 강제 모델에서
    /// `effort: "none"`은 400("Reasoning is mandatory")을 유발한다.
    /// 응답 trace만 빼는 `exclude: true`로 요청해야 한다.
    #[tokio::test]
    async fn request_uses_reasoning_exclude() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .and(body_partial_json(serde_json::json!({
                "reasoning": { "exclude": true }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_llm_response()))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.summarize("Title", "Content").await;
        assert!(
            result.is_ok(),
            "request body must include reasoning.exclude=true: {result:?}"
        );
    }

    #[tokio::test]
    async fn network_failure() {
        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", "http://127.0.0.1:1");
        let result = adapter.summarize("Title", "Content").await;
        assert!(result.is_err());
    }
}
