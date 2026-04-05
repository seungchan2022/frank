---
name: step-6
description: 구현. 서브태스크 코드 작성.
context: fork
allowed-tools:
  - Read
  - Edit
  - Write
  - Glob
  - Grep
  - Bash
  - mcp__context7__resolve-library-id
  - mcp__context7__query-docs
  - mcp__serena__find_symbol
  - mcp__serena__get_symbols_overview
  - mcp__serena__find_referencing_symbols
  - mcp__serena__read_file
  - mcp__serena__list_dir
  - mcp__exa__web_search_exa
---

# Step 6: 구현

## 수행 작업

1. **구현 계획 확인**: 서브태스크 문서의 구현 방법 확인
2. **라이브러리 문서 조회**: context7 MCP로 최신 API 확인
3. **기존 코드 분석**: serena MCP로 코드 구조 파악
4. **코드 작성**: 규칙을 준수하며 구현

## 구현 원칙

- 프로젝트 아키텍처 패턴 준수
- TDD: 테스트 먼저, 구현 후
- 문서에 명시되지 않은 기능 추가 금지

## MCP 활용

```
# 라이브러리 문서 조회
context7 -> resolve-library-id -> query-docs

# 기존 코드 분석
serena -> get_symbols_overview -> find_symbol

# 대규모 코드 패턴 검색
morph-mcp -> warpgrep_codebase_search

# 외부 리서치 (새 기술 도입 시)
exa -> web_search_exa
perplexity -> perplexity_ask
```

## 다음 단계

→ `/step-7` (리팩토링)
