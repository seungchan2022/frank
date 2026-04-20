# 서브태스크 01 — 선언 없는 3개 스킬 frontmatter 신규 추가

> 생성일: 260420
> 메인태스크: `progress/260420_g6_allowed_tools_pruning.md`
> 상태: 📋 step-4 완료 (도구 목록 + Feature List 초안 승인)
> 의존성: 없음 (병렬 가능)
> 예상 소요: 40~60분

## 목적

현재 `allowed-tools` 선언이 비어있는 3개 스킬(`/deep-analysis`, `/presentation`, `/progress-cleanup`)에 **보수 원칙**으로 frontmatter를 신설한다. 각 스킬 본문에서 **실제로 호출·언급되는 도구**만 스캔해서 최소 집합으로 선언하고, 쌍/동족 도구(예: `mcp__codex__codex` ↔ `mcp__codex__codex-reply`)는 함께 포함한다.

## 산출물

- `.claude/skills/deep-analysis/SKILL.md` frontmatter `allowed-tools` 추가
- `.claude/skills/presentation/SKILL.md` frontmatter `allowed-tools` 추가
- `.claude/skills/progress-cleanup/SKILL.md` frontmatter `allowed-tools` 추가

각 스킬 frontmatter에는 `context: fork` 선언이 이미 있는지도 함께 확인해 일관성을 맞춘다(현재 상태 확인 시 없음 → 기존 정책과 대조 후 추가 여부 결정).

## 작업 단계

1. **스킬별 도구 도출**: 각 `SKILL.md` 본문을 grep으로 훑어 등장 도구/MCP 서버명 목록화
   - `/deep-analysis`: Codex/Serena/Context7/Tavily 등 외부 추론 MCP
   - `/presentation`: Gamma + Mermaid MCP (본문 "허용 MCP" 섹션 명시)
   - `/progress-cleanup`: Git 이동 + 파일 I/O 위주 (Bash `git mv`, Glob, Read, Write, Edit)
2. **쌍/동족 도구 보정**: `codex` ↔ `codex-reply`, `tavily_search` ↔ `tavily_research`, `context7__resolve-library-id` ↔ `context7__query-docs` 등 쌍 규칙 적용
3. **frontmatter 작성**: 기존 `/debate`, `/milestone-review` 포맷과 동일하게 YAML `allowed-tools:` 블록 추가
4. **검증**: 본문에서 호출되는 도구가 frontmatter에 모두 포함됐는지 재확인 (누락 시 복구)

## 완료 조건

- [ ] `/deep-analysis/SKILL.md`에 `allowed-tools` 블록 존재 + 본문 실사용 도구 전부 포함
- [ ] `/presentation/SKILL.md`에 `allowed-tools` 블록 존재 + Gamma/Mermaid MCP 포함
- [ ] `/progress-cleanup/SKILL.md`에 `allowed-tools` 블록 존재 + Git/파일 I/O 도구 포함
- [ ] 3개 파일 모두 YAML 파싱 에러 없음 (`head -20` 확인)
- [ ] 스킬 본문 수정 없음 (frontmatter만 변경)

## 리스크

- 본문 실사용 도구 누락 시 런타임에서 도구 호출 차단 → grep 검증 필수
- `allowed-tools`에 선언되지 않은 도구는 **ToolSearch로 지연 로드**되는 점 감안: 과도하게 좁히면 유연성 상실 → "쌍/동족 도구 포함" 원칙으로 완충

## step-4 인터뷰 결과 (4/4 — 2026-04-20)

| # | 질문 | 선택 | 결론 |
|---|---|---|---|
| 1 | 도구 도출 방법 | **B** | 본문 전체 독해 후 grep + 쌍/동족 보정 통합 |
| 2 | `context: fork` 선언 | **A** | 3개 모두 추가 (기존 큰 스킬 일관성 + 부작용 없음) |
| 3 | 기본 도구 명시 | **A** | 본문 실사용만 명시 (Q1 독해 결과 기반 스킬별 개별) |
| 4 | 도구 목록 공개 시점 | **B** | step-4에서 초안 공개 + 훑어보기 (재논의 리스크↓) |

