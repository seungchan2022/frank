//! 영구/간헐 판정 + 에러 카테고리 분류.
//!
//! SSOT 진단 설계 §실패 처리:
//! - 재시도 1회 (HTTP 5xx / timeout / network 한정)
//! - 200+빈결과 또는 4xx는 재시도 제외
//! - 같은 카테고리 반복 → 영구, 다른 결과 → 간헐
//!
//! 본 모듈은 **분류 로직만** 담당. 실제 재시도 실행은 `runner` 가 한다.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    RateLimit, // 429
    Auth,      // 401 / 403
    ServerErr, // 5xx
    Timeout,   // 요청 타임아웃
    Network,   // 연결/DNS/소켓
    JsonParse, // 응답 파싱 실패
    /// partial 파일 쓰기 실패 등 — runner에서 직접 분류·기록 시 사용 예정.
    #[allow(dead_code)]
    FileIo,
    /// DB 조회 실패 — main에서 직접 매핑·기록 시 사용 예정.
    #[allow(dead_code)]
    Db,
}

impl ErrorCategory {
    pub fn as_label(self) -> &'static str {
        match self {
            ErrorCategory::RateLimit => "rate_limit",
            ErrorCategory::Auth => "auth",
            ErrorCategory::ServerErr => "5xx",
            ErrorCategory::Timeout => "timeout",
            ErrorCategory::Network => "network",
            ErrorCategory::JsonParse => "json_parse",
            ErrorCategory::FileIo => "file_io",
            ErrorCategory::Db => "db",
        }
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_label())
    }
}

/// 에러 메시지 / 상태 코드를 카테고리로 분류.
/// SearchPort 어댑터가 `AppError::Internal(String)` 으로 메시지를 wrap 하므로
/// 문자열 패턴으로 분기. (정밀도 < 신뢰성: 진단 보고서에서 narrative로 보강)
pub fn classify(message: &str) -> ErrorCategory {
    let lower = message.to_lowercase();
    if lower.contains("429") || lower.contains("rate limit") {
        ErrorCategory::RateLimit
    } else if lower.contains("401")
        || lower.contains("403")
        || lower.contains("unauthor")
        || lower.contains("forbidden")
    {
        ErrorCategory::Auth
    } else if lower.contains("500")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("5xx")
    {
        ErrorCategory::ServerErr
    } else if lower.contains("timeout") || lower.contains("timed out") {
        ErrorCategory::Timeout
    } else if lower.contains("dns")
        || lower.contains("connect")
        || lower.contains("network")
        || lower.contains("tcp")
    {
        ErrorCategory::Network
    } else if lower.contains("parse") || lower.contains("json") {
        ErrorCategory::JsonParse
    } else {
        // 기본값: 네트워크로 분류해 재시도 후보로 둠 (보고서에서 narrative 보강)
        ErrorCategory::Network
    }
}

/// 해당 카테고리가 재시도 대상인지. SSOT: 5xx / timeout / network 한정.
pub fn is_retryable(cat: ErrorCategory) -> bool {
    matches!(
        cat,
        ErrorCategory::ServerErr | ErrorCategory::Timeout | ErrorCategory::Network
    )
}

/// 영구/간헐 판정. 재시도 1회 후 1차/2차 카테고리 비교.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    /// 재시도 안 함 (4xx / json_parse / 기타) — 즉시 영구로 본다.
    PermanentNoRetry,
    /// 재시도했고 같은 카테고리 → 영구
    PermanentSameCategory,
    /// 재시도했고 성공 → 간헐
    Intermittent,
    /// 재시도했는데 카테고리만 다름 → 간헐 (외부 요인 시사)
    IntermittentDifferent,
}

pub fn judge_with_retry(
    first: ErrorCategory,
    second: Option<Result<(), ErrorCategory>>,
) -> FailureKind {
    if !is_retryable(first) {
        return FailureKind::PermanentNoRetry;
    }
    match second {
        None => FailureKind::PermanentNoRetry, // 재시도 정책상 retryable이지만 시도 안 함 (이론상 호출 안됨)
        Some(Ok(())) => FailureKind::Intermittent,
        Some(Err(c2)) if c2 == first => FailureKind::PermanentSameCategory,
        Some(Err(_)) => FailureKind::IntermittentDifferent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T-03: 카테고리 분류
    #[test]
    fn classify_buckets() {
        assert_eq!(
            classify("Tavily returned 429: rate limit"),
            ErrorCategory::RateLimit
        );
        assert_eq!(
            classify("Exa returned 401 unauthorized"),
            ErrorCategory::Auth
        );
        assert_eq!(
            classify("Tavily returned 503 Service Unavailable"),
            ErrorCategory::ServerErr
        );
        assert_eq!(classify("request timed out"), ErrorCategory::Timeout);
        assert_eq!(classify("dns lookup failed"), ErrorCategory::Network);
        assert_eq!(
            classify("Tavily parse failed: invalid json"),
            ErrorCategory::JsonParse
        );
    }

    // T-03: 재시도 대상은 5xx / timeout / network
    #[test]
    fn retryable_only_for_three_categories() {
        assert!(is_retryable(ErrorCategory::ServerErr));
        assert!(is_retryable(ErrorCategory::Timeout));
        assert!(is_retryable(ErrorCategory::Network));
        assert!(!is_retryable(ErrorCategory::RateLimit));
        assert!(!is_retryable(ErrorCategory::Auth));
        assert!(!is_retryable(ErrorCategory::JsonParse));
        assert!(!is_retryable(ErrorCategory::FileIo));
        assert!(!is_retryable(ErrorCategory::Db));
    }

    // T-03: 영구/간헐 판정
    #[test]
    fn judge_no_retry_for_4xx() {
        let kind = judge_with_retry(ErrorCategory::Auth, None);
        assert_eq!(kind, FailureKind::PermanentNoRetry);
    }

    #[test]
    fn judge_intermittent_when_retry_succeeds() {
        let kind = judge_with_retry(ErrorCategory::Timeout, Some(Ok(())));
        assert_eq!(kind, FailureKind::Intermittent);
    }

    #[test]
    fn judge_permanent_when_same_category_repeats() {
        let kind = judge_with_retry(
            ErrorCategory::ServerErr,
            Some(Err(ErrorCategory::ServerErr)),
        );
        assert_eq!(kind, FailureKind::PermanentSameCategory);
    }

    #[test]
    fn judge_intermittent_different_when_category_changes() {
        let kind = judge_with_retry(ErrorCategory::Network, Some(Err(ErrorCategory::Timeout)));
        assert_eq!(kind, FailureKind::IntermittentDifferent);
    }
}
