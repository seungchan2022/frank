---
name: step-3
description: 서브태스크 분리. 메인태스크를 독립적 서브태스크로 분해.
context: fork
allowed-tools:
  - Read
  - Write
  - Bash
  - mcp__sequential-thinking__sequentialthinking
  - mcp__codex__codex
  - mcp__codex__codex-reply
  - mcp__serena__find_symbol
  - mcp__serena__get_symbols_overview
  - mcp__serena__list_dir
  - mcp__serena__search_for_pattern
  - Task
---

# Step 3: 서브태스크 분리

## 사전 작업 (자동, 묻지 않음)

```bash
# 1. main 최신 형상으로 갱신
git checkout main
git pull origin main

# 2. feature 브랜치 생성 + 이동
git checkout -b feature/$(date +%y%m%d)_{메인태스크_요약}
```

## 수행 작업

```
[0] (자동) main pull + feature 브랜치 생성
       ↓
[1] 메인태스크 분석
       ↓
[2] (선택) 필요 시 /debate 명령으로 3자 토론 가능
       ↓
[3] 태스크 분해 (sequential_thinking)
       ↓
[4] 의존성 분석
       ↓
[5] 서브태스크 목록 문서화
       ↓
[6] (자동) D2로 의존성 DAG 다이어그램 생성
```

## 분리 원칙

- **단일 책임**: 각 서브태스크는 하나의 목적
- **독립 실행**: 가능한 한 독립적으로 실행 가능
- **명확한 산출물**: 각 서브태스크의 결과물 정의
- **적정 크기**: 1-2시간 내 완료 가능한 단위

## 다음 단계

→ `/step-4` (서브태스크 인터뷰)
