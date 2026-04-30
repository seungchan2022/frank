---
name: progress-cleanup
description: "진행상황 정리. 완료 마일스톤 아카이빙 + stale TODO 감지 + history/INDEX.md 갱신. 트리거 키워드: progress 정리, cleanup, 아카이빙."
context: fork
allowed-tools:
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - Bash
---

# /progress-cleanup 스킬

완료된 마일스톤 문서를 history/로 아카이빙하고, INDEX.md를 갱신한다.

## 프로세스

### 1. 스캔
- `progress/` 하위 파일 전체 스캔
- 각 파일의 상태 판별: COMPLETED / STALE / ACTIVE

### 2. 아카이빙 (사용자 확인 후)
- 완료된 마일스톤 관련 파일을 `history/{마일스톤명}/`으로 이동
- `git mv`로 이동 (git 이력 보존)
- 폴더명: 소문자, 간결하게 (예: `mvp1`, `mvp1.5`, `mvp2`). **0.5 단위는 점 살림** (`mvp1.5` ≠ 정수 `mvp15`)

### 3. Stale 감지
- 7일 이상 미수정 + 상태가 "진행중"인 파일 → STALE로 표시
- STALE 파일 목록을 사용자에게 보고

### 4. INDEX.md 갱신
`history/INDEX.md`를 갱신한다:

```markdown
## {마일스톤명} — {한줄 설명} ({기간})

> {기간/규모 요약}

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 기획 | [{제목}]({경로}) | {설명} |
| 분석 | [{제목}]({경로}) | {설명} |
| 회고 | [{제목}]({경로}) | {설명} |
```

하단에 "빠른 검색 가이드" 테이블도 갱신:

```markdown
## 빠른 검색 가이드

| 찾고 싶은 것 | 참조 파일 |
|-------------|----------|
| ... | ... |
```

### 5. 결과 보고
- 이동된 파일 수
- STALE 파일 목록
- INDEX.md 갱신 내용 요약

## 규칙
- `progress/` 루트에는 **진행 중인 작업 문서만** 유지
- 아카이빙 전 반드시 사용자 확인
- `git mv`로 이동하여 이력 보존
