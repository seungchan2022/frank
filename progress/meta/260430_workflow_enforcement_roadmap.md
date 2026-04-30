# 워크플로우 강제성 도입 로드맵 (메타태스크)

> 작성일: 2026-04-30
> 활성 Phase: Phase 1 (진행 예정)
> 메타태스크 면제: 워크플로우 9단계 진행 면제 (룰 자체 수정이라 자기참조 회피)
> 세션 resume: 본 문서의 `## 변경 로그`와 각 Phase 체크박스로 진행 상태 복원
> 인터뷰 완료: 2026-04-30 (Q1~Q6 결정 §1에 반영)

---

## 1. 결정사항 (Decision Log)

### 인터뷰 결정 (Q1~Q6, 2026-04-30 확정)

| # | 결정 | 인터뷰 답 | 근거 |
|---|---|---|---|
| D1 | **9단계 유지, 5단계는 §3 한 줄로 축소 (Codex 내부 사고 모델로 명시)** | Q1: A | 9단계 운영 자산(skill, agents.md 매핑, documentation.md 매핑) 이미 존재. 5단계는 Codex 호출 시 자동 작동하는 사고 모델이라 별도 강제 불필요. 룰 압축 + 단일 SSOT 원칙 부합. |
| D2 | **강제 룰 SSOT = `.claude/skills/workflow/SKILL.md`** | Q2: A | 9단계 진입점 = workflow 스킬. 강제 룰이 같이 있으면 워크플로우 호출 시 자동 로드. `context: fork`라 평소엔 안 로드 = "필요할 때만 무겁게". |
| D3 | **메타태스크 면제 경로 = `.claude/`, `scripts/`, `rules/`, `progress/meta/` + 메모류(`progress/notes.md`, `bugs.md`, `debts.md`, `future-ideas.md`) + `history/`** | Q3: B | 인터뷰는 새 메인태스크 코드 작업 시 발생. 메타·메모·아카이브는 인터뷰와 무관하니 차단 면제. |
| D4 | **bypass 환경변수 = `WORKFLOW_BYPASS=1`** | Q4: A | 기존 `KPI_BYPASS=1` 패턴 일관성. |
| D5 | **stale expire = 24h** | Q5: A | 마일스톤 평균 1~2일 사이클이라 24h면 평소 작업 충분 커버. |
| D6 | **Phase 1 즉시 시작, 오늘 내 Phase 1~3 다 진행** | Q6: A | 메타태스크라 워크플로우 면제. Phase 1은 위험 0이라 즉시 가능. 각 Phase 완료마다 본 문서 갱신해 세션 간 완벽 인계. |

### 구조 결정 (인터뷰 외)

| # | 결정 | 근거 |
|---|---|---|
| D7 | **메타태스크 위치 = `progress/meta/`** | progress 루트는 MVP/마일스톤용. 메타(룰·hook·skill 수정)는 분리. |
| D8 | **Phase 분리 진행 (1→2→3)** | Phase 3(차단)은 Phase 2(state file)가 전제. 한 번에 다 하면 디버깅 난해. |
| D9 | **메타태스크 면제 정책 유지** | 본 작업 + 향후 hook·rules 수정은 워크플로우 9단계 면제. step-4 SKILL 기존 면제 조항과 일관. |

---

## 2. 배경 — 4가지 문제

1. **워크플로우 강제성 안 지켜짐** — 룰 텍스트로 "절대 준수" 적어도 LLM이 일관되게 안 지킴
2. **5↔9단계 모순** — `0_CODEX_RULES.md §3`은 5단계, `§9` + workflow skill은 9단계. 같은 파일 안에서 자기모순
3. **인터뷰 망각** — step-1/4 인터뷰 도중 한 질문 깊이 토론하다 다른 Q 잊고 step-6 직진
4. **룰 길이 인플레이션** — 룰 추가할수록 attention dilution, MUST NOT 50개 → 각각 무게 1/50

**원인 본질**: LLM은 자기 컨텍스트 안에서 자기를 100% 강제 못 함. 자기-감시 한계. 외부 강제(hook + state file)로 빼야 풀림.

