# G6 스킬 allowed-tools 예방적 정비

> 생성일: 260420
> 완료일: 260420
> 상태: ✅ 완료 (01~04 구현 + step-7 간소화 통과 + step-8 검증 완료 — 런타임 항목은 C 방식으로 deferred)
> 소속: MVP 마일스톤 외부 단일 태스크 (하네스 엔지니어링 격상 시리즈)
> 근거 문서: `progress/analysis/260417_하네스_비교분석.md` §5 G6 · §7 우선순위표 2번
> 5단계 매핑(`0_CODEX_RULES.md §3`): 본 문서는 **Inspect 산출물**이며, Specify는 `§8 예외 선언`(스킬 frontmatter + 문서 변경)으로 대체. Implement/Verify/Report는 `/step-4`~`/step-9`에서 순차 수행.

## 요약

하네스 비교분석에서 확인된 "선언 없는 스킬 3개 + 과다 선언 스킬 3개"의 `allowed-tools`를 **보수 원칙**(본문 명시 도구 + 쌍/동족 도구 + 맥락상 필요 도구)으로 정비한다. 본 작업은 **토큰 비용 체감이 발생한 상황이 아니라 예방적 정비**로 수행 — 지금 정비해두면 이후 신규 스킬 작성·유지보수 시 기준점이 된다. 작업 결과는 `rules/sub/agents.md`에 3~5줄 원칙으로 박아 재발 방지.

## 상세 내용

### 배경
- 분석 문서(`260417_하네스_비교분석.md`) 실행 순서: `G1 → G6 → 8.2` — G1은 260420 완료, 이번 사이클은 G6
- 사용자 의도: "토큰 비용 체감보다는 미리 설정해두면 좋을 것 같아서" → **예방적 정비**로 범위 한정
- 과투자 경계: 토큰 실측 세션 비교·정기 스캔 스크립트 등은 본 사이클 제외

### 3자 토론 합의 (Claude + Codex + Serena)
| 항목 | 결정 |
|---|---|
| 핵심 목표 | 6개 스킬 `allowed-tools`를 "본문 실사용 + 최소 유틸"로 좁히기 |
| 산출물 1 | 선언 없는 3개 스킬 frontmatter 신규 작성 |
| 산출물 2 | 과다 3개 스킬 frontmatter pruning (본문 분석 기반) |
| 산출물 3 | before/after 지표 표 (라인 수 + MCP 서버 수) |
| 산출물 4 | 분석 문서 3곳(요약표·§5 G6·§7 우선순위표) 태그 동기화 |
| 부차 산출물 | `rules/sub/agents.md` 원칙 섹션 추가 (3~5줄) |
| 리스크 관리 | 스킬 본문 실사용 도구 검증 필수 — 제거된 도구가 런타임에 필요하면 복구 |

### 인터뷰 결과 (5/5 — 2026-04-20)
| # | 질문 | 선택 | 결론 |
|---|---|---|---|
| 1 | 토큰 측정 방식 | **A** | 간이 지표만 (`allowed-tools` 라인 수 + MCP 서버 수 before/after) |
| 2 | 과다 스킬 pruning 공격성 | **A** | 보수 (본문 명시 + 쌍/동족 도구 예: `codex`↔`codex-reply` + 맥락상 필요 도구) |
| 3 | 선언 없는 3개 스킬 초기 범위 | **B** | 스킬별 완전 개별 설계 (Q2 원칙 일관 적용, 공통 베이스라인 없음) |
| 4 | `agents.md` 동기화 | **B** | 짧게 원칙만 추가 (3~5줄) |
| 5 | 이후 유지보수 주기 | **A** | 주기 정하지 않음 (필요 시 개별 점검) |

### 대상 스킬 6개

**선언 없는 3개 (frontmatter 신규 추가)**:
- `/deep-analysis` — 심층 분석, Codex/Serena 등 외부 추론 MCP 예상
- `/presentation` — Gamma + Mermaid MCP 사용 (CLAUDE.md 목록에 명시)
- `/progress-cleanup` — 진행상황 정리, Git + 파일 I/O 위주

**과다 선언 3개 (pruning)**:
- `/milestone` — 마일스톤 플로우 (Discovery-First: 탐색 → 브레인스토밍 → 수렴)
- `/debate` — 3자 토론 (Claude + Codex + Serena 왕복)
- `/milestone-review` — 로드맵 진행 검토 + 마일스톤 재조정

