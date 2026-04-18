---
name: milestone-review
description: "마일스톤 리뷰/갱신. 로드맵 진행 상황 검토 + 마일스톤 재조정. 트리거 키워드: 마일스톤 리뷰, milestone review, 로드맵 리뷰, 진행 상황."
context: fork
allowed-tools:
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - Bash
  - mcp__sequential-thinking__sequentialthinking
  - mcp__codex__codex
  - mcp__codex__codex-reply
  - mcp__serena__find_symbol
  - mcp__serena__get_symbols_overview
  - mcp__serena__list_dir
  - mcp__serena__search_for_pattern
  - mcp__tavily__tavily_search
  - mcp__exa__web_search_exa
  - mcp__mermaid__mermaid-mcp-app
  - Task
---

# 마일스톤 리뷰 (/milestone-review)

> 기존 로드맵의 진행 상황을 검토하고, 아이템별 상태를 추적하며, 필요 시 재조정한다.

## 실행 흐름

```
/milestone-review {프로젝트명?}
       ↓
[1] 로드맵 + 마일스톤 + 아이템 상태 수집
       ↓
[2] 진행 상황 대시보드
       ↓
[3] 건강도 평가
       ↓
[4] 재조정 제안 (필요 시)
       ↓
[5] 문서 갱신
       ↓
→ 다음 아이템/마일스톤 안내
```

---

## [1] 상태 수집

1. `progress/milestones/` 에서 프로젝트 디렉토리 탐색
   - 인자 없으면: 가장 최근 수정된 프로젝트 자동 선택
   - 여러 프로젝트 존재 시: 사용자에게 선택 요청
2. `_roadmap.md` 로드
3. 각 `M{N}_*.md`에서 마일스톤 상태 + 아이템 테이블 추출
4. 아이템 유형별 실행 결과 수집:
   - `feature` → `progress/` 메인태스크 문서 진행 상황
   - `research` → `progress/analysis/` 보고서 존재 여부
   - `decision` → 의사결정 기록 존재 여부
   - `chore` → Git 로그에서 관련 커밋 확인
   - `spike` → 프로토타입 결과 + 전환/폐기 여부

---

## [2] 진행 상황 대시보드

```
# 로드맵 진행 상황: {프로젝트명}

## 전체 진행률: {N}% ({완료}/{전체} 마일스톤)

### M1: {마일스톤명} [진행중]
| # | 아이템 | 유형 | 실행 | 상태 |
|---|--------|------|------|------|
| 1 | 북마크 시스템 | feature | /workflow | 진행중 (step-6) |
| 2 | DB 구조 결정 | decision | /debate | 완료 |
| 3 | CI 수정 | chore | 직접 | 완료 |

### M2: {마일스톤명} [대기]
| # | 아이템 | 유형 | 실행 | 상태 |
|---|--------|------|------|------|
| 1 | 퀴즈 생성 | feature | /workflow | 대기 |
| 2 | SR 알고리즘 조사 | research | /deep-analysis | 대기 |
```

---

## [3] 건강도 평가

### 평가 기준

| 지표 | 녹색 | 황색 | 적색 |
|------|------|------|------|
| 일정 | 예정대로 | 1~3일 지연 | 1주+ 지연 |
| 스코프 | 변경 없음 | 소폭 조정 | 대폭 변경 |
| 의존성 | 차단 없음 | 우회 가능 | 차단됨 |
| 아이템 진행 | 유형별 균형 | 특정 유형 지연 | 다수 아이템 정체 |
| spike 결과 | 검증 완료 | 일부 미완 | 핵심 spike 실패 |

### 건강도 표시

```
전체 건강도: [녹색/황색/적색]

- 일정: [상태] — {설명}
- 스코프: [상태] — {설명}
- 아이템 진행: [상태] — {유형별 현황}
- spike: [상태] — {검증 결과 요약}
```

---

## [4] 재조정 제안 (필요 시)

건강도가 황색/적색일 때:

### 재조정 유형

