# 피드 품질 개선 기능 구현 현황 + 부채 정합성 검증

> 생성: 2026-05-04  
> 분석 유형: code-quality  
> 대상: server/src/api/feed.rs, infra/, services/, .claude/skills/e2e/, web/e2e/, ios/FrankUITests/

---

## 1. 피드 품질 개선 4가지 기능 구현 현황

### 1-1. 키워드 매핑 정밀화 (`tag_search_keyword()`)

**판정: 구현됨**

- 파일: `server/src/api/feed.rs:33-49`
- 한국어 태그명 → 영어 검색 키워드 매핑 함수 완전 구현
- 12개 태그 매핑 (모바일 개발, 웹 개발, AI/ML, 클라우드/인프라, 보안, 데이터 사이언스, 블록체인, 스타트업, 투자/VC, 프로덕트, UX/디자인, 오픈소스)
- 미매핑 태그는 원본 그대로(`other => other`) 반환 — fallback 처리
- 실제 쿼리 구성: `"{keyword} latest news{personalization_suffix}"` (feed.rs:305)
- 테스트에서도 `tag_search_keyword()` 반환값 기반으로 expected query 구성 (feed.rs:1441, 1498, 1554 등)

### 1-2. 중복 기사 제거 (dedup)

**판정: 구현됨**

- 파일: `server/src/api/feed.rs:384-387`
- URL 정규화 기반 중복 제거: `normalize_url()` 함수 (feed.rs:451-466)
- 정규화 범위: 스킴(http/https) 제거, `www.` 제거, trailing slash 제거, 소문자 통일
- `HashSet<String>` 기반 `retain` 패턴으로 인플레이스 중복 제거
- 단위 테스트: `get_feed_deduplicates_by_url` (feed.rs:799)

### 1-3. 발행 날짜 필터 (7일 이상 제외)

**판정: 미구현**

- `feed.rs` 전체에서 `published_at`을 기준으로 오래된 기사를 필터링하는 로직 없음
- `published_at`은 `FeedItem` 구조체 필드로 존재하고 응답에 포함되지만, 서버 측에서 날짜 기반 필터링을 수행하지 않음
- 날짜 관련 코드(days 포함)는 카운터 reset_at 계산 전용으로만 사용됨
- 검색 엔진(Tavily/Exa)이 반환한 기사의 발행일을 서버가 후처리로 거르지 않음
- **리스크**: 검색 쿼리에 "latest news" suffix를 붙여 최신성을 유도하지만, 서버 단 하드 컷오프는 없어 7일 이상 된 기사도 피드에 노출될 수 있음

### 1-4. 소스 다양성 제한 (동일 도메인 편중)

**판정: 미구현**

- 도메인 카운트 / per-domain 상한 로직 없음
- 기사 필터 체인은 listing URL 차단, 홈페이지 URL 차단, 짧은 snippet 차단(30자 미만) 세 가지뿐
- 동일 도메인에서 다수 기사가 반환되어도 그대로 피드에 포함
- 구현 참고: HashSet 기반 도메인 카운터 + 상한(예: 도메인당 3개)으로 `retain` 패턴 추가 가능

---

## 2. OPEN 부채 구현 여부 검증

### DEBT-08: E2E 자동화 기반 (`/e2e` 스킬)

**판정: 구현됨 (부채 해소)**

- `.claude/skills/e2e/SKILL.md` 존재 확인
- 스킬 상태: "M4 시나리오 완성 (웹 W-01~W-03, iOS I-01~I-05)"
- 웹: `web/e2e/` 디렉토리 — `feed-like.spec.ts`, `feed-summary.spec.ts`, `smoke.spec.ts`, `tag-navigation.spec.ts` 4개 파일 확인
- iOS: `ios/Frank/FrankUITests/` — `CrossFeatureFlowUITest.swift`, `FeedRefreshUITest.swift`, `LoginFlowUITest.swift`, `M3UXImprovementsUITest.swift`, `OnboardingFlowUITest.swift` 5개 UITest 파일 확인
- **결론: debts.md DEBT-08은 OPEN이지만 실제로는 스킬과 테스트 파일이 모두 존재. 상태를 RESOLVED로 갱신 필요.**

### DEBT-MVP15-01: `retry_classifier` 402 분류

**판정: 미구현 (부채 유효)**

- 파일: `server/src/bin/diagnose/retry_classifier.rs:52-83`
- `classify()` 함수: 401/403만 `Auth`로 분기. 402 Payment Required는 명시 처리 없음
- 402가 포함된 메시지는 `"429"`/`"rate limit"` 아닌 경우 → 숫자 문자열 "402"가 아무 조건에도 매칭 안 됨 → 기본값 `ErrorCategory::Network`로 fall락
- 실제 테스트(`classify_buckets`)에도 402 케이스 없음
- **결론: 부채 설명 정확. `quota_exhausted` 카테고리 신설 필요.**

### DEBT-MVP15-06: `infra → services` 역방향 레이어 의존 위반

