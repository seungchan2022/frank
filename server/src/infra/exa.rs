use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::SearchResult;
use crate::domain::ports::SearchPort;
use crate::infra::http_utils::{RetryConfig, read_body_limited, send_with_retry};

#[derive(Debug, Clone)]
pub struct ExaAdapter {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct ExaResponse {
    results: Vec<ExaResult>,
}

#[derive(Debug, Deserialize)]
struct ExaResult {
    title: Option<String>,
    url: String,
    highlights: Option<Vec<String>>,
    #[serde(rename = "publishedDate")]
    published_date: Option<String>,
}

impl ExaAdapter {
    pub fn new(api_key: &str) -> Self {
        Self::with_base_url(api_key, "https://api.exa.ai")
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

impl SearchPort for ExaAdapter {
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
                "numResults": max_results,
                "contents": {
                    "highlights": {
                        "numSentences": 3,
                        "highlightsPerUrl": 1
                    }
                }
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
                            .header("x-api-key", &api_key)
                            .header("Content-Type", "application/json")
                            .json(&body)
                    }
                },
                &config,
            )
            .await
            .map_err(|e| AppError::Internal(format!("Exa request failed: {e}")))?;

            let status = resp.status();
            let body = read_body_limited(resp, config.max_response_size)
                .await
                .map_err(|e| AppError::Internal(format!("Exa read failed: {e}")))?;

            if !status.is_success() {
                return Err(AppError::Internal(format!("Exa returned {status}: {body}")));
            }

            let data: ExaResponse = serde_json::from_str(&body)
                .map_err(|e| AppError::Internal(format!("Exa parse failed: {e}")))?;

            Ok(data
                .results
                .into_iter()
                .map(|r| SearchResult {
                    title: r.title.unwrap_or_default(),
                    url: r.url,
                    snippet: r.highlights
                        .and_then(|h| h.into_iter().next())
                        .map(|s| {
                            // 마크다운 헤더(#) 제거 후 공백 정리, 300자 제한
                            let cleaned = s
                                .lines()
                                .filter(|l| !l.trim_start().starts_with('#'))
                                .collect::<Vec<_>>()
                                .join(" ")
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ");
                            cleaned.chars().take(300).collect::<String>()
                        }),
                    published_at: r.published_date,
                    image_url: None, // Exa API는 image_url 미제공
                })
                .collect())
        })
    }

    fn source_name(&self) -> &str {
        "exa"
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
            .respond_with(ResponseTemplate::new(502))
            .up_to_n_times(2)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "results": [{"title": "Test", "url": "https://example.com", "highlights": ["snippet"], "publishedDate": null}]
                })),
            )
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
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

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn non_2xx_error_message() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(403).set_body_string("Forbidden"))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Exa returned"));
    }

    #[tokio::test]
    async fn invalid_json_parse_failure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse failed"));
    }

    #[tokio::test]
    async fn request_error_network_failure() {
        let adapter = ExaAdapter::with_base_url("test-key", "http://127.0.0.1:1");
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn title_none_defaults_to_empty_string() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"title": null, "url": "https://example.com", "highlights": ["content"], "publishedDate": null}]
            })))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await.unwrap();
        assert_eq!(results[0].title, "");
    }

    #[tokio::test]
    async fn successful_search_maps_fields() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    {"title": "Article 1", "url": "https://a.com", "highlights": ["snippet 1"], "publishedDate": "2026-01-01"}
                ]
            })))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Article 1");
        assert_eq!(results[0].url, "https://a.com");
        assert_eq!(results[0].snippet, Some("snippet 1".to_string()));
        assert_eq!(results[0].published_at, Some("2026-01-01".to_string()));
    }
}
