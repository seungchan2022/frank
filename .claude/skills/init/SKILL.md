---
name: init
description: 프로젝트 초기화. 현재 상태 파악 및 컨텍스트 로드.
allowed-tools:
  - Read
  - Glob
  - Grep
  - Bash
---

# 프로젝트 초기화 (/init)

## 수행 작업

1. **CLAUDE.md 확인**: 프로젝트 규칙 및 가이드라인 로드
2. **rules/ 폴더 확인**: `rules/0_CODEX_RULES.md` + `rules/sub/` 서브 룰북 존재 여부 확인
3. **진행 중인 작업 자동 감지 (세션 Resume)**:
   - 현재 Git 브랜치명에서 태스크 키워드 추출
   - `progress/` 폴더에서 현재 브랜치와 관련된 최신 메인태스크 문서 자동 로딩
   - `git diff --stat` + `git status`로 미커밋 변경사항 파악
   - 문서의 상태를 읽어 "마지막 세션 요약" 자동 출력
4. **Git 상태 확인**: 현재 브랜치 및 변경사항 파악
5. **피드백 확인**: `feedback/` 폴더의 열린 피드백 목록 표시 (status: open인 항목만)

## 출력

```
# 프로젝트 상태

- CLAUDE.md: {존재 여부}
- rules/: {파일 수}개 서브 룰북
- Git 브랜치: {브랜치명}
- 변경사항: {staged/unstaged 수}

# 세션 Resume
- 진행 중 태스크: {태스크명}
- 현재 단계: {Step N — 설명}
- 다음 단계: {추천 액션}
```

## 절대 금지 규칙 (매 세션 준수)

- **`Co-Authored-By:` 커밋 태그 절대 금지**
- **`git push` 절대 금지** — 푸시는 사용자가 직접 수행
- **`git commit` 자동 실행 금지** — 사용자 허락 후에만 커밋

## 다음 단계

- 새 태스크 시작: `/workflow`
- 기존 태스크 계속: `/step-{N}` 또는 `/next`
