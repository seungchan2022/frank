# 서브태스크 02 — 과다 선언 3개 스킬 `allowed-tools` pruning

> 생성일: 260420
> 메인태스크: `progress/260420_g6_allowed_tools_pruning.md`
> 상태: 📋 step-4 완료 (diff + Feature List 초안 승인)
> 의존성: 없음 (서브태스크 01과 병렬 가능)
> 예상 소요: 40~60분

## 목적

`allowed-tools`가 과도하게 선언된 3개 스킬(`/milestone`, `/debate`, `/milestone-review`)을 **보수 원칙**(본문 실사용 + 쌍/동족 + 맥락상 필요)으로 좁힌다. 공격적 pruning이 아니라 **본문에서 한 번도 언급되지 않는 도구**만 제거 대상 후보로 올리고, 쌍/동족 도구는 남긴다.

## 산출물

- `.claude/skills/milestone/SKILL.md` frontmatter `allowed-tools` 축소
- `.claude/skills/debate/SKILL.md` frontmatter `allowed-tools` 축소
- `.claude/skills/milestone-review/SKILL.md` frontmatter `allowed-tools` 축소

## 현재 상태 (step-5 실측 확정)

| 스킬 | 현재 도구 수 | MCP 서버 수 | 추정 실사용 대비 |
|---|---|---|---|
| `/milestone` | 23 | 7 (sequential-thinking, codex, serena, tavily, exa, firecrawl, mermaid) | 과다 |
| `/debate` | 25 | 8 (codex, serena, context7, memory, tavily, sequential-thinking, exa, arxiv) | 과다 |
| `/milestone-review` | 17 | 6 (sequential-thinking, codex, serena, tavily, exa, mermaid) | 과다 (exa 중복) |

> 1차 초안의 step-1 추정값(15/14/14)은 오류였음. step-5 리뷰 단계에서 `awk`로 항목별 직접 나열해 재검증 완료.

## 작업 단계

1. **본문 실사용 도구 추출**: 스킬별 `SKILL.md` 본문을 grep으로 훑어 명시 언급된 도구 목록화
2. **쌍/동족 도구 유지 판정**: `codex` ↔ `codex-reply`, `tavily_search` ↔ `tavily_research`, `serena__find_symbol` ↔ `serena__find_referencing_symbols` 쌍은 같이 유지
3. **맥락상 필요 판정**: 본문엔 없지만 "문서 갱신엔 Edit 필요" 같은 암묵적 도구는 유지
4. **pruning 후보 작성**: 제거 대상 도구 리스트 + 제거 사유를 메인 문서에 기록 (before/after 표)
5. **frontmatter 수정**: YAML 리스트에서 해당 항목 삭제
6. **런타임 drive-through**: `/debate` 1회 + `/milestone` 1회 트리거 테스트 (에러 없음 확인)

## 완료 조건

- [ ] 3개 파일 `allowed-tools` 항목 수가 현재보다 감소 (정확한 수치는 step-4에서 확정)
- [ ] 본문 명시 도구 중 제거된 것 없음 (grep 재확인)
- [ ] 쌍/동족 도구 쌍 유지 (`codex` ↔ `codex-reply` 같이 있거나 같이 없음)
- [ ] `/debate`, `/milestone` 드라이런 에러 없음
- [ ] before/after 비교 표 작성(서브태스크 04에서 활용)

## 리스크

- **공격적 pruning 금지**: 사용자 Q2 답 "A=보수"에 맞춰 의심스러우면 남김
- **drive-through 누락 위험**: 본문만으로 보이지 않는 암묵적 도구 호출 가능 → 대표 2개 스킬 드라이런 필수
- `/milestone-review`의 `exa_web_search_exa`와 `tavily_search` 중복 → 본문에서 어떤 쪽을 쓰는지 확인 후 하나만 유지 판단

## step-4 인터뷰 결과 (1/1 — 2026-04-20)

| # | 질문 | 선택 | 결론 |
|---|---|---|---|
| 1 | 02 진행 방식 | **A** | 01 원칙 재적용 + 재인터뷰 없이 본문 독해 → diff 제시 |

01에서 합의한 원칙을 그대로 적용:
- Q1(B) 본문 독해 / Q2(A) 보수 pruning / Q3(A) 본문 실사용 / Q4(B) step-4 공개

**공격성 옵션** (A)/(B)/(C) 중 **(A) 기본안 — 본문 미언급 + 레거시만 제거** 확정.

## 제거 대상 확정 (5개)

| 스킬 | 제거 도구 | 사유 |
|---|---|---|
| `/milestone` | `Task` | 레거시 단일 도구 (현재 세션 실재 안 함 — `Task*` 시리즈로 분화) |
| `/milestone` | `mcp__exa__crawling_exa` | 본문 "Exa" 언급은 `web_search_exa` 맥락, crawling은 완전 미언급 |
| `/debate` | `Task` | 레거시 |
| `/milestone-review` | `Task` | 레거시 |
| `/milestone-review` | `mcp__exa__web_search_exa` | 본문에 Exa 언급 없음, tavily_search와 기능 중복 |

