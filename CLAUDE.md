# CLAUDE.md

이 파일은 Claude Code가 본 저장소에서 작업할 때 참조할 **진입점**이다. 상세 룰은 `rules/`로 위임한다.

## 언어

Xenops(Companion)는 항상 **한국어**로만 응답한다.

## 프로젝트 개요

AI 기반 나만의 뉴스 스크랩 스터디앱. 서버(Rust/Axum) + 웹(Svelte) + iOS(SwiftUI/Tuist) 3개 앱.
- 아키텍처: 에코 서버 + 포트/어댑터 패턴. DB는 sqlx PgPool 직접, Supabase SDK는 Auth 전용
- 의존 방향: `api → services → domain(ports) ← infra(adapters)` (단방향)
- 포트/경로: API 8080, Web 5173, iOS(시뮬레이터)

## 주요 명령

```bash
# 통합 배포 (항상 이걸 사용 — cargo run / npm run dev 직접 실행 금지)
scripts/deploy.sh                        # 전체 (ios + api + front)
scripts/deploy.sh --target=ios           # iOS만
scripts/deploy.sh --target=api,front --native   # Docker 없이

# 검증 (커밋 전 필수)
cd server && cargo clippy -- -D warnings && cargo fmt --check && cargo test
cd web && npm run lint && npm run check && npm run test
cd ios/Frank && tuist generate --no-open && xcodebuild test -workspace Frank.xcworkspace -scheme Frank -destination 'platform=iOS Simulator,name=iPhone 17 Pro'

# KPI 확인 (active MVP의 Hard 지표 검증)
bash scripts/kpi-check.sh     # Hard 미달 시 exit 1
bash scripts/kpi-report.sh    # 전체 대시보드 (non-blocking)
```

## KPI 게이트 (2층: 마일스톤 + MVP 최종)

KPI를 **두 층위**로 관리. 각 마일스톤 단위로 독립 검증되며, MVP 최종은 마지막에 통합 검증한다.

### 상태 파일 2개

| 파일 | 포맷 | 용도 |
|---|---|---|
| `progress/active_mvp.txt` | `11:in-progress` | MVP 번호·상태 |
| `progress/active_milestone.txt` | `M2:in-progress` / `none` | 현재 마일스톤·상태 |

상태: `planning → in-progress → completing → done`

### 검증 대상 자동 선택

- `MVP:completing` → **MVP 최종 KPI**(`_roadmap.md`) 검증
- 그 외 → **현재 활성 마일스톤 KPI**(`M{X}_*.md`) 검증

### 상태별 게이트 자동 조정

- `planning` / `in-progress`: 점진 지표(커버리지·회고·성능) Soft 강등, 회귀형(테스트 통과)은 Hard
- `completing`: 선언 그대로 엄격 적용
- `done`: 게이트 없음
- 측정 불가(`-`/`missing`)는 `completing` 제외 자동 Soft 강등

### 기타

- 템플릿: `progress/TEMPLATE_mvp_kpi.md` (마일스톤용 + MVP 최종용 분리)
- pre-commit이 `scripts/kpi-check.sh`로 적용된 Hard 지표 미달 시 **커밋 차단**
- 우회: `KPI_BYPASS=1 git commit ...` + `progress/kpi/bypass.log`에 사유 필수 기록
- `docs:` / `chore:` 커밋은 자동 스킵
- 상태 전이 주체:
  - `/milestone` → MVP 기획 시 자동 초기화
  - `/workflow` → 각 마일스톤 단위 자동 전이 (이전 M done, 새 M in-progress, 마지막 M 후 MVP 완료 안내)
  - `/milestone-review` → MVP 사이클 경계(새 MVP 시작 전)에 한 번 호출
- 상세: `rules/0_CODEX_RULES.md §3.5`

## 규칙 체계

- **`rules/0_CODEX_RULES.md`** — 최상위 강제 규칙 (워크플로우 9단계 + KPI 게이트 + 보안)
- **`rules/sub/`** — 서브 룰북 (INDEX.md 참조)
  - `agents.md` — 에이전트 + MCP 정책 (Supabase, Chrome DevTools 포함)
  - `workflow.md` — 개발 사고 가이드
  - `git.md` — Git 커밋 규칙
  - `milestone.md` — 마일스톤 플로우
  - `documentation.md`, `session_scope.md`, `sub_agent_usage.md`
- **`.claude/rules/`** — 언어별 스코프 룰 (rust/svelte/swift/python-ml)

## 커밋·브랜치

- **feature 브랜치 필수**: main 직접 커밋은 pre-commit hook이 차단
- **커밋 본문 필수**: 제목만으로 끝내지 않음. 이유·범위 3~4줄
- **커밋 단위 분리**: feat/fix/test/docs/chore 각각 분리 (`feedback_commit_granularity`)
- **커밋 전 검증 필수**: 린트 + 타입체크 + 테스트 + `scripts/kpi-check.sh` 모두 통과
- **`git commit` 자동 실행 금지** — 반드시 사용자 허락 후 커밋

settings.json/hooks 기계적 강제:
- deny: `git push`, `git add -A`, `rm -rf`, `git reset --hard`, `.env` 수정
- pre-commit: main 직접 커밋 차단, 테스트 미통과 차단, **KPI Hard 미달 차단**
- commit-msg: Co-Authored-By 태그 차단, 포맷 검증

## 워크플로우

본 저장소의 **강제 워크플로우는 9단계 (step-1~step-9)**. 강제 룰의 SSOT는 `.claude/skills/workflow/SKILL.md`. `rules/0_CODEX_RULES.md §3`은 이를 가리키고, §9는 핵심 게이트만 보유.

핵심 명령 (`.claude/skills/`):
- `/milestone "설명"` — Discovery → 로드맵 → 마일스톤 정의
- `/milestone-review` — 로드맵 진행 검토
- `/workflow "태스크"` — 9단계 진입
- `/kpi` — 현재 MVP KPI 상태 대시보드
- `/debate`, `/deep-analysis` — 3자 토론·심층 분석
- `/study "MVPN"` — MVP별 흐름·개념 학습 (객관식 자가 확인 + 자동 critical-review)
- `/init`, `/next`, `/status` — 세션 초기화·진행

## 작업 원칙

- 모호한 요청은 **확인 후 실행**
- 워크플로우 단계 건너뛰지 않음 (`feedback_enforce_workflow`)
- "진행" 같은 짧은 지시에도 현재 단계·다음 동작을 명시
- 민감정보 하드코딩/로그 노출 금지
- 명시적 요청 없이 대규모 리팩토링 금지
- E2E 검증 필수 (단위 테스트만으로 완료 처리 금지 — `feedback_e2e_before_commit`)
