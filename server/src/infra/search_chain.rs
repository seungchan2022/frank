use crate::domain::error::AppError;
use crate::domain::models::SearchResult;
use crate::domain::ports::SearchPort;

/// 폴백 체인: 순서대로 시도, 첫 성공 결과 반환
pub struct SearchFallbackChain {
    sources: Vec<Box<dyn SearchPort>>,
}

impl std::fmt::Debug for SearchFallbackChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&str> = self.sources.iter().map(|s| s.source_name()).collect();
        f.debug_struct("SearchFallbackChain")
            .field("sources", &names)
            .finish()
    }
}

impl SearchFallbackChain {
    pub fn new(sources: Vec<Box<dyn SearchPort>>) -> Self {
        Self { sources }
    }

    pub async fn search(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<(Vec<SearchResult>, String), AppError> {
        let mut last_error = None;

        for source in &self.sources {
            match source.search(query, max_results).await {
                Ok(results) if !results.is_empty() => {
                    tracing::info!(
                        source = source.source_name(),
                        count = results.len(),
                        "search succeeded"
                    );
                    return Ok((results, source.source_name().to_string()));
                }
                Ok(_) => {
                    tracing::warn!(source = source.source_name(), "search returned empty");
                }
                Err(e) => {
                    tracing::warn!(source = source.source_name(), error = %e, "search failed, trying next");
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AppError::Internal("All search sources returned empty results".to_string())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::fake_search::FakeSearchAdapter;

    #[tokio::test]
    async fn fallback_chain_uses_first_success() {
        let failing = FakeSearchAdapter::new("failing", vec![], true);
        let working = FakeSearchAdapter::new(
            "working",
            vec![SearchResult {
                title: "Test".to_string(),
                url: "https://example.com".to_string(),
                snippet: Some("snippet".to_string()),
                published_at: None,
            }],
            false,
        );

        let chain = SearchFallbackChain::new(vec![Box::new(failing), Box::new(working)]);
        let (results, source) = chain.search("test query", 5).await.unwrap();

        assert_eq!(source, "working");
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn fallback_chain_all_fail() {
        let fail1 = FakeSearchAdapter::new("fail1", vec![], true);
        let fail2 = FakeSearchAdapter::new("fail2", vec![], true);

        let chain = SearchFallbackChain::new(vec![Box::new(fail1), Box::new(fail2)]);
        let result = chain.search("test query", 5).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn fallback_chain_skips_empty_results() {
        let empty = FakeSearchAdapter::new("empty", vec![], false);
        let working = FakeSearchAdapter::new(
            "working",
            vec![SearchResult {
                title: "Result".to_string(),
                url: "https://example.com/result".to_string(),
                snippet: None,
                published_at: None,
            }],
            false,
        );

        let chain = SearchFallbackChain::new(vec![Box::new(empty), Box::new(working)]);
        let (results, source) = chain.search("query", 5).await.unwrap();

        assert_eq!(source, "working");
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn fallback_chain_all_empty_returns_error() {
        let empty1 = FakeSearchAdapter::new("empty1", vec![], false);
        let empty2 = FakeSearchAdapter::new("empty2", vec![], false);

        let chain = SearchFallbackChain::new(vec![Box::new(empty1), Box::new(empty2)]);
        let result = chain.search("query", 5).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("All search sources returned empty"));
    }

    #[tokio::test]
    async fn fallback_chain_first_source_succeeds() {
        let working = FakeSearchAdapter::new(
            "first",
            vec![SearchResult {
                title: "First".to_string(),
                url: "https://example.com/first".to_string(),
                snippet: None,
                published_at: None,
            }],
            false,
        );
        let second = FakeSearchAdapter::new("second", vec![], false);

        let chain = SearchFallbackChain::new(vec![Box::new(working), Box::new(second)]);
        let (results, source) = chain.search("query", 5).await.unwrap();

        // tracing::info path is covered
        assert_eq!(source, "first");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn debug_format_shows_source_names() {
        let chain = SearchFallbackChain::new(vec![
            Box::new(FakeSearchAdapter::new("tavily", vec![], false)),
            Box::new(FakeSearchAdapter::new("exa", vec![], false)),
        ]);
        let debug = format!("{:?}", chain);
        assert!(debug.contains("tavily"));
        assert!(debug.contains("exa"));
    }
}
