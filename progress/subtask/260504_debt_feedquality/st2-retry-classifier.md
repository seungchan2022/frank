# ST-2: B-1 retry_classifier 402 → quota_exhausted

> 브랜치: feature/260504_feed-quality-debt-cleanup  
> 파일: server/src/bin/diagnose/retry_classifier.rs

## 요약
Exa 쿼터 초과(402) 응답을 Network 폴백이 아닌 QuotaExhausted 카테고리로 분류.

## Feature List
<!-- size: 소형 | count: 5 | skip: true -->

### 기능
- [x] F-01 ErrorCategory enum에 QuotaExhausted 추가
- [x] F-02 classify()에 "402" / "payment required" 패턴 매핑 — 401 분기보다 앞에 삽입 (흡수 방지)
- [x] F-03 `QuotaExhausted`에 대해 `as_label()` → `"quota_exhausted"`, `Display` → `"quota_exhausted"` 반환
- [x] F-04 `is_retryable()` match에 `QuotaExhausted => false` arm 추가

### 엣지
- [x] E-01 "Payment Required" 대소문자 무관 매핑 (to_lowercase 후 비교)

### 에러
- [x] R-01 기존 분류(Network, Auth 등) 회귀 없음

### 테스트
- [x] T-01 classify("402 payment required") == QuotaExhausted 단위 테스트
- [x] T-02 is_retryable(QuotaExhausted) == false 검증
- [x] T-03 as_label(QuotaExhausted) == "quota_exhausted" 검증
