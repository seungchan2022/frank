use std::time::Duration;

use tokio::time::timeout;
use url_jail::{Policy, validate};
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::LlmResponse;
use crate::domain::ports::{CrawlPort, FavoritesPort, LlmPort};

const SUMMARIZE_TIMEOUT_SECS: u64 = 30;

/// URL 크롤링 + LLM 요약 오케스트레이션.
///
/// - SSRF 방어: url_jail::validate로 private IP / loopback / cloud metadata 차단
/// - 타임아웃: crawl + LLM 전체를 30초로 제한 → AppError::Timeout
/// - favorites 업데이트: url이 favorites에 없어도 에러 없음 (0행 업데이트)
/// - favorites 업데이트 실패 시: warn 로그만 남기고 요약 결과 정상 반환
pub async fn summarize<C, L, F>(
    url: &str,
    title: &str,
    user_id: Uuid,
    crawl: &C,
    llm: &L,
    favorites: &F,
) -> Result<LlmResponse, AppError>
where
    C: CrawlPort + ?Sized,
    L: LlmPort + ?Sized,
    F: FavoritesPort + ?Sized,
{
    // SSRF 방어
    validate(url, Policy::PublicOnly)
        .await
        .map_err(|e| AppError::BadRequest(format!("URL not allowed: {e}")))?;

    // crawl + LLM 전체를 30초 타임아웃으로 감싸기
    let result = timeout(Duration::from_secs(SUMMARIZE_TIMEOUT_SECS), async {
        let content = crawl
            .scrape(url)
            .await
            .map_err(|e| AppError::Internal(format!("crawl failed: {e}")))?;

        let response = llm
            .summarize(title, &content)
            .await
            .map_err(|e| AppError::Internal(format!("llm failed: {e}")))?;

        Ok::<LlmResponse, AppError>(response)
    })
    .await
    .map_err(|_| AppError::Timeout("요약 요청이 시간을 초과했습니다 (30초)".to_string()))??;

    // favorites DB 업데이트 (실패해도 사용자 응답에 영향 없음)
    if let Err(e) = favorites
        .update_favorite_summary(
            user_id,
            url,
            &result.summary.summary,
            &result.summary.insight,
        )
        .await
    {
        tracing::warn!(
            user_id = %user_id,
            url = %url,
            error = %e,
            "favorites summary update failed — returning result anyway"
        );
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_favorites::FakeFavoritesAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;

    #[tokio::test]
    async fn happy_path_returns_summary() {
        let crawl = FakeCrawlAdapter::new();
        let llm = FakeLlmAdapter::new();
        let favorites = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = summarize(
            "https://example.com/article",
            "Test Article",
            user_id,
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(favorites.update_call_count(), 1);
    }

    #[tokio::test]
    async fn ssrf_loopback_blocked() {
        let crawl = FakeCrawlAdapter::new();
        let llm = FakeLlmAdapter::new();
        let favorites = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = summarize(
            "http://127.0.0.1/secret",
            "Test",
            user_id,
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    #[tokio::test]
    async fn ssrf_private_ip_blocked() {
        let crawl = FakeCrawlAdapter::new();
        let llm = FakeLlmAdapter::new();
        let favorites = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = summarize(
            "http://10.0.0.1/internal",
            "Test",
            user_id,
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    #[tokio::test]
    async fn ssrf_cloud_metadata_blocked() {
        let crawl = FakeCrawlAdapter::new();
        let llm = FakeLlmAdapter::new();
        let favorites = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = summarize(
            "http://169.254.169.254/latest/meta-data/",
            "Test",
            user_id,
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    #[tokio::test]
    async fn crawl_failure_returns_internal_error() {
        let crawl = FakeCrawlAdapter::failing();
        let llm = FakeLlmAdapter::new();
        let favorites = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = summarize(
            "https://example.com/article",
            "Test",
            user_id,
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(matches!(result, Err(AppError::Internal(_))));
    }

    #[tokio::test]
    async fn llm_failure_returns_internal_error() {
        let crawl = FakeCrawlAdapter::new();
        let llm = FakeLlmAdapter::failing();
        let favorites = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = summarize(
            "https://example.com/article",
            "Test",
            user_id,
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(matches!(result, Err(AppError::Internal(_))));
    }

    #[tokio::test(start_paused = true)]
    async fn timeout_returns_timeout_error() {
        use std::sync::Arc;

        let crawl = Arc::new(FakeCrawlAdapter::sleeping());
        let llm = Arc::new(FakeLlmAdapter::new());
        let favorites = Arc::new(FakeFavoritesAdapter::new());
        let user_id = Uuid::new_v4();

        let task = tokio::spawn({
            let crawl = Arc::clone(&crawl);
            let llm = Arc::clone(&llm);
            let favorites = Arc::clone(&favorites);
            async move {
                summarize(
                    // IP 직접 사용 → url_jail DNS 해석 불필요 → start_paused 환경에서 안전
                    "https://8.8.8.8/article",
                    "Test",
                    user_id,
                    &*crawl,
                    &*llm,
                    &*favorites,
                )
                .await
            }
        });

        // 30초 타임아웃 초과
        tokio::time::advance(Duration::from_secs(31)).await;

        let result = task.await.unwrap();
        assert!(matches!(result, Err(AppError::Timeout(_))));
    }

    #[tokio::test]
    async fn favorites_update_failure_still_returns_summary() {
        let crawl = FakeCrawlAdapter::new();
        let llm = FakeLlmAdapter::new();
        let favorites = FakeFavoritesAdapter::failing();
        let user_id = Uuid::new_v4();

        // favorites 업데이트 실패해도 200 정상 반환
        let result = summarize(
            "https://example.com/article",
            "Test",
            user_id,
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(favorites.update_call_count(), 1);
    }
}
