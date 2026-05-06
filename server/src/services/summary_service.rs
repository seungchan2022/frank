use std::time::Duration;

use tokio::time::timeout;
use url_jail::{Policy, validate};
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::LlmResponse;
use crate::domain::ports::{CrawlPort, FavoritesPort, LlmPort};

/// MVP7 M1: 60초로 증가 (OpenRouter HTTP timeout 60초와 일치)
const SUMMARIZE_TIMEOUT_SECS: u64 = 60;

/// 레거시: API 핸들러에서는 미사용 — 활성 엔드포인트는 `summarize_with_occupation` 사용.
/// insight 필드를 파싱하지 않음 (항상 None). 테스트 및 하위 호환성 목적으로 유지.
///
/// - SSRF 방어: url_jail::validate로 private IP / loopback / cloud metadata 차단
/// - 타임아웃: crawl + LLM 전체를 60초로 제한 → AppError::Timeout
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

    // crawl + LLM 전체를 60초 타임아웃으로 감싸기
    let result = timeout(Duration::from_secs(SUMMARIZE_TIMEOUT_SECS), async {
        let content = crawl.scrape(url).await.map_err(|e| {
            AppError::UnprocessableEntity(format!("콘텐츠를 가져올 수 없습니다: {e}"))
        })?;

        let response = llm.summarize(title, &content).await.map_err(|e| {
            AppError::ServiceUnavailable(format!("요약 서비스를 사용할 수 없습니다: {e}"))
        })?;

        Ok::<LlmResponse, AppError>(response)
    })
    .await
    .map_err(|_| AppError::Timeout("요약 요청이 시간을 초과했습니다 (60초)".to_string()))??;

    // favorites DB 업데이트 (실패해도 사용자 응답에 영향 없음)
    if let Err(e) = favorites
        .update_favorite_summary(
            user_id,
            url,
            &result.summary.summary,
            result.summary.insight.as_deref(),
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

/// C3: occupation 무관 항상 insight non-null 반환.
///
/// - occupation 인자는 시그니처 호환 유지 목적으로 수신하나 LLM 호출에 전달하지 않음.
///   (향후 `rewrite_with_occupation` 경로와의 인터페이스 일관성 보존)
/// - DB 장애 시 occupation=None으로 degrade 가능 — insight 반환에 영향 없음.
#[allow(clippy::too_many_arguments)]
pub async fn summarize_with_occupation<'a, C, L, F>(
    url: &'a str,
    title: &'a str,
    snippet: Option<&'a str>,
    user_id: Uuid,
    occupation: Option<&'a str>,
    crawl: &'a C,
    llm: &'a L,
    favorites: &'a F,
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

    let occ_owned = occupation.map(|s| s.to_string());

    let result = timeout(Duration::from_secs(SUMMARIZE_TIMEOUT_SECS), async {
        let content = match crawl.scrape(url).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(url = %url, error = %e, "Firecrawl 실패 — snippet/title 폴백");
                snippet
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or(title)
                    .to_string()
            }
        };

        let response = llm
            .summarize_with_occupation(title, &content, occ_owned.as_deref())
            .await
            .map_err(|e| {
                AppError::ServiceUnavailable(format!("요약 서비스를 사용할 수 없습니다: {e}"))
            })?;

        Ok::<LlmResponse, AppError>(response)
    })
    .await
    .map_err(|_| AppError::Timeout("요약 요청이 시간을 초과했습니다 (60초)".to_string()))??;

    if let Err(e) = favorites
        .update_favorite_summary(
            user_id,
            url,
            &result.summary.summary,
            result.summary.insight.as_deref(),
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
    async fn crawl_failure_with_snippet_falls_back_and_succeeds() {
        let crawl = FakeCrawlAdapter::failing();
        let llm = FakeLlmAdapter::new();
        let favorites = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = summarize_with_occupation(
            "https://example.com/article",
            "Test Article",
            Some("snippet content"),
            user_id,
            None,
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(result.is_ok(), "snippet 폴백 시 성공: {result:?}");
    }

    #[tokio::test]
    async fn crawl_failure_without_snippet_falls_back_to_title() {
        let crawl = FakeCrawlAdapter::failing();
        let llm = FakeLlmAdapter::new();
        let favorites = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = summarize_with_occupation(
            "https://example.com/article",
            "Test Article",
            None,
            user_id,
            None,
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(result.is_ok(), "title 폴백 시 성공: {result:?}");
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
    async fn legacy_summarize_crawl_failure_returns_unprocessable_entity() {
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

        assert!(
            matches!(result, Err(AppError::UnprocessableEntity(_))),
            "크롤 실패는 422 UnprocessableEntity: {result:?}"
        );
    }

    #[tokio::test]
    async fn llm_failure_returns_service_unavailable() {
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

        assert!(
            matches!(result, Err(AppError::ServiceUnavailable(_))),
            "LLM 실패는 503 ServiceUnavailable: {result:?}"
        );
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

        // MVP7 M1: 60초 타임아웃 초과
        tokio::time::advance(Duration::from_secs(61)).await;

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
