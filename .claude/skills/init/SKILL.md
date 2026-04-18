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
2. **rules/ 폴더 확인**: `rules/0_CODEX_RULES.md` + `rules/sub/` 서브 룰북 존재 여부
3. **★ 활성 MVP + 마일스톤 자동 로드 (2층 컨텍스트 복원)**:
   - `progress/active_mvp.txt` 읽기 → MVP 번호·상태 (예: `11:in-progress`)
   - `progress/active_milestone.txt` 읽기 → 현재 마일스톤·상태 (예: `M2:in-progress`)
   - 해당 마일스톤 기획 문서 자동 탐색: `progress/mvp{N}/M{X}_*.md`
   - 문서가 있으면 내용 일부 로드해 "이 마일스톤이 뭐 하는 건지" 요약
   - `bash scripts/kpi-report.sh --quick` 실행 → Hard 게이트 현재값 한 줄 요약
4. **진행 중인 작업 자동 감지 (세션 Resume)**:
   - Git 브랜치명에서 태스크 키워드 추출 (예: `feature/M2-feed-perf`)
   - `progress/` 폴더에서 현재 브랜치·마일스톤과 관련된 최신 문서 자동 로딩
   - `git status` + `git diff --stat`으로 미커밋 변경사항
5. **피드백 확인**: 메모리 `feedback_*.md` 최근 3개

## 출력

```
# 프로젝트 상태

- CLAUDE.md: {존재}
- rules/: {파일 수}
- Git 브랜치: {브랜치명}
- 변경사항: {staged/unstaged 수}

# MVP·마일스톤 컨텍스트  ← 신규
- 활성 MVP: MVP{N} [{state}]
- 활성 마일스톤: {M{X}} [{state}]
- 마일스톤 문서: progress/mvp{N}/M{X}_*.md
- 마일스톤 목표: "{문서에서 추출한 한 문장}"
- KPI 요약: {kpi-report --quick 출력}

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
