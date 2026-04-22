# MVP1 학습 로드맵

> 생성: 2026-04-22
> MVP: MVP1 — AI 기반 키워드 뉴스 자동 수집+요약 웹앱

## 전체 학습 흐름

사용자 행동 기준으로 구성. 기술 레이어 순서 아님.

| # | 단계 | 핵심 개념 | 상태 | 상세 파일 |
|---|------|-----------|------|-----------|
| 1 | 로그인 (인증) | JWT, httpOnly 쿠키, Supabase Auth, safeGetSession, Bearer 토큰, Rust require_auth | 🔄 진행 중 | mvp1_stage1_notes.md |
| 2 | 온보딩 (태그/키워드 등록) | Supabase RLS, 낙관적 업데이트, SvelteKit Form Action | 🔲 미시작 | — |
| 3 | 뉴스 수집 (검색 파이프라인) | 폴백 체인 (Tavily→Exa→Firecrawl), 포트/어댑터 패턴, 배치 처리 | 🔲 미시작 | — |
| 4 | LLM 요약 + 인사이트 | OpenRouter, 프롬프트 엔지니어링, 스트리밍 응답 | 🔲 미시작 | — |
| 5 | 피드 표시 (매거진 UI) | Svelte 5 runes, stale-while-revalidate, 가상 스크롤 | 🔲 미시작 | — |
| 6 | 기사 상세 | og:image 크롤링, 캐싱 전략, 상세 페이지 라우팅 | 🔲 미시작 | — |

## 현재 위치

**1단계 로그인** 진행 중 — 흐름도 완성, 개념 학습 시작 전

### 1단계 개념 학습 진행

| 개념 | 상태 |
|---|---|
| Supabase Auth (서버 역할) | 🔲 미시작 |
| JWT (JSON Web Token) | 🔲 미시작 |
| httpOnly 쿠키 | 🔲 미시작 |
| safeGetSession | 🔲 미시작 |
| Bearer 토큰 | 🔲 미시작 |
| Rust require_auth / Axum Extension | 🔲 미시작 |
| iOS Keychain vs httpOnly 쿠키 비교 | 🔲 미시작 |
| Apple OAuth vs 이메일 로그인 차이 | 🔲 미시작 |

## 세션 재개 방법

새 세션에서 이어서 시작하려면:
1. 이 파일로 현재 위치 파악
2. 해당 단계 상세 파일 (`mvp1_stage{N}_notes.md`) 로드
3. 중단된 개념부터 미니사이클 재개
