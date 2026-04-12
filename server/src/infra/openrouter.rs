use std::future::Future;
use std::pin::Pin;

use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::{LlmResponse, LlmSummary, QuizConcept, QuizQuestion, QuizResult};
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

const KEYWORD_EXTRACT_MODEL: &str = "meta-llama/llama-3.3-70b-instruct:free";
const MAX_KEYWORD_LEN: usize = 100;

const SYSTEM_PROMPT: &str = r#"You are a news analyst for a Korean-speaking audience. Given an article title and content, provide a JSON response with exactly three fields:
1. "title_ko": 한국어로 번역한 기사 제목 (원문의 핵심을 살리되 한국 독자가 바로 이해할 수 있는 자연스러운 표현)
2. "summary": 기사 핵심 내용을 한국어로 쉽게 풀어서 설명 (전문 용어는 괄호 안에 원문 병기, 3-5문장). Use limited Markdown in the string value: **bold** for key terms, *italic* for emphasis, - for bullet lists, blank lines between paragraphs.
3. "insight": 이 기사가 왜 중요한지, 독자에게 어떤 의미인지 한국어로 분석 (2-3문장). Use limited Markdown in the string value: **bold** for key terms, *italic* for emphasis.

Respond ONLY with valid JSON. Do NOT use Markdown outside of the string values (no code blocks, no headings, no tables)."#;

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

impl OpenRouterAdapter {
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
            "model": KEYWORD_EXTRACT_MODEL,
            "messages": [
                {
                    "role": "user",
                    "content": user_content
                }
            ],
            "temperature": 0.3,
            "max_tokens": 100,
            "response_format": { "type": "json_object" },
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
        .map_err(|e| {
            AppError::Internal(format!("OpenRouter keyword extract request failed: {e}"))
        })?;

        let status = resp.status();
        let body_text = read_body_limited(resp, config.max_response_size)
            .await
            .map_err(|e| {
                AppError::Internal(format!("OpenRouter keyword extract read failed: {e}"))
            })?;

        if !status.is_success() {
            return Err(AppError::Internal(format!(
                "OpenRouter keyword extract API error ({status}): {body_text}"
            )));
        }

        let chat_resp: ChatResponse = serde_json::from_str(&body_text).map_err(|e| {
            AppError::Internal(format!("OpenRouter keyword extract parse failed: {e}"))
        })?;

