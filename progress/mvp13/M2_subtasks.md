# MVP13 M2 — 서브태스크 분해

> 생성일: 2026-04-28
> 브랜치: feature/260428_mvp13-m2-client-feed-sync

---

## 서브태스크 목록

### T-01 웹 — WrongAnswer 타입 + 필터 전환

**목적**: favorites 브릿지(tagMap) 완전 제거 → WrongAnswer.tagId 직접 사용

**대상 파일**:
- `web/src/lib/types/quiz.ts`
- `web/src/lib/utils/favorites-filter.ts`
- `web/src/lib/components/QuizModal.svelte`
- `web/src/routes/favorites/+page.svelte`

**작업 상세**:
1. `WrongAnswer` 인터페이스에 `tagId: string | null` 필드 추가
2. `SaveWrongAnswerBody`에 `tag_id: string | null` 필드 추가
3. `favorites-filter.ts`:
   - `buildWrongAnswerTagMap()` 함수 **삭제** (favorites 브릿지 전용)
   - `filterWrongAnswers(wrongAnswers, tagMap, selectedTagId)` → `filterWrongAnswers(wrongAnswers, selectedTagId)` 시그니처 변경 (`wa.tagId === selectedTagId` 로 교체)
   - `buildWrongAnswerFilterTags(allTags, wrongAnswers, tagMap)` → `buildWrongAnswerFilterTags(allTags, wrongAnswers)` 시그니처 변경 (`wrongAnswers.map(wa => wa.tagId)` 직접 집계)
4. `QuizModal.svelte`:
   - `Props`에 `tagId?: string | null` 추가
   - `saveWrongAnswer()` 호출 시 `tag_id: tagId ?? null` 전달
5. `+page.svelte`:
   - `wrongAnswerTagMap` $derived 제거
   - `filteredWrongAnswers` → `filterWrongAnswers(wrongAnswers, selectedTagId)` 호출로 교체
   - `wrongAnswerFilterTags` → `buildWrongAnswerFilterTags(allTags, wrongAnswers)` 호출로 교체
   - `buildWrongAnswerTagMap` import 제거

**산출물**: favorites 브릿지 없이 tagId 기반으로 오답 필터 동작

---

### T-02 웹 — 피드 새로고침 전환

**목적**: 초기 로드 시 태그별 분리 저장 + refresh 현재 탭만 재요청

**대상 파일**:
- `web/src/lib/stores/feedStore.svelte.ts`

**작업 상세**:
1. `loadFeed()`:
   - 기존: `tagCache = new Map([['all', makeTabState(items)]])`
   - 변경: items를 `item.tag_id` 기준으로 그룹핑 → `tagCache['all']` + `tagCache[tagId]` 각각 저장
2. `feedItems` $derived:
   - 기존: `activeTagId ? allItems.filter(item => item.tag_id === activeTagId) : allItems`
   - 변경: `tagCache.get(activeTagId ?? 'all')?.items ?? []` (클라이언트 필터 제거, 태그별 캐시 직접 참조)
3. `hasMore` $derived: `tagCache.get(activeTagId ?? 'all')?.hasMore ?? true`
4. `isLoadingMore` $derived: `tagCache.get(activeTagId ?? 'all')?.status === 'loadingMore'`
5. `loadMore()`:
   - `key = 'all'` 고정 유지 (태그 탭에서 loadMore 없음 — iOS와 동일하게 통일)
   - 태그 탭에서는 초기 분리 저장분만 표시
6. `refresh()`:
   - 항상 `fetchFeed(undefined, { noCache: true })` 전체 재요청
   - 응답으로 `tagCache` 전체 재분리 저장 (단순하고 일관성 있음)

**산출물**: 탭 전환 즉시 표시(API 없음) + refresh 현재 탭만 갱신 + 전체 탭 반영

---

### T-03 iOS — WrongAnswer 모델 + 필터 전환

**목적**: favorites 브릿지 제거 → WrongAnswer.tagId 직접 사용

**대상 파일**:
- `ios/Frank/Frank/Sources/Core/Models/WrongAnswer.swift`
- `ios/Frank/Frank/Sources/Features/Favorites/WrongAnswerTagFilter.swift`
- `ios/Frank/Frank/Sources/Features/Favorites/FavoritesView.swift`

