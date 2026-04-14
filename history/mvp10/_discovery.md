# Discovery: Frank MVP10

> 생성일: 260414

## 코드베이스 분석 결과

### 버그 확인

#### 1. 검색 키워드 오염 (server/src/api/feed.rs L89-108)
- `tag_id` 없을 때만 `keyword_suffix` 적용 로직 존재 (태그별 피드는 오염 안 됨)
- **실제 문제**: `get_top_keywords`가 전체 like 기록에서 키워드를 추출하므로, 태그를 바꿔도 이전 태그 기사에서 누른 좋아요 키워드가 전체 피드(tag_id=none) 쿼리에 계속 붙음
- 수정 방향: `keyword_suffix`를 현재 사용자 태그 맥락으로 한정하거나, 태그별 키워드를 분리 저장

#### 2. 요약 에러 메시지 (server/src/domain/error.rs, web)
- 서버 `AppError::Internal`은 "Internal server error" 숨김 처리로 이미 올바름
- 웹 클라이언트에서 에러 JSON `{ "error": "Internal server error" }` 수신 시 어떻게 표시하는지 확인 필요
- 크롤링 실패 / LLM 실패 → `AppError::Internal` → 500 응답 — 웹이 이를 그대로 띄우는 것이 문제

#### 3. iOS 퀴즈 완료 화면 버튼 누락 (ios/Frank/Frank/Sources/Features/Quiz/QuizView.swift L211-268)
- `finishedView`에 닫기 버튼만 있음
- 웹 `article/+page.svelte` L343, L350에는 "↺ 다시 풀기" + "오답 보기" 버튼 존재
- 수정: `finishedView`에 다시 풀기(generateQuiz 재호출) + 오답 보기(WrongAnswersView 표시) 버튼 추가

#### 4. iOS 퀴즈 배지 미표시 (ios/Frank/Frank/Sources/Core/Adapters/APIFavoritesAdapter.swift)
- `FavoriteItem` 모델에 `quizCompleted` 필드 정상 정의 + CodingKey `quiz_completed` 매핑 존재
- `listFavorites()` API 응답 decode 로직 존재
- **의심 포인트**: 서버 `/api/me/favorites` GET 응답에 `quiz_completed` 필드가 실제로 포함되는지 확인 필요
- `favorites.rs` API 핸들러 확인 필요

#### 5. 피드 캐시 없음 (server/src/api/feed.rs)
- 매 GET `/me/feed` 요청마다 외부 검색 API 병렬 호출
- 서버 전체에 캐시 레이어 없음 확인
- 방향: 태그별 검색 결과를 `Arc<Mutex<HashMap>>` + `Instant` 기반 TTL 캐시로 인메모리 저장

## 수렴 결과

### 이번에 넣을 것 (In)

| # | 아이템 | 유형 | 실행 스킬 | 마일스톤 |
|---|--------|------|----------|---------|
| 1 | iOS 퀴즈 완료 화면 — 다시 풀기 + 오답 보기 버튼 | feature | /workflow | M1 |
| 2 | iOS 퀴즈 배지 미표시 수정 | feature | /workflow | M1 |
| 3 | 검색 keyword_weights 오염 수정 (서버) | feature | /workflow | M2 |
| 4 | 요약 에러 메시지 사용자 친화적 개선 (웹) | feature | /workflow | M2 |
| 5 | 피드 서버 TTL 캐시 도입 | feature | /workflow | M3 |

### 다음에 할 것 (Next)

| # | 아이템 | 메모 |
|---|--------|------|
| 1 | Groq 스트리밍 퀴즈 생성 | M1 Groq 교체 완료 후 확장. 우선순위 낮음 |
| 2 | 피드 SWR 강화 | 서버 캐시로 일단 해결. 추가 개선은 추후 |

### 안 할 것 (Out)

| # | 아이템 | 사유 |
|---|--------|------|
| 1 | 신규 기능 추가 | MVP10 컨셉: 버그 수정 + 완성도 전용 |
| 2 | Tavily API 복구 | 한도 초과 상태, Exa fallback으로 동작 중 — 비용 정책 재검토 별도 |
