---
name: step-7
description: 리팩토링+코드리뷰. 리팩토링 3R → 리뷰 3R (Codex 최적화).
context: fork
allowed-tools:
  - Read
  - Edit
  - Write
  - Glob
  - Grep
  - Bash
  - mcp__codex__codex
  - mcp__codex__codex-reply
  - mcp__serena__find_symbol
  - mcp__serena__get_symbols_overview
  - mcp__serena__find_referencing_symbols
  - mcp__serena__read_file
  - mcp__serena__search_for_pattern
  - mcp__morph-mcp__warpgrep_codebase_search
  - Task
---

# Step 7: 리팩토링 + 코드 리뷰

> 조건: 코드 수정/추가가 있는 경우에만 실행

## 전체 흐름

```
[Phase 1] 리팩토링 3R (Claude → Codex/Serena 피드백 → 반영) x 3
       ↓
[Phase 2] 코드 리뷰 3R (보안/성능 → 아키텍처 → 합동 최종)
       ↓
[최종] 사용자 검수
```

## Codex 최적화 규칙

- **전체 파일 금지** — `git diff` 결과만 Codex에 전달
- **병렬 실행** — Claude 리팩토링과 Codex 리뷰를 Agent 도구로 병렬
- **소규모 변경 스킵** — 파일 3개 이하 + 50줄 이하 시 Codex 호출 생략

## Phase 1: 리팩토링 3R

```
R1: Claude 리팩토링 → Codex diff 피드백 (병렬) → 반영
R2: Claude 리팩토링 → Serena 구조 피드백 → 반영
R3: Claude 최종 리팩토링 → Codex 검증 → 확정
```

원칙: KISS / DRY / YAGNI. 기능 변경 금지.

## Phase 2: 코드 리뷰 3R

```
R1: Claude 보안/성능 리뷰 → 수정
R2: Codex 아키텍처/스타일 리뷰 (diff 기반, 병렬) → 수정
R3: Claude + Codex 합동 최종 → 승인/반려
```

## 다음 단계

→ `/step-8` (테스트)
