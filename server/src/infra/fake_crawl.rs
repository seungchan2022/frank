use std::future::Future;
use std::pin::Pin;

use crate::domain::error::AppError;
use crate::domain::ports::CrawlPort;

#[derive(Debug, Clone)]
pub struct FakeCrawlAdapter {
    should_fail: bool,
}

impl FakeCrawlAdapter {
    pub fn new() -> Self {
        Self { should_fail: false }
    }

    pub fn failing() -> Self {
        Self { should_fail: true }
    }
}

impl Default for FakeCrawlAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CrawlPort for FakeCrawlAdapter {
    fn scrape(
        &self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + '_>> {
        let url = url.to_string();

        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal("Fake crawl failure".to_string()));
            }

            Ok(format!(
                "# Fake crawled content\n\nThis is the crawled markdown content for {url}."
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_crawl_returns_markdown() {
        let crawl = FakeCrawlAdapter::new();
        let result = crawl.scrape("https://example.com/article").await.unwrap();
        assert!(result.contains("example.com/article"));
    }

    #[tokio::test]
    async fn fake_crawl_failing_returns_error() {
        let crawl = FakeCrawlAdapter::failing();
        let result = crawl.scrape("https://example.com/article").await;
        assert!(result.is_err());
    }
}
