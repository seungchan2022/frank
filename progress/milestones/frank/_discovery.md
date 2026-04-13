# Discovery: Frank MVP9

> 생성일: 260413
> 진입점: MVP8 완료 (웹 203 / iOS 200 테스트 통과) → MVP9 기획

---

## 영감 보드

### 기술 트렌드

| 트렌드 | 적용 가능성 | 설명 |
|--------|-----------|------|
| Groq LPU 추론 | **H** | llama-3.3-70b-versatile 기준 394 TPS. 현재 OpenRouter free tier 대기열 30초+ → Groq로 교체 시 3~5초 예상. 무료 플랜 30 RPM / 1,000 RPD — Frank 사용 패턴에 충분. |
| Tavily time_range 파라미터 | **H** | `time_range: "week"` 파라미터 1줄 추가로 최근 7일 기사만 반환. 현재 tavily.rs에 날짜 필터 없음. `topic: "news"` + `days: 7` 조합도 사용 가능. |
| LLM-powered snippet cleaning | **M** | regex hell 없이 LLM에 noisy snippet → clean summary 변환. 단, 비용·레이턴시 추가. 현재 단계는 regex 강화로 대응. |
| 퀴즈 delayed feedback (Codecademy 패턴) | **H** | 퀴즈 완료 후 오답 인라인 리뷰. Codecademy "post-quiz review" 기능: 틀린 문제만 즉시 노출, 오답 노트 탭 이동 불필요. MVP8에서 오답 노트를 완성했으므로 완료 화면에 인라인 연동이 자연스러운 다음 단계. |
| NotebookLM 플래시카드 패턴 | **M** | 오답을 플래시카드로 전환. 현재 MVP에서 실현하기엔 복잡도 과다 → 다음 MVP 검토. |

### 유사 서비스 UX 패턴

| 서비스 | 핵심 기능 | Frank에 영감 |
|--------|----------|------------|
| Codecademy post-quiz review | 완료 화면에서 오답 즉시 인라인 노출 | Q1 퀴즈 완료 오답 인라인 표시 |
| Quizlet MCQ | 오답 재학습 루프 | 오답 보기 시트(Q2) |
| NotebookLM 퀴즈 | 소스 기반 퀴즈 자동 생성 | 퀴즈 품질 개선 참고 |

### 현재 코드베이스 확장점 (Serena 분석)

| 파일 | 확장 방향 |
|------|---------|
| `server/src/infra/tavily.rs:66` | body에 `time_range: "week"` 1줄 추가 → 피드 다양성 즉시 개선 |
| `server/src/infra/exa.rs:42` `clean_snippet()` | 노이즈 패턴 필터링 규칙 추가 (네비게이션 문구, 아이콘 이름, 시간표 패턴) |
| `server/src/infra/openrouter.rs` | Groq 어댑터로 교체 또는 병렬 추가 (OpenAI compatible API) |
| 클라이언트 퀴즈 완료 화면 (웹+iOS) | 완료 시 보유한 로컬 오답 데이터 → 인라인 렌더링 |
| 클라이언트 기사 상세 퀴즈 버튼 (웹+iOS) | `quiz_completed` 플래그 활용 → 두 버튼 분기 |

---

## 아이디어 풀 (11개)

| # | 아이디어 | 출처 | 흥미도 | 실현성 |
|---|---------|------|--------|--------|
| 1 | Groq llama-3.3-70b로 LLM 교체 (요약·퀴즈 30초→5초) | 인터뷰 S1/S2 + Groq 공식 | ★★★ | ★★★ |
| 2 | Tavily `time_range: "week"` 추가 (피드 다양성) | 인터뷰 A3 + Tavily 문서 | ★★★ | ★★★ |
| 3 | snippet 노이즈 패턴 필터 강화 (네비게이션·아이콘·시간표) | 인터뷰 A2 + AlterLab RAG 정제 패턴 | ★★★ | ★★★ |
| 4 | 퀴즈 완료 화면 오답 인라인 표시 | 인터뷰 Q1 + Codecademy 패턴 | ★★★ | ★★★ |
| 5 | 기사 상세 퀴즈 버튼 재설계 (다시 풀기 / 오답 보기) | 인터뷰 Q2 | ★★☆ | ★★★ |
| 6 | Tavily snippet을 Groq로 clean (LLM snippet 정제) | LLM-powered cleaning 트렌드 | ★★☆ | ★★☆ |
| 7 | 피드 초기 로딩 stale-while-revalidate 강화 | 인터뷰 A1 | ★★☆ | ★★☆ |
| 8 | 오답을 플래시카드로 변환 (NotebookLM 패턴) | NotebookLM 영감 | ★★★ | ★☆☆ |
| 9 | 퀴즈 스트리밍 생성 (streaming API) | Groq streaming 지원 | ★★☆ | ★★☆ |
| 10 | 연속 퀴즈 모드 (오답만 다시 풀기) | Quizlet MCQ 영감 | ★★☆ | ★★☆ |
| 11 | 피드 개인화 가중치 (조회 기사 키워드 부스팅) | MVP6 MVP7 흐름 연장 | ★★☆ | ★★☆ |

---

## 수렴 결과

### Impact/Effort 배치

```
        높은 임팩트
            │
   #1 Groq  │  #3 snippet
   #2 days  │  #4 Q1 오답인라인
  ──────────┼──────────
   #5 Q2버튼 │  #6 LLM snippet
   #7 SWR   │  #8 플래시카드
            │
        낮은 임팩트
← 낮은 노력     높은 노력 →
```

### 이번에 넣을 것 (In) — MVP9 범위

| # | 아이템 | 유형 | 실행 스킬 | 마일스톤 |
|---|--------|------|----------|---------|
| 1 | Groq LLM 어댑터 교체 (요약·퀴즈) | feature | /workflow | M1 |
| 2 | Tavily `time_range: "week"` 파라미터 추가 | feature | /workflow | M1 |
| 3 | snippet 노이즈 패턴 필터 강화 | feature | /workflow | M1 |
| 4 | 퀴즈 완료 화면 오답 인라인 표시 (웹+iOS) | feature | /workflow | M2 |
| 5 | 기사 상세 퀴즈 버튼 재설계 (웹+iOS) | feature | /workflow | M2 |

### 다음에 할 것 (Next)

| # | 아이템 | 메모 |
|---|--------|------|
| 6 | Groq streaming 퀴즈 생성 | M1 Groq 교체 완료 후 streaming 확장 |
| 7 | 피드 SWR 강화 (A1) | MVP6에서 이미 병렬화. 추가 개선 여지는 있으나 우선순위 낮음 |
| 8 | LLM snippet 정제 | regex 강화로 우선 대응, 비용 검토 후 도입 |

### 안 할 것 (Out)

| # | 아이템 | 사유 |
|---|--------|------|
| 8 | 오답 플래시카드 변환 | 구현 복잡도 대비 임팩트 불명확. MVP10+ 검토 |
| 10 | 연속 퀴즈 모드 | 오답 인라인(Q1) + 오답 보기(Q2)로 학습 루프는 충분히 완성 |
| 11 | 피드 개인화 가중치 부스팅 | MVP7에서 기본 개인화 완료. 추가 복잡도 투입 시점 아님 |
