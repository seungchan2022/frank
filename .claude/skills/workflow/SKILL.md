---
name: workflow
description: 9단계 워크플로우 시작. 요구사항 수신 즉시 3자 토론 자동 시작.
context: fork
allowed-tools:
  - Read
  - Write
  - mcp__sequential-thinking__sequentialthinking
  - mcp__codex__codex
  - mcp__codex__codex-reply
  - mcp__serena__find_symbol
  - mcp__serena__get_symbols_overview
  - mcp__serena__list_dir
  - mcp__serena__search_for_pattern
  - Task
---

# 워크플로우 시작 (/workflow)

> **본 SKILL은 9단계 워크플로우의 진입점이자 강제 룰의 SSOT.**
> 스텝 헤더 / 인터뷰 완료 게이트 / 피드백 후 구현 진입 / 서브에이전트 알림 등 모든 강제 규칙의 정확한 형식·예외·복구 절차는 본 파일에서 정의된다.
> `rules/0_CODEX_RULES.md §9`, `CLAUDE.md`, `step-1/SKILL.md`, `step-4/SKILL.md`는 본 SSOT를 가리키는 링크만 보유.

## 실행 흐름

```
/workflow "M{X}-{요구사항}"
       ↓
[0] ★ active_milestone 자동 갱신
   └ 요구사항에서 "M{X}" 추출해 progress/active_milestone.txt를 "M{X}:in-progress"로 전이
     (이미 같은 상태면 유지. 마일스톤 기획 문서 자동 로드)
       ↓
[1] 워크플로우 개요 표시
       ↓
[2] 3자 토론: 요구사항 해석 (자동)
       ↓
[3] 합의 기반 인터뷰 질문 생성
       ↓
[4] 사용자 인터뷰
       ↓
[5] 메인태스크 문서 생성 (마일스톤 문서 참조 링크 포함)
       ↓
→ /step-2 안내
```

## [0] 마일스톤 상태 자동 전이

`/workflow "M2-피드성능"` 같이 호출되면 다음을 수행:

1. **요구사항 문자열에서 `M{숫자}` 패턴 추출** (없으면 현재 active_milestone 유지)

2. **이전 마일스톤 자동 done 처리**:
   - `progress/active_milestone.txt`를 읽어 이전 M 확인
   - 이전이 `M{Y}:in-progress` 이고 새 호출이 다른 `M{X}`면:
     → 시스템이 **"이전 M{Y} 완료된 것으로 처리합니다. 수동 E2E 테스트 통과 + 문서 갱신 + 커밋 완료됐나요? (y/n)"** 확인
     → `y`면 **반드시 아래 명령 실행** 후 계속:
       ```bash
       echo "M{Y}:done" > progress/active_milestone.txt
       ```
       그리고 이 변경을 `docs:` 커밋으로 즉시 추가 (또는 다음 커밋에 포함)
     → `n`이면 중단하고 사용자에게 M{Y} 마무리 안내

3. **새 마일스톤 active 전이**:
   - `progress/mvp{N}/M{X}_*.md` 파일 존재 확인 — 없으면 `/milestone`으로 먼저 기획 요청
   - `echo "M{X}:in-progress" > progress/active_milestone.txt`

4. **active_subtask.txt 자동 갱신 (Option A — self-contained)**:
   - `progress/mvp{N}/M{X}_*.md` glob으로 파일 탐색
   - 파일 발견 시: `echo "progress/mvp{N}/M{X}_{마일스톤명}.md" > progress/active_subtask.txt`
   - 파일 없을 시: `active_subtask.txt` 갱신 생략 (이미 `/milestone`이 기록했거나 없는 경우)
   - 이미 올바른 M{X} 경로가 기록돼 있으면 재기록 생략

5. 이후 step-1~step-9는 이 마일스톤 KPI로 게이트 검증

6. **active_step 설정** (Phase 2 — state file 기반 강제):
   ```bash
   echo "step-1" > progress/active_step.txt
   ```
   이후 각 step SKILL이 진입 시 자기 step으로 갱신할 책임. step-9 SKILL이 커밋 성공 후 `none`으로 cleanup.

### 인터뷰 라이프사이클 (state file 기반 — Phase 2)

[3] 인터뷰 질문 생성 직후 (총 N개 질문 확정 시점):

```bash
cat > progress/active_interview.json <<EOF
{
  "step": "step-1",
  "current": 1,
  "total": {N},
  "remaining": ["{Q1 제목}", "{Q2 제목}", "..."],
  "started_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
```

[4] 매 답변 후: `current` 1 증가, `remaining` 첫 항목 제거.

인터뷰 완료 (마지막 답변 후): `echo '{}' > progress/active_interview.json`

> 위 절차는 Phase 3 PreToolUse hook이 인터뷰 미완료 상태를 감지해 Edit/Write를 차단하는 데 사용된다. 누락 시 hook이 stale로 오판해 잘못된 차단 발생 가능.

### step-9 (커밋) 직후 MVP 완료 자동 체크

step-9 커밋 성공 시 시스템이 자동으로:
1. `progress/mvp{N}/_roadmap.md`에서 전체 마일스톤 목록 추출
2. `active_milestone.txt` 기준으로 **남은 M이 없나** 확인
3. 모든 M이 done 상태로 전이 가능(또는 이미 done)이면:
   ```
   📋 MVP{N} 로드맵의 모든 마일스톤이 끝난 것으로 보입니다.
   MVP 완료 프로세스를 시작할까요?

   - MVP{N}:completing으로 전이
   - MVP 최종 KPI 엄격 검증 (_roadmap.md)
   - history/mvp{N}/{YYMMDD}_mvp{N}_completion_retro.md 회고 작성 안내
   - progress/mvp{N}/ → history/mvp{N}/ 아카이빙
   - active_mvp.txt를 다음 MVP로 초기화

   진행? (y/n)
   ```