        let content = chat_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| {
                AppError::Internal("OpenRouter keyword extract returned no choices".to_string())
            })?;

        // {"keywords": [...]} 래퍼 JSON 파싱
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
            .collect::<std::collections::HashSet<_>>() // 중복 제거
            .into_iter()
            .collect::<Vec<_>>();

        Ok(keywords)
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
            .map_err(|e| AppError::Internal(format!("OpenRouter quiz request failed: {e}")))?;

            let status = resp.status();
            let body_text = read_body_limited(resp, config.max_response_size)
                .await
                .map_err(|e| AppError::Internal(format!("OpenRouter quiz read failed: {e}")))?;

            if !status.is_success() {
                return Err(AppError::Internal(format!(
                    "OpenRouter quiz API error ({status}): {body_text}"
                )));
            }

            let chat_resp: ChatResponse = serde_json::from_str(&body_text)
                .map_err(|e| AppError::Internal(format!("OpenRouter quiz parse failed: {e}")))?;

            let content_str = chat_resp
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .ok_or_else(|| {
                    AppError::Internal("OpenRouter quiz returned no choices".to_string())
                })?;

            let parsed: serde_json::Value = serde_json::from_str(&content_str).map_err(|e| {
                AppError::Internal(format!(
                    "Quiz response is not valid JSON: {e}, content: {content_str}"
                ))
            })?;

            // concepts 파싱
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

            // questions 파싱
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

    #[test]
    fn system_prompt_contains_markdown_instructions() {
        assert!(
            SYSTEM_PROMPT.contains("**bold**"),
            "SYSTEM_PROMPT must instruct LLM to use bold markdown"
        );
        assert!(
            SYSTEM_PROMPT.contains("*italic*"),
            "SYSTEM_PROMPT must instruct LLM to use italic markdown"
        );
        assert!(
            SYSTEM_PROMPT.contains("- for bullet"),
            "SYSTEM_PROMPT must instruct LLM to use bullet list markdown"
        );
    }

    #[tokio::test]
    async fn markdown_in_summary_and_insight_parses_correctly() {
        let mock_server = MockServer::start().await;

        let markdown_response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": "{\"title_ko\": \"AI 혁신\", \"summary\": \"**핵심 내용**: AI가 발전했습니다.\\n- 첫 번째 항목\\n- 두 번째 항목\", \"insight\": \"이 기사는 *매우 중요*합니다. **주목**할 필요가 있습니다.\"}"
                }
            }],
            "model": "test-model",
            "usage": {"prompt_tokens": 100, "completion_tokens": 80}
        });

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(markdown_response))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.summarize("AI 기사", "내용").await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert!(resp.summary.summary.contains("**핵심 내용**"));
        assert!(resp.summary.insight.contains("*매우 중요*"));
    }

    // extract_keywords 테스트

    fn valid_keyword_response() -> serde_json::Value {
        serde_json::json!({
            "choices": [{
                "message": {
                    "content": "{\"keywords\": [\"iOS\", \"Swift\", \"SwiftUI\", \"Apple\", \"Xcode\"]}"
                }
            }],
            "model": "meta-llama/llama-3.3-70b-instruct:free",
            "usage": {"prompt_tokens": 50, "completion_tokens": 30}
        })
    }

    #[tokio::test]
    async fn extract_keywords_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_keyword_response()))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter
            .extract_keywords("iOS 기사 제목", Some("Swift 관련 스니펫"))
            .await;
        assert!(
            result.is_ok(),
            "extract_keywords should succeed: {result:?}"
        );
        let keywords = result.unwrap();
        assert!(!keywords.is_empty());
        assert!(keywords.contains(&"iOS".to_string()));
    }

    #[tokio::test]
    async fn extract_keywords_no_snippet() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_keyword_response()))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.extract_keywords("title only", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn extract_keywords_filters_too_long() {
        let mock_server = MockServer::start().await;

        let long_keyword = "a".repeat(101);
        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": format!("{{\"keywords\": [\"valid\", \"{long_keyword}\"]}}")
                }
            }],
            "model": "test-model",
            "usage": null
        });

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.extract_keywords("title", None).await.unwrap();
        // 100자 초과 키워드는 제거
        assert!(result.contains(&"valid".to_string()));
        assert!(!result.iter().any(|k| k.len() > 100));
    }

    #[tokio::test]
    async fn extract_keywords_deduplicates() {
        let mock_server = MockServer::start().await;

        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": "{\"keywords\": [\"iOS\", \"iOS\", \"Swift\"]}"
                }
            }],
            "model": "test-model",
            "usage": null
        });

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.extract_keywords("title", None).await.unwrap();
        // 중복 제거 후 iOS는 1개만
        let ios_count = result.iter().filter(|k| *k == "iOS").count();
        assert_eq!(ios_count, 1);
    }

    #[tokio::test]
    async fn extract_keywords_missing_keywords_field_returns_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{"message": {"content": "{\"wrong_field\": []}"}}],
                "model": "test-model",
                "usage": null
            })))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.extract_keywords("title", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("keywords"));
    }

    #[tokio::test]
    async fn extract_keywords_non_2xx_returns_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .up_to_n_times(10) // retry 포함
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.extract_keywords("title", None).await;
        assert!(result.is_err());
    }

    // generate_quiz 테스트

    fn valid_quiz_response() -> serde_json::Value {
        serde_json::json!({
            "choices": [{
                "message": {
                    "content": "{\"concepts\": [{\"term\": \"Swift\", \"explanation\": \"애플 언어\"}], \"questions\": [{\"question\": \"질문?\", \"options\": [\"A\",\"B\",\"C\",\"D\"], \"answer_index\": 0, \"explanation\": \"해설\"}]}"
                }
            }],
            "model": "test-model",
            "usage": {"prompt_tokens": 100, "completion_tokens": 200}
        })
    }

    #[tokio::test]
    async fn generate_quiz_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(valid_quiz_response()))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.generate_quiz("테스트 제목", "테스트 내용").await;
        assert!(result.is_ok(), "generate_quiz should succeed: {result:?}");
        let quiz = result.unwrap();
        assert_eq!(quiz.concepts.len(), 1);
        assert_eq!(quiz.concepts[0].term, "Swift");
        assert_eq!(quiz.questions.len(), 1);
        assert_eq!(quiz.questions[0].options.len(), 4);
        assert_eq!(quiz.questions[0].answer_index, 0);
    }

    #[tokio::test]
    async fn generate_quiz_missing_concepts_returns_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{"message": {"content": "{\"questions\": []}"}}],
                "model": "test-model",
                "usage": null
            })))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.generate_quiz("title", "content").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("concepts"));
    }

    #[tokio::test]
    async fn generate_quiz_missing_questions_returns_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{"message": {"content": "{\"concepts\": []}"}}],
                "model": "test-model",
                "usage": null
            })))
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.generate_quiz("title", "content").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("questions"));
    }

    #[tokio::test]
    async fn generate_quiz_non_2xx_returns_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
            .up_to_n_times(10)
            .mount(&mock_server)
            .await;

        let adapter =
            OpenRouterAdapter::with_base_url("test-key", "test-model", &mock_server.uri());
        let result = adapter.generate_quiz("title", "content").await;
        assert!(result.is_err());
    }
}
