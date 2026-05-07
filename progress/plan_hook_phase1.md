# 훅 시스템 보완 Phase 1 구현 계획

> 작성: 2026-05-07
> 관련 부채: DEBT-HOOK-01, DEBT-HOOK-02, DEBT-HOOK-04

---

## 범위

| 부채 | 내용 | 포함 |
|------|------|------|
| DEBT-HOOK-01 | 각 step SKILL.md에 갱신 지시 추가 (A안 백업) | ✅ |
| DEBT-HOOK-02 | PostToolUse 훅으로 Skill 호출 시 active_step.txt 자동 갱신 (B안 근본) | ✅ |
| DEBT-HOOK-04 | step-8 자동화 가능 테스트를 Claude가 직접 실행 | ✅ |
| DEBT-HOOK-03 | /next 자동 흐름 재설계 | ❌ 제외 (Phase 2로 분리) |

---

## 작업 목록

### ST-1. PostToolUse 훅 작동 검증 (착수 전 필수)

훅에서 `tool_input.skill` 값을 실제로 읽을 수 있는지 검증.
B안 전체가 이 전제 위에 있으므로 코드 작성 전에 확인.

```bash
# 검증용 임시 훅: tool_input 구조 파악
echo "$HOOK_INPUT" >> /tmp/hook_debug.log
```

검증 통과 시 ST-2 진행. 실패 시 A안(DEBT-HOOK-01)만으로 진행.

---

### ST-2. PostToolUse 훅 구현 (DEBT-HOOK-02)

**파일**: `.claude/hooks/update-active-step.sh` 신규 생성

**로직**:
- `tool_name == "Skill"` 이고 `tool_input.skill`이 `step-[1-9]` 패턴일 때만 갱신
- `/next`, `/workflow`, `/init` 등 다른 Skill 호출은 갱신 안 함 (필터 필수)

**설정**: `.claude/settings.json` PostToolUse 훅으로 등록

---

### ST-3. 각 step SKILL.md 갱신 지시 추가 (DEBT-HOOK-01, A안 백업)

**대상**: step-2, step-3, step-5, step-6, step-7, step-8

각 SKILL.md 진입부 첫 번째 액션으로 추가:
```bash
echo "step-N" > progress/active_step.txt
```

step-9는 이미 `echo "none"` 코드가 있으나 실행 안 된 선례 있음 → B안 훅이 주, A안은 백업.

---

### ST-4. step-8 SKILL.md 자동화 항목 직접 실행 (DEBT-HOOK-04)

**현재 문제**: 자동화 가능 항목(린트, 빌드, 단위 테스트, E2E)을 Claude가 사용자에게 위임.

**수정 방향**:
- 자동화 가능 항목 목록 명시 → Bash 도구로 Claude가 직접 실행
- 실패 시 → 해당 step(step-6 또는 step-7)으로 복귀 안내
- 시각 확인(UI 렌더링, 레이아웃) 항목만 사용자에게 요청

---

## 완료 조건

- [x] ST-1: 훅 작동 검증 통과 — stdin JSON에서 `tool_input.skill` 읽기 확인 (2026-05-07)
- [x] ST-2: `/step-2` 호출 시 active_step.txt = "step-2"으로 자동 갱신 확인 (2026-05-07 E2E 통과)
- [x] ST-3: step-2,3,5,6,7,8 SKILL.md 갱신 지시 추가 (2026-05-07)
- [x] ST-4: step-8 자동 실행 원칙 blockquote 명시 (2026-05-07)
- [x] DEBT-HOOK-01, 02, 04 상태 → RESOLVED 갱신 (2026-05-07)

---

## 제외 항목 (Phase 2)

- DEBT-HOOK-03: /next 자동 흐름 재설계 — 별도 설계 후 착수
  - 선행 조건: Phase 1 완료 후 active_step.txt 정확도 검증
  - 설계 필요: step 완료 시그널 정의, 자동 진행/멈춤 기준 구현
