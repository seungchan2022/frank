use reqwest::{RequestBuilder, Response, StatusCode};
use std::future::Future;
use std::time::Duration;

use crate::domain::error::AppError;

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
}