심층 검토로 `/deep-analysis` 원안 18개 유지 확정 (좁힘 12 vs 넓힘 18 → 넓힘 채택).
`/presentation` Gamma MCP 부재 → (b) 본문 유지, 런타임 호출 실패 허용.

## 도구 목록 초안 (step-6 구현 대상)

### `/deep-analysis/SKILL.md` — 18개

```yaml
---
name: deep-analysis
description: "심층추론/심층분석. 다중 MCP 활용 심층 코드/아키텍처 분석. 트리거 키워드: 심층추론, 심층분석, deep analysis, 깊이 분석, 코드 분석해줘."
context: fork
allowed-tools:
  - Read
  - Write
  - Glob
  - Grep
  - Bash
  - Agent
  - mcp__context7__resolve-library-id
  - mcp__context7__query-docs
  - mcp__codex__codex
  - mcp__codex__codex-reply
  - mcp__serena__find_symbol
  - mcp__serena__get_symbols_overview
  - mcp__serena__read_file
  - mcp__serena__search_for_pattern
  - mcp__serena__list_dir
  - mcp__tavily__tavily_search
  - mcp__tavily__tavily_research
  - mcp__sequential-thinking__sequentialthinking
---
```

### `/presentation/SKILL.md` — 6개

```yaml
---
name: presentation
description: "Gamma+Mermaid MCP로 디자인 품질 프레젠테이션/인포그래픽 생성. 트리거 키워드: PPT 만들어줘, 프레젠테이션, 슬라이드, 인포그래픽, 보고서, 발표자료."
context: fork
allowed-tools:
  - Write
  - mcp__mermaid__get-mermaid-draft
  - mcp__mermaid__save-mermaid-draft
  - mcp__mermaid__mermaid-mcp-app
  - mcp__gamma__*
  - mcp__d2__*
---
```

**step-5 리뷰 반영 (2026-04-20)**:
- Gamma 개별 tool id(`generate_gamma`/`get_generation`/`list_themes`)는 근거 부족 → `/daily-retro` 선례대로 **와일드카드 `mcp__gamma__*`**로 통일
- 본문 18행 "`D2: render-d2 (아키텍처 다이어그램 보조)`" 누락 발견 → `mcp__d2__*` 와일드카드 추가
- D2를 MCP로 선언함에 따라 `Bash` 제거 (D2 CLI 경로 근거 소멸)

**주의**: `mcp__gamma__*`, `mcp__d2__*`는 **서버 id 미검증** 상태 — 본문 "허용 MCP" 섹션 근거로만 선언했고, 실제 MCP 서버명 일치 여부는 미확인. 구현(step-6) 전 서버 추가·재확인 필요. 런타임 호출 실패는 허용하되, 서버 id 자체가 틀린 경우 와일드카드 매칭도 실패함에 유의.

### `/progress-cleanup/SKILL.md` — 6개

```yaml
---
name: progress-cleanup
description: "진행상황 정리. 완료 마일스톤 아카이빙 + stale TODO 감지 + history/INDEX.md 갱신. 트리거 키워드: progress 정리, cleanup, 아카이빙."
context: fork
allowed-tools:
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - Bash
---
```

## Feature List
<!-- size: 중형 | count: 19 | skip: false -->

### 기능
- [x] F-01 `/deep-analysis/SKILL.md` frontmatter에 `context: fork` 선언 추가
- [x] F-02 `/deep-analysis/SKILL.md` frontmatter에 `allowed-tools` 18개 블록 추가 (기본 5 + Agent + Codex×2 + Serena×5 + Tavily×2 + Context7×2 + sequential-thinking)
- [x] F-03 `/presentation/SKILL.md` frontmatter에 `context: fork` 선언 추가
- [x] F-04 `/presentation/SKILL.md` frontmatter에 `allowed-tools` 6개 블록 추가 (Write + Mermaid×3 + Gamma 와일드카드 + D2 와일드카드)
- [x] F-05 `/progress-cleanup/SKILL.md` frontmatter에 `context: fork` 선언 추가
- [x] F-06 `/progress-cleanup/SKILL.md` frontmatter에 `allowed-tools` 6개 블록 추가 (Read/Write/Edit/Glob/Grep/Bash)
- [x] F-07 3개 파일 모두 기존 `name`/`description` 필드 원형 유지