---

## 3. 전체 영향 범위

### 손대는 파일 (총 16개)

```
[문서] 9개 — Phase 1
  rules/0_CODEX_RULES.md
  CLAUDE.md
  rules/sub/agents.md
  rules/sub/workflow.md
  rules/sub/milestone.md
  rules/sub/INDEX.md
  rules/sub/documentation.md  (혹시 5단계 언급 있으면)
  .claude/skills/workflow/SKILL.md
  .claude/skills/study/SKILL.md
  .claude/agents/debate.md

[SKILL] 4개 — Phase 2
  .claude/skills/workflow/SKILL.md   (재수정)
  .claude/skills/step-1/SKILL.md
  .claude/skills/step-4/SKILL.md
  .claude/skills/step-9/SKILL.md

[hook + state] — Phase 2, 3
  .claude/settings.json               (hook 추가)
  .claude/hooks/inject-step-status.sh  (신규)
  .claude/hooks/block-on-interview.sh  (신규)
  progress/active_step.txt             (신규 state)
  progress/active_interview.json       (신규 state)
```

### 안 건드리는 것 (25개 스킬 중 20개)

```
/init, /status, /kpi, /milestone, /milestone-review, /next,
/study, /daily-retro, /critical-review, /debate, /deep-analysis,
/e2e, /readme-update, /progress-cleanup, /notes, /presentation,
/step-2, /step-3, /step-5, /step-6, /step-7, /step-8,
pre-commit hook + scripts/kpi-*.sh
```

---

## 4. Phase 1 — 문서 정리 (위험 0)

### 목표

5↔9단계 모순 제거. 강제 룰 SSOT를 workflow SKILL로 통일. 중복 룰을 한 곳만 유지.

### 작업 목록

- [ ] **1.1 `rules/0_CODEX_RULES.md §3` 정리** (5단계 → 1줄로 축소)
  - 변경 전: 59~67라인 5단계 상세 + "유일한 강제 워크플로우" 문구
  - 변경 후: "외부 Codex용 5단계 사고 원칙(Inspect→Specify→Implement→Verify→Report)은 §9 9단계 워크플로우 안에서 자연스럽게 충족된다." 1~2줄
  - 영향: §9 9단계가 진짜 SSOT라는 것 명시

- [ ] **1.2 `rules/0_CODEX_RULES.md §9` 9단계 SSOT 강화**
  - "본 §9가 본 저장소의 강제 워크플로우. 세부 규칙은 `.claude/skills/workflow/SKILL.md`를 참조" 추가
  - 인터뷰 게이트·스텝 헤더·피드백 처리 같은 세부 규칙은 workflow SKILL로 이관 후 §9는 링크만

- [ ] **1.3 `CLAUDE.md` "워크플로우" 섹션 정리**
  - "5단계가 유일한 강제" 문구 제거
  - "9단계 워크플로우. 세부는 `.claude/skills/workflow/SKILL.md` + `rules/0_CODEX_RULES.md §9` 참조" 1줄

- [ ] **1.4 `.claude/skills/workflow/SKILL.md` 강제 룰 SSOT 강화**
  - 인터뷰 게이트 / 스텝 헤더 / 피드백 처리 / 스텝 강제 원칙 → 본 파일 한 곳에만 유지
  - 다른 곳 중복 룰 제거

- [ ] **1.5 `.claude/skills/step-1/SKILL.md`, `step-4/SKILL.md` 중복 룰 제거**
  - "인터뷰 완료 게이트" 등 본문 → `.claude/skills/workflow/SKILL.md` 링크로 대체
  - 본 SKILL은 자기 단계 고유 내용만

- [ ] **1.6 `rules/sub/{workflow,milestone,agents,INDEX}.md`, `study/SKILL.md`, `agents/debate.md` 잔재 정리**
  - "5단계" 표현 grep으로 찾아 9단계 표현으로 통일 또는 제거

### 검증

