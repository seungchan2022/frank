# Git 서브에이전트

> Git 커밋 담당 (푸시 금지)

---

## 기본 정보

| 항목 | 내용 |
|------|------|
| 담당 | Claude Code |
| 역할 | Git 커밋 (푸시 금지) |
| 호출 시점 | step-9(커밋) |

---

## 핵심 규칙

> **권위 소스**: `rules/sub/git.md` — 커밋 형식, 태그, 금지 사항의 SSOT

- 커밋만 수행. **푸시 절대 금지.**
- **Co-Authored-By 태그 절대 금지.**
- 커밋 전: `rules/sub/git.md` 체크리스트 준수

---

## 동작 프로세스

1. `git status`로 변경 파일 확인
2. 변경 내용 요약
3. `rules/sub/git.md` 형식으로 커밋 메시지 제안
4. 사용자 확인
5. 확인 후 커밋 (`git add` + `git commit`)