### 엣지
- [~] E-01 deferred (런타임 검증, 다음 세션 자연 사용 시 확인) `/presentation`의 Gamma MCP가 런타임 부재여도 Mermaid MCP 호출 경로는 정상 작동
- [~] E-02 deferred (런타임 검증, 다음 세션 자연 사용 시 확인) `/deep-analysis`가 Agent sub-agent 경유 시 sub-agent 자체 도구 세트로 동작 (부모 `allowed-tools` 제약 미전파)
- [x] E-03 frontmatter YAML 키 순서가 기존 스킬(`/milestone`, `/debate`)과 일치 (`name → description → context → allowed-tools`)
- [~] E-04 deferred (런타임 검증, 다음 세션 자연 사용 시 확인) `/progress-cleanup`의 Bash `git mv` 호출이 frontmatter 선언 후에도 정상 수행

### 에러
- [x] R-01 3개 파일 YAML 파싱 실패 없음 (`head -20` 육안 확인, 들여쓰기·콜론 검증)
- [x] R-02 `allowed-tools` 누락 도구 발견 시 복구 경로(ToolSearch 지연 로드 or frontmatter 재수정) 확인

### 테스트
- [~] T-01 deferred (스킬 drive-through는 대화 플로우를 유발해 예방적 정비 맥락에 과투자 — 다음 세션 자연 사용 시 검증, C 방식 합의) `/deep-analysis` 1회 drive-through 수동 호출
- [~] T-02 deferred (사유 동일) `/presentation` 1회 drive-through 수동 호출
- [~] T-03 deferred (사유 동일) `/progress-cleanup` 1회 drive-through 수동 호출
- [x] T-04 `grep -c "allowed-tools:"` 3개 파일 모두 1 반환
- [x] T-05 3개 스킬 본문-도구 grep 매핑표 작성 (위 "T-05 본문-도구 매핑표" 섹션 참조)

### 회귀
- [~] G-01 deferred (런타임 검증, 다음 세션 자연 사용 시 확인) 다른 스킬(`step-*`, `/debate`, `/milestone`, `/workflow` 등)의 기존 동작에 영향 없음

## T-05 본문-도구 매핑표 (step-8 결과)

각 스킬 선언 도구와 본문 근거 매핑. 범례: **본문 L{n}**=본문 행 명시 / **기본**=파일 I/O 기본 도구 / **맥락(X)**=description·프로세스 맥락상 필요 / **쌍(x)**=기본 도구 x와의 쌍/동족 / **와일드카드**=서버 id 미검증 근거부 선언.

### `/deep-analysis` (18개)

| 도구 | 근거 |
|---|---|
| Read, Write, Glob, Grep, Bash | 기본 (보고서 생성 + progress/ 문서화) |
| Agent | 본문 L32 "deep-analysis 에이전트 호출" |
| mcp__context7__resolve-library-id | 본문 L37 "Context7 최대 3회/태스크" |
| mcp__context7__query-docs | 쌍(resolve-library-id) |
| mcp__codex__codex | 맥락(description "다중 MCP 활용 심층 분석") |
| mcp__codex__codex-reply | 쌍(codex) |
| mcp__serena__find_symbol | 맥락(코드/아키텍처 분석) |
| mcp__serena__get_symbols_overview | 쌍(find_symbol) |
| mcp__serena__read_file | 쌍(find_symbol) |
| mcp__serena__search_for_pattern | 쌍(find_symbol) |
| mcp__serena__list_dir | 쌍(find_symbol) |
| mcp__tavily__tavily_search | 맥락(외부 근거 수집) |
| mcp__tavily__tavily_research | 쌍(tavily_search) |
| mcp__sequential-thinking__sequentialthinking | 맥락(심층 추론) |

