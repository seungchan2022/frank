use std::future::Future;
use std::pin::Pin;

use crate::domain::error::AppError;
use crate::domain::models::LlmSummary;
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
        _snippet: &str,
    ) -> Pin<Box<dyn Future<Output = Result<LlmSummary, AppError>> + Send + '_>> {
        let title = title.to_string();

        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal("Fake LLM failure".to_string()));
            }

            Ok(LlmSummary {
                summary: format!("[요약] {title}에 대한 테스트 요약입니다."),
                insight: format!("[인사이트] {title}에 대한 테스트 분석입니다."),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_llm_returns_deterministic_result() {
        let llm = FakeLlmAdapter::new();
        let result = llm.summarize("AI News", "Some snippet").await.unwrap();
        assert!(result.summary.contains("AI News"));
        assert!(result.insight.contains("AI News"));
    }

    #[tokio::test]
    async fn fake_llm_failing_returns_error() {
        let llm = FakeLlmAdapter::failing();
        let result = llm.summarize("AI News", "Some snippet").await;
        assert!(result.is_err());
    }
}