**판정: 미구현 (부채 유효)**

- 파일: `server/src/infra/counted_search.rs:31`
- `use crate::services::notification_service::{AlertDispatch, dispatch_threshold_alert};` 직접 import 확인
- CLAUDE.md 명시 방향 `api → services → domain ← infra` 위반 실증
- **결론: 부채 설명 정확. 현재 코드에 그대로 위반 존재.**

### DEBT-MVP15-07: 피드 snippet invalid JSON escape

**판정: 부분 구현 (부채 유효, 완전 해소 아님)**

- `clean_snippet()` 함수는 `server/src/infra/exa.rs:43`에 구현, `tavily.rs`도 동일 함수 재사용
- 정제 범위: HTML 태그 제거, 마크다운 헤더, 플레이스홀더, 공백 정규화, 300자 절단
- **그러나**: `\4`, `\_` 같은 invalid JSON escape sequence 제거 로직은 `clean_snippet()` 내에 없음
- backslash 처리 코드는 `imessage.rs`의 AppleScript 전용 escape뿐, snippet JSON escape 처리와 무관
- **결론: snippet 클린업 기반은 있으나 invalid backslash escape 특화 처리는 미구현. 부채 유효.**

### DEBT-MVP15-02: 진단 바이너리 실행 환경 가정

**판정: 미구현 (부채 유효)**

- `server/src/bin/diagnose_search.rs` 존재 확인 (디렉토리: `server/src/bin/`)
- `scripts/run-diagnose.sh` 미존재, env 외부화 처리 없음
- 부채 설명의 임시 해결책이 여전히 유일한 실행 방법

### DEBT-MVP15-03: Tavily `effective_max` 동적 감지

**판정: 미구현 (부채 유효)**

- `tavily.rs`에서 Tavily 응답 헤더 기반 plan limit 추출 없음

### DEBT-MVP15-04: Exa `reset_at` NULL semantics

**판정: 미구현 (부채 유효)**

- `server/src/infra/counted_search.rs:157-163` `current_period_start()` 주석: "reset_at은 사용하지 않음 — Exa(NULL reset_at)도 동일 로직 적용 가능"
- 실제로 Postgres/InMemory 구현 모두 첫 record_call에서 reset_at을 month+1로 세팅 (의도 미반영)

### DEBT-MVP15-05: notification_service task leak

**판정: 미구현 (부채 유효)**

- `notification_service.rs:131` `tokio::task::spawn_blocking` + `tokio::time::timeout` 패턴 그대로
- `osascript` hang 시 blocking 스레드 누적 가능성 해결 안 됨

---

## 3. 요약 표

| 항목 | 판정 | 근거 위치 |
|------|------|-----------|
| tag_search_keyword() | ✅ 구현됨 | feed.rs:33-49 |
| 중복 기사 제거 (dedup) | ✅ 구현됨 | feed.rs:384-387, 451-466 |
| 발행 날짜 필터 (7일) | ❌ 미구현 | — |
| 소스 다양성 제한 | ❌ 미구현 | — |
| DEBT-08: /e2e 스킬 | ✅ 이미 구현 (부채 해소) | .claude/skills/e2e/SKILL.md, web/e2e/, ios/FrankUITests/ |
| DEBT-MVP15-01: 402 분류 | ❌ 미구현 | retry_classifier.rs:52-83 |
| DEBT-MVP15-02: 진단 실행 환경 | ❌ 미구현 | — |
| DEBT-MVP15-03: Tavily 동적 감지 | ❌ 미구현 | — |
| DEBT-MVP15-04: Exa reset_at | ❌ 미구현 | counted_search.rs:157 주석 |
| DEBT-MVP15-05: task leak | ❌ 미구현 | notification_service.rs:131 |
| DEBT-MVP15-06: infra→services | ❌ 미구현 | counted_search.rs:31 |
| DEBT-MVP15-07: snippet escape | ⚠️ 부분 구현 | exa.rs:43, tavily.rs:146 |

---

## 4. 즉시 조치 권장 사항

### A안: DEBT-08 상태 갱신 (낮은 비용, 즉각 정합성 회복)
`progress/debts.md`에서 DEBT-08을 RESOLVED로 변경. 스킬과 테스트 파일이 이미 존재하므로 코드 변경 없이 문서만 수정.

### B안: 발행 날짜 필터 + 소스 다양성 추가 (중간 비용)
`feed.rs` 기사 수집 루프 이후에 두 필터를 체인 추가:
1. `published_at` 7일 이상 → 제거 (None은 통과)
2. 도메인 추출 후 HashSet 카운트 → 도메인당 3개 초과 시 제거

### C안: DEBT-MVP15-07 snippet escape 수정 (낮은 비용, 중간 리스크 완화)
`clean_snippet()` 내에 invalid backslash sequence (`\` 뒤에 JSON 허용 문자 `"\/bfnrtu` 외의 문자) 제거 1줄 추가. M3 envelope 전환 전에 처리 필요.
