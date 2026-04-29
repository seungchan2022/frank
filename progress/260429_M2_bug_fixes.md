# 메인태스크: M2 실사용 버그 수정

> 날짜: 2026-04-29
> 마일스톤: MVP14 M2
> 상태: in-progress
> 브랜치: feature/260429_mvp14_m2_bug_fixes
> 참조: progress/mvp14/M2_bug_fixes.md

## 목적

실사용(260429) 중 발견된 버그 4건(BUG-006~008, BUG-010)을 수정한다.
코드 탐색 결과 BUG-006/007/010은 코드 버그가 아님을 확인. BUG-008 iOS만 실제 수정 필요.

## 코드 탐색 결과 (step-3 분석)

### BUG-006 (에러 캐시 재시도 불가)
- 서버: `summarize.rs`에 요약 결과 캐시 없음. 서버 측 에러 캐싱 없음.
- iOS: `ArticleDetailFeature.loadSummary()`에서 에러 시 `cache.set()` 미호출 → 에러 미캐싱.
  "다시 시도" 버튼 → `feature.loadSummary()` → 캐시 미스 → 새 API 호출 → 정상 동작.
- 웹: `handleSummarize()`에서 에러 시 `summaryCache.set()` 미호출 → 에러 미캐싱. 재시도 가능.
- **결론**: 코드 버그 없음. "재시도 시 계속 실패"는 특정 URL 크롤링/LLM 자체 실패. ST-1에서 문서화.

### BUG-007 (pull-to-refresh 미반영)
- iOS: `FeedFeature.refresh()` → `noCache: true` → `APIArticleAdapter`에서 `Cache-Control: no-cache` 헤더 추가.
- 서버: `is_no_cache()` 감지 → `feed_cache.get()` 스킵 → 검색 API 호출 → 결과 캐시 저장.
- `rebuildTagStates(from: items)` 완전 재구성. 현재 선택 탭 없으면 별도 fetch.
- **결론**: 코드 정상. "목록 불변" 현상은 검색 엔진이 동일 결과 반환하는 것일 가능성. ST-1에서 문서화.

### BUG-008 (탭 전환 깜빡임)
- **iOS 실제 버그 확인**:
  - `FeedView.mainContent`: `feature.articles.isEmpty` 시 `EmptyStateView()` 렌더링.
  - `FeedFeature.selectTag()` 캐시 미스 시: `tagStates[key] = TagState(status: .loading)` (items: []) 삽입 → `selectedTagId = tagId` → `feature.articles = []` → `isLoading=false` → `EmptyStateView` 렌더링!
  - 이후 API 완료 후 items 채워짐 → 깜빡임 발생.
  - **수정 필요**: FeedFeature에 현재 탭 로딩 중 여부 property 노출 → FeedView에서 조건 추가.
- **웹**: `feedStore.isTagLoading` 이미 존재. 피드 페이지에서 `(loading || isTagLoading) && feedItems.length === 0` 조건으로 "Loading feed..." 표시 → 정상 처리됨. 단 실제 동작 확인 필요.

### BUG-010 (태그 전환 자동 변경)
- iOS: `selectTag()` 캐시 미스 시 서버 새 fetch 실행 → 이전과 다른 결과 반환 가능. 의도된 동작.
- **결론**: 코드 버그 아님. 캐시 미스 후 새 검색 결과 차이. ST-1에서 문서화.

---

## 완료 기준

- [ ] ST-1: 캐시 계층 맵 문서화 + BUG-006/007/010 코드 분석 완료 기록 (bugs.md 갱신)
- [ ] ST-2: BUG-008 iOS FeedFeature `isTabChanging` property 추가 + FeedView 조건 수정
- [ ] ST-3: BUG-008 웹 isTagLoading 처리 실제 동작 검증 (필요 시 수정)
- [ ] ST-4: 서버·웹·iOS 전체 테스트 통과 (`cargo test` + `vitest` + `xcodebuild test`)
- [ ] ST-5: bugs.md BUG-006/007/008/010 최종 상태 갱신

---

## 서브태스크 상세

### ST-1: 캐시 계층 맵 + 버그 코드 분석 문서화

| 항목 | 내용 |
|---|---|
| 타입 | research + docs |
| 플랫폼 | 전체 |
| Phase | 1 (선행) |
| 의존 | 없음 |

**작업 내용**
- `progress/analysis/260429_cache_map.md` 생성:
  - 서버: `InMemoryFeedCache` (FeedItem, TTL 5분, key=`{user_id}:{tag_ids}`)
  - iOS: `SummarySessionCache` (SummaryResult, 세션 내 인메모리) + `tagStates` in FeedFeature
  - 웹: `summaryCache` (SummaryResult, Svelte $state) + `tagCache` in feedStore
- BUG-006/007/010 코드 분석 결과 기록 (위 결론 반영)
- bugs.md BUG-006/007/010 상태 갱신 (OPEN → 분석 완료 기록)

