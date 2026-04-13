use futures::future::join_all;
use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::SearchResult;
use crate::domain::ports::SearchPort;
use crate::infra::http_utils::{
    OG_IMAGE_TIMEOUT_SECS, RetryConfig, fetch_og_image, read_body_limited, send_with_retry,
};

#[derive(Debug, Clone)]
pub struct ExaAdapter {
    client: Client,
    /// og:image 크롤링 전용 클라이언트 (짧은 타임아웃)
    crawl_client: Client,
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

/// MVP9 M1: snippet 정제 함수.
/// 1. HTML 태그(<...>) 제거 — 문자 단위 파싱
/// 2. 줄바꿈·연속 공백 정리
/// 3. 300자 문장 경계 절단 (마침표/느낌표/물음표 기준, 초과 시 단어 경계로 폴백)
pub fn clean_snippet(s: &str) -> String {
    // 1. HTML 태그 제거 (문자 단위 파싱)
    let mut no_html = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => no_html.push(ch),
            _ => {}
        }
    }

    // 2. 줄바꿈 → 공백, 연속 공백 정리
    let normalized: String = no_html.split_whitespace().collect::<Vec<_>>().join(" ");

    // 3. 300자 이하면 그대로 반환
    if normalized.chars().count() <= 300 {
        return normalized;
    }

    // 4. 300자 이내에서 문장 경계로 절단
    let cutoff: String = normalized.chars().take(300).collect();

    // 마지막 문장 종결 부호 위치 (바이트 인덱스)
    if let Some(pos) = cutoff.rfind(['.', '!', '?']) {
        // 종결 부호 포함하여 반환 (ASCII 단일 바이트)
        cutoff[..pos + 1].to_string()
    } else {
        // 문장 경계 없으면 단어 경계로 폴백
        if let Some(last_space) = cutoff.rfind(' ') {
            cutoff[..last_space].to_string()
        } else {
            cutoff
        }
    }
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
                    title: r.title.unwrap_or_default(),
                    url: r.url,
                    snippet: r
                        .highlights
                        .and_then(|h| h.into_iter().next())
                        .map(|s| clean_snippet(&s)),
                    published_at: r.published_date,
                    image_url,
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

    // --- clean_snippet 단위 테스트 ---

    #[test]
    fn clean_snippet_removes_html_tags() {
        let input = "<p>본문</p> <a href='https://example.com'>링크</a>";
        let result = clean_snippet(input);
        assert!(!result.contains('<'));
        assert!(!result.contains('>'));
        assert!(result.contains("본문"));
        assert!(result.contains("링크"));
    }

    #[test]
    fn clean_snippet_normalizes_whitespace() {
        let input = "첫째 줄\n둘째 줄\n\n셋째 줄";
        let result = clean_snippet(input);
        assert!(!result.contains('\n'));
        assert!(result.contains("첫째 줄"));
        assert!(result.contains("셋째 줄"));
    }

    #[test]
    fn clean_snippet_short_input_unchanged() {
        let input = "짧은 텍스트입니다.";
        let result = clean_snippet(input);
        assert_eq!(result, "짧은 텍스트입니다.");
    }

    #[test]
    fn clean_snippet_cuts_at_sentence_boundary() {
        // 300자 초과 시 마침표 기준으로 절단
        let long = "첫 번째 문장입니다. ".repeat(20); // 300자 초과
        let result = clean_snippet(&long);
        assert!(result.chars().count() <= 300);
        assert!(result.ends_with('.'), "문장 경계로 절단되어야 함: {result}");
    }

    #[test]
    fn clean_snippet_sentence_boundary_no_ellipsis() {
        // 문장 경계 절단 시 "…" 없어야 함
        let sentences = "Apple이 새로운 AI 기능을 발표했다. 이번 발표에서 iOS 업데이트가 포함됐다. 사용자 경험 개선이 핵심이다. ".repeat(5);
        let result = clean_snippet(&sentences);
        assert!(!result.ends_with('…'), "문장 경계 절단 시 … 없어야 함");
        assert!(result.ends_with('.'), "마침표로 끝나야 함");
    }

    #[test]
    fn clean_snippet_normal_text_preserved() {
        let input = "Apple이 새로운 AI 기능을 iOS 18에 추가했다. 이 기능은 사용자 경험을 크게 개선할 것으로 예상된다.";
        let result = clean_snippet(input);
        assert!(result.contains("Apple이"));
        assert!(result.contains("iOS 18"));
        assert!(result.contains("AI 기능"));
    }

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

    #[tokio::test]
    async fn search_results_include_og_image() {
        let mock_server = MockServer::start().await;

        // Exa 검색 응답 — mock_server URL 반환
        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    {
                        "title": "Article 1",
                        "url": format!("{}/article1", mock_server.uri()),
                        "highlights": ["snippet 1"],
                        "publishedDate": "2026-01-01"
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        // article1 페이지 — og:image 포함
        Mock::given(method("GET"))
            .and(path("/article1"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"<html><head><meta property="og:image" content="https://cdn.example.com/thumb.jpg" /></head></html>"#,
            ))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].image_url,
            Some("https://cdn.example.com/thumb.jpg".to_string())
        );
    }

    #[tokio::test]
    async fn og_image_none_when_crawl_fails() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    {
                        "title": "Blocked",
                        "url": format!("{}/blocked", mock_server.uri()),
                        "highlights": ["snippet"],
                        "publishedDate": null
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        // 크롤링 차단 (403)
        Mock::given(method("GET"))
            .and(path("/blocked"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await.unwrap();

        assert_eq!(results[0].image_url, None);
    }
}