| 유형 | 설명 | 예시 |
|------|------|------|
| 스코프 축소 | 아이템을 다음 마일스톤으로 이동 | "M1에서 퀴즈 기능을 M3로 이동" |
| 마일스톤 분할 | 큰 마일스톤을 둘로 | "M1을 M1a + M1b로 분할" |
| 순서 변경 | 의존성 재조정 | "M3를 M2보다 먼저" |
| 유형 전환 | 아이템 유형 변경 | "spike 성공 → feature로 승격" |
| 마일스톤 병합 | 작은 것들 합치기 | "M3 + M4 → M3" |
| 타임라인 조정 | 기간 변경 | "M2를 1주 → 2주" |
| 아이템 폐기 | spike 실패 등 | "SR 알고리즘 spike 폐기" |

### 제안 형식

```
## 재조정 제안

현재 상황: {문제 요약}

제안 A (추천): {설명}
  - 영향: {영향 분석}

제안 B: {설명}
  - 영향: {영향 분석}

제안 C: 현행 유지
  - 리스크: {설명}
```

---

## [5] 문서 갱신

사용자 확정 후:

1. `_roadmap.md` 갱신 (타임라인 + 변경 이력)
2. 영향받는 `M{N}_*.md` 갱신 (아이템 상태/유형 변경)
3. 의존성 그래프 재생성 (D2/Mermaid)
4. `_discovery.md`의 수렴 결과 업데이트 (아이템 이동/폐기 시)

---

## [6] MVP 사이클 초기화 (다음 MVP 방향성)

**`/milestone-review`는 MVP 한 사이클의 끝/시작 경계에서 한 번 실행하는 스킬이다.**
각 마일스톤(M1, M2, ...) 단위 전이는 `/workflow`가 자동 처리하므로 여기서는 다루지 않는다.

### 호출 시점

1. 이전 MVP가 history로 아카이빙된 **직후**
2. 다음 MVP 방향성을 잡기 전 — 전체 진행 상황 한 번 훑기

### 수행 작업

1. `progress/active_mvp.txt` 확인
   - 값이 `{N+1}:planning` 또는 `none`이면 → 새 MVP 사이클 시작
   - 값이 `{N}:in-progress`이고 모든 마일스톤이 done 상태라면 → MVP 완료 프로세스 안내 (아래)

2. 이전 MVP 완료 마감 안내 (해당 시):
   ```
   📋 MVP{N}의 모든 마일스톤이 끝난 것으로 보입니다.
   다음 순서로 마무리하세요 (수동):
     1. progress/mvp{N}/_roadmap.md에서 모든 M done 상태 확인
     2. 필요한 문서 업데이트 (회고·INDEX 등)
     3. progress/mvp{N}/* → history/mvp{N}/ 이동 (progress-cleanup 스킬 사용 가능)
     4. history/mvp{N}/{YYMMDD}_mvp{N}_completion_retro.md 작성
     5. MVP 최종 KPI 검증 필요 시 active_mvp.txt를 "{N}:completing"으로 바꾼 뒤 /kpi 실행
     6. 마감 커밋 후 active_mvp.txt를 "{N+1}:planning", active_milestone.txt를 "none"으로 초기화
   ```

3. 다음 MVP 방향성 잡기:
   - 이전 회고에서 "다음에 할 것(Next)" 목록 추출
   - 현재 활성 부채(`progress/debt.md`) 목록 로드
   - 사용자 인터뷰 후 `/milestone "MVP{N+1} ..."` 호출 안내

### 이 스킬이 하지 않는 것

- M1, M2 각각의 completing 전이 (→ `/workflow`가 자동 처리)
- 매일 작업 상황 대시보드 (→ `/status` 또는 `/kpi`)
- 기획 상세화 (→ `/milestone`)

---

## 다음 단계

- **새 MVP 기획**: `/milestone "MVP{N+1} 설명"`
- **현재 상태 확인**: `/status` / `/kpi`
- **상세 분석**: `/deep-analysis`

---

