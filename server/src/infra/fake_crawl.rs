use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use crate::domain::error::AppError;
use crate::domain::ports::CrawlPort;

#[derive(Debug, Clone)]
pub struct FakeCrawlAdapter {
    should_fail: bool,
    sleep_secs: Option<u64>,
}

impl FakeCrawlAdapter {
    pub fn new() -> Self {
        Self { should_fail: false, sleep_secs: None }
    }

    pub fn failing() -> Self {
        Self { should_fail: true, sleep_secs: None }
    }

    /// tokio::time::sleep으로 지연하는 어댑터.
    /// `tokio::time::pause()` + `advance()` 타임아웃 테스트용.
    pub fn sleeping() -> Self {
        Self { should_fail: false, sleep_secs: Some(60) }
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
        let should_fail = self.should_fail;
        let sleep_secs = self.sleep_secs;

        Box::pin(async move {
            if let Some(secs) = sleep_secs {
                tokio::time::sleep(Duration::from_secs(secs)).await;
            }

            if should_fail {
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

    #[tokio::test]
    async fn fake_crawl_default() {
        let crawl = FakeCrawlAdapter::default();
        let result = crawl.scrape("https://example.com").await.unwrap();
        assert!(result.contains("example.com"));
    }
}
