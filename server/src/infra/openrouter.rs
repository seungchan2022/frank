use std::future::Future;
use std::pin::Pin;

use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::LlmSummary;
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
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

const SYSTEM_PROMPT: &str = r#"You are a news analyst. Given an article title and snippet, provide a JSON response with exactly two fields:
1. "summary": A concise Korean summary (2-3 sentences)
2. "insight": A unique insight or analysis in Korean (1-2 sentences)

Respond ONLY with valid JSON, no markdown or extra text. Example:
{"summary": "요약 내용", "insight": "인사이트 내용"}"#;

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
        snippet: &str,
    ) -> Pin<Box<dyn Future<Output = Result<LlmSummary, AppError>> + Send + '_>> {
        let title = title.to_string();
        let snippet = snippet.to_string();

        Box::pin(async move {
            let user_message = format!("Title: {title}\nSnippet: {snippet}");

            let body = serde_json::json!({
                "model": self.model,
                "messages": [
                    { "role": "system", "content": SYSTEM_PROMPT },
                    { "role": "user", "content": user_message },
                ],
                "temperature": 0.3,
                "max_tokens": 500,
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

            Ok(LlmSummary { summary, insight })
        })
    }
}
