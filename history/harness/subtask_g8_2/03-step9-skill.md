# ST-03 — step-9 SKILL.md 수정 (미체크 목록 출력 + 확인 대기)

> 상위: 260420_g8_2_commit_block
> 번호: 03
> 상태: ✅ 완료 (260420)
> 유형: 수정

## 스코프

### In-Scope
- `.claude/skills/step-9/SKILL.md` 수정
- Feature List 파싱 → 미체크 목록 출력 + 사용자 확인 대기
- 미체크 없으면 git commit 진행
- 커밋 성공 후 `progress/active_subtask.txt` → `none` 클리어
- 세 가지 선택지 안내:
  1. /step-8로 돌아가 직접 검증 후 재진입
  2. 해당 항목을 `[~] deferred (사유)` 또는 `[-] N/A (사유)` 처리 후 재진입
  3. `--skip-manual` + 사유 입력 (긴급 우회)

### Out-of-Scope
- 실제 파싱 로직 (feature-list-check.sh — ST-01)
- pre-commit hook (ST-02)

## 인터뷰 결정사항

| 항목 | 결정 |
|---|---|
| 차단 후 안내 | B — 세 가지 선택지 (deferred/N-A 정식 경로 포함) |
| active_subtask.txt 클리어 | 커밋 성공 직후 `none` 초기화 |
| --skip-manual 우회 | bypass.log에 사유 기록 필수 |

## 변경 파일

| 파일 | 변경 유형 |
|---|---|
| `.claude/skills/step-9/SKILL.md` | 수정 |

## Feature List

<!-- size: 소형 | count: 12 | skip: false -->

### 기능 (F)
- [x] F-01 step-9 진입 시 active_subtask.txt 읽어 Feature List 파싱
- [x] F-02 미체크 항목 카테고리별 목록 출력 후 사용자 확인 대기
- [x] F-03 미체크 없으면 확인 없이 git commit 진행
- [x] F-04 모든 커밋(feat/test/docs) 완료 후 active_subtask.txt → `none` 클리어
- [x] F-05 세 가지 선택지 안내 (step-8 / deferred·N-A / --skip-manual)
- [x] F-06 --skip-manual 선택 시 사유 입력 받아 bypass.log 기록

### 엣지 케이스 (E)
- [x] E-01 active_subtask.txt 없거나 `none`이면 Feature List 체크 생략
- [x] E-02 Feature List 섹션 없는 서브태스크 → 체크 없이 진행
- [x] E-03 `docs:`/`chore:` 커밋 태그 → Feature List 체크 생략

### 테스트 (T)
- [x] T-01 미체크 있는 상태 → 목록 출력 + 대기 확인 (drive-through)
- [x] T-02 모든 항목 체크 완료 → 바로 커밋 진행 확인
- [x] T-03 --skip-manual 입력 → bypass.log 기록 확인
