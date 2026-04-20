# §8.2 — Feature List 미체크 시 /step-9 커밋 차단 구현

> 생성일: 260420
> 완료일: 260420
> 상태: ✅ 완료
> 소속: MVP 마일스톤 외부 단일 태스크 (하네스 엔지니어링 격상 시리즈 — 마지막)
> 근거 문서:
> - `progress/analysis/260417_하네스_비교분석.md` §8.2 + §7 우선순위표 3번
> - `progress/subtask/260420_g1_feature_list_skill/03-step9-commit-block.md` (참고용 이관 초안)

## 5단계 워크플로우 매핑 (`0_CODEX_RULES.md §3`)

| 단계 | 내용 |
|---|---|
| **Inspect** | 본 문서 (근거: 비교분석 §8.2 + G1 03 이관 초안) |
| **Specify** | 인터뷰 5/5 + 설계 확정 (이 섹션들) |
| **Implement** | 서브태스크 01~04 구현 (`/step-6`) |
| **Verify** | Feature List 기반 drive-through (`/step-8`) |
| **Report** | 분석 문서 동기화 + feat/docs 커밋 분리 (`/step-9`) |

## KPI 게이트

- MVP11 상태: `planning` → Hard 지표 Soft 강등 (측정 불가 자동 통과)
- 마일스톤: `none` → 게이트 없음
- `docs:` / `chore:` 커밋은 pre-commit KPI 스킵 자동 적용

## TDD 계획

| 대상 파일 | 테스트 방법 | 예외 사유 |
|---|---|---|
| `scripts/feature-list-check.sh` | 수동 drive-through (bash 스크립트) | 단위 테스트 프레임워크 미적용 영역 |
| `.git/hooks/pre-commit` | 수동 drive-through | hook 환경 자동화 불가 |
| `.claude/skills/step-9/SKILL.md` | §8 TDD 예외 | 마크다운 지시문 변경 (코드 없음) |
| `.claude/skills/step-4/SKILL.md` | §8 TDD 예외 | 마크다운 지시문 변경 (코드 없음) |

drive-through 시나리오 (step-8에서 수행):
1. 미체크 `[ ]` 있는 서브태스크 → hook 차단 확인
2. 파싱 실패(ID 포맷 오류) → 차단 + 위치 안내 확인
3. `FEATURE_LIST_BYPASS=1` → 통과 + bypass.log 기록 확인
4. `docs:` 커밋 → 통과 확인
5. Feature List 없는 서브태스크 → 통과 확인
6. 모든 항목 체크 완료 → 정상 커밋 확인

## 요약

G1(Feature List 생성) + G6(allowed-tools 정비)에 이어 하네스 엔지니어링 격상 시리즈 마지막 단계.
`/step-9` 진입 시점에 서브태스크 Feature List 파싱 → 미체크 `[ ]` 존재 시 차단.
pre-commit hook으로 직접 `git commit` 경로도 동일하게 차단.
G1에서 확립한 안정 ID 규약·4상태 모델·HTML 메타·파싱 실패 정책을 그대로 재사용.

## 인터뷰 결과 (5/5 — 2026-04-20)

| # | 질문 | 선택 | 결론 |
|---|---|---|---|
| 1 | 구현 범위 | **B** | step-9 스킬 + pre-commit hook 양쪽 구현 (외부 git commit 차단 포함) |
| 2 | 활성 서브태스크 식별 | **A** | `progress/active_subtask.txt` 신규 도입 (active_mvp.txt 패턴 재사용) |
| 3 | active_subtask.txt 기록 주체 | **B** | step-4 스킬에서 Feature List 초안 생성 직후 기록 |
| 4 | 파싱 실패 시 hook 동작 | **A** | 차단 + 에러 안내 (사용자가 안내 보고 Claude에 수정 요청) |
| 5 | step-9 스킬 UX | **A** | 미체크 목록 출력 후 사용자 확인 대기 후 git commit 진행 |

## 설계 확정

### 우회 체계 (이중 구조)
- **스킬 레이어**: 사용자가 `--skip-manual` + 사유 입력 시 통과
- **hook 레이어**: `FEATURE_LIST_BYPASS=1` 환경변수 (KPI_BYPASS 동일 패턴)
- **bypass.log**: `progress/feature-list/bypass.log` (KPI log와 분리)

### active_subtask.txt 규약
- 포맷: `progress/subtask/{경로}/{파일명}.md` (절대경로 아닌 progress 상대경로)
- 기록: step-4 Feature List 초안 생성 직후
- 클리어: step-9 커밋 성공 직후 (`none` 으로 초기화)

### 파싱 실패 정책 (G1 그대로 재사용)
| 실패 유형 | 처리 |
|---|---|
| ID 포맷 오류 (`F01` → `F-01`) | 차단 + 위치 안내 |
| 상태 기호 불일치 | 차단 + 위치 안내 |
| `[~]`/`[-]` 뒤 사유 누락 | 차단 + 위치 안내 |
| HTML 메타 누락 | 차단 + 위치 안내 |
| count 불일치 | 차단 + 위치 안내 |
| Feature List 섹션 없음 | 통과 (skip: true 처리와 동일) |

### 커밋 태그 판정
- 차단 대상: `feat` / `fix` / `test`
- 통과: `docs` / `chore` / `style` / `refactor`

## 서브태스크 목록

| # | 제목 | 상태 |
|---|---|---|
| 01 | `scripts/feature-list-check.sh` 신규 구현 | ✅ (260420) |
| 02 | pre-commit hook에 feature-list-check.sh 연계 | ✅ (260420) |
| 03 | step-9 SKILL.md 수정 (미체크 목록 출력 + 확인 대기) | ✅ (260420) |
| 04 | step-4 SKILL.md 수정 (active_subtask.txt 기록) | ✅ (260420) |
| 05 | 분석 문서 §8.2 + §7 + 요약표 동기화 | ✅ (260420) |

## 변경 파일 (예상)

| 파일 | 변경 유형 |
|---|---|
| `scripts/feature-list-check.sh` | 신규 생성 |
| `.git/hooks/pre-commit` | 수정 (hook 추가) |
| `.claude/skills/step-9/SKILL.md` | 수정 |
| `.claude/skills/step-4/SKILL.md` | 수정 |
| `progress/feature-list/bypass.log` | 신규 생성 |
| `progress/active_subtask.txt` | 신규 생성 |
| `progress/analysis/260417_하네스_비교분석.md` | 수정 (§8.2 동기화) |

## 완료 조건

- [x] `scripts/feature-list-check.sh` — 파싱 + 4상태 판정 + 차단 + 에러 안내
- [x] pre-commit hook — feature-list-check.sh 호출 + `FEATURE_LIST_BYPASS=1` 우회
- [x] step-9 SKILL.md — 미체크 목록 출력 + 확인 대기 + `--skip-manual` 우회 안내
- [x] step-4 SKILL.md — Feature List 생성 직후 `active_subtask.txt` 기록 단계 추가
- [x] 분석 문서 §8.2 `⏳ → ✅` + §7 우선순위 3번 + 요약표 동기화
- [x] feat + docs 커밋 분리 (G1/G6 패턴)
