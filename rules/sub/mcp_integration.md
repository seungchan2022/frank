# MCP 통합 활용 정책 (v1.0)

이 문서는 프로젝트에서 사용하는 MCP 서버의 활용 정책, 우선순위 체인, 에이전트-MCP 매핑을 정의한다.

---

## 1. MCP 서버 인벤토리

| # | 서버 | 용도 | 우선순위 | 상태 |
|---|------|------|---------|------|
| 1 | **Context7** | 라이브러리 공식 문서 조회 | 외부 문서 1순위 | 설치됨 |
| 2 | **Sequential Thinking** | 복잡한 분석 구조화 | 분석/설계 필수 | 설치됨 |
| 3 | **Serena** | 코드 심볼/참조/구조 분석 | 코드 분석 1순위 | 설치됨 |
| 4 | **Memory** | 과거 컨텍스트/결정 기록 및 참조 | 컨텍스트 유지 | 설치됨 |
| 5 | **Tavily** | 외부 검색/리서치 | 웹 검색 1순위 | 설치됨 |
| 6 | **Playwright** | 브라우저 자동화/E2E 테스트 | E2E 테스트 | 설치됨 |
| 7 | **Codex** | 2차 검증 (Claude 이후) | 검증 전용 | 설치됨 |
| 8 | **Firecrawl** | 웹 스크래핑/크롤링 | 데이터 수집 | 설치됨 |
| 9 | **Chrome DevTools** | 프론트엔드 디버깅/성능 | 프론트엔드 분석 | 설치됨 |
| 10 | **arXiv MCP** | arXiv 논문 검색/다운로드/전문 읽기 | 논문 검색 1순위 | 설치됨 |
| 11 | **Exa** | AI 네이티브 시맨틱 웹 검색 (의미 기반) | 시맨틱 검색 | 설치됨 |
| 12 | **Gamma** | AI 프레젠테이션/문서 생성 (PPT/PDF export) | 프레젠테이션 | 설치됨 |
| — | ~~D2~~ | D2 다이어그램 렌더링 | — | 미설치 |
| — | ~~Magic~~ | 21st.dev UI 컴포넌트 | — | 미설치 (API 키 없음) |
| — | ~~Morph MCP~~ | 코드 검색 서브에이전트 | — | 미설치 (API 키 없음) |
| — | ~~Perplexity~~ | 딥 리서치 | — | 미설치 (API 키 없음) |
| — | ~~Paper Search~~ | 멀티소스 논문 검색 | — | 미설치 |
| — | ~~Mermaid~~ | Mermaid 다이어그램 렌더링 | — | 미설치 |
| — | ~~Supabase~~ | DB 관리 | — | 미설치 (미사용) |

> 프로젝트 상황에 맞게 서버를 추가/제거한다. `.mcp.json`과 `settings.local.json`에 반영 필수.

---

## 2. 우선순위 체인 (Priority Chains)

### 2.1 코드 분석

```
Serena → Grep/Glob → Morph WarpGrep → file read (Read 도구)
```

- **Serena**: 심볼 수준 분석 (함수 시그니처, 참조, 의존성)
- **Grep/Glob**: 패턴 기반 텍스트 검색 (정확한 키워드/패턴)
- **Morph WarpGrep**: 자연어 기반 코드 검색 (흐름 추적, 대규모 탐색)
- **Read**: 직접 파일 읽기

### 2.2 외부 문서 확인

```
Context7 → Tavily → Firecrawl → WebFetch
```

- **Context7**: 라이브러리 공식 문서 (resolve-library-id → query-docs)
- **Tavily**: 일반 기술 검색/리서치
- **Firecrawl**: 특정 사이트 스크래핑 (Context7 미지원 시)
- **WebFetch**: 단일 URL 직접 접근

### 2.3 웹 검색

```
Exa (시맨틱) → Tavily (키워드) → WebSearch
```

- **Exa**: AI 시맨틱 검색 (의미 기반, 자연어 쿼리에 강점)
- **Tavily**: 구조화된 키워드 검색
- **WebSearch**: 일반 웹 검색 (상위 불가 시 대안)

### 2.4 논문/학술 리서치

```
arXiv MCP → Paper Search → Exa (학술 검색)
```

- **arXiv MCP**: arXiv 논문 직접 검색/다운로드/전문 읽기 (1순위)
- **Paper Search**: 멀티소스 통합 검색 (arXiv+PubMed+bioRxiv+Google Scholar+Semantic Scholar)
- **Exa**: 학술 문서 시맨틱 검색 (보조)

### 2.5 딥 리서치 (종합 조사)

```
Perplexity (Deep Research) → Exa + Tavily + Firecrawl 병렬
```

- **Perplexity**: 인용 기반 종합 리서치 (1순위)
- **Exa + Tavily**: 병렬 시맨틱+키워드 검색 (검증/보완)
- **Firecrawl**: 특정 페이지 상세 수집 (필요 시)

### 2.6 프론트엔드 분석

```
Playwright → Chrome DevTools
```

- **Playwright**: E2E 테스트, 브라우저 자동화
- **Chrome DevTools**: 성능 트레이스, Lighthouse, 메모리 분석

