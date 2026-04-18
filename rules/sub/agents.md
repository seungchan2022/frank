# 에이전트 + MCP 정책 (v2.0)

이 문서는 개발 워크플로우에서 사용하는 에이전트 역할, MCP 서버 목록·안전·비용 제한, 우선순위 체인, 토론 프로토콜을 **단일 소스**로 정의한다.
(이전 `mcp_integration.md` 내용 통합 완료, 해당 파일은 삭제됨)

---

## 1. 역할별 에이전트

| 에이전트 | 담당 | 역할 | 호출 시점 |
|----------|------|------|----------|
| interviewer | Claude Code | 요구사항 수집, A/B/C 인터뷰 | step-1, step-4 |
| reviewer | Claude Code + Codex (병렬) | 문서/코드 리뷰, rules/ 준수 검증 | step-2, step-5, step-7 |
| refactorer | Claude Code (1차) → Codex (2차) | 코드 리팩토링, 품질 개선 | step-7 |
| tester | Claude Code | 테스트 실행, 결과 검증, KPI 수집 | step-8 |
| git | Claude Code | Git 커밋 (푸시 금지, KPI 게이트 통과 후) | step-9 |

---

## 2. MCP 서버 목록 (현재 활성)

| MCP | 용도 | 호출 시점 |
|---|---|---|
| **codex** | 2차 리뷰·리팩토링, 기술적 분석 | reviewer/refactorer 2차 |
| **context7** | 라이브러리 공식 문서 (React/Svelte/Axum 등) | 구현 중 API 사용법 확인 |
| **tavily** | 외부 검색·딥 리서치 | 최신 정보·모범 사례 조사 |
| **tuist** | iOS 프로젝트 관리 (generate, clean) | iOS 작업 |
| **supabase** | DB 스키마·마이그레이션·SQL·타입 생성·로그 | DB 작업, M6 profiles 이관 등 |
| **chrome-devtools** | 브라우저 스크린샷·네트워크·성능·콘솔 | 웹(Svelte) 변경 후 시각·성능 검증 |

### 2.1 사용 시나리오별 매핑

| 시나리오 | 주 MCP | 보조 |
|---|---|---|
| DB 스키마 변경 | supabase (`apply_migration`, `get_advisors`) | — |
| 웹 UI 변경 후 검증 | chrome-devtools (`take_screenshot`, `list_network_requests`) | supabase(로그) |
| iOS 변경 | tuist (`generate`) | — |
| 성능 측정 | chrome-devtools (`performance_start_trace`) | — |
| 라이브러리 API 조회 | context7 | tavily(대안 검토) |
| 외부 트렌드 조사 | tavily | context7 |

---

## 3. 안전 제한 (MUST)

| MCP | 제한 | 이유 |
|-----|------|------|
| chrome-devtools | **로컬/스테이징만** 접근 | 프로덕션 접근 금지 |
| supabase | DDL 변경(DROP/ALTER/TRUNCATE) **사전 승인 필요** | 데이터 안전 |
| supabase | 프로덕션 데이터 직접 수정 금지 | 데이터 안전 |
| supabase | access token은 **환경변수**로만 주입 (코드·문서 하드코딩 금지) | 보안 |
| codex | **항상 2차 패스**로만 사용 (Claude 1차 분석 이후) | 검증 전용 |
| memory | 민감 정보(API 키, 비밀번호, 토큰) 기록 금지 | 보안 |

---

## 4. 비용 제한

| MCP | 제한 | 이유 |
|-----|------|------|
| context7 | `query-docs` **3회/태스크** | API 호출 비용 절감 |
| tavily | 연속 호출 시 결과 캐싱 · 반복 호출 금지 | 비용 절감 |

---

## 5. 우선순위 체인 (Fallback)

### 5.1 코드 분석
```
Grep/Glob (심볼·패턴 검색) → Read (직접 읽기)
```

### 5.2 외부 문서
```
context7 (라이브러리 공식 문서) → tavily (기술 검색) → WebFetch (단일 URL)
```

### 5.3 외부 웹 데이터 검색
```
tavily (1순위) → firecrawl(필요 시 설치) → WebSearch/WebFetch (최종 폴백)
```
- 1순위가 정상 응답하면 하위 도구를 호출하지 않는다
- API 오류·타임아웃·빈 결과 시에만 다음 순위로 넘어간다

### 5.4 UI 검증
```
chrome-devtools (1순위) → 수동 시뮬레이터 검증 (iOS 등 비웹)
```

### 5.5 DB 작업
```
supabase MCP (마이그레이션·SQL) → sqlx 직접 (코드 레벨)
```

---

## 6. 에이전트 호출 규칙

### 6.1 일반
- 에이전트는 지정된 호출 시점에서만 호출한다
- MCP 도구는 `.claude/settings.local.json`에 허용된 것만 사용한다
- 에이전트 간 순환 호출 금지

### 6.2 병렬 실행
- reviewer: Claude Code + Codex **병렬** 실행 후 결과 취합
- refactorer: Claude Code 1차 → Codex 2차 **순차**
- 독립 레이어(서버/웹/iOS) 구현은 **병렬 에이전트 선호** (`feedback_parallel_agents.md`)

### 6.3 결과 형식
- 모든 에이전트 결과는 마크다운으로 문서화
- 결과는 해당 서브태스크 문서에 추가

---

## 7. 3자 토론 프로토콜

### 7.1 참여자

| 참여자 | 관점 | 주 도구 |
|--------|------|---------|
| Claude | 사용자 의도, 문서 일치성 | Read, Write, Glob, Grep |
| Codex | 기술적 타당성, 구현 가능성 | mcp__codex__codex |
| Serena | 구조적 영향, 코드베이스 분석 (필요 시) | mcp__serena__* |

### 7.2 보조 도구

| 도구 | 용도 |
|------|------|
| context7 | 라이브러리 공식 문서 참조 |
| tavily | 외부 모범 사례·패턴 조사 (내부 코드 정보 쿼리 금지) |

### 7.3 절차

1. **주제 제시**: 토론 주제와 맥락
2. **가설 수집**: 각 참여자 독립 가설 (보조 도구로 근거 수집 가능)
3. **상호 반박**: 약점 지적, 대안 제시
4. **합의 도출**: 공통 동의 사항, 쟁점별 결론, 채택할 접근법
5. **결론 문서화**: `progress/debate/{YYMMDD}_{단계}_{주제}.md` 저장

### 7.4 토론 규칙
- 토론 중 memory **쓰기 금지** (읽기만 허용)
- 토론 후 결론을 해당 워크플로우 단계에 반영
- 필요 시 토론 후 memory에 결정사항 저장

---

## 8. 단계별 에이전트 연계 (0_CODEX_RULES.md §3과 매핑)

| 5단계 | 연계 에이전트·MCP |
|---|---|
| Inspect | interviewer + sequential_thinking(선택) + memory(읽기) + serena(선택) |
| Specify | tester(테스트 설계) + context7 |
| Implement | context7 + supabase(DB) + tuist(iOS) |
| Verify | tester + chrome-devtools(웹) + supabase `get_advisors`(DB) + **kpi-check.sh** |
| Report | git(커밋) — pre-commit hook이 KPI Hard 게이트 검증 |

---

## 9. 금지 사항

- MCP 서버를 **우회하여 직접 API 호출** 금지
- **프로덕션 환경**에 접근하는 MCP 호출 금지
- MCP 토큰·키를 코드·로그·문서에 하드코딩 금지
