# 에이전트 역할 및 MCP 도구 사용 정책 (v1.0)

이 문서는 개발 워크플로우에서 사용하는 에이전트 역할, MCP 도구 사용 정책, 토론 프로토콜을 정의한다.

---

## 1. 역할별 에이전트

| 에이전트 | 담당 | 역할 | 호출 시점 |
|----------|------|------|----------|
| interviewer | Claude Code | 요구사항 수집, A/B/C 인터뷰 | step-1, step-4 |
| reviewer | Claude Code + Codex (병렬) | 문서/코드 리뷰, rules/ 준수 검증 | step-2, step-5, step-7 |
| refactorer | Claude Code (1차) → Codex (2차) | 코드 리팩토링, 품질 개선 | step-7 |
| tester | Claude Code | 테스트 실행, 결과 검증 | step-8 |
| git | Claude Code | Git 커밋 (푸시 금지) | step-9 |

## 2. MCP 도구 에이전트

| 에이전트 | MCP 서버 | 역할 | 호출 시점 |
|----------|---------|------|----------|
| codex | codex mcp-server | 2차 리뷰, 2차 리팩토링, 기술적 분석 | reviewer/refactorer 2차 작업 시 |
| context7 | context7 | 라이브러리 최신 API 문서 검색 | 구현 중 라이브러리 사용법 확인 시 |
| sequential_thinking | sequential-thinking | 복잡한 태스크 논리적 분해 | 메인→서브태스크 분리, 복잡한 문제 해결 시 |
| serena | serena | 코드베이스 구조 분석 | 기존 코드 분석, 아키텍처 영향 분석 시 |
| memory | memory | 이전 결정사항 참조 (토론 중 읽기 전용) | 긴 프로젝트에서 이전 컨텍스트 필요 시 |
| tavily | tavily-mcp | 외부 모범 사례 조사, 최신 정보 조회 | 최신 정보/기술 문서 조사 시 |
| playwright | playwright | 브라우저 E2E 검증, UI 회귀 확인 | UI/클라이언트 변경 시, step-8 검증 시 |
| firecrawl | firecrawl | 웹 문서·외부 데이터 크롤링 | 외부 웹 문서 수집이 필요할 때 |
| d2 | d2 | D2 언어 기반 다이어그램 렌더링 (SVG/PNG) | 아키텍처/플로우 시각화 시 |
| exa | exa | AI 시맨틱 웹 검색 | 시맨틱 검색, 학술 보조 검색 시 |
| perplexity | perplexity | 딥 리서치, 인용 기반 종합 조사 | 리서치/팩트체크 시 |
| supabase | supabase | DB 스키마/마이그레이션/SQL/타입 생성 관리 | DB 작업 시 |

## 3. 에이전트 호출 규칙

### 3.1 일반 규칙
- 에이전트는 지정된 호출 시점에서만 호출한다.
- MCP 도구는 `.claude/settings.local.json`에 허용된 도구만 사용한다.
- 에이전트 간 순환 호출을 금지한다.

### 3.2 외부 검색 우선순위 (Fallback Chain)

외부 웹 데이터 검색이 필요할 때 아래 순서를 따른다.

| 순위 | 도구 | 용도 | 비고 |
|------|------|------|------|
| 1순위 | **tavily** | 웹 검색·딥 리서치 | 실패 시 2순위로 폴백 |
| 2순위 | **firecrawl** | 웹 문서 스크래핑·크롤링·구조화 추출 | tavily 실패 시 사용 |
| 3순위 | **Claude Code Explorer** (WebSearch/WebFetch) | 빌트인 웹 검색·페이지 조회 | firecrawl 실패 시 최종 폴백 |

- 1순위가 정상 응답하면 하위 도구를 호출하지 않는다.
- API 오류·타임아웃·빈 결과 등 실패 시에만 다음 순위로 넘어간다.

### 3.3 병렬 실행
- reviewer: Claude Code + Codex를 병렬로 실행하고 결과를 취합한다.
- refactorer: Claude Code 1차 → Codex 2차 순차 실행한다.

### 3.4 에이전트 결과 형식
- 모든 에이전트 결과는 마크다운 형식으로 문서화한다.
- 결과는 해당 서브태스크 문서에 추가한다.

## 4. 3자 토론 프로토콜

### 4.1 참여자

| 참여자 | 관점 | 주 도구 |
|--------|------|---------|
| Claude | 사용자 의도, 문서 일치성 | Read, Write, Glob, Grep |
| Codex | 기술적 타당성, 구현 가능성 | mcp__codex__codex |
| Serena | 구조적 영향, 코드베이스 분석 | mcp__serena__* |

### 4.2 보조 도구

| 도구 | 용도 |
|------|------|
| context7 | 라이브러리 공식 문서 참조 |
| memory (읽기 전용) | 이전 토론/결정 사항 참조 |
| tavily | 외부 모범 사례/패턴 조사 (내부 코드 정보 쿼리 금지) |
| sequential-thinking | 복잡한 쟁점 분해 |
| playwright | 브라우저 상호작용 기반 E2E/시각 검증 |
| firecrawl | 웹 문서·외부 데이터 크롤링 |

### 4.3 토론 절차

1. **주제 제시**: 토론 주제와 맥락 제시
2. **가설 수집**: 각 참여자가 독립적으로 가설 제시 (보조 도구로 근거 수집 가능)
3. **상호 반박**: 각 가설의 약점 지적, 대안 제시
4. **합의 도출**: 공통 동의 사항, 쟁점별 결론, 채택할 접근법
5. **결론 문서화**: `progress/debate/{YYMMDD}_{단계}_{주제}.md`에 저장

### 4.4 토론 규칙
- 토론 중 memory 쓰기 금지 (읽기만 허용)
- 토론 후 결론을 해당 단계에 반영
- 필요 시 토론 후 memory에 결정사항 저장

## 5. 단계별 에이전트 연계

```
step-1 (인터뷰) → interviewer + sequential_thinking + memory
step-2 (룰즈 검증) → reviewer (Claude + Codex)
step-3 (서브태스크 분리) → sequential_thinking
step-4 (서브태스크 인터뷰) → interviewer + memory
step-5 (서브태스크 리뷰) → reviewer (Claude + Codex) + serena
step-6 (구현) → context7 + serena
step-7 (리팩토링+코드리뷰 3R+3R) → refactorer + reviewer (Claude + Codex) + serena
step-8 (테스트) → tester + playwright(UI 변경 시)
step-9 (커밋) → git
```
