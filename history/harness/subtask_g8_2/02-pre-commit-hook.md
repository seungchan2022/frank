# ST-02 — pre-commit hook에 feature-list-check.sh 연계

> 상위: 260420_g8_2_commit_block
> 번호: 02
> 상태: ✅ 완료 (260420)
> 유형: 수정

## 스코프

### In-Scope
- `.git/hooks/pre-commit` 수정
- `kpi-check.sh` 이후에 `feature-list-check.sh` 호출 추가
- `docs:`/`chore:` 커밋 자동 스킵 (기존 kpi-check 정책 일치)

### Out-of-Scope
- feature-list-check.sh 내부 구현 (ST-01)

## 인터뷰 결정사항

| 항목 | 결정 |
|---|---|
| 실행 순서 | kpi-check.sh → feature-list-check.sh |
| 우회 환경변수 | `FEATURE_LIST_BYPASS=1` (ST-01에서 처리) |

## 변경 파일

| 파일 | 변경 유형 |
|---|---|
| `.git/hooks/pre-commit` | 수정 |

## Feature List

<!-- size: 소형 | count: 10 | skip: false -->

### 기능 (F)
- [x] F-01 kpi-check.sh 성공 후 feature-list-check.sh 호출
- [x] F-02 feature-list-check.sh exit 1 시 커밋 차단
- [x] F-03 active_subtask.txt가 `none`이면 feature-list-check.sh 조기 종료 (통과)

### 엣지 케이스 (E)
- [x] E-01 kpi-check.sh 실패 시 feature-list-check.sh 미호출 (순서 보장)
- [x] E-02 feature-list-check.sh 파일 없을 때 경고 출력 후 통과

### 테스트 (T)
- [x] T-01 미체크 있는 상태에서 `git commit -m "feat: ..."` → 차단 확인
- [x] T-02 active_subtask.txt가 `none`인 상태에서 커밋 → 통과 확인
- [x] T-03 `FEATURE_LIST_BYPASS=1 git commit` → 통과 확인

### 회귀 (G)
- [x] G-01 기존 kpi-check.sh 차단 동작 영향 없음 확인
- [x] G-02 commit-msg hook 동작 영향 없음 확인
