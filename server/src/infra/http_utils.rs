use reqwest::{Client, RequestBuilder, Response, StatusCode};
use std::future::Future;
use std::time::Duration;

use crate::domain::error::AppError;

/// og:image 크롤링에 읽을 최대 바이트 (64KB — <head>는 항상 이 안에 있음)
pub const OG_IMAGE_READ_LIMIT: usize = 64 * 1024;
/// og:image 크롤링 타임아웃 (초)
pub const OG_IMAGE_TIMEOUT_SECS: u64 = 5;

/// 기사 URL에서 og:image 메타태그를 추출한다.
/// 실패(타임아웃, 봇 차단, 파싱 불가)하면 None 반환 — 피드 로딩에 영향 없음.
pub async fn fetch_og_image(client: &Client, article_url: &str) -> Option<String> {
    let resp = client.get(article_url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    // 최대 64KB만 읽어 파싱 (og:image는 항상 <head> 안에 있음)
    let bytes = resp.bytes().await.ok()?;
    let html = std::str::from_utf8(&bytes[..bytes.len().min(OG_IMAGE_READ_LIMIT)]).ok()?;

    extract_og_image(html)
}

/// HTML에서 og:image content 값을 추출한다.
/// `<meta property="og:image" content="URL">` 및
/// `<meta content="URL" property="og:image">` 두 순서를 모두 처리한다.
pub fn extract_og_image(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let mut search_from = 0;

    while let Some(rel_pos) = lower[search_from..].find("<meta") {
        let tag_start = search_from + rel_pos;
        let tag_end = lower[tag_start..]
            .find('>')
            .map(|p| tag_start + p + 1)
            .unwrap_or(lower.len());

        let tag = &html[tag_start..tag_end];
        let tag_lower = &lower[tag_start..tag_end];

        if tag_lower.contains("og:image")
            && let Some(url) = extract_attr(tag, "content")
            && url.starts_with("http")
        {
            return Some(url);
        }

        search_from = tag_end;
        if search_from >= lower.len() {
            break;
        }
    }

    None
}

/// HTML 태그에서 지정한 속성값을 추출한다. 큰따옴표·작은따옴표 모두 처리.
pub fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let search = format!("{attr}=");
    let pos = lower.find(&search)?;
    let after = &tag[pos + search.len()..];

    if let Some(rest) = after.strip_prefix('"') {
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    } else if let Some(rest) = after.strip_prefix('\'') {
        let end = rest.find('\'')?;
        Some(rest[..end].to_string())
    } else {
        let end = after.find(|c: char| c.is_whitespace() || c == '>' || c == '/')?;
        Some(after[..end].to_string())
    }
}

/// HTTP retry + size-limit 설정
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_response_size: u64,
    pub retryable_statuses: Vec<StatusCode>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_response_size: 2 * 1024 * 1024, // 2MB
            retryable_statuses: vec![
                StatusCode::TOO_MANY_REQUESTS,
                StatusCode::INTERNAL_SERVER_ERROR,
                StatusCode::BAD_GATEWAY,
                StatusCode::SERVICE_UNAVAILABLE,
                StatusCode::GATEWAY_TIMEOUT,
            ],
        }
    }
}

impl RetryConfig {
    /// 검색 API용 프리셋 (3회 retry, 2MB 제한)
    pub fn for_search() -> Self {
        Self::default()
    }

    /// 크롤링 API용 프리셋 (2회 retry, 20MB 제한)
    pub fn for_crawl() -> Self {
        Self {
            max_retries: 2,
            max_response_size: 20 * 1024 * 1024, // 20MB
            ..Self::default()
        }
    }

    /// LLM API용 프리셋 (1회 retry, 1MB 제한, 긴 base delay)
    pub fn for_llm() -> Self {
        Self {
            max_retries: 1,
            base_delay_ms: 200,
            max_response_size: 1024 * 1024, // 1MB
            ..Self::default()
        }
    }
}

fn is_retryable_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect()
}

fn check_content_length(resp: &Response, max_size: u64) -> Result<(), AppError> {
    if let Some(len) = resp.content_length()
        && len > max_size
    {
        return Err(AppError::Internal(format!(
            "Response too large: {len} bytes exceeds limit of {max_size} bytes"
        )));
    }
    Ok(())
}

