use std::future::Future;
use std::pin::Pin;

use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::{LlmResponse, LlmSummary};
use crate::domain::ports::LlmPort;

#[derive(Debug, Clone)]
pub struct OpenRouterAdapter {
    client: Client,
    api_key: String,
    model: String,
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
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
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
            });

            let resp = self
                .client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("OpenRouter request failed: {e}")))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(AppError::Internal(format!(
                    "OpenRouter API error ({status}): {body}"
                )));
            }

            let chat_resp: ChatResponse = resp
                .json()
                .await
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
