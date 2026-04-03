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
  - mcp__morph-mcp__warpgrep_codebase_search
  - mcp__exa__web_search_exa
  - mcp__perplexity__perplexity_search
  - mcp__perplexity__perplexity_ask
  - Task
---

# Step 1: 메인태스크 설정

> **권장**: 새 워크플로우 시작 시 `/workflow`를 사용하세요.

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

### 기본 질문 (항상 포함)
- 무엇을 구현하고 싶으신가요? (구체화)
- 이 기능의 목적은 무엇인가요?
- 예상되는 입력/출력은 무엇인가요?
- 제약사항이 있나요?

## 산출물

`progress/{YYMMDD}_{메인태스크명}.md`

## 다음 단계

→ `/step-2` (룰즈 검증)
