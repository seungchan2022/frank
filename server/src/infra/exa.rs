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

/// MVP8 M1: snippet 정제 함수.
/// 1. 마크다운 헤더(# 시작 행) 제거
/// 2. HTML 태그(<...>) 제거 — 문자 단위 파싱
/// 3. URL 토큰(http:// / https:// 시작) 제거
/// 4. 이메일 토큰(@ 포함) 제거
/// 5. 공백 정리
/// 6. 200자 단어 경계 절단 (초과 시 … 추가)
pub fn clean_snippet(s: &str) -> String {
    // 1. 마크다운 헤더 제거 + 줄 수집
    let lines: Vec<&str> = s
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect();

    let raw = lines.join(" ");

    // 2. HTML 태그 제거 (문자 단위 파싱)
    let mut no_html = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => no_html.push(ch),
            _ => {}
        }
    }

    // 3, 4. URL/이메일 토큰 제거 + 공백 정리
    let cleaned: Vec<&str> = no_html
        .split_whitespace()
        .filter(|token| {
            !token.starts_with("http://") && !token.starts_with("https://") && !token.contains('@')
        })
        .collect();

    let joined = cleaned.join(" ");

    // 6. 200자 단어 경계 절단
    if joined.chars().count() <= 200 {
        return joined;
    }

    // 200자 이하에서 마지막 공백 찾기
    let cutoff: String = joined.chars().take(200).collect();
    if let Some(last_space) = cutoff.rfind(' ') {
        format!("{}…", &cutoff[..last_space])
    } else {
        format!("{}…", cutoff)
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
    fn clean_snippet_removes_markdown_headers() {
        let input = "# 제목\n## 소제목\n본문 내용입니다.";
        let result = clean_snippet(input);
        assert!(!result.contains('#'));
        assert!(result.contains("본문 내용입니다."));
    }

    #[test]
    fn clean_snippet_removes_html_tags() {
        let input = "<p>본문</p> <a href='https://example.com'>링크</a>";
        let result = clean_snippet(input);
        assert!(!result.contains('<'));
        assert!(!result.contains('>'));
        assert!(result.contains("본문"));
    }

    #[test]
    fn clean_snippet_removes_urls() {
        let input = "기사 내용 https://example.com/article 이후 텍스트";
        let result = clean_snippet(input);
        assert!(!result.contains("https://"));
        assert!(result.contains("기사 내용"));
        assert!(result.contains("이후 텍스트"));
    }

    #[test]
    fn clean_snippet_removes_http_urls() {
        let input = "기사 http://example.com 내용";
        let result = clean_snippet(input);
        assert!(!result.contains("http://"));
    }

    #[test]
    fn clean_snippet_removes_emails() {
        let input = "문의: contact@example.com 또는 test@test.org";
        let result = clean_snippet(input);
        assert!(!result.contains('@'));
        assert!(result.contains("문의:"));
    }

    #[test]
    fn clean_snippet_word_boundary_cut_at_200() {
        // 200자를 초과하는 입력 — 단어 경계에서 절단
        let words: Vec<String> = (0..50).map(|i| format!("word{i}")).collect();
        let input = words.join(" ");
        let result = clean_snippet(&input);
        assert!(result.chars().count() <= 204); // 200 + "…" (1char) + 여유
        assert!(result.ends_with('…') || result.chars().count() <= 200);
    }

    #[test]
    fn clean_snippet_short_input_unchanged() {
        let input = "짧은 텍스트";
        let result = clean_snippet(input);
        assert_eq!(result, "짧은 텍스트");
    }

    #[test]
    fn clean_snippet_cleans_combined() {
        let input = "# 헤더\n<b>굵은</b> 텍스트 https://url.com 그리고 email@test.com 이후";
        let result = clean_snippet(input);
        assert!(!result.contains('#'));
        assert!(!result.contains('<'));
        assert!(!result.contains("https://"));
        assert!(!result.contains('@'));
        assert!(result.contains("굵은"));
        assert!(result.contains("텍스트"));
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