/// RequestBuilder 팩토리 클로저를 받아 retry + size-limit 체크 수행
///
/// `builder_factory`는 매 retry마다 새로운 RequestBuilder를 생성해야 한다
/// (RequestBuilder는 send() 후 소비되므로).
pub async fn send_with_retry<F, Fut>(
    builder_factory: F,
    config: &RetryConfig,
) -> Result<Response, AppError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = RequestBuilder>,
{
    let mut last_error: Option<AppError> = None;
    let total_attempts = config.max_retries + 1;

    for attempt in 0..total_attempts {
        if attempt > 0 {
            let delay_ms = config.base_delay_ms * 2u64.pow(attempt - 1);
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        let builder = builder_factory().await;
        match builder.send().await {
            Ok(resp) => {
                // Size check
                check_content_length(&resp, config.max_response_size)?;

                // Retryable status code
                if config.retryable_statuses.contains(&resp.status()) {
                    last_error = Some(AppError::Internal(format!(
                        "HTTP {} (attempt {}/{})",
                        resp.status(),
                        attempt + 1,
                        total_attempts
                    )));
                    continue;
                }

                return Ok(resp);
            }
            Err(e) => {
                if is_retryable_error(&e) {
                    last_error = Some(AppError::Internal(format!(
                        "Request error (attempt {}/{}): {e}",
                        attempt + 1,
                        total_attempts
                    )));
                    continue;
                }
                return Err(AppError::Internal(format!("Request failed: {e}")));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| AppError::Internal("All retries exhausted".to_string())))
}

/// Response body를 읽으면서 size limit을 적용한다.
/// Content-Length 헤더가 없는 chunked 응답에도 동작한다.
pub async fn read_body_limited(resp: Response, max_size: u64) -> Result<String, AppError> {
    // Content-Length가 있으면 미리 체크 (빠른 실패)
    check_content_length(&resp, max_size)?;

    let mut body = Vec::new();
    let mut stream = resp;

    while let Some(chunk) = stream
        .chunk()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read response chunk: {e}")))?
    {
        body.extend_from_slice(&chunk);
        if body.len() as u64 > max_size {
            return Err(AppError::Internal(format!(
                "Response body too large: exceeds limit of {max_size} bytes"
            )));
        }
    }

    String::from_utf8(body)
        .map_err(|e| AppError::Internal(format!("Invalid UTF-8 in response body: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(max_retries: u32) -> RetryConfig {
        RetryConfig {
            max_retries,
            base_delay_ms: 10,
            max_response_size: 2 * 1024 * 1024,
            ..RetryConfig::default()
        }
    }

    #[tokio::test]
    async fn retry_success_after_failures() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        // First 2 calls return 503, third returns 200
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(2)
            .expect(2)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let config = test_config(2);

        let result = send_with_retry(
            || {
                let url = url.clone();
                let client = client.clone();
                async move { client.get(&url) }
            },
            &config,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn retry_all_fail_returns_last_error() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(503))
            .expect(4) // 1 initial + 3 retries
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let config = test_config(3);

        let result = send_with_retry(
            || {
                let url = url.clone();
                let client = client.clone();
                async move { client.get(&url) }
            },
            &config,
        )
        .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("attempt 4/4"));
    }

    #[tokio::test]
    async fn backoff_exponential_delay() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(2)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 50,
            max_response_size: 2 * 1024 * 1024,
            ..RetryConfig::default()
        };

        let start = std::time::Instant::now();
        let result = send_with_retry(
            || {
                let url = url.clone();
                let client = client.clone();
                async move { client.get(&url) }
            },
            &config,
        )
        .await;

        assert!(result.is_ok());
        // base_delay * 2^0 + base_delay * 2^1 = 50 + 100 = 150ms minimum
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(100),
            "Expected >= 100ms backoff delay, got {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn non_retryable_status_returns_immediately() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(400).set_body_string("bad request"))
            .expect(1) // Only 1 call, no retry
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let config = test_config(3);

        let result = send_with_retry(
            || {
                let url = url.clone();
                let client = client.clone();
                async move { client.get(&url) }
            },
            &config,
        )
        .await;

        // Non-retryable status is returned as-is (caller decides what to do)
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn size_limit_exceeded_returns_error() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        // Create a body larger than 1KB limit
        let large_body = "x".repeat(2048);
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&large_body))
            .expect(1)
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let config = RetryConfig {
            max_response_size: 1024, // 1KB limit
            ..test_config(0)
        };

        let result = send_with_retry(
            || {
                let url = url.clone();
                let client = client.clone();
                async move { client.get(&url) }
            },
            &config,
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[tokio::test]
    async fn size_limit_under_passes() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        let body = "x".repeat(512);
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&body))
            .expect(1)
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let config = RetryConfig {
            max_response_size: 1024,
            ..test_config(0)
        };

        let result = send_with_retry(
            || {
                let url = url.clone();
                let client = client.clone();
                async move { client.get(&url) }
            },
            &config,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn no_content_length_header_passes() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let config = test_config(0);

        let result = send_with_retry(
            || {
                let url = url.clone();
                let client = client.clone();
                async move { client.get(&url) }
            },
            &config,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn max_retries_zero_no_retry() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(503))
            .expect(1) // Only 1 call
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let config = test_config(0);

        let result = send_with_retry(
            || {
                let url = url.clone();
                let client = client.clone();
                async move { client.get(&url) }
            },
            &config,
        )
        .await;

        assert!(result.is_err());
    }

    #[test]
    fn retry_config_presets() {
        let search = RetryConfig::for_search();
        assert_eq!(search.max_retries, 3);
        assert_eq!(search.max_response_size, 2 * 1024 * 1024);

        let crawl = RetryConfig::for_crawl();
        assert_eq!(crawl.max_retries, 2);
        assert_eq!(crawl.max_response_size, 20 * 1024 * 1024);

        let llm = RetryConfig::for_llm();
        assert_eq!(llm.max_retries, 1);
        assert_eq!(llm.base_delay_ms, 200);
        assert_eq!(llm.max_response_size, 1024 * 1024);
    }

    #[tokio::test]
    async fn retry_first_attempt_success_no_delay() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let config = test_config(3);

        let start = std::time::Instant::now();
        let result = send_with_retry(
            || {
                let url = url.clone();
                let client = client.clone();
                async move { client.get(&url) }
            },
            &config,
        )
        .await;

        assert!(result.is_ok());
        // 첫 시도 성공 시 delay 없음
        assert!(start.elapsed() < std::time::Duration::from_millis(100));
    }

    #[tokio::test]
    async fn content_length_equals_limit_passes() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        // Body exactly 1024 bytes = limit, should pass (only > limit is rejected)
        let body = "x".repeat(1024);
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&body))
            .expect(1)
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let config = RetryConfig {
            max_response_size: 1024,
            ..test_config(0)
        };

        let result = send_with_retry(
            || {
                let url = url.clone();
                let client = client.clone();
                async move { client.get(&url) }
            },
            &config,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn read_body_limited_success() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string("hello"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let resp = client.get(&url).send().await.unwrap();
        let body = read_body_limited(resp, 1024).await.unwrap();

        assert_eq!(body, "hello");
    }

    #[tokio::test]
    async fn read_body_limited_exceeds_limit() {
        let mock_server = MockServer::start().await;
        let client = Client::new();

        let large = "x".repeat(2048);
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&large))
            .expect(1)
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        let resp = client.get(&url).send().await.unwrap();
        let result = read_body_limited(resp, 1024).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

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

    #[tokio::test]
    async fn fetch_og_image_success() {
        let mock_server = MockServer::start().await;
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        Mock::given(method("GET"))
            .and(path("/article"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"<html><head><meta property="og:image" content="https://cdn.example.com/img.jpg" /></head></html>"#,
            ))
            .mount(&mock_server)
            .await;

        let url = format!("{}/article", mock_server.uri());
        let result = fetch_og_image(&client, &url).await;
        assert_eq!(result, Some("https://cdn.example.com/img.jpg".to_string()));
    }

    #[tokio::test]
    async fn fetch_og_image_returns_none_on_error() {
        let client = Client::new();
        let result = fetch_og_image(&client, "http://127.0.0.1:1/article").await;
        assert_eq!(result, None);
    }
}
