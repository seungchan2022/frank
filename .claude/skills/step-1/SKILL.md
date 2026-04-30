---
name: step-1
description: 메인태스크 설정. 요구사항 인터뷰 후 태스크 분해.
context: fork
allowed-tools:
  - Read
  - Write
  - mcp__sequential-thinking__sequentialthinking
  - mcp__codex__codex
  - mcp__codex__codex-reply
  - mcp__serena__find_symbol
  - mcp__serena__get_symbols_overview
  - mcp__serena__list_dir
  - mcp__serena__search_for_pattern
  - mcp__exa__web_search_exa
  - Task
---

# Step 1: 메인태스크 설정

> **권장**: 새 워크플로우 시작 시 `/workflow`를 사용하세요.
>
> **인터뷰 강제 룰 SSOT**: 인터뷰 완료 게이트, Q{M}/{T} 카운터, 스텝 헤더 등 공통 강제 규칙은 `.claude/skills/workflow/SKILL.md` 참조. 본 SKILL은 step-1 고유 절차만 정의.

## 수행 작업

```
[1] 요구사항 수신
       ↓
[2] (선택) 필요 시 /debate 명령으로 3자 토론 가능
       ↓
[3] 인터뷰 질문 생성 (A/B/C 추천 형식, 1개씩)
       ↓
[4] 사용자 인터뷰
       ↓
[5] (선택) 새 기술 도입 시 Exa/Perplexity로 외부 리서치
       ↓
[6] 태스크 분해 (sequential_thinking)
       ↓
[7] 메인태스크 문서 생성
```

## 인터뷰 질문

### 질문 포맷 (필수 준수)

모든 인터뷰 질문은 **반드시 아래 형식**으로 출력한다:

```
Q{N}/{총수} — {질문 제목}

{질문 배경 1~2줄}

A. {선택지 설명}
B. {선택지 설명}  ← (추천)
C. {선택지 설명}
```

- 선택지는 2~3개, 추천 옵션에는 반드시 `← (추천)` 마킹
- 추천 근거는 선택지 뒤 한 줄로 간략히 명시
- 질문은 **1개씩** 제시 (한 번에 여러 개 금지)
- 사용자가 번호/알파벳으로 답하면 다음 질문으로 진행

### 인터뷰 라이프사이클 (state file — Phase 2)

단독 호출 시 (보통 `/workflow`가 진행하지만 step-1 단독 진입 가능):

1. 인터뷰 시작 (총 질문 수 확정 직후):
   ```bash
   echo "step-1" > progress/active_step.txt
   cat > progress/active_interview.json <<EOF
   {
     "step": "step-1",
     "current": 1,
     "total": {N},
     "remaining": ["{Q1 제목}", "{Q2 제목}", "..."],
     "started_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
   }
   EOF
   ```

2. 매 답변 후: `current` 1 증가, `remaining` 첫 항목 제거.

3. 인터뷰 완료: `echo '{}' > progress/active_interview.json`

이 절차는 Phase 3 PreToolUse hook이 인터뷰 미완료 시 Edit/Write 차단에 사용. 누락 시 stale 오판으로 잘못 차단됨.

### 기본 질문 영역 (항상 포함)
- 무엇을 구현하고 싶으신가요? (구체화)
- 이 기능의 목적은 무엇인가요?
- 예상되는 입력/출력은 무엇인가요?
- 제약사항이 있나요?

## 산출물

`progress/{YYMMDD}_{메인태스크명}.md`

## 다음 단계

→ `/step-2` (룰즈 검증)
