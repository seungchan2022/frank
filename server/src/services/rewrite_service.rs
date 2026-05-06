use std::time::Duration;

use tokio::time::timeout;
use url_jail::{Policy, validate};
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::ports::{CrawlPort, FavoritesPort, LlmPort};

/// MVP7 M1과 동일한 60초 타임아웃 사용
const REWRITE_TIMEOUT_SECS: u64 = 60;

/// MVP15 M3: 직업 시각 재작성 오케스트레이션.
///
/// - occupation은 호출자(핸들러)가 사전 검증 후 전달 (빈 문자열 불가)
/// - SSRF 방어: url_jail::validate로 private IP / loopback / cloud metadata 차단
/// - 타임아웃: crawl + LLM 전체를 60초로 제한 → AppError::Timeout
/// - favorites 업데이트: url이 favorites에 없어도 에러 없음 (upsert — C3)
/// - favorites 업데이트 실패 시: warn 로그만 남기고 재작성 결과 정상 반환
#[allow(clippy::too_many_arguments)]
pub async fn rewrite_with_occupation<C, L, F>(
    url: &str,
    title: &str,
    snippet: Option<&str>,
    user_id: Uuid,
    occupation: &str,
    crawl: &C,
    llm: &L,
    favorites: &F,
) -> Result<String, AppError>
where
    C: CrawlPort + ?Sized,
    L: LlmPort + ?Sized,
    F: FavoritesPort + ?Sized,
{
    // SSRF 방어
    validate(url, Policy::PublicOnly)
        .await
        .map_err(|e| AppError::BadRequest(format!("URL not allowed: {e}")))?;

    let occ_owned = occupation.to_string();

    // crawl + LLM 전체를 60초 타임아웃으로 감싸기
    let result = timeout(Duration::from_secs(REWRITE_TIMEOUT_SECS), async {
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

        let rewrite = llm
            .rewrite_with_occupation(title, &content, &occ_owned)
            .await
            .map_err(|e| {
                AppError::ServiceUnavailable(format!("재작성 서비스를 사용할 수 없습니다: {e}"))
            })?;

        Ok::<String, AppError>(rewrite)
    })
    .await
    .map_err(|_| AppError::Timeout("재작성 요청이 시간을 초과했습니다 (60초)".to_string()))??;

    // favorites DB 업데이트 (실패해도 사용자 응답에 영향 없음 — best-effort)
    // MVP16 M3: occupation도 함께 저장 (C2-bug 수정 — 재작성 당시 직업 추적)
    if let Err(e) = favorites
        .update_favorite_rewrite(user_id, url, &result, Some(occupation))
        .await
    {
        tracing::warn!(
            user_id = %user_id,
            url = %url,
            error = %e,
            "favorites rewrite update failed — returning result anyway"
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
    async fn happy_path_returns_rewrite() {
        let crawl = FakeCrawlAdapter::new();
        let llm = FakeLlmAdapter::new();
        let favorites = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = rewrite_with_occupation(
            "https://example.com/article",
            "Test Article",
            None,
            user_id,
            "iOS 개발자",
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("iOS 개발자"));
        assert!(text.contains("Test Article"));
        // rewrite 결과에 occupation과 title이 포함되어 있는지 확인
        assert!(text.contains("Test Article"));
    }

    #[tokio::test]
    async fn ssrf_loopback_blocked() {
        let crawl = FakeCrawlAdapter::new();
        let llm = FakeLlmAdapter::new();
        let favorites = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = rewrite_with_occupation(
            "http://127.0.0.1/secret",
            "Test",
            None,
            user_id,
            "개발자",
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    #[tokio::test]
    async fn crawl_failure_with_snippet_falls_back_and_succeeds() {
        let crawl = FakeCrawlAdapter::failing();
        let llm = FakeLlmAdapter::new();
        let favorites = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = rewrite_with_occupation(
            "https://example.com/article",
            "Test",
            Some("snippet content"),
            user_id,
            "iOS 개발자",
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

        let result = rewrite_with_occupation(
            "https://example.com/article",
            "Test",
            None,
            user_id,
            "iOS 개발자",
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(result.is_ok(), "title 폴백 시 성공: {result:?}");
    }

    #[tokio::test]
    async fn llm_failure_returns_service_unavailable() {
        let crawl = FakeCrawlAdapter::new();
        let llm = FakeLlmAdapter::failing();
        let favorites = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = rewrite_with_occupation(
            "https://example.com/article",
            "Test",
            None,
            user_id,
            "iOS 개발자",
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(matches!(result, Err(AppError::ServiceUnavailable(_))));
    }

    #[tokio::test]
    async fn favorites_update_failure_still_returns_rewrite() {
        let crawl = FakeCrawlAdapter::new();
        let llm = FakeLlmAdapter::new();
        let favorites = FakeFavoritesAdapter::failing();
        let user_id = Uuid::new_v4();

        let result = rewrite_with_occupation(
            "https://example.com/article",
            "Test",
            None,
            user_id,
            "iOS 개발자",
            &crawl,
            &llm,
            &favorites,
        )
        .await;

        assert!(result.is_ok());
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
                rewrite_with_occupation(
                    "https://8.8.8.8/article",
                    "Test",
                    None,
                    user_id,
                    "iOS 개발자",
                    &*crawl,
                    &*llm,
                    &*favorites,
                )
                .await
            }
        });

        tokio::time::advance(Duration::from_secs(61)).await;

        let result = task.await.unwrap();
        assert!(matches!(result, Err(AppError::Timeout(_))));
    }
}
