---
name: weekly-report
description: "비개발자용 주간 리포트 자동 생성. 금주 한일 + 차주 계획을 비즈니스 관점으로 요약. 트리거 키워드: 주간보고, 주간리포트, weekly report."
context: fork
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - mcp__codex__codex
  - mcp__codex__codex-reply
  - mcp__sequential-thinking__sequentialthinking
---

# 비개발자용 주간 리포트 생성 (/weekly-report)

> `/weekly-report` 또는 `/weekly-report 3/23~3/29`로 호출.

## 수행 절차

### Phase 1: 데이터 수집 (병렬)
- Git 히스토리 수집
- progress/ 파일 수집
- feedback/ 파일 수집

### Phase 2: 초안 작성
- 비개발자가 이해할 수 있는 용어로 작성

### Phase 3: 3자 페르소나 검증 (병렬)
- 비개발 페르소나에게 피드백 수집

### Phase 4: 최종본 작성

## 작성 규칙

- **"뭘 했다"가 아니라 "그래서 뭐가 좋아졌다"로 쓴다**
- 가능하면 전후 수치를 포함
- 기술 용어 → 일상 언어로 변환
- 기술 내부 작업은 관련 항목의 부연으로

## 출력: `progress/weekly/{YYMMDD}_weekly_report.md`
