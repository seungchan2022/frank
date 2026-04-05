use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::SearchResult;
use crate::domain::ports::SearchPort;
use crate::infra::http_utils::{RetryConfig, send_with_retry};

#[derive(Debug, Clone)]
pub struct TavilyAdapter {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: Option<String>,
    published_date: Option<String>,
}

impl TavilyAdapter {
    pub fn new(api_key: &str) -> Self {
        Self::with_base_url(api_key, "https://api.tavily.com")
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

impl SearchPort for TavilyAdapter {
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
                "max_results": max_results,
                "search_depth": "advanced",
                "include_answer": false,
            });

            let config = RetryConfig::for_search();

            let api_key = self.api_key.clone();
            let url = format!("{}/search", self.base_url);

            let resp = send_with_retry(
                || {
                    let url = url.clone();
                    let body = body.clone();
                    let api_key = api_key.clone();
                    let client = self.client.clone();
                    async move {
                        client
                            .post(&url)
                            .header("Content-Type", "application/json")
                            .bearer_auth(&api_key)
                            .json(&body)
                    }
                },
                &config,
            )
            .await
            .map_err(|e| AppError::Internal(format!("Tavily request failed: {e}")))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(AppError::Internal(format!(
                    "Tavily returned {status}: {body}"
                )));
            }

            let data: TavilyResponse = resp
                .json()
                .await
                .map_err(|e| AppError::Internal(format!("Tavily parse failed: {e}")))?;

            Ok(data
                .results
                .into_iter()
                .map(|r| SearchResult {
                    title: r.title,
                    url: r.url,
                    snippet: r.content,
                    published_at: r.published_date,
                })
                .collect())
        })
    }

    fn source_name(&self) -> &str {
        "tavily"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn retry_on_retryable_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(2)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "results": [{"title": "Test", "url": "https://example.com", "content": "snippet", "published_date": null}]
                })),
            )
            .mount(&mock_server)
            .await;

        let adapter = TavilyAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn size_limit_exceeded() {
        let mock_server = MockServer::start().await;

        let large_body = "x".repeat(3 * 1024 * 1024);
        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&large_body))
            .mount(&mock_server)
            .await;

        let adapter = TavilyAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("request failed"));
    }

    #[tokio::test]
    async fn non_2xx_error_message() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        let adapter = TavilyAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Tavily returned"));
    }

    #[tokio::test]
    async fn invalid_json_parse_failure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock_server)
            .await;

        let adapter = TavilyAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse failed"));
    }

    #[tokio::test]
    async fn request_error_network_failure() {
        // Use a port that's not listening
        let adapter = TavilyAdapter::with_base_url("test-key", "http://127.0.0.1:1");
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn successful_search_maps_fields() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    {"title": "Article 1", "url": "https://a.com", "content": "snippet 1", "published_date": "2026-01-01"},
                    {"title": "Article 2", "url": "https://b.com", "content": null, "published_date": null}
                ]
            })))
            .mount(&mock_server)
            .await;

        let adapter = TavilyAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Article 1");
        assert_eq!(results[0].url, "https://a.com");
        assert_eq!(results[0].snippet, Some("snippet 1".to_string()));
        assert_eq!(results[0].published_at, Some("2026-01-01".to_string()));
        assert_eq!(results[1].snippet, None);
    }
}