```bash
# 5단계 잔재 0건이어야 함 (예외: §3 한 줄 축소본만 허용)
grep -rn "5단계" .claude/ rules/ CLAUDE.md | grep -v "원칙(Inspect"

# 강제 룰 SSOT 단일성 — workflow SKILL에만 있어야 함
grep -rln "인터뷰 완료 게이트" .claude/ rules/ CLAUDE.md
# 기대: .claude/skills/workflow/SKILL.md 한 줄만
```

### 커밋 단위

`docs: 5↔9단계 모순 제거 + 강제 룰 SSOT 통일 (Phase 1)`

### 롤백

`git revert {Phase1 커밋}` — 단일 커밋, 동작 변경 없으므로 안전.

---

## 5. Phase 2 — state file + UserPromptSubmit hook (위험 매우 낮음)

### 목표

활성 step + 인터뷰 진행 상태를 외부 파일로 추적. 매 turn LLM 컨텍스트 첫 줄에 자동 주입해 망각 방지.

### state file 명세

**`progress/active_step.txt`**
```
포맷: 단일 줄. step-1 / step-2 / ... / step-9 / none
초기값: none
갱신 주체:
  - workflow [0]단계 시작 시 → step-1
  - 각 step-N 종료 시 → step-{N+1}
  - step-9 커밋 성공 시 → none (cleanup)
```

**`progress/active_interview.json`**
```json
{
  "step": "step-1",
  "current": 2,
  "total": 4,
  "remaining": ["Q3: 제약사항", "Q4: 테스트 범위"],
  "started_at": "2026-04-30T14:23:00Z"
}
```
```
갱신 주체:
  - step-1/4 인터뷰 시작 시 → 작성 (current=1, total=N, remaining=[Q1~Qn])
  - 매 답변 후 → current 증가, remaining 첫 항목 제거
  - 마지막 답변 후 → {} 빈 객체 (또는 파일 삭제)
  - step-1/4 종료 후 step-2/5 진입 시 → 강제 cleanup
초기값: 빈 객체 {} 또는 미존재
```

### 작업 목록

- [ ] **2.1 `progress/active_step.txt` 생성 + `none` 초기화**

- [ ] **2.2 `progress/active_interview.json` 생성 + `{}` 초기화**

- [ ] **2.3 `.claude/skills/workflow/SKILL.md` [0]단계에 active_step 갱신 명시**
  - "echo 'step-1' > progress/active_step.txt" 명령 추가

- [ ] **2.4 `.claude/skills/step-1/SKILL.md` 인터뷰 라이프사이클 추가**
  - 인터뷰 시작 시: `active_interview.json` 작성 (Write tool로 JSON 직접 작성)
  - 매 답변 후: current 갱신
  - 인터뷰 완료 시: `{}`로 비우기

- [ ] **2.5 `.claude/skills/step-4/SKILL.md` 동일 라이프사이클 추가**

- [ ] **2.6 `.claude/skills/step-9/SKILL.md` 커밋 성공 후 cleanup 추가**
  - `echo 'none' > progress/active_step.txt`

- [ ] **2.7 `.claude/hooks/inject-step-status.sh` 작성**
  ```bash
  #!/bin/bash
  ROOT="${CLAUDE_PROJECT_DIR:-$(pwd)}"
  STEP=$(tr -d '[:space:]' < "$ROOT/progress/active_step.txt" 2>/dev/null)
  [ -z "$STEP" ] || [ "$STEP" = "none" ] && exit 0

  INT_FILE="$ROOT/progress/active_interview.json"
  if [ -f "$INT_FILE" ] && [ "$(jq -r '.current // empty' "$INT_FILE" 2>/dev/null)" ]; then
    CUR=$(jq -r '.current' "$INT_FILE")
    TOT=$(jq -r '.total' "$INT_FILE")
    REM=$(jq -r '.remaining | join(", ")' "$INT_FILE")
    MSG="활성: $STEP | 인터뷰 Q$CUR/$TOT | 남은 질문: $REM"
  else
    MSG="활성: $STEP"
  fi

  jq -n --arg msg "$MSG" '{systemMessage: $msg}'
  ```