4. 사용자 `y` → 위 절차 자동 수행. 실패 시 안내.
5. `n` → 그냥 커밋만 완료

**사용자는 수동 E2E 테스트 완료 + 다음 `/workflow` 호출만으로 M 전이 발생.** 별도 명령어 불필요.

---

---

## 9단계 워크플로우 개요

| 단계 | 명령 | 설명 | 유연성 |
|------|------|------|--------|
| 1 | /step-1 | 메인태스크 설정 (3자 토론 포함) | 필수 |
| 2 | /step-2 | 룰즈 검증 | 필수 |
| 3 | /step-3 | 서브태스크 분리 | 필수 |
| 4 | /step-4 | 서브태스크 인터뷰 | 선택 |
| 5 | /step-5 | 서브태스크 리뷰 | 필수 |
| 6 | /step-6 | 구현 | 필수 |
| 7 | /step-7 | 리팩토링 + 코드 리뷰 (3R+3R) | 유연 |
| 8 | /step-8 | 테스트 | 유연 |
| 9 | /step-9 | 커밋 | 필수 |

## 스텝 진행 강제 규칙 (절대 준수)

- **한 번에 한 스텝만**: 현재 스텝 내용을 모두 표시한 뒤 반드시 멈춘다.
- **스텝 전환 전 사용자 확인 필수**: 다음 스텝으로 넘어가기 전에 항상
  > "다음 단계 /step-N 으로 넘어갈까요?"
  라고 물어보고, 사용자가 명시적으로 허락한 뒤에만 진행한다.
- **스킵 금지**: 설령 다음 스텝이 명백히 간단해 보여도 사용자 확인 없이 건너뛰지 않는다.
- **자동 연속 실행 금지**: "계속", "진행", "다음" 같은 지시를 받아도 한 스텝씩만 처리하고 다시 확인한다.
- **현재 스텝 명시 (필수)**: 매 응답 **첫 줄**에 반드시 아래 형식으로 표시한다.
  ```
  [Step N/9 — /step-N | 인터뷰 M/T 완료]   ← 인터뷰 중일 때
  [Step N/9 — /step-N]                      ← 인터뷰 외
  ```

### 스텝 강제 원칙 (기획 문서 존재 여부 무관)

- **무조건 step 1부터 시작**: M{X}_*.md나 기획 문서가 이미 있어도 step 1부터 순서대로 실행
- **해당 없는 스텝 명시 선언**: 건너뛸 때 "이 스텝은 해당 없어 패스합니다 — [이유]" 명시 후 다음으로
- **스텝 압축 금지**: 사용자 확인 없이 여러 스텝 묶음 처리 금지

### 인터뷰 완료 게이트 (절대 준수)

step-1 또는 step-4 인터뷰 진행 중:
- **인터뷰 진행 상황 표시**: 매 질문마다 `Q{M}/{T}` 카운터 표시 (예: `Q2/4`)
- **모든 질문 완료 전 다음 스텝 전환 절대 금지**: 사용자가 Q1에만 답해도 Q2~QT를 끝까지 진행
- **사용자가 답변 도중 다른 것을 물어보거나 추가 논의를 요청하면**: 해당 논의를 먼저 처리하되, 인터뷰로 반드시 복귀. "Q{M}/{T} 로 돌아가겠습니다" 명시
- **인터뷰 완료 선언**: 마지막 질문 답변 후 "인터뷰 완료 (T/T)" 명시 후에만 다음 단계 진행

### 피드백 후 구현 진입 규칙 (절대 준수)

사용자가 테스트 결과·버그 보고·피드백을 전달했을 때:
1. **즉시 구현 금지** — 수정 방향을 먼저 분석·제시한다
2. **방향 확인 후 진행**: "이 방향으로 수정할까요?" 확인 후 구현 진입
3. **설계 변경이 필요한 경우**: step-1 또는 step-3로 되돌아가 범위를 재정의한다

---

## [2] 자동 토론: 요구사항 해석

### 토론 참여자

| 참여자 | 관점 | 질문 |
|--------|------|------|
| Claude | 사용자 의도 | "사용자가 진짜 원하는 것은 무엇인가?" |
| Codex | 기술적 의미 | "기술적으로 이것은 무엇을 의미하는가?" |
| Serena | 코드베이스 영향 | "기존 코드베이스에 어떤 구조적 영향을 미치는가?" |

### 토론 절차

**1단계: 각자 해석 제시**
**2단계: 상호 반박** — 각 해석의 약점 지적, 대안 제시
**3단계: 합의 도출**

---

## [3] 인터뷰 질문 생성

- **반드시 1문1답**: 한 번에 1개 질문만 (여러 질문 동시 금지)
- **A/B/C 선택지**: 2~4개 옵션, 추천 옵션에 `(추천)` 표시
- **질문 순서**: 큰 덩어리(범위/방향) → 중간(기능/제약) → 세부(구현/예외)
- **카운터 필수**: 각 질문 앞에 `Q{현재}/{총}` 표시 (예: `Q2/4 — 구현 방식`)
- **총 질문 수 사전 고지**: 인터뷰 시작 시 "총 N개 질문" 안내

---

## [5] 산출물: 메인태스크 문서

`progress/{YYMMDD}_{메인태스크명}.md`

## 다음 단계

→ `/step-2` (룰즈 검증)