**완료 기준**: 분석 문서 생성 + bugs.md 갱신 완료

---

### ST-2: BUG-008 iOS 탭 전환 깜빡임 제거

| 항목 | 내용 |
|---|---|
| 타입 | feature |
| 플랫폼 | iOS (Swift/SwiftUI) |
| Phase | 2-A |
| 의존 | ST-1 |
| 병렬 | ST-3과 병렬 |

**작업 내용**
- `FeedFeature.swift`: `isTabChanging: Bool` computed property 추가
  ```swift
  var isTabChanging: Bool {
      tagStates[currentKey]?.status == .loading
  }
  ```
- `FeedView.swift`: `mainContent`에서 깜빡임 조건 수정
  ```swift
  // Before
  } else if feature.articles.isEmpty {
      EmptyStateView()
  
  // After
  } else if feature.articles.isEmpty && !feature.isTabChanging {
      EmptyStateView()
  // 또는 isTabChanging 중에는 ShimmerListView() 표시
  ```

**완료 기준**
- 탭 전환 시 "기사가 없습니다" 순간 미표시
- `xcodebuild build` 통과

---

### ST-3: BUG-008 웹 탭 전환 깜빡임 검증

| 항목 | 내용 |
|---|---|
| 타입 | verify |
| 플랫폼 | 웹 (Svelte/TS) |
| Phase | 2-B |
| 의존 | ST-1 |
| 병렬 | ST-2와 병렬 |

**작업 내용**
- 웹 피드 페이지 실제 동작 확인:
  - 탭 전환 시 `isTagLoading` 조건이 실제 깜빡임 방지하는지 Playwright 스크린샷으로 확인
  - `feedStore.selectTag()`가 `async` 함수인데 웹에서 `selectTag` 호출 방식 확인
- 깜빡임 발생 시: feedStore selectTag() 또는 피드 페이지 UI 조건 수정
- 깜빡임 미발생 시: 검증 완료 기록

**완료 기준**
- 탭 전환 깜빡임 미발생 확인
- `npm run lint && npm run check` 통과

---

### ST-4: 전체 테스트 통과 확인

| 항목 | 내용 |
|---|---|
| 타입 | verify |
| 플랫폼 | 전체 |
| Phase | 3 |
| 의존 | ST-2, ST-3 |

**작업 내용**
- `cd server && cargo clippy -- -D warnings && cargo test`
- `cd web && npm run lint && npm run check && npm run test`
- `xcodebuild test -workspace Frank.xcworkspace -scheme Frank -destination 'platform=iOS Simulator,name=iPhone 17 Pro'`

**완료 기준**: 세 가지 모두 통과

---

### ST-5: bugs.md 최종 상태 갱신

| 항목 | 내용 |
|---|---|
| 타입 | docs |
| 플랫폼 | — |
| Phase | 4 |
| 의존 | ST-4 |

**작업 내용**
- BUG-006: OPEN → RESOLVED (에러 캐싱 없음, 재시도 코드 정상, 크롤링/LLM 원인)
- BUG-007: OPEN → RESOLVED (pull-to-refresh noCache=true 정상 동작, 검색 API 동일 결과 반환)
- BUG-008: OPEN → RESOLVED (iOS FeedView 조건 수정, 웹 isTagLoading 처리 완료)
- BUG-010: OPEN → RESOLVED (의도된 동작 — 캐시 미스 후 새 검색 결과)

**완료 기준**: bugs.md 4건 모두 갱신 완료

---

## 실행 단계 (의존성 기반)

```
Phase 1 ─ 단독 블로커
  └─ ST-1: 캐시 맵 문서화 + BUG-006/007/010 분석 완료 기록

Phase 2 ─ 병렬
  ┌─ Agent A (iOS)
  │    ST-2: BUG-008 iOS 탭 전환 깜빡임 수정
  └─ Agent B (웹)
       ST-3: BUG-008 웹 깜빡임 검증

Phase 3 ─ 통합
  └─ ST-4: 전체 테스트 통과

Phase 4 ─ 문서화
  └─ ST-5: bugs.md 최종 갱신
```

## 의존성 DAG

```
ST-1 ──┬──> ST-2 ──┬──> ST-4 ──> ST-5
       └──> ST-3 ──┘
```

---

## 리스크

| 리스크 | 영향 | 대응 |
|--------|------|------|
| ST-2 수정 후 ShimmerListView 대신 EmptyStateView가 의도치 않게 사라지는 케이스 발생 | M | 실제 빈 피드(구독 태그 없음) 케이스와 "탭 로딩 중" 케이스 분기 명확히 |
| BUG-007이 실제로 코드 버그인 경우 ST-1 분석 오류 | L | ST-1에서 실제 시뮬레이터 pull-to-refresh 재현 확인 |
| 웹 탭 전환 깜빡임이 isTagLoading 외 다른 조건으로 발생하는 경우 | M | ST-3에서 Playwright 실제 확인 |