**작업 상세**:
1. `WrongAnswer.swift`:
   - `tagId: UUID?` 필드 추가
   - `CodingKeys`에 `case tagId = "tag_id"` 추가
   - `SaveWrongAnswerParams`에 `tagId: UUID?` 필드 추가 + `CodingKeys.tagId = "tag_id"` 추가
2. `WrongAnswerTagFilter.swift`:
   - `buildTagMap(from:)` 함수 **삭제**
   - `apply(items:tagMap:selectedTagId:)` → `filter(items:selectedTagId:)` 로 교체
     - 내부: `wa.tagId == selectedTagId` 로 필터
3. `FavoritesView.swift`:
   - `wrongAnswerTagMap` computed 프로퍼티 **삭제**
   - `wrongAnswerTags` computed: `wrongAnswersFeature.items.compactMap(\.tagId)` 기준으로 태그 집계 → `feature.tags.filter { tagIds.contains($0.id) }`
   - `filteredWrongAnswers` computed: `WrongAnswerTagFilter.filter(items: wrongAnswersFeature.items, selectedTagId: selectedTagId)` 호출

**산출물**: favorites 로드 없이 tagId 기반 오답 필터 동작

---

### T-04 iOS — 피드 초기 로드 + 새로고침 전환

**목적**: 초기 로드 시 태그별 분리 저장 + selectTag 캐시 히트 보장 + refresh 전체 탭 반영

**대상 파일**:
- `ios/Frank/Frank/Sources/Features/Feed/FeedFeature.swift`

**작업 상세**:
1. `loadInitial()`:
   - 기존: `tagStates["all"] = .firstPage(items: items, pageSize: PAGE_SIZE)`
   - 변경: items를 `article.tagId` 기준으로 Dictionary(grouping:by:) 그룹핑
     → `tagStates["all"] = .firstPage(items: items, pageSize: PAGE_SIZE)`
     → 각 그룹: `tagStates[tagId.uuidString] = .firstPage(items: group, pageSize: group.count)` (hasMore = false — 초기 전체 fetch이므로)
2. `selectTag()`:
   - 캐시 히트 시 즉시 표시 (기존 동작 유지)
   - 캐시 미스 시 서버 재요청 폴백 **유지** (초기 20개에 해당 태그 없을 경우 대비)
   - 변경 없음 — 단, loadInitial 분리 저장으로 캐시 히트율이 높아짐
3. `refresh()`:
   - 항상 `fetchFeed(tagId: nil)` 전체 재요청
   - 응답으로 `tagStates` 전체 재분리 저장 (단순하고 일관성 있음)
4. `loadMore()`:
   - 태그 탭에서 loadMore 불가 (`'all'` 탭 기준으로만 동작 — 웹과 동일하게 통일)
   - 태그 탭에서는 초기 분리 저장분만 표시 (`hasMore = false`)

**산출물**: 초기 로드 후 모든 태그 탭 즉시 전환 가능 + refresh 전체 탭 반영 + 희귀 태그 폴백 유지

---

### T-05 웹 테스트

**목적**: T-01, T-02 변경 사항 검증

**대상 파일**:
- `web/src/routes/favorites/__tests__/favorites-tag-filter.test.ts`
- `web/src/lib/stores/feedStore.test.ts`

**작업 상세**:
1. `favorites-tag-filter.test.ts`:
   - `filterWrongAnswers(wrongAnswers, selectedTagId)` — tagId 기반 필터 동작 (브릿지 파라미터 제거 반영)
   - `buildWrongAnswerFilterTags(allTags, wrongAnswers)` — WrongAnswer.tagId 직접 집계 확인
   - E-01: tagId null인 오답은 태그 선택 시 필터 제외
   - E-02: selectedTagId null이면 전체 반환 (tagId null 포함)
2. `feedStore.test.ts`:
   - `loadFeed()` 후 태그별 캐시 분리 저장 확인
   - `refresh()` 후 현재 탭만 갱신, 다른 탭 캐시 유지 확인
   - `refresh()` 후 tagCache['all'] 해당 태그 기사 교체 확인
   - `loadMore()` 현재 탭 기준 페이지네이션 확인