- [ ] **2.8 `.claude/settings.json`에 UserPromptSubmit hook 추가**
  - 기존 "실제 기기" hook과 같은 배열에 추가
  ```json
  {
    "hooks": [
      {
        "type": "command",
        "command": "bash \"${CLAUDE_PROJECT_DIR:-$(pwd)}/.claude/hooks/inject-step-status.sh\""
      }
    ]
  }
  ```

- [ ] **2.9 검증 — 모의 워크플로우**
  - `echo 'step-1' > progress/active_step.txt`
  - `echo '{"current":2,"total":4,"remaining":["Q3","Q4"]}' > progress/active_interview.json`
  - 새 채팅 시작 → 시스템 메시지에 `활성: step-1 | 인터뷰 Q2/4 | 남은 질문: Q3, Q4` 출력 확인
  - cleanup: state 둘 다 초기화

### 커밋 단위

```
feat: state file 도입 (active_step + active_interview)
feat: UserPromptSubmit hook으로 step 상태 자동 주입
```
2개 분리 커밋.

### 롤백

```bash
# settings.json hook 1개 제거
# .claude/hooks/inject-step-status.sh 삭제
# progress/active_step.txt, active_interview.json 삭제
# step-1/4/9 SKILL 변경 git revert
```

---

## 6. Phase 3 — PreToolUse 자물쇠 + 안전장치 6개 (위험 중간)

### 목표

인터뷰 미완료 상태에서 Edit/Write tool 호출 시 차단. 잘못된 차단 방지를 위한 안전장치 6개 동시 도입.

### 안전장치 6개 명세

| # | 장치 | 구현 |
|---|---|---|
| S1 | matcher 좁히기 | `Edit\|Write`만. Read·Bash·Grep·MultiEdit 영향 없음 |
| S2 | 활성 step 체크 | `active_step.txt = none` or 비어있으면 무조건 통과 |
| S3 | step 전환 시 자동 cleanup | step-2~9 진입 시 `active_interview.json = {}` 강제 클리어 (step-1/4 SKILL 책임) |
| S4 | 메타태스크 면제 | tool input의 file_path가 다음 중 하나로 시작/매칭되면 통과: `.claude/`, `scripts/`, `rules/`, `progress/meta/`, `history/`, `progress/notes.md`, `progress/bugs.md`, `progress/debts.md`, `progress/future-ideas.md` (D3 인터뷰 결정) |
| S5 | bypass 환경변수 | `WORKFLOW_BYPASS=1` 시 무조건 통과 (긴급용) — D4 인터뷰 결정 |
| S6 | 자동 expire | `active_interview.json`의 `started_at` + 24h < 현재 → stale, 무시 — D5 인터뷰 결정 |

### 작업 목록