### 완료 조건
- [x] `/deep-analysis` `SKILL.md`에 `allowed-tools` frontmatter 신규 추가 (18개)
- [x] `/presentation` `SKILL.md`에 `allowed-tools` frontmatter 신규 추가 (6개)
- [x] `/progress-cleanup` `SKILL.md`에 `allowed-tools` frontmatter 신규 추가 (6개)
- [x] `/milestone` `SKILL.md` `allowed-tools` pruning (23→21, Task + exa_crawling 제거)
- [x] `/debate` `SKILL.md` `allowed-tools` pruning (25→24, Task 제거)
- [x] `/milestone-review` `SKILL.md` `allowed-tools` pruning (17→15, Task + exa_web_search 제거)
- [x] `rules/sub/agents.md §2.2`에 "스킬 `allowed-tools` 최소 집합 원칙" 섹션 추가 (4줄)
- [x] `progress/analysis/260417_하네스_비교분석.md`에 before/after 지표 표 추가 (6개 스킬 × 라인 수 + MCP 서버 수)
- [x] 분석 문서 3곳 ⏳ → ✅ 동기화 (요약 표 G6 / §5 G6 / §7 우선순위 2번)
- [ ] feat 커밋 + docs 커밋 분리 (step-9 수행)

### KPI / 게이트
현재 `active_mvp.txt=11:planning`, `active_milestone.txt=none` 상태이므로 `0_CODEX_RULES.md §3.5.3`에 따라 KPI 게이트는 **Soft 강등 적용**(planning 상태 + 활성 마일스톤 없음). 별도 전용 KPI는 선언하지 않는다. 기존 Frank 커밋 전 필수 검증(린트+타입+테스트) + `scripts/kpi-check.sh`는 그대로 적용되되 Hard 차단은 발생하지 않는다.

### TDD 예외 (`0_CODEX_RULES.md §8`)
본 태스크는 전부 **스킬 frontmatter 메타데이터 + 문서 변경**이므로 TDD 예외 적용. 단위 테스트 작성 대신 아래로 검증:
- 각 스킬 `allowed-tools` 수정 후 **본문에서 호출하는 도구 grep 체크** — 선언 누락 시 복구
- 대표 스킬 2개(`/debate`, `/milestone`) 수동 drive-through 1회 — 런타임 에러 없음 확인
- 최종 보고에 TDD 예외 사유 명시

## 서브태스크

`/step-3`에서 확정. 상세 문서는 `progress/subtask/260420_g6_allowed_tools_pruning/` 참조, 의존성 DAG은 동 디렉토리 `dag.svg`.

| # | 서브태스크 | 의존성 | 상세 | 커밋 분류 | 상태 |
|---|---|---|---|---|---|
| 01 | 선언 없는 3개 스킬 frontmatter 신규 추가 (`/deep-analysis`, `/presentation`, `/progress-cleanup`) | 없음 (병렬) | [01-frontmatter-add-3skills.md](subtask/260420_g6_allowed_tools_pruning/01-frontmatter-add-3skills.md) | `feat:` | ✅ |
| 02 | 과다 3개 스킬 `allowed-tools` pruning (`/milestone`, `/debate`, `/milestone-review`) | 없음 (병렬) | [02-frontmatter-prune-3skills.md](subtask/260420_g6_allowed_tools_pruning/02-frontmatter-prune-3skills.md) | `feat:` | ✅ |
| 03 | `rules/sub/agents.md` 원칙 섹션 추가 (3~5줄) | 01, 02 | [03-agents-md-principle.md](subtask/260420_g6_allowed_tools_pruning/03-agents-md-principle.md) | `feat:` | ✅ |
| 04 | 분석 문서 before/after 표 + 3곳 ⏳→✅ 동기화 | 01, 02, 03 | [04-analysis-doc-sync.md](subtask/260420_g6_allowed_tools_pruning/04-analysis-doc-sync.md) | `docs:` | ✅ |

**실행 순서**: `(S1 ∥ S2) → S3 → S4` — 01·02는 병렬 가능, 03은 두 결과 합치고, 04가 최종 문서 동기화.

**커밋 전략** (`feedback_commit_granularity`):
- `feat:` 커밋 1개 — 서브태스크 01·02·03 묶음 (스킬 frontmatter + agents.md 원칙)
- `docs:` 커밋 1개 — 서브태스크 04 (분석 문서 동기화)

## 워크플로우 설정

### 자동 토론
| 단계 | 상태 |
|------|------|
| step-1 | OFF (수동 3자 토론 완료) |
| step-3 | OFF |
| step-5 | OFF |
| step-7 | OFF |

토론 깊이: standard

Specify(step-2~3) 단계에서 `/critical-review` 1회 호출 권장 — 본문 실사용 도구 추출 단계에서 **누락 리스크**를 적대적으로 점검.

## 다음 단계

1. `/step-2` — 룰즈 검증 (Codex로 `rules/0_CODEX_RULES.md` + `rules/sub/` 전반 부합 여부)
2. `/step-3` — 서브태스크 분리 (위 표 확정)
3. `/step-4` — 서브태스크 인터뷰 + Feature List 초안 생성 (G1 산출물 적용)