**산출물**: 단위 테스트 전체 통과

---

### T-06 iOS 테스트

**목적**: T-03, T-04 변경 사항 검증

**대상 파일**:
- `ios/Frank/Frank/Tests/WrongAnswerTagFilterTests.swift` (기존 업데이트)
- `ios/Frank/Frank/Tests/FeedFeatureTests.swift` (기존 업데이트)

**작업 상세**:
1. `WrongAnswerTagFilterTests.swift`:
   - `filter(items:selectedTagId:)` — tagId 기반 필터 동작 확인
   - selectedTagId nil → 전체 반환 확인
   - tagId nil인 오답 → 태그 선택 시 필터 제외 확인
   - buildTagMap 함수 관련 기존 테스트 제거
2. `FeedFeatureTests.swift`:
   - `loadInitial()` 후 tagStates["all"] + 각 tagId별 분리 저장 확인
   - `selectTag()` — 캐시 히트 시 서버 요청 없음 확인
   - `selectTag()` — 캐시 미스 시 빈 상태 표시 (서버 요청 없음) 확인
   - `refresh()` 후 tagStates["all"] 해당 태그 기사 교체 확인

**산출물**: 단위 테스트 전체 통과

---

## 의존성 DAG

```
T-01 ─┐
      ├─→ T-05 (웹 테스트)
T-02 ─┘

T-03 ─┐
      ├─→ T-06 (iOS 테스트)
T-04 ─┘
```

**병렬 실행 가능 그룹**:
- `T-01 ‖ T-02` (둘 다 독립)
- `T-03 ‖ T-04` (둘 다 독립)
- `(T-01+T-02) ‖ (T-03+T-04)` (웹/iOS 완전 독립)

**직렬 순서**:
- 웹: T-01 → T-05, T-02 → T-05 (T-05는 T-01, T-02 완료 후)
- iOS: T-03 → T-06, T-04 → T-06 (T-06은 T-03, T-04 완료 후)

---

## 완료 기준 매핑

| 서브태스크 | 완료 기준 |
|---|---|
| T-01 | F-01, F-02, F-03, G-02 |
| T-02 | F-07, F-08, F-09, G-03 |
| T-03 | F-04, F-05, F-06, G-02 |
| T-04 | F-10, F-11, F-12, G-03 |
| T-05 | T-01+T-02 단위 테스트 통과 |
| T-06 | T-03+T-04 단위 테스트 통과 |

---

## 인터뷰 결정

| Q | 결정 | 내용 |
|---|---|---|
| Q1 | A | `feedItem?.tag_id` → QuizModal `tagId` prop 전달 |
| Q2 | A | 태그 탭 `loadMore()` → `fetchFeed(activeTagId, offset)` |
| Q3 | A | iOS `loadMore()` → `fetchFeed(tagId: currentTagId, offset: currentCount)` |
| Q4 | A | `FavoritesView` computed property 직접 수정 (`wrongAnswerTagMap` 삭제) |

---

## 주요 판단 사항

1. **T-02 loadMore() 수정 범위**: `key = 'all'` 하드코딩을 `key = activeTagId ?? 'all'`로 변경하여 태그 탭에서 loadMore가 각 탭 독립적으로 동작하도록 함. G-03(무한 스크롤 회귀) 달성에 필수.

2. **T-04 selectTag() 캐시 미스 처리**: 서버 재요청 제거 후 캐시 미스(해당 태그 기사 없음) = 빈 목록 표시. loadInitial에서 전체 fetch로 이미 모든 태그 분리 저장하므로, 캐시 미스는 실제로 해당 태그 기사가 0개임을 의미.

3. **`SaveWrongAnswerParams.tagId`**: 기존 오답 저장 API는 `tag_id`를 받지 않음. 서버 M1에서 이미 `tag_id` 컬럼 추가 완료이므로 클라이언트도 전달하도록 변경 필요.

---

## Feature List
<!-- size: 대형 | count: 30 | skip: false -->