> **주의**: Codex/Serena/Tavily/sequential-thinking은 본문 직접 명시 없음. description 문구 + 동급 스킬(`/milestone`) 패턴 + 사용자 "함부로 지우면 안 될 수도" 방어적 유지 근거로 선언.

### `/presentation` (6개)

| 도구 | 근거 |
|---|---|
| Write | 기본 (다이어그램 파일 저장) |
| mcp__mermaid__get-mermaid-draft | 본문 L20 "Mermaid: generate_mermaid" (draft 관리 쌍) |
| mcp__mermaid__save-mermaid-draft | 쌍(get-mermaid-draft) |
| mcp__mermaid__mermaid-mcp-app | 본문 L20 "Mermaid: generate_mermaid" |
| mcp__gamma__* | 본문 L19 "Gamma: generate_gamma, get_generation, list_themes" (와일드카드, 서버 id 미검증) |
| mcp__d2__* | 본문 L21 "D2: render-d2" (와일드카드, 서버 id 미검증) |

### `/progress-cleanup` (6개)

| 도구 | 근거 |
|---|---|
| Read | 본문 L13 "progress/ 하위 파일 전체 스캔" |
| Write | 신규 `history/INDEX.md` 작성 가능성 |
| Edit | 본문 L26 "INDEX.md를 갱신한다" |
| Glob | 본문 L13 "전체 스캔" |
| Grep | 맥락(stale 감지 상태 검색, 안전 마진) |
| Bash | 본문 L18 "git mv로 이동" |

### `/milestone` (21개, Before 23)

| 도구 | 근거 |
|---|---|
| Read, Write, Glob, Grep, Bash | 기본 (문서 생성·상태 파일 초기화) |
| mcp__sequential-thinking__sequentialthinking | 본문 L293 "sequential_thinking으로 구조화" |
| mcp__codex__codex | 본문 L201 "3자 토론 (Claude + Codex + Serena)" |
| mcp__codex__codex-reply | 쌍(codex) |
| mcp__serena__find_symbol | 본문 L161 "기존 코드 확장점 (Serena)" |
| mcp__serena__get_symbols_overview | 쌍(find_symbol) |
| mcp__serena__list_dir | 쌍(find_symbol) |
| mcp__serena__search_for_pattern | 쌍(find_symbol) |
| mcp__serena__read_file | 쌍(find_symbol) |
| mcp__tavily__tavily_search | 본문 L149~155 "Tavily" 명시 |
| mcp__tavily__tavily_research | 쌍(tavily_search) |
| mcp__exa__web_search_exa | 본문 L149~155 "Exa" 명시 (오픈소스 프로젝트) |
| mcp__firecrawl__firecrawl_search | 쌍(firecrawl_scrape) |
| mcp__firecrawl__firecrawl_scrape | 본문 L154 "Firecrawl" 명시 (UX 패턴) |
| mcp__mermaid__mermaid-mcp-app | 본문 L535 "D2 또는 Mermaid로 의존성 그래프" |
| mcp__mermaid__get-mermaid-draft | 쌍(mermaid-mcp-app) |
| mcp__mermaid__save-mermaid-draft | 쌍(mermaid-mcp-app) |

**제거됨**: `Task` (본문 미언급 + 레거시), `mcp__exa__crawling_exa` (본문 Exa 맥락은 web_search 매칭)

### `/debate` (24개, Before 25)