- [ ] **3.1 `.claude/hooks/block-on-interview.sh` 작성**
  ```bash
  #!/bin/bash
  set -e
  ROOT="${CLAUDE_PROJECT_DIR:-$(pwd)}"

  # S5: bypass
  [ "$WORKFLOW_BYPASS" = "1" ] && exit 0

  # S2: 활성 step 없으면 통과
  STEP=$(tr -d '[:space:]' < "$ROOT/progress/active_step.txt" 2>/dev/null)
  [ -z "$STEP" ] || [ "$STEP" = "none" ] && exit 0

  # 인터뷰 진행 중인지 확인
  INT_FILE="$ROOT/progress/active_interview.json"
  [ -f "$INT_FILE" ] || exit 0
  CUR=$(jq -r '.current // 0' "$INT_FILE" 2>/dev/null)
  TOT=$(jq -r '.total // 0' "$INT_FILE" 2>/dev/null)
  [ "$TOT" = "0" ] || [ "$CUR" -ge "$TOT" ] && exit 0

  # S6: 24h expire
  STARTED=$(jq -r '.started_at // empty' "$INT_FILE" 2>/dev/null)
  if [ -n "$STARTED" ]; then
    NOW=$(date -u +%s)
    START_EPOCH=$(date -u -j -f "%Y-%m-%dT%H:%M:%SZ" "$STARTED" +%s 2>/dev/null || echo 0)
    AGE=$(( NOW - START_EPOCH ))
    [ "$AGE" -gt 86400 ] && exit 0  # 24h 넘으면 stale
  fi

  # S4: 메타태스크 면제 (tool_input의 file_path 검사) — D3 인터뷰 결정 (B안)
  TOOL_INPUT=$(jq -r '.tool_input.file_path // empty')
  case "$TOOL_INPUT" in
    *"/.claude/"*|*"/scripts/"*|*"/rules/"*|*"/progress/meta/"*|*"/history/"*)
      exit 0 ;;
    */progress/notes.md|*/progress/bugs.md|*/progress/debts.md|*/progress/future-ideas.md)
      exit 0 ;;
  esac

  # 차단
  REM=$(jq -r '.remaining | join(", ")' "$INT_FILE" 2>/dev/null || echo "")
  echo "❌ 인터뷰 Q$CUR/$TOT 미완료. 남은 질문: $REM" >&2
  echo "   인터뷰 끝낸 후 Edit/Write 진입. 우회: WORKFLOW_BYPASS=1" >&2
  exit 2
  ```

- [ ] **3.2~3.7 안전장치 6개 위 스크립트에 모두 포함됨** (S1은 settings.json matcher로)

- [ ] **3.8 `.claude/settings.json`에 PreToolUse hook 추가**
  ```json
  "PreToolUse": [
    {
      "matcher": "Edit|Write",
      "hooks": [
        {
          "type": "command",
          "command": "bash \"${CLAUDE_PROJECT_DIR:-$(pwd)}/.claude/hooks/block-on-interview.sh\""
        }
      ]
    }
  ]
  ```

- [ ] **3.9 시나리오 테스트 5개**

  | # | 시나리오 | 기대 동작 |
  |---|---|---|
  | T1 | 정상 — active_step=none | Edit 통과 |
  | T2 | 차단 — step-1 + 인터뷰 Q2/4 | Edit 차단 + Q3, Q4 안내 |
  | T3 | 메타 면제 — Edit 대상이 `.claude/skills/foo.md` | Edit 통과 (S4) |
  | T4 | bypass — `WORKFLOW_BYPASS=1` | Edit 통과 (S5) |
  | T5 | expire — started_at 25h 전 | Edit 통과 (S6) |

### 커밋 단위

`feat: PreToolUse hook으로 인터뷰 미완료 차단 + 안전장치 6개 (Phase 3)`

### 롤백

```bash
# settings.json PreToolUse 배열만 제거
# .claude/hooks/block-on-interview.sh 삭제
# UserPromptSubmit hook + state file은 유지 (Phase 2 효과 보존)
```

---

## 7. 함정·주의사항

| 함정 | 회피 방법 |
|---|---|
| Phase 2 도중 step-1 SKILL이 active_interview.json을 안 갱신하고 종료 → 다음 세션에서 stale | S6(24h expire) + step-9 cleanup |
| Phase 3 도입 후 hook 자체 디버깅이 차단됨 | S4(메타 면제) — `.claude/hooks/` 수정은 항상 통과 |
| `jq` 미설치 환경 | macOS는 brew로 보장, hook 스크립트 시작에 `command -v jq` 체크 추가 검토 |
| 기존 UserPromptSubmit hook("실제 기기")와 충돌 | 같은 배열에 추가, 각 hook 독립 실행됨 — 충돌 없음 |
| settings.json JSON syntax 깨짐 | 변경 후 `jq . .claude/settings.json` 검증 필수 |
| Phase 1에서 5단계 인용 짧게 한 게 §9 9단계 의미 깨짐 | grep으로 다른 곳 5단계 의존 없는지 사전 확인 |
| step-2/3/5/6/7/8이 active_step.txt 갱신 책임 누락 | Phase 2.3에서 workflow SKILL이 [0]에서만 step-1 설정. 각 step 종료 시 step+1 자동 갱신은 step-N 본문이 책임 — 단계별 추가 시 누락 위험 → 후속 작업으로 별도 추적 |

