# MVP1 학습 로드맵

> 생성: 2026-04-22
> MVP: MVP1 — AI 기반 키워드 뉴스 자동 수집+요약 웹앱

## 전체 학습 흐름

사용자 행동 기준으로 구성. 기술 레이어 순서 아님.

| # | 단계 | 핵심 개념 | 상태 | 상세 파일 |
|---|------|-----------|------|-----------|
| 1 | 로그인 (인증) | JWT, httpOnly 쿠키, Supabase Auth, safeGetSession, Bearer 토큰, Rust require_auth | ✅ 완료 | mvp1_stage1_notes.md |
| 2 | 온보딩 (태그/키워드 등록) | Promise.all, $state/$derived/$effect, Bearer 토큰, 트랜잭션, UPSERT, 피드 캐시 무효화 | ✅ 완료 | mvp1_stage2_notes.md |
| 3 | 뉴스 수집 (검색 파이프라인) | 폴백 체인 (Tavily→Exa→Firecrawl), 포트/어댑터 패턴, 배치 처리 | 🔲 미시작 | — |
| 4 | LLM 요약 + 인사이트 | OpenRouter, 프롬프트 엔지니어링, 스트리밍 응답 | 🔲 미시작 | — |
| 5 | 피드 표시 (매거진 UI) | Svelte 5 runes, stale-while-revalidate, 가상 스크롤 | 🔲 미시작 | — |
| 6 | 기사 상세 | og:image 크롤링, 캐싱 전략, 상세 페이지 라우팅 | 🔲 미시작 | — |

## 현재 위치

**3단계 뉴스 수집 (검색 파이프라인)** 미시작

### 1단계 개념 학습 진행 (완료)

| 개념 | 상태 |
|---|---|
| Supabase Auth (서버 역할) | ✅ 완료 |
| JWT (JSON Web Token) | ✅ 완료 |
| httpOnly 쿠키 | ✅ 완료 |
| safeGetSession — getSession(로컬) + getUser(Supabase 서버 검증) 이중 확인 | ✅ 완료 |
| Bearer 토큰 | ✅ 완료 |
| Rust require_auth / Axum Extension | ✅ 완료 |
| iOS Keychain vs httpOnly 쿠키 비교 | ✅ 완료 |
| Apple OAuth vs 이메일 로그인 차이 | ✅ 완료 |

### 2단계 개념 학습 진행 (완료)

| 개념 | 상태 |
|---|---|
| safeGetSession 이중 확인 구조 (getSession + getUser) | ✅ 완료 |
| 인증 가드 — 서버(+layout.server.ts) + 클라이언트($effect) 이중 방어 | ✅ 완료 |
| Promise.all — 독립 요청 병렬 발송 | ✅ 완료 |
| 마운트 / onMount (iOS viewDidLoad 대응) | ✅ 완료 |
| $state — 참조 비교, Set/Array 재할당 필수 | ✅ 완료 |
| $derived — $state 기반 자동 계산 (iOS 계산 프로퍼티 대응) | ✅ 완료 |
| $effect — $state 변화 반응 (iOS .onChange(of:) 대응) | ✅ 완료 |
| Bearer 토큰 — Rust API 모든 요청에 Authorization 헤더 직접 첨부 | ✅ 완료 |
| 트랜잭션 (BEGIN/COMMIT/ROLLBACK) — 원자적 쿼리 묶음 | ✅ 완료 |
| SQL 명령어 (SELECT/INSERT/UPDATE/DELETE/UPSERT) | ✅ 완료 |
| 피드 캐시 무효화 — 태그 변경 시 항상 호출, 설정 페이지 재사용 구조 | ✅ 완료 |

## 세션 재개 방법

새 세션에서 이어서 시작하려면:
1. 이 파일로 현재 위치 파악
2. 해당 단계 상세 파일 (`mvp1_stage{N}_notes.md`) 로드
3. 중단된 개념부터 미니사이클 재개
