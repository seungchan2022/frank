# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Companion (Xenops) 설정
- Xenops는 항상 **한국어**로만 대화한다.

## 프로젝트 개요

AI 기반 나만의 뉴스 스크랩 기반 스터디앱.
상세 기획: `progress/260404_MVP1_기획.md` 참조.

## 주요 명령

```bash
# 서버 린트/체크
cd server && cargo clippy -- -D warnings
cd server && cargo fmt --check

# 웹 프론트 린트/타입체크
cd web && npm run lint && npm run check

# 전체 테스트
cd server && cargo test
cd web && npm run test

# 빌드
cd server && cargo build --release
cd web && npm run build

# 로컬 실행
cd server && cargo run                     # :8080
cd web && npm run dev                      # :5173
```

## 테스트 커버리지 기준 (90%)

| 영역 | 대상 | 도구 | 기준 |
|------|------|------|------|
| 서버(Rust) | `server/src/` | cargo-tarpaulin | **90% 이상** |
| 웹 프론트 | `web/src/lib/` | vitest | **90% 이상** |

- 새 기능 구현 시 반드시 테스트 먼저 작성 (TDD)
- 커버리지 90% 미만으로 떨어뜨리는 커밋 금지
- 인프라 초기화 코드는 통합 테스트 영역으로 제외 허용

## 아키텍처

### 앱 구조

| 앱 | 경로 | 유형 | 포트 |
|---|---|---|---|
| API 서버 | `server/` | Rust (Axum) | 8080 |
| 웹 프론트 | `web/` | Svelte | 5173 |

### 에코 서버 + 포트/어댑터 패턴

모든 앱(서버/웹/iOS)에 동일 패턴 적용:
- 앱은 에코 서버 — 포트 호출 + 응답 변환만
- 모든 외부 호출을 포트(trait/protocol)로 추상화
- State injection으로 프로덕션 어댑터와 Fake 어댑터 교체
- 포트는 관심사별 분리, 어댑터는 통일
- DB 접근은 Supabase REST API (sqlx 미사용)

### 디렉토리 구조

```
frank/
├── server/              # Rust API 서버 (에코 서버)
│   └── src/
│       ├── api/         # HTTP 핸들러 (얇게: 파싱→포트 호출→응답)
│       ├── services/    # 유스케이스 오케스트레이션
│       ├── domain/      # 비즈니스 모델 + 포트(trait) 정의
│       └── infra/       # 어댑터 구현체 (전부 reqwest HTTP)
├── web/                 # Svelte 웹 프론트엔드
│   └── src/
├── supabase/            # DB 마이그레이션
├── progress/            # 작업 문서 (진행 중)
├── history/             # 완료 마일스톤 아카이브
└── rules/               # 강제 규칙
```

**의존 방향**: `api → services → domain(ports) ← infra(adapters)` (단방향, 상향 참조 금지)

## 규칙 체계

강제 규칙과 서브 룰북을 반드시 따른다:
- **`rules/0_CODEX_RULES.md`** — 최상위 강제 규칙
- **`rules/sub/`** — 서브 룰북 (agents, workflow, git, documentation, mcp_integration, sub_agent_usage 등)

## 커밋 형식

상세: `rules/sub/git.md` 참조

```
tag_en: 제목(한글)

본문 3~4줄 요약(한글)
```

태그: feat, fix, docs, refactor, test, chore, style, perf

## 워크플로우

스킬(`.claude/skills/`), 에이전트(`.claude/agents/`). 핵심 명령:

### 마일스톤 플로우 (Discovery + 전략)

- `/milestone "프로젝트 설명"` — 탐색 → 브레인스토밍 → 수렴 → 로드맵
- `/milestone-review` — 로드맵 진행 상황 검토 + 아이템별 상태 추적 + 재조정

### 워크플로우 (전술 실행)

- `/workflow "태스크"` — 9단계 워크플로우 시작
- `/step-{1~9}` — 개별 단계
- `/next` — 다음 단계

### 유틸리티

- `/debate` — 3자 토론 (Claude + Codex + Serena)
- `/deep-analysis` — 심층 코드/아키텍처 분석
- `/init` — 프로젝트 초기화 + 세션 Resume

### 계층 관계

```
/milestone (발견 + 전략)
비전 → Discovery → 로드맵 → 마일스톤
                              └→ 아이템 (유형별 라우팅)
                                  ├─ feature  → /workflow (메인태스크)
                                  ├─ research → /deep-analysis
                                  ├─ decision → /debate
                                  └─ chore    → 직접 실행
```

## 브랜치 & 머지 규칙 (절대 준수)

- **main에 직접 커밋 금지** — 반드시 로컬 feature 브랜치에서 작업
- **main에 머지 전 반드시 최신 상태로 갱신 + rebase 머지**:
  ```bash
  git checkout main && git pull origin main
  git checkout feature/작업명
  git rebase main
  git checkout main && git merge feature/작업명    # fast-forward
  ```

## 금지 사항

- **`git commit` 자동 실행 절대 금지** — 반드시 사용자에게 커밋 여부를 먼저 물어보고, 명시적 허락을 받은 후에만 커밋
- **`Co-Authored-By:` 커밋 태그 절대 금지**
- **`git push` 금지** — 사용자가 직접 수행
- 민감정보 하드코딩/로그 노출 금지
- 명시적 요청 없이 대규모 리팩토링 금지
- 테스트 미통과 상태에서 작업 완료 표시 금지
- **커밋 전 검증 필수 (절대 생략 금지)**: 린트 + 타입체크 + 테스트 모두 통과 확인

## 작업 원칙

- **모호한 요청 시 확인 후 실행** — 여러 해석이 가능한 지시는 이해한 바를 간결히 확인한 후 진행
- **워크플로우 단계 건너뛰지 않기** — 순서를 변경하려면 사용자에게 먼저 알림
- **"진행" 같은 짧은 지시에도 현재 단계와 다음 동작을 명시**
