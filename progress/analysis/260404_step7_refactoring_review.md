# Step 7: 리팩토링 3R + 코드 리뷰 3R 통합 보고서

- 날짜: 2026-04-04
- 대상: 서버(Rust) 전체 + 웹(Svelte) 전체

## Critical (즉시 수정)

| # | 영역 | 이슈 | 파일 |
|---|------|------|------|
| 1 | 서버 | **HTTP 타임아웃 미설정** — 외부 API 무응답 시 스레드 무한 대기 | infra 전체 (tavily, exa, firecrawl, openrouter) |
| 2 | 서버 | **auth 미들웨어: 매 요청마다 Client 생성 + 원격 검증만** — Supabase 장애 시 전체 마비 | `middleware/auth.rs` |
| 3 | 웹 | **인증 가드 race condition** — `$effect` 간 실행 순서 비보장 | `feed/+page.svelte`, `+page.svelte` |

## Major (다음 스프린트)

| # | 영역 | 이슈 | 파일 |
|---|------|------|------|
| 4 | 서버 | `sqlx::FromRow`가 domain 모델에 존재 — 의존 방향 위반 | `domain/models.rs` |
| 5 | 서버 | 크롤링 **직렬 처리** — N건 순차 HTTP로 수십 초 지연 | `collect_service.rs` |
| 6 | 서버 | LLM 요약 **직렬 처리** — 동일 병렬화 필요 | `summary_service.rs` |
| 7 | 서버 | Internal 에러 상세 메시지 **클라이언트 노출** | `domain/error.rs` |
| 8 | 서버 | `SearchFallbackChain` 구체 타입 의존 — services → infra 위반 | `collect_service.rs` |
| 9 | 서버 | 검색 어댑터 3개 코드 중복 (HTTP 호출/에러 처리 동일) | `tavily.rs`, `exa.rs`, `firecrawl.rs` |
| 10 | 웹 | onboarding/settings **태그 선택 UI 완전 중복** | 2개 페이지 |
| 11 | 웹 | API 프록시 라우트 2개 **100% 동일** | `api/collect`, `api/summarize` |
| 12 | 웹 | feed/+page.svelte **283줄 과대** — 분리 필요 | `feed/+page.svelte` |
| 13 | 웹 | 모든 UI 텍스트 **i18n 하드코딩** — 규칙 위반 | 전체 `.svelte` |
| 14 | 웹 | `saveMyTags` **비원자적** DELETE+INSERT | `api.ts` |
| 15 | 웹 | `onAuthStateChange` **구독 해제 누락** | `auth.svelte.ts` |

## Minor

| # | 영역 | 이슈 |
|---|------|------|
| 16 | 서버 | `update_article_summary` 파라미터 7개 → DTO 구조체 필요 |
| 17 | 서버 | Article 필드 17개 — 의미 그룹별 중첩 구조체 분리 |
| 18 | 서버 | `save_articles` 반환값(시도 건수)과 실제 저장 건수 불일치 |
| 19 | 서버 | 매직넘버 하드코딩 (1000, 50, 5) |
| 20 | 웹 | `tagMap` $derived 중복 (feed, feed/[id]) |
| 21 | 웹 | 에러 핸들링 패턴 10회 이상 반복 |
| 22 | 웹 | 무한 스크롤 에러 시 무한 재시도 루프 가능 |
| 23 | 웹 | 컴포넌트 테스트 전무 — 90% 커버리지 미달 |

## 긍정적 측면

**서버**: 포트/어댑터 골격 우수, 모든 포트에 Fake 어댑터 존재, `?` 연산자 일관 사용, SearchFallbackChain 폴백 견고
**웹**: Svelte 5 runes 정확 사용, XSS 자동 방어, TypeScript 타입 깔끔, API 프록시로 서버 URL 미노출
