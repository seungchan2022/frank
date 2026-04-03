# 3자 과학적 토론 에이전트

> Claude + Codex + Serena 기반 다중 관점 토론 프로토콜

---

## 기본 정보

| 항목 | 내용 |
|------|------|
| 참여자 | Claude (주장자), Codex (검증자), Serena (코드 분석가) |
| 역할 | 기술적 의사결정을 위한 구조화된 토론 수행 |
| 트리거 | step-1, step-3, step-5, step-7 |

---

## 참여자 구성

### 기본 참여자 (3인)

| 참여자 | 역할 | MCP 도구 |
|--------|------|----------|
| **Claude** | 주장자 (Proposer) | Sequential Thinking (구조화) |
| **Codex** | 검증자 (Verifier) | Codex MCP (2차 검증) |
| **Serena** | 코드 분석가 (Analyst) | Serena MCP (심볼/참조 분석) |

### 확장 참여자 (depth: deep 모드)

| 추가 도구 | 역할 | 활성화 조건 |
|-----------|------|------------|
| **Memory** | 과거 토론/결정 참조 | debate_depth: deep |
| **Perplexity** | 딥 리서치, 추론 기반 검증 | debate_depth: deep |
| **Tavily** | 외부 사례/키워드 검색 | debate_depth: deep |
| **Exa** | 시맨틱 검색, 학술 자료 발견 | debate_depth: deep |
| **Context7** | 라이브러리 공식 문서 확인 | debate_depth: deep |

---

## 설정 (Config)

```yaml
auto_debate: true          # 자동 토론 활성화 여부
debate_depth: normal       # normal | deep
max_rebuttal_rounds: 3     # 최대 반박 라운드 수
consensus_threshold: 2/3   # 합의 기준 (3명 중 2명)
```

---

## 토론 프로세스

### 1단계: 주제 설정 (Topic Definition)

```
입력: 기술적 의사결정이 필요한 주제
```

### 2단계: 구조화 (Sequential Thinking)

Sequential Thinking MCP를 사용하여 토론 구조를 설계한다:
- 주제 분해 (하위 논점 3~5개)
- 각 참여자별 분석 범위 할당
- 평가 기준 정의

### 3단계: 가설 수집 (Hypothesis Collection) — 병렬 실행

| 참여자 | 분석 방법 | 출력 |
|--------|----------|------|
| Claude | 추론 기반 설계 분석 | 가설 + 근거 |
| Codex | 코드 레벨 검증 | 구현 가능성 평가 |
| Serena | 기존 코드베이스 심볼 분석 | 현재 구현 상태 + 영향 범위 |

### 4단계: 반박 라운드 (Rebuttal Rounds) — 최대 3회

```
라운드 N:
  1. 각 참여자가 다른 참여자의 가설에 대해 반론 제시
  2. 반론에 대한 재반박
  3. 수정된 입장 정리

종료 조건:
  - 합의 도달 (2/3 이상 동의)
  - 최대 라운드 소진
```

### 5단계: 합의 도출 (Consensus)

```
합의 문서 구조:
  - 최종 결론 (한 문장)
  - 근거 요약 (3~5줄)
  - 반대 의견 요약 (소수 의견 기록)
  - 리스크 및 제약 조건
  - 실행 계획 (구체적 Action Items)
```

---

## 출력 형식

```
progress/debate/{YYMMDD}_{step}_{topic}.md
```

---

## 금지 사항

| 금지 | 이유 |
|------|------|
| 합의 없이 구현 착수 | 검증되지 않은 설계 리스크 |
| 반박 없이 동의 | 형식적 토론 방지 |
| 외부 데이터 없이 deep 모드 | deep 모드의 의미 상실 |
| 토론 문서 미작성 | 의사결정 추적 불가 |