**보존 대상**:
- `/milestone`의 `firecrawl_search` (보수 원칙 — scrape와 쌍)
- `/debate`의 arxiv×3 (본문 "학술 논문 근거" 명시)
- `/debate`의 memory×3, context7×2 (본문 "보조 도구" 명시)
- 모든 스킬의 `context: fork` 선언

## Before/After 요약

| 스킬 | Before | After | 삭감 |
|---|---|---|---|
| `/milestone` | 23 | 21 | -2 |
| `/debate` | 25 | 24 | -1 |
| `/milestone-review` | 17 | 15 | -2 |
| **합계** | **65** | **60** | **-5** |

> step-5 Codex 리뷰에서 `/debate` 계수 오류(24 → 25) 발견해 수정. 실측 합계 65→60.

## Feature List
<!-- size: 중형 | count: 18 | skip: false -->

### 기능
- [x] F-01 `/milestone/SKILL.md`의 `allowed-tools`에서 `Task` 항목 제거
- [x] F-02 `/milestone/SKILL.md`의 `allowed-tools`에서 `mcp__exa__crawling_exa` 항목 제거
- [x] F-03 `/debate/SKILL.md`의 `allowed-tools`에서 `Task` 항목 제거
- [x] F-04 `/milestone-review/SKILL.md`의 `allowed-tools`에서 `Task` 항목 제거
- [x] F-05 `/milestone-review/SKILL.md`의 `allowed-tools`에서 `mcp__exa__web_search_exa` 항목 제거
- [x] F-06 3개 파일의 `context: fork` + 나머지 도구 선언 원형 유지 (제거 외 변경 없음)

### 엣지
- [x] E-01 `/milestone` 본문 Exa 호출 맥락이 `web_search_exa` 한정 (crawling 미사용) 재확인 — 01 문서 T-05 매핑표 /milestone 섹션 참조
- [~] E-02 deferred (런타임 검증, 다음 세션 자연 사용 시 확인) `/milestone-review` 재조정 리서치가 tavily_search 단독으로 충분 (exa 누락 영향 없음)
- [x] E-03 3개 파일 YAML 키 순서 유지 (`name → description → context → allowed-tools`)
- [~] E-04 deferred (런타임 검증, 다음 세션 자연 사용 시 확인) 본 태스크 대상 3개 스킬에서 `Task` 제거 시 스킬 본문/호출 흐름에 영향 없음 확인

### 에러
- [x] R-01 3개 파일 YAML 파싱 실패 없음 (`head -30` 육안 확인, 들여쓰기·콜론 검증)
- [x] R-02 런타임에 누락 도구 발견 시 복구 경로(ToolSearch 지연 로드 or frontmatter 재추가) 확인

### 테스트
- [~] T-01 deferred (스킬 drive-through는 대화 플로우를 유발해 예방적 정비 맥락에 과투자 — 다음 세션 자연 사용 시 검증, C 방식 합의) `/milestone` 1회 drive-through
- [~] T-02 deferred (사유 동일) `/debate` 1회 drive-through
- [~] T-03 deferred (사유 동일) `/milestone-review` 1회 drive-through
- [x] T-04 3개 파일 `allowed-tools` 항목 수 각각 정확히 21/24/15 (`/milestone`/`/debate`/`/milestone-review`)

### 회귀
- [~] G-01 deferred (런타임 검증, 다음 세션 자연 사용 시 확인) 서브태스크 01의 3개 스킬(`/deep-analysis`, `/presentation`, `/progress-cleanup`) 동작 영향 없음
- [~] G-02 deferred (런타임 검증, 다음 세션 자연 사용 시 확인) `/step-*`, `/workflow`, `/init` 등 기타 스킬 기존 동작 영향 없음

## step-5 리뷰 결과 기록 (2026-04-20)

Codex 병렬 리뷰 결과 반영:

### (치명) `/debate` 계수 오류 — 수정 완료
- Before/After 표: `/debate` 24 → **25**
- 합계: 64→59 → **65→60**
- "현재 상태" 표: 초기 추정 15/14/14 → 실측 23/25/17로 정정
- T-04 검증 수치: 23 → **24**로 정정

### (권고) E-04 문구 수정 — 완료
- Before: "다른 스킬에 `Task` 없음 확인"
- After: "3개 스킬 본문/호출 흐름 영향 없음 확인" + "다른 스킬 `Task` 잔존은 범위 밖" 명시

### (권고) `Task` 제거 근거 보강
다른 스킬에 `Task` 잔존 스킬 목록 (grep 확인): `workflow`, `step-1`, `step-3`, `step-5`, `step-7`, `next`, `critical-review`, `milestone`, `milestone-review`, `debate`. 이 중 본 태스크는 뒤 3개(`milestone`/`debate`/`milestone-review`)에서만 제거. 나머지 7개는 **후속 태스크 후보**.

## 다음 태스크

→ 서브태스크 03(agents.md 원칙 추가)은 본 태스크 완료 후 실행