### 기능
- [x] F-01 웹 `WrongAnswer` 타입에 `tagId: string | null` 필드 파싱
- [x] F-02 웹 오답 필터가 `wa.tagId === selectedTagId` 기반으로 동작
- [x] F-03 웹 오답 태그 칩이 `WrongAnswer.tagId` 기반으로 계산됨
- [x] F-04 웹 `QuizModal`에 `tagId` prop 추가 + 오답 저장 시 `tag_id` 전달
- [x] F-05 웹 `loadFeed()` 후 `tagCache`에 `'all'` + 각 태그별 캐시 분리 저장
- [x] F-06 웹 태그 탭 클릭 시 서버 요청 없이 즉시 표시
- [x] F-07 웹 `refresh()` 전체 재요청 + `buildTagCache`로 `tagCache` 완전 재구성 (Q3 확정: 전체 재요청 정책)
- [x] F-08 웹 태그 탭에서 `loadMore()` 없음 (전체 탭에서만 동작 — iOS와 통일)
- [x] F-09 iOS `WrongAnswer` 모델에 `tagId: UUID?` 필드 파싱
- [x] F-10 iOS 오답 필터가 `wa.tagId == selectedTagId` 기반으로 동작
- [x] F-11 iOS 오답 태그 칩이 `wrongAnswersFeature.items.compactMap(\.tagId)` 기반으로 계산됨
- [x] F-12 iOS `loadInitial()` 후 `tagStates["all"]` + 각 tagId별 캐시 분리 저장
- [x] F-13 iOS 태그 탭 클릭 시 캐시 히트면 즉시 표시, 캐시 미스면 서버 재요청 (폴백 유지)
- [x] F-14 iOS `refresh()` 전체 재요청 + `rebuildTagStates`로 `tagStates` 완전 재구성 (Q3 확정: 전체 재요청 정책)

### 엣지
- [x] E-01 `tagId: null`인 오답은 태그 선택 시 필터 제외 (웹+iOS)
- [x] E-02 `selectedTagId: null`(전체 탭)이면 tagId null 포함 전체 오답 반환
- [x] E-03 구독 태그가 1개일 때 피드 분리 저장 정상 동작
- [x] E-04 특정 태그에 해당하는 기사가 0개일 때 빈 목록 표시 (iOS 캐시 미스)
- [x] E-05 전체 탭에서 `refresh()` 시 전체 재fetch (`activeTagId = null` → `fetchFeed(undefined)`)

### 에러
- [x] R-01 `loadMore()` 중 `refresh()` 호출 차단 (race condition 방지) 유지
- [x] R-02 `tag_id` 없는 기존 오답 저장 요청도 정상 처리 (null 전달)
- [x] R-03 iOS `SaveWrongAnswerParams.tagId = nil` 인코딩 시 서버 정상 수신 (서버 통합 테스트에서 tag_id: null 케이스 커버됨)

### 테스트
- [x] T-01 웹 `filterWrongAnswers(wrongAnswers, selectedTagId)` unit test — tagId 기반 필터
- [x] T-02 웹 `buildWrongAnswerFilterTags(allTags, wrongAnswers)` unit test — tagId 직접 집계
- [x] T-03 웹 `feedStore.loadFeed()` unit test — 태그별 캐시 분리 저장 확인
- [x] T-04 웹 `feedStore.refresh()` unit test — 전체 재요청 + tagCache 완전 재구성 확인
- [x] T-05 웹 `feedStore.loadMore()` unit test — 현재 탭 기준 페이지네이션 확인
- [x] T-06 iOS `WrongAnswerTagFilter.filter(items:selectedTagId:)` unit test
- [x] T-07 iOS `FeedFeature.loadInitial()` unit test — tagStates 분리 저장 확인
- [x] T-08 iOS `FeedFeature.selectTag()` unit test — 캐시 히트/미스 동작 확인
- [x] T-09 iOS `FeedFeature.refresh()` unit test — tagStates 완전 재구성 확인

### 회귀
- [x] G-01 즐겨찾기 탭 태그 필터 기존 동작 유지 (favorites.tagId 기반 — 변경 없음)
- [x] G-02 오답 저장 전체 플로우 정상 동작 (tag_id 전달 포함)
- [x] G-03 피드 무한 스크롤 정상 동작 (전체 탭 + 태그 탭 모두)
