---
name: step-9
description: 커밋. Git 커밋 (푸시 금지).
allowed-tools:
  - Bash
  - Read
  - Write  # 쓰기 범위: active_subtask.txt 클리어(none) 전용 — bypass.log append는 Bash 사용
---

# Step 9: 커밋

> 커밋 형식/태그/금지 사항: `rules/sub/git.md` (SSOT)

## 수행 작업

```
[1] Feature List 검증 (active_subtask.txt 기반)
      ↓
[2] 변경사항 확인: git status, git diff
      ↓
[3] 커밋 메시지 작성 (rules/sub/git.md 형식)
      ↓
[4] 사용자 확인 후 커밋
      ↓
[5] 커밋 성공 후 active_subtask.txt → none 클리어
```

> **절대 금지**: `Co-Authored-By:` 태그, `git push`

---

## [1] Feature List 검증

커밋 전 **반드시** 아래 순서로 진행.

### 1-A. active_subtask.txt 확인

```bash
cat progress/active_subtask.txt 2>/dev/null || echo "none"
```

- `none` 또는 파일 없음 → Feature List 검증 생략, [2]로 이동
- 경로 있음 → 해당 서브태스크 문서의 `## Feature List` 섹션 파싱

### 1-B. 태그 판정

| 커밋 태그 | Feature List 검증 |
|-----------|-------------------|
| `feat` / `fix` / `test` | **필수** |
| `docs` / `chore` / `style` / `refactor` | **생략** (통과) |

`docs:` / `chore:` 단독 커밋이면 Feature List 검증 없이 [2]로 이동.

### 1-C. 미체크 항목 확인

Feature List 섹션의 `- [ ]` 미체크 항목이 있으면:

```
⚠️ Feature List 미체크 항목 N개

  [카테고리]
  - [ ] F-01 항목 설명
  - [ ] T-02 항목 설명

→ 아래 세 가지 방법 중 하나를 선택하세요:

  1️⃣  /step-8 로 돌아가 실측 검증 후 재진입 (권장)
  2️⃣  해당 항목을 [~] deferred (사유) 또는 [-] N/A (사유) 처리 후 재진입
  3️⃣  긴급 우회: "--skip-manual" 입력 + 사유 입력
       → progress/feature-list/bypass.log 에 자동 기록
```

사용자 선택 대기. **사용자 확인 없이 커밋 진행 금지**.

### 1-D. --skip-manual 우회 처리

사용자가 `--skip-manual` 선택 시:

1. 사유 입력 받기 (필수 — 빈 사유 거부)
2. `progress/feature-list/bypass.log` 에 Bash로 append (Write 도구 사용 금지 — 덮어쓰기 위험):
   ```bash
   mkdir -p progress/feature-list
   echo "$(date +%Y-%m-%dT%H:%M:%S) BYPASS | channel: manual | subtask: {경로} | unchecked: N | reason: {사유}" >> progress/feature-list/bypass.log
   ```
3. 커밋 진행

---

## [2] 변경사항 확인

```bash
git status
git diff --cached
```

---

## [3] 커밋 메시지 작성

`rules/sub/git.md` 형식 준수.

---

## [4] 사용자 확인 후 커밋

```bash
git add {파일 목록}
git commit -m "..."
```

---

## [5] 커밋 성공 후 active_subtask.txt 클리어

커밋이 성공하면:

```bash
echo "none" > progress/active_subtask.txt
```

Write 도구로 직접 갱신해도 된다. **이 단계를 잊으면 다음 커밋에서 오검증이 발생하므로 반드시 수행**.

---

## 완료

서브태스크 완료 시:
1. 서브태스크 상태를 "완료"로 변경
2. 다음 서브태스크로 이동
3. 모든 서브태스크 완료 시 메인태스크 "완료"로 변경
