# ST-01 — scripts/feature-list-check.sh 신규 구현

> 상위: 260420_g8_2_commit_block
> 번호: 01
> 상태: ✅ 완료 (260420)
> 유형: 신규 기능

## 스코프

### In-Scope
- `scripts/feature-list-check.sh` 신규 생성
- `progress/active_subtask.txt` 읽어 파싱 대상 경로 획득
- `## Feature List` 섹션 파싱 → 4상태 판정
- 미체크 `[ ]` 1개 이상 시 한국어 에러 출력 + exit 1
- 파싱 실패 시 차단 + 위치 안내 (한국어)
- 커밋 태그 판정: `feat/fix/test` → 차단, `docs/chore/style/refactor` → 통과
- `FEATURE_LIST_BYPASS=1` 환경변수 우회 + `progress/feature-list/bypass.log` 기록
- Feature List 섹션 없음 → 통과 (skip 처리)

### Out-of-Scope
- pre-commit hook 연결 (ST-02)
- active_subtask.txt 쓰기/클리어 (ST-03, ST-04)

## 인터뷰 결정사항

| 항목 | 결정 |
|---|---|
| 입력 소스 | `progress/active_subtask.txt` (active_mvp.txt 패턴) |
| 에러 출력 언어 | 한국어 |
| 파싱 실패 처리 | 차단 + 위치 안내 |
| 우회 | `FEATURE_LIST_BYPASS=1` + bypass.log 기록 |

## 파싱 실패 유형 (G1 규약 재사용)

| 실패 유형 | 처리 |
|---|---|
| ID 포맷 오류 (`F01` → `F-01`) | 차단 + 위치 안내 |
| 상태 기호 불일치 | 차단 + 위치 안내 |
| `[~]`/`[-]` 뒤 사유 누락 | 차단 + 위치 안내 |
| HTML 메타 누락 | 차단 + 위치 안내 |
| count 불일치 | 차단 + 위치 안내 |
| Feature List 섹션 없음 | 통과 |

## 출력 포맷 (미체크 시)

```
❌ Feature List 미체크 항목 N개 — 커밋 차단

  [카테고리]
  F-01 [ ] 항목 설명
  E-02 [ ] 항목 설명

→ /step-8로 돌아가 검증 후 재진입하거나
   FEATURE_LIST_BYPASS=1 + progress/feature-list/bypass.log 기록 후 진행
```

## 변경 파일

| 파일 | 변경 유형 |
|---|---|
| `scripts/feature-list-check.sh` | 신규 생성 |
| `progress/feature-list/bypass.log` | 신규 생성 (디렉토리 포함) |

## Feature List

<!-- size: 중형 | count: 18 | skip: false -->

### 기능 (F)
- [x] F-01 active_subtask.txt 읽기 성공 시 경로 파싱
- [x] F-02 active_subtask.txt 없거나 none일 때 통과 처리
- [x] F-03 `## Feature List` 섹션 탐지 및 파싱
- [~] deferred (카테고리 헤더 없이 ID 순 목록 출력으로 구현 — 실용상 충분) F-04 미체크 `[ ]` 항목 집계 및 카테고리별 분류 출력
- [x] F-05 `FEATURE_LIST_BYPASS=1` 환경변수 우회 + bypass.log 기록

### 엣지 케이스 (E)
- [x] E-01 Feature List 섹션 없음 → 통과
- [x] E-02 모든 항목 `[x]`/`[~]`/`[-]` → 통과
- [x] E-03 active_subtask.txt 파일 없음 → 통과 (경고 없이)
- [x] E-04 active_subtask.txt 값이 `none`이면 체크 없이 통과

### 에러 처리 (R)
- [x] R-01 ID 포맷 오류 감지 시 한국어 에러 + 라인 번호 출력
- [x] R-02 `[~]`/`[-]` 사유 누락 감지 시 한국어 에러 + 라인 번호 출력
- [x] R-03 bypass.log 디렉토리 없으면 자동 생성

### 테스트 (T)
- [x] T-01 미체크 있는 fixture → exit 1 확인
- [x] T-02 모든 항목 체크된 fixture → exit 0 확인
- [x] T-03 `FEATURE_LIST_BYPASS=1` → exit 0 + bypass.log 기록 확인
- [x] T-04 active_subtask.txt가 `none`일 때 → exit 0 확인
- [x] T-05 Feature List 없는 fixture → exit 0 확인

### 회귀 (G)
- [x] G-01 kpi-check.sh 동작에 영향 없음 확인
