---
name: next
description: 다음 단계 자동 진행. 현재 단계 완료 처리 후 다음 단계 실행.
context: fork
allowed-tools:
  - Read
  - Write
  - Edit
  - Glob
  - Bash
  - mcp__sequential-thinking__sequentialthinking
  - mcp__codex__codex
  - mcp__codex__codex-reply
  - mcp__serena__find_symbol
  - mcp__serena__get_symbols_overview
  - mcp__serena__list_dir
  - mcp__serena__search_for_pattern
  - mcp__context7__resolve-library-id
  - mcp__context7__query-docs
  - Task
---

# 다음 단계 진행 (/next)

## 수행 작업

```
/next
   ↓
[1] 현재 단계 확인 (/status 로직)
   ↓
[2] 현재 단계 완료 조건 검증
   ↓
[3] 완료 처리 (문서 업데이트)
   ↓
[4] 다음 단계 자동 실행
```

## 단계별 완료 조건

| 단계 | 완료 조건 |
|------|----------|
| step-1 | 메인태스크 문서 생성됨 |
| step-2 | Codex 리뷰 완료, 위반사항 해결 |
| step-3 | 서브태스크 목록 작성됨 |
| step-4 | 현재 서브태스크 인터뷰 완료 |
| step-5 | 리뷰 완료, 합의 도출 |
| step-6 | 코드 구현 완료 |
| step-7 | 리팩토링 3R + 코드 리뷰 3R 완료 |
| step-8 | 린트 + 타입체크 + 테스트 통과 |
| step-9 | 커밋 완료 |

## 서브태스크 사이클

step-4 ~ step-9는 서브태스크별로 반복:

```
서브태스크 1: step-4 → step-5 → step-6 → step-7 → step-8 → step-9
                                                          ↓ /next
서브태스크 2: step-4 → step-5 → step-6 → step-7 → step-8 → step-9
                                                          ↓
                                                    모두 완료 시 메인태스크 완료
```
