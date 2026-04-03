---
name: step-9
description: 커밋. Git 커밋 (푸시 금지).
allowed-tools:
  - Bash
  - Read
---

# Step 9: 커밋

> 커밋 형식/태그/금지 사항: `rules/sub/git.md` (SSOT)

## 수행 작업

1. **변경사항 확인**: `git status`, `git diff`
2. **커밋 메시지 작성**: `rules/sub/git.md` 형식 준수
3. **사용자 확인 후 커밋**: `git add` + `git commit`

> **절대 금지**: `Co-Authored-By:` 태그, `git push`

## 완료

서브태스크 완료 시:
1. 서브태스크 상태를 "완료"로 변경
2. 다음 서브태스크로 이동
3. 모든 서브태스크 완료 시 메인태스크 "완료"로 변경
