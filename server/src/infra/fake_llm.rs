use std::future::Future;
use std::pin::Pin;

use crate::domain::error::AppError;
use crate::domain::models::{LlmResponse, LlmSummary};
use crate::domain::ports::LlmPort;

#[derive(Debug, Clone)]
pub struct FakeLlmAdapter {
    should_fail: bool,
}

impl FakeLlmAdapter {
    pub fn new() -> Self {
        Self { should_fail: false }
    }

    pub fn failing() -> Self {
        Self { should_fail: true }
    }
}

impl Default for FakeLlmAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmPort for FakeLlmAdapter {
    fn summarize(
        &self,
        title: &str,
        _content: &str,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse, AppError>> + Send + '_>> {
        let title = title.to_string();

        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal("Fake LLM failure".to_string()));
            }

            Ok(LlmResponse {
                summary: LlmSummary {
                    title_ko: format!("[한국어] {title}"),
                    summary: format!(
                        "**핵심**: {title}에 대한 테스트 요약입니다.\n- 첫 번째 항목\n- 두 번째 항목"
                    ),
                    insight: format!("*중요*: {title}에 대한 **테스트 분석**입니다."),
                },
                model: "fake-model".to_string(),
                prompt_tokens: 100,
                completion_tokens: 50,
            })
        })
    }

    fn extract_keywords<'a>(
        &'a self,
        _title: &'a str,
        _snippet: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, AppError>> + Send + 'a>> {
        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal("Fake LLM failure".to_string()));
            }
            Ok(vec![
                "iOS".to_string(),
                "Swift".to_string(),
                "SwiftUI".to_string(),
            ])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_llm_returns_deterministic_result() {
        let llm = FakeLlmAdapter::new();
        let result = llm.summarize("AI News", "Some content").await.unwrap();
        assert!(result.summary.summary.contains("AI News"));
        assert!(result.summary.insight.contains("AI News"));
        assert!(result.summary.title_ko.contains("AI News"));
        assert!(
            result.summary.summary.contains("**핵심**"),
            "summary should contain markdown bold"
        );
        assert!(
            result.summary.insight.contains("*중요*"),
            "insight should contain markdown italic"
        );
        assert_eq!(result.model, "fake-model");
        assert_eq!(result.prompt_tokens, 100);
        assert_eq!(result.completion_tokens, 50);
    }

    #[tokio::test]
    async fn fake_llm_failing_returns_error() {
        let llm = FakeLlmAdapter::failing();
        let result = llm.summarize("AI News", "Some content").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn fake_llm_default() {
        let llm = FakeLlmAdapter::default();
        let result = llm.summarize("Test", "content").await.unwrap();
        assert!(result.summary.title_ko.contains("Test"));
    }

    #[tokio::test]
    async fn fake_llm_extract_keywords_returns_fixed_list() {
        let llm = FakeLlmAdapter::new();
        let result = llm
            .extract_keywords("iOS Swift 기사", Some("SwiftUI 관련"))
            .await
            .unwrap();
        assert_eq!(result, vec!["iOS", "Swift", "SwiftUI"]);
    }

    #[tokio::test]
    async fn fake_llm_extract_keywords_no_snippet() {
        let llm = FakeLlmAdapter::new();
        let result = llm.extract_keywords("title only", None).await.unwrap();
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn fake_llm_extract_keywords_failing_returns_error() {
        let llm = FakeLlmAdapter::failing();
        let result = llm.extract_keywords("title", None).await;
        assert!(result.is_err());
    }
}