### 2.7 시각화

```
D2 (아키텍처/플로우) → Mermaid (타임라인/간트/파이/시퀀스)
```

- **D2**: 선언형 D2 언어로 고품질 다이어그램 렌더링 (SVG/PNG, 자동 레이아웃)
- **Mermaid**: 22종 다이어그램 (플로우, 시퀀스, 간트, 파이, 타임라인, 마인드맵 등)

### 2.8 프레젠테이션/인포그래픽

```
Gamma (디자인 품질 프레젠테이션) + D2/Mermaid (다이어그램 보조)
```

- **Gamma**: 프롬프트 → 디자인 품질 PPT/PDF 자동 생성
- **D2/Mermaid**: 다이어그램 이미지를 프레젠테이션에 보조 삽입

### 2.9 DB 관리

```
Supabase MCP (스키마/마이그레이션/SQL/타입)
```

- **Supabase MCP 필수 사용**: Supabase 관련 모든 작업은 반드시 MCP를 통해 수행한다
- curl/REST API 직접 호출, Dashboard 수동 실행 안내 금지 -- MCP가 유일한 경로
- DDL 변경(DROP/ALTER)은 사전 승인 필요, 프로덕션 데이터 직접 수정 금지

### 2.10 UI 컴포넌트

```
Magic (21st.dev)
```

- **Magic**: UI 컴포넌트 빌더/영감/리파이너

### 2.11 복잡한 분석/설계

```
Sequential Thinking
```

- **Sequential Thinking**: 다단계 추론, 구조화된 사고 과정

### 2.12 과거 컨텍스트

```
Memory
```

- **Memory**: 과거 분석/결정/엔티티 기록 및 조회

### 2.13 2차 검증

```
Codex (항상 Claude 1차 이후)
```

- **Codex**: Claude의 분석/코드를 독립적으로 재검증

---

## 3. 사용 제한 (Limits)

| MCP | 제한 | 기준 |
|-----|------|------|
| Context7 | `query-docs` 최대 **3회/태스크** | API 호출 비용 절감 |
| Memory | 토론(debate) 중에는 **읽기 전용** | 토론 무결성 보장 |
| Codex | **항상 2차 패스**로만 사용 | Claude 1차 분석 이후 검증용 |
| Firecrawl | 크롤링 대상 **사전 승인 필요** | 불필요한 외부 접근 방지 |
| Chrome DevTools | **로컬/스테이징만** 접근 | 프로덕션 접근 금지 |
| Playwright | **로컬/스테이징만** 접근 | 프로덕션 접근 금지 |
| Morph WarpGrep | 자연어 검색 전용, 정확한 패턴은 **Grep/Glob 우선** | 토큰 비용 절감 |
| arXiv MCP | 논문 다운로드 시 **storage-path** 용량 관리 | 디스크 절약 |
| Paper Search | Semantic Scholar API 키 **권장** (레이트 리밋) | 안정성 |
| Exa | 시맨틱 검색 전용, 정확한 키워드는 **Tavily 우선** | 비용 절감 |
| Perplexity | Deep Research는 **고비용**, 단순 검색에 사용 금지 | 비용 절감 |
| Gamma | Pro 플랜 크레딧 소모, **대량 생성 자제** | 크레딧 절감 |
| Mermaid | 제한 없음 (로컬 렌더링) | -- |
| Supabase | DDL 변경(DROP/ALTER)은 사전 승인 필요. 프로덕션 데이터 직접 수정 금지 | 데이터 안전 |

---

## 4. 에이전트-MCP 매핑 테이블

| 에이전트 | 필수 MCP | 선택 MCP |
|---------|---------|---------|
| **debate** | Sequential Thinking, Codex, Serena | Memory, Tavily, Context7 (deep 모드) |
| **deep-analysis** | Serena, Sequential Thinking | Context7, Tavily, Memory, D2, Chrome DevTools, Morph WarpGrep |
| **chrome-devtools** | Chrome DevTools | -- |
| **git** | -- | -- |
| **codex** | Codex | -- |
| **context7** | Context7 | -- |
| **firecrawl** | Firecrawl | -- |
| **frontend** | Playwright | Chrome DevTools |
| **interviewer** | Sequential Thinking | Memory |
| **memory** | Memory | -- |
| **refactorer** | Serena | Sequential Thinking |
| **reviewer** | Serena, Codex | Context7 |
| **sequential_thinking** | Sequential Thinking | -- |
| **serena** | Serena | -- |
| **tavily** | Tavily | -- |
| **tester** | -- | Context7 |
| **researcher** | Perplexity, Exa | arXiv MCP, Paper Search, Tavily, Firecrawl |
| **paper-search** | arXiv MCP, Paper Search | Exa, Perplexity |
| **supabase** | Supabase | -- |

---

## 5. 금지 사항

- MCP 서버를 **우회하여 직접 API 호출** 금지
- **프로덕션 환경**에 접근하는 MCP 호출 금지
- Memory에 **민감 정보**(API 키, 비밀번호, 토큰) 기록 금지
- Context7 **3회 초과** 호출 금지 (태스크당)
- Codex를 **1차 분석 도구**로 사용 금지 (항상 2차)
