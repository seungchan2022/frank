# ST-04 — step-4 SKILL.md 수정 (active_subtask.txt 기록)

> 상위: 260420_g8_2_commit_block
> 번호: 04
> 상태: ✅ 완료 (260420)
> 유형: 수정

## 스코프

### In-Scope
- `.claude/skills/step-4/SKILL.md` 수정
- Feature List 초안 생성 직후 `progress/active_subtask.txt`에 서브태스크 문서 경로 기록
- 경로 포맷: `progress/subtask/{경로}/{파일명}.md` (progress 상대경로)

### Out-of-Scope
- active_subtask.txt 클리어 (ST-03 — step-9 커밋 성공 후)
- 이번 §8.2 사이클 자체는 수동으로 진행 (부트스트랩 딜레마 — 생략 결정)

## 인터뷰 결정사항

| 항목 | 결정 |
|---|---|
| 기록 시점 | Feature List 초안 생성 직후 |
| 이번 사이클 | active_subtask.txt 생략 (부트스트랩) |
| 클리어 위치 | ST-03 (step-9) 담당 |

## 변경 파일

| 파일 | 변경 유형 |
|---|---|
| `.claude/skills/step-4/SKILL.md` | 수정 |
| `progress/active_subtask.txt` | 신규 생성 (초기값 `none`) |

## Feature List

<!-- size: 소형 | count: 8 | skip: false -->

### 기능 (F)
- [x] F-01 Feature List 초안 생성 단계 직후 active_subtask.txt 기록 단계 추가
- [x] F-02 기록 포맷: `progress/subtask/{경로}/{파일명}.md`
- [x] F-03 `progress/active_subtask.txt` 초기 파일 생성 (값: `none`)

### 엣지 케이스 (E)
- [x] E-01 소형 서브태스크 skip 시 active_subtask.txt 기록 생략 안내

### 테스트 (T)
- [x] T-01 step-4 완료 후 active_subtask.txt에 경로 기록 확인 (drive-through)
- [x] T-02 경로 포맷이 progress 상대경로인지 확인

### 회귀 (G)
- [x] G-01 기존 step-4 인터뷰 + Feature List 생성 흐름 영향 없음 확인
- [x] G-02 step-8 순회 검증 흐름 영향 없음 확인
