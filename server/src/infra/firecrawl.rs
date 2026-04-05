use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::SearchResult;
use crate::domain::ports::{CrawlPort, SearchPort};
use crate::infra::http_utils::{RetryConfig, read_body_limited, send_with_retry};

#[derive(Debug, Clone)]
pub struct FirecrawlAdapter {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct FirecrawlResponse {
    data: Vec<FirecrawlResult>,
}

#[derive(Debug, Deserialize)]
struct FirecrawlResult {
    title: Option<String>,
    url: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FirecrawlScrapeResponse {
    data: Option<FirecrawlScrapeData>,
}

#[derive(Debug, Deserialize)]
struct FirecrawlScrapeData {
    markdown: Option<String>,
}

impl FirecrawlAdapter {
    pub fn new(api_key: &str) -> Self {
        Self::with_base_url(api_key, "https://api.firecrawl.dev")
    }

    pub fn with_base_url(api_key: &str, base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
        }
    }
}

impl SearchPort for FirecrawlAdapter {
    fn search(
        &self,
        query: &str,
        max_results: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<SearchResult>, AppError>> + Send + '_>,
    > {
        let query = query.to_string();
        Box::pin(async move {
            let body = serde_json::json!({
                "query": query,
                "limit": max_results,
            });

            let config = RetryConfig::for_search();

            let api_key = self.api_key.clone();
            let url = format!("{}/v1/search", self.base_url);

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
            .map_err(|e| AppError::Internal(format!("Firecrawl request failed: {e}")))?;

            let status = resp.status();
            let body = read_body_limited(resp, config.max_response_size)
                .await
                .map_err(|e| AppError::Internal(format!("Firecrawl read failed: {e}")))?;

            if !status.is_success() {
                return Err(AppError::Internal(format!(
                    "Firecrawl returned {status}: {body}"
                )));
            }

            let data: FirecrawlResponse = serde_json::from_str(&body)
                .map_err(|e| AppError::Internal(format!("Firecrawl parse failed: {e}")))?;

            Ok(data
                .data
                .into_iter()
                .filter_map(|r| {
                    Some(SearchResult {
                        title: r.title.unwrap_or_default(),
                        url: r.url?,
                        snippet: r.description,
                        published_at: None,
                    })
                })
                .collect())
        })
    }

    fn source_name(&self) -> &str {
        "firecrawl"
    }
}

impl CrawlPort for FirecrawlAdapter {
    fn scrape(
        &self,
        url: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, AppError>> + Send + '_>>
    {
        let url = url.to_string();
        Box::pin(async move {
            let body = serde_json::json!({
                "url": url,
                "formats": ["markdown"],
                "onlyMainContent": true,
            });

            let config = RetryConfig::for_crawl();

            let api_key = self.api_key.clone();
            let scrape_url = format!("{}/v1/scrape", self.base_url);

            let resp = send_with_retry(
                || {
                    let scrape_url = scrape_url.clone();
                    let body = body.clone();
                    let api_key = api_key.clone();
                    let client = self.client.clone();
                    async move {
                        client
                            .post(&scrape_url)
                            .header("Authorization", format!("Bearer {}", api_key))
                            .header("Content-Type", "application/json")
                            .json(&body)
                    }
                },
                &config,
            )
            .await
            .map_err(|e| AppError::Internal(format!("Firecrawl scrape request failed: {e}")))?;

            let status = resp.status();
            let scrape_config = RetryConfig::for_crawl();
            let body = read_body_limited(resp, scrape_config.max_response_size)
                .await
                .map_err(|e| AppError::Internal(format!("Firecrawl scrape read failed: {e}")))?;

            if !status.is_success() {
                return Err(AppError::Internal(format!(
                    "Firecrawl scrape returned {status}: {body}"
                )));
            }

            let data: FirecrawlScrapeResponse = serde_json::from_str(&body)
                .map_err(|e| AppError::Internal(format!("Firecrawl scrape parse failed: {e}")))?;

            data.data.and_then(|d| d.markdown).ok_or_else(|| {
                AppError::Internal("Firecrawl scrape returned no markdown".to_string())
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // --- Search tests ---

    #[tokio::test]
    async fn search_retry_on_retryable_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"title": "Test", "url": "https://example.com", "description": "desc"}]
            })))
            .mount(&mock_server)
            .await;

        let adapter = FirecrawlAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn search_size_limit_exceeded() {
        let mock_server = MockServer::start().await;

        let large_body = "x".repeat(3 * 1024 * 1024);
        Mock::given(method("POST"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&large_body))
            .mount(&mock_server)
            .await;

        let adapter = FirecrawlAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn search_non_2xx_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(429).set_body_string("Rate limited"))
            .mount(&mock_server)
            .await;

        let adapter = FirecrawlAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test", 5).await;
        // 429 is retryable, so after retries it should error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn search_invalid_json() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock_server)
            .await;

        let adapter = FirecrawlAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse failed"));
    }

    #[tokio::test]
    async fn search_url_none_filtered_out() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"title": "Has URL", "url": "https://example.com", "description": "desc"},
                    {"title": "No URL", "url": null, "description": "desc"}
                ]
            })))
            .mount(&mock_server)
            .await;

        let adapter = FirecrawlAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Has URL");
    }

    #[tokio::test]
    async fn search_network_failure() {
        let adapter = FirecrawlAdapter::with_base_url("test-key", "http://127.0.0.1:1");
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
    }

    // --- Scrape tests ---

    #[tokio::test]
    async fn scrape_retry_on_retryable_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/scrape"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v1/scrape"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"markdown": "# Hello World"}
            })))
            .mount(&mock_server)
            .await;

        let adapter = FirecrawlAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.scrape("https://example.com").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "# Hello World");
    }

    #[tokio::test]
    async fn scrape_non_2xx_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/scrape"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not found"))
            .mount(&mock_server)
            .await;

        let adapter = FirecrawlAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.scrape("https://example.com").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Firecrawl scrape returned")
        );
    }

    #[tokio::test]
    async fn scrape_invalid_json() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/scrape"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock_server)
            .await;

        let adapter = FirecrawlAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.scrape("https://example.com").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("scrape parse failed")
        );
    }

    #[tokio::test]
    async fn scrape_no_markdown_returns_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/scrape"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"markdown": null}
            })))
            .mount(&mock_server)
            .await;

        let adapter = FirecrawlAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.scrape("https://example.com").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no markdown"));
    }

    #[tokio::test]
    async fn scrape_data_null_returns_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/scrape"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": null
            })))
            .mount(&mock_server)
            .await;

        let adapter = FirecrawlAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.scrape("https://example.com").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no markdown"));
    }

    #[tokio::test]
    async fn scrape_network_failure() {
        let adapter = FirecrawlAdapter::with_base_url("test-key", "http://127.0.0.1:1");
        let result = adapter.scrape("https://example.com").await;
        assert!(result.is_err());
    }
}
