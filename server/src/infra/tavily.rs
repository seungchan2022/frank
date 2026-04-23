use futures::future::join_all;
use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::SearchResult;
use crate::domain::ports::SearchPort;
use crate::infra::exa::clean_snippet;
use crate::infra::http_utils::{
    OG_IMAGE_TIMEOUT_SECS, RetryConfig, fetch_og_image, read_body_limited, send_with_retry,
};

#[derive(Debug, Clone)]
pub struct TavilyAdapter {
    client: Client,
    /// og:image 크롤링 전용 클라이언트 (짧은 타임아웃)
    crawl_client: Client,
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
            crawl_client: Client::builder()
                .timeout(std::time::Duration::from_secs(OG_IMAGE_TIMEOUT_SECS))
                .user_agent("Mozilla/5.0 (compatible; FrankBot/1.0)")
                .build()
                .expect("Failed to build crawl client"),
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
                "time_range": "week",
                // Tavily API 파라미터명: topic (Exa는 category) — 각 API 스펙 차이
                "topic": "news",
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

            let status = resp.status();
            let body = read_body_limited(resp, config.max_response_size)
                .await
                .map_err(|e| AppError::Internal(format!("Tavily read failed: {e}")))?;

            if !status.is_success() {
                return Err(AppError::Internal(format!(
                    "Tavily returned {status}: {body}"
                )));
            }

            let data: TavilyResponse = serde_json::from_str(&body)
                .map_err(|e| AppError::Internal(format!("Tavily parse failed: {e}")))?;

            // 각 기사 URL에서 og:image 병렬 크롤링
            let crawl_futures: Vec<_> = data
                .results
                .iter()
                .map(|r| fetch_og_image(&self.crawl_client, &r.url))
                .collect();
            let image_urls: Vec<Option<String>> = join_all(crawl_futures).await;

            Ok(data
                .results
                .into_iter()
                .zip(image_urls)
                .map(|(r, image_url)| SearchResult {
                    title: r.title,
                    url: r.url,
                    snippet: r.content.map(|s| clean_snippet(&s)),
                    published_at: r.published_date,
                    image_url,
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
    use crate::infra::http_utils::extract_og_image;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // MARK: - og:image 파싱 단위 테스트

    #[test]
    fn og_image_standard_order() {
        let html =
            r#"<head><meta property="og:image" content="https://example.com/img.jpg" /></head>"#;
        assert_eq!(
            extract_og_image(html),
            Some("https://example.com/img.jpg".to_string())
        );
    }

    #[test]
    fn og_image_reversed_attr_order() {
        let html =
            r#"<head><meta content="https://example.com/img.jpg" property="og:image" /></head>"#;
        assert_eq!(
            extract_og_image(html),
            Some("https://example.com/img.jpg".to_string())
        );
    }

    #[test]
    fn og_image_not_present() {
        let html = r#"<head><meta property="og:title" content="Hello" /></head>"#;
        assert_eq!(extract_og_image(html), None);
    }

    #[test]
    fn og_image_relative_url_ignored() {
        let html = r#"<head><meta property="og:image" content="/img/thumb.jpg" /></head>"#;
        assert_eq!(extract_og_image(html), None);
    }

    // MARK: - Tavily 어댑터 통합 테스트

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
        let adapter = TavilyAdapter::with_base_url("test-key", "http://127.0.0.1:1");
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn successful_search_with_og_image() {
        let mock_server = MockServer::start().await;

        // Tavily 검색 응답
        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    {"title": "Article 1", "url": format!("{}/article1", mock_server.uri()), "content": "snippet 1", "published_date": "2026-01-01"},
                    {"title": "Article 2", "url": format!("{}/article2", mock_server.uri()), "content": null, "published_date": null}
                ]
            })))
            .mount(&mock_server)
            .await;

        // article1: og:image 있음
        Mock::given(method("GET"))
            .and(path("/article1"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"<html><head><meta property="og:image" content="https://cdn.example.com/thumb.jpg" /></head></html>"#,
            ))
            .mount(&mock_server)
            .await;

        // article2: og:image 없음 (403 차단)
        Mock::given(method("GET"))
            .and(path("/article2"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&mock_server)
            .await;

        let adapter = TavilyAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await.unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Article 1");
        assert_eq!(
            results[0].image_url,
            Some("https://cdn.example.com/thumb.jpg".to_string())
        );
        assert_eq!(results[1].image_url, None);
    }

    // MARK: - ST-2: time_range 파라미터 검증

    #[tokio::test]
    async fn search_request_includes_time_range_week() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .and(body_partial_json(serde_json::json!({
                "time_range": "week"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"title": "Test", "url": "https://example.com", "content": "snippet", "published_date": null}]
            })))
            .mount(&mock_server)
            .await;

        let adapter = TavilyAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test query", 5).await;
        assert!(
            result.is_ok(),
            "time_range: week 파라미터 포함 요청이 성공해야 함"
        );
        assert_eq!(result.unwrap().len(), 1);
    }

    // MARK: - ST-2: topic:"news" 파라미터 검증

    #[tokio::test]
    async fn tavily_request_includes_topic_news() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .and(body_partial_json(serde_json::json!({
                "topic": "news"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"title": "News Article", "url": "https://example.com/news/article", "content": "snippet", "published_date": null}]
            })))
            .mount(&mock_server)
            .await;

        let adapter = TavilyAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test query", 5).await;
        assert!(
            result.is_ok(),
            "topic: news 파라미터 포함 요청이 성공해야 함: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn tavily_null_snippet_remains_none() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    {
                        "title": "No Snippet",
                        "url": "https://example.com/no-snippet",
                        "content": null,
                        "published_date": null
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/no-snippet"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&mock_server)
            .await;

        let adapter = TavilyAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await.unwrap();

        assert_eq!(results[0].snippet, None, "null content는 None으로 유지");
    }
}