---

## 8. 변경 로그

> 각 Phase 완료 시 **반드시 다음 양식 채워서 추가**. 다음 세션이 이 로그만 읽고도 100% 흐름 파악 가능해야 함.

### 양식 (Phase 완료 시 필수 기록)

```markdown
### YYYY-MM-DD — Phase {N} 완료

**커밋**: {git commit hash} ({커밋 메시지 요약})
**변경 파일**: {N개}
- {file_path}: {한 줄 요약}
- ...

**완료된 작업 항목**: {Phase {N}의 모든 [x] 항목 그대로 복사}

**검증 결과**:
- {검증 명령어}: {OK/FAIL}
- ...

**발견한 함정·이슈** (있을 시):
- {이슈}: {대응 또는 후속 작업}

**다음 Phase 진입 시 주의사항**:
- {뭘 신경써야 하는지}

**활성 Phase 갱신**:
- Phase {N+1} 진입 (또는 전체 완료)
```

### 실제 로그

| 일시 | 이벤트 |
|---|---|
| 2026-04-30 | 로드맵 작성 |
| 2026-04-30 | 인터뷰 완료 (Q1~Q6) — §1 결정사항 반영 |
| - | Phase 1 시작 |
| - | Phase 1 완료 |
| - | Phase 2 시작 |
| - | Phase 2 완료 |
| - | Phase 3 시작 |
| - | Phase 3 완료 |

(Phase 완료 시 위 양식으로 본 섹션에 상세 추가)

---

## 9. 새 채팅에서 즉시 이어가기 위한 resume 절차

> 핵심 요구사항 (D6, Q6): **다른 채팅에서 상황 설명 없이 100% 이어갈 수 있어야 함.**

### 새 채팅 시작 시 자동 절차 (이 문서를 읽는 Claude의 행동 강제)

1. **현재 위치 파악**
   - 본 문서 `## 변경 로그`의 마지막 항목 확인 → 마지막 완료 Phase 식별
   - 본 문서 헤더 "활성 Phase: ..." 확인
   - `git log --oneline -10`로 최근 커밋 확인 (변경 로그와 일치 여부)

2. **state 일관성 검증**
   - `cat progress/active_step.txt` (Phase 2 후부터 의미 있음)
   - `cat progress/active_interview.json` (Phase 2 후부터 의미 있음)
   - hook 파일 존재 여부: `ls .claude/hooks/` (Phase 2 = inject-step-status.sh, Phase 3 = block-on-interview.sh)
   - `.claude/settings.json`의 hooks 배열 확인

3. **의도된 상태와 비교 → 불일치 시 사용자 확인**
   - 변경 로그상 Phase 2 완료인데 hook 파일 없음 → 롤백 흔적? 사용자에게 확인
   - 변경 로그상 Phase 1 완료인데 git log에 해당 커밋 없음 → 다른 브랜치? 미커밋?

4. **다음 작업 명시**
   - 활성 Phase의 첫 미완료 `[ ]` 항목 찾기
   - 사용자에게 **"현재 위치: Phase {N} 작업 {M.K}부터 진행 가능. 시작할까요?"** 한 줄로 안내

### 새 채팅 시작 시 사용자가 해야 할 것 (이상적으로는 0개)

1. 본 문서 경로만 알려주거나, `/init` 호출
2. Claude가 위 절차로 자동 파악 → 다음 작업 제시
3. 사용자는 "OK" 또는 "잠깐, 그 전에 ..."만 말하면 됨

### 핵심 원칙

- **본 문서가 SSOT** — 다른 곳에 진행 상태 기록 금지 (분산 SSOT 함정 피함)
- **각 Phase 완료 시 변경 로그 양식 필수** — 누락하면 다음 세션 인계 깨짐
- **사용자는 본 문서 경로 한 번만 알려주면 됨** — 그 외 컨텍스트 설명 불필요