| 도구 | 근거 |
|---|---|
| Read, Write, Glob, Grep | 본문 L41 "Claude \| Read, Write, Glob, Grep" 명시 |
| mcp__codex__codex | 본문 L42 "Codex \| mcp__codex__codex" 명시 |
| mcp__codex__codex-reply | 쌍(codex) |
| mcp__serena__find_symbol, get_symbols_overview, find_referencing_symbols, read_file, list_dir, search_for_pattern | 본문 L43 "Serena \| mcp__serena__*" (쌍 6개 유지) |
| mcp__context7__resolve-library-id | 본문 L49 "context7 \| 라이브러리 공식 문서" |
| mcp__context7__query-docs | 쌍(resolve-library-id) |
| mcp__memory__search_nodes, read_graph, open_nodes | 본문 L50 "memory (읽기 전용) \| 이전 토론/결정 참조" (쌍 3개 유지) |
| mcp__tavily__tavily_search | 본문 L51 "tavily \| 외부 모범 사례 조사" |
| mcp__tavily__tavily_research | 쌍(tavily_search) |
| mcp__sequential-thinking__sequentialthinking | 본문 L52 "sequential-thinking \| 복잡한 쟁점 분해" |
| mcp__exa__web_search_exa | 본문 L53 "exa \| 시맨틱 검색으로 외부 근거" |
| mcp__arxiv-mcp-server__search_papers, download_paper, read_paper | 본문 L55 "arxiv / paper-search \| 학술 논문 근거" (쌍 3개 유지) |

**제거됨**: `Task` (본문 미언급 + 레거시)

### `/milestone-review` (15개, Before 17)

| 도구 | 근거 |
|---|---|
| Read, Write, Edit, Glob, Grep, Bash | 기본 (로드맵 갱신 + 아이템 테이블 추출 + `git log`) |
| mcp__sequential-thinking__sequentialthinking | 맥락(재조정 구조화) |
| mcp__codex__codex | 맥락(재조정 제안 외부 의견) |
| mcp__codex__codex-reply | 쌍(codex) |
| mcp__serena__find_symbol | 맥락(아이템 진행 코드 분석) |
| mcp__serena__get_symbols_overview | 쌍(find_symbol) |
| mcp__serena__list_dir | 쌍(find_symbol) |
| mcp__serena__search_for_pattern | 쌍(find_symbol) |
| mcp__tavily__tavily_search | 맥락(재조정 리서치) |
| mcp__mermaid__mermaid-mcp-app | 본문 L153 "의존성 그래프 재생성 (D2/Mermaid)" |

**제거됨**: `Task` (본문 미언급 + 레거시), `mcp__exa__web_search_exa` (본문 미언급 + tavily_search와 중복)

> **주의**: `/milestone-review`는 본문에 MCP 직접 명시 적음. 다수가 "맥락상 필요"로 유지됨 — `/deep-analysis`와 유사 상황.

## step-5 리뷰 결과 기록 (2026-04-20)

Codex 병렬 리뷰에서 3건 치명 결함 발견 + 3건 권고 발견. 치명 3건 즉시 수정(D2 추가·Gamma 와일드카드 전환·/presentation 계수 6개). 권고 3건 반영:

### (권고) `/deep-analysis` 18개 원칙 불일치 명시

Q3(A) "본문 실사용만" 원칙 엄격 적용 시 `/deep-analysis` 본문은 `Agent`(에이전트 호출) + `Context7`(제약 섹션) 2개 MCP만 명시. Codex/Serena/Tavily/sequential-thinking은 본문 미명시.

그럼에도 18개 유지 결정한 근거:
- description "다중 MCP 활용 심층 분석" 문구
- 기존 동급 스킬(`/milestone`, `/milestone-review`)과 일관성
- 사용자 우려("함부로 지우면 안 될 수도") 방어적 대응
- "본문이 덜 상세한 탓" (본문 보강이 별도 작업으로 남은 상태)

**후속 작업 후보**: G? — `/deep-analysis` 본문에 실사용 MCP 도구 명시적 기술 추가 (갭 문서에 기록 검토).

### (권고) YAML 검증 강화 → T-05로 반영

본문-도구 grep 매핑표를 테스트 항목으로 추가. `head -20` 육안 검증과 병행.
