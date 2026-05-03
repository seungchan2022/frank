use std::future::Future;
use std::pin::Pin;

use crate::domain::error::AppError;
use crate::domain::models::{LlmResponse, LlmSummary, QuizConcept, QuizQuestion, QuizResult};
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
                    insight: Some(format!("*중요*: {title}에 대한 **테스트 분석**입니다.")),
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

    fn generate_quiz<'a>(
        &'a self,
        _title: &'a str,
        _content: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<QuizResult, AppError>> + Send + 'a>> {
        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal("Fake LLM failure".to_string()));
            }
            Ok(QuizResult {
                concepts: vec![QuizConcept {
                    term: "테스트 용어".to_string(),
                    explanation: "테스트 설명입니다.".to_string(),
                }],
                questions: vec![QuizQuestion {
                    question: "테스트 질문?".to_string(),
                    options: vec![
                        "A".to_string(),
                        "B".to_string(),
                        "C".to_string(),
                        "D".to_string(),
                    ],
                    answer_index: 0,
                    explanation: "테스트 해설입니다.".to_string(),
                }],
            })
        })
    }

    fn summarize_with_occupation<'a>(
        &'a self,
        title: &'a str,
        content: &'a str,
        occupation: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse, AppError>> + Send + 'a>> {
        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal("Fake LLM failure".to_string()));
            }
            let insight = occupation.map(|occ| {
                format!("[{occ} 시각] *중요*: {title}에 대한 **직업 맞춤 인사이트**입니다.")
            });
            Ok(LlmResponse {
                summary: LlmSummary {
                    title_ko: format!("[한국어] {title}"),
                    summary: content.to_string(),
                    insight,
                },
                model: "fake-model".to_string(),
                prompt_tokens: 100,
                completion_tokens: 50,
            })
        })
    }

    fn rewrite_with_occupation<'a>(
        &'a self,
        title: &'a str,
        _content: &'a str,
        occupation: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>> {
        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal("Fake LLM failure".to_string()));
            }
            Ok(format!(
                "[{occupation} 시각으로 재작성] {title}: 직업 맞춤 관점에서 설명합니다."
            ))
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
        assert!(
            result
                .summary
                .insight
                .as_deref()
                .unwrap_or("")
                .contains("AI News")
        );
        assert!(result.summary.title_ko.contains("AI News"));
        assert!(
            result.summary.summary.contains("**핵심**"),
            "summary should contain markdown bold"
        );
        assert!(
            result
                .summary
                .insight
                .as_deref()
                .unwrap_or("")
                .contains("*중요*"),
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

    #[tokio::test]
    async fn fake_llm_generate_quiz_returns_result() {
        let llm = FakeLlmAdapter::new();
        let result = llm
            .generate_quiz("테스트 제목", "테스트 내용")
            .await
            .unwrap();
        assert_eq!(result.concepts.len(), 1);
        assert_eq!(result.concepts[0].term, "테스트 용어");
        assert_eq!(result.questions.len(), 1);
        assert_eq!(result.questions[0].options.len(), 4);
        assert_eq!(result.questions[0].answer_index, 0);
    }

    #[tokio::test]
    async fn fake_llm_generate_quiz_failing_returns_error() {
        let llm = FakeLlmAdapter::failing();
        let result = llm.generate_quiz("title", "content").await;
        assert!(result.is_err());
    }

    // MVP15 M3: summarize_with_occupation 테스트 (T-02)

    #[tokio::test]
    async fn summarize_with_occupation_some_returns_insight() {
        let llm = FakeLlmAdapter::new();
        let result = llm
            .summarize_with_occupation("AI 기사", "내용입니다", Some("iOS 개발자"))
            .await
            .unwrap();
        assert!(result.summary.insight.is_some());
        let insight = result.summary.insight.unwrap();
        assert!(insight.contains("iOS 개발자"));
        assert!(insight.contains("AI 기사"));
    }

    #[tokio::test]
    async fn summarize_with_occupation_none_returns_no_insight() {
        let llm = FakeLlmAdapter::new();
        let result = llm
            .summarize_with_occupation("AI 기사", "내용입니다", None)
            .await
            .unwrap();
        assert!(result.summary.insight.is_none());
    }

    #[tokio::test]
    async fn rewrite_with_occupation_returns_string() {
        let llm = FakeLlmAdapter::new();
        let result = llm
            .rewrite_with_occupation("AI 기사", "내용입니다", "iOS 개발자")
            .await
            .unwrap();
        assert!(result.contains("iOS 개발자"));
        assert!(result.contains("AI 기사"));
    }

    #[tokio::test]
    async fn rewrite_with_occupation_failing_returns_error() {
        let llm = FakeLlmAdapter::failing();
        let result = llm
            .rewrite_with_occupation("title", "content", "개발자")
            .await;
        assert!(result.is_err());
    }
}
