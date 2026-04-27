# MVP12 M3 착수 전 현황 분석

> 생성일: 260427
> 분석 유형: code-quality (착수 전 현황 파악)
> 분석 대상: 웹 오답 탭 태그 칩 / iOS-웹 구현 갭 / M3 범위 경계

---

## 1. 웹 오답 탭 태그 칩이 안 나오는 이유 (wrongAnswerTagMap 구조 추적)

### 1-1. 렌더링 조건 추적

`+page.svelte:399-421`:
```
{#if wrongAnswerFilterTags.length > 0}
  <!-- 태그 칩 렌더링 -->
{/if}
```

칩 영역은 존재한다. `wrongAnswerFilterTags.length === 0`일 때만 사라진다.

### 1-2. wrongAnswerFilterTags 파생 경로

```
allTags (onMount fetchTags)
  ↓
wrongAnswers (loadWrongAnswers, 오답 탭 최초 진입 시)
  ↓
wrongAnswerTagMap = buildWrongAnswerTagMap(favoritesStore.favorites)
  │  {url → tagId} 매핑. favorites에 tagId가 있는 기사만 포함.
  ↓
wrongAnswerFilterTags = buildWrongAnswerFilterTags(allTags, wrongAnswers, wrongAnswerTagMap)
  ↓  wrongAnswers 각각의 articleUrl을 wrongAnswerTagMap에서 조회
  ↓  매핑 있는 것만 tagId Set으로 수집
  ↓  allTags에서 해당 tagId만 필터
  → 결과: 실제 오답에 매핑된 태그만 칩으로 노출
```

### 1-3. 칩이 비는 3가지 원인

| 원인 | 조건 | 버그 여부 |
|---|---|---|
| A. allTags 미로드 | onMount fetchTags 실패/지연 | 비크리티컬 (설계 의도 — 태그 로드 실패 시 필터 숨김) |
| B. wrongAnswers 없음 | 오답 탭 미진입 또는 로드 실패 | 정상 동작 |
| C. favorites에 odalUrl 매핑 없음 | 오답 기사가 즐겨찾기 해제됨 | **M2 확정 정책 (의도된 동작)** |

**핵심**: 세 번째 원인(C)이 실사용에서 가장 흔한 시나리오다. 오답을 틀린 기사를 즐겨찾기에서 해제하면 해당 오답은 태그 칩에서 사라진다. 이는 BUG-F 수정(MVP12 M2)에서 확정한 정책 — "즐겨찾기 해제된 기사의 오답은 태그 선택 시 제외"의 직접 결과다. **버그가 아니라 데이터 의존성**이다.

### 1-4. wrongAnswerTagMap 구조

`buildWrongAnswerTagMap(favorites: Favorite[]): Record<string, string>`
```ts
// favorites-filter.ts:15-20
return Object.fromEntries(
  favorites
    .filter((f) => f.tagId !== null)
    .map((f) => [f.url, f.tagId as string])
);
```

- 키: 기사 URL
- 값: 태그 ID (string)
- tagId=null인 즐겨찾기는 맵에서 제외
- 즐겨찾기에 없는 기사(해제된 기사)는 당연히 제외

---

## 2. iOS 현재 구현과 웹 구현의 차이 매핑

### 2-1. 오답 필터링 정책 (핵심 갭)

| 항목 | 웹 (M2 확정) | iOS (현재 MVP11 M4) | 갭 종류 |
|---|---|---|---|
| favorites 미등록 오답 + 태그 선택 | **제외** (`tagId === selectedTagId`가 false) | **항상 포함** (`if tagMap[wrongAnswer.articleUrl] == nil { return true }`) | **정책 정반대** |
| 정책 근거 | BUG-F 수정 시 OQ3 확정: "분류 없음 → 제외" | MVP11 M4 설계: "favorites 미등록 오답은 항상 표시" | - |

**iOS 코드 위치**: `WrongAnswerTagFilter.swift:44`
```swift
// favorites에 없는 오답은 항상 표시
if tagMap[wrongAnswer.articleUrl] == nil { return true }
```

이 한 줄이 웹과 정반대 동작을 만든다. M3에서 이 줄을 제거해야 한다.

### 2-2. 오답 탭 태그 칩 소스

| 항목 | 웹 (M2 확정) | iOS (현재) | 갭 종류 |
|---|---|---|---|
| 칩 소스 | `wrongAnswerFilterTags` (오답에 실제 매핑된 태그만) | `feature.tags` (즐겨찾기 기사의 태그 전체) | iOS에 동등물 없음 |
| 칩 비는 조건 | 오답에 매핑된 tagId가 allTags에 없을 때 | `feature.tags`가 비거나 items가 빌 때 | - |

**iOS 코드 위치**: `FavoritesView.swift:293-306`
```swift
private func tagChipBar(onSelect: @escaping (UUID?) -> Void) -> some View {
    if !feature.tags.isEmpty {
        TagChipBarView(
            tags: feature.tags,  // ← 오답 탭에서도 이걸 그대로 사용
            ...
        )
    }
}
```

기사 탭과 오답 탭이 `feature.tags`를 공유하고 있다. 웹은 오답 탭에서 `wrongAnswerFilterTags`를 별도로 계산한다.

### 2-3. selectedTagId 자동 초기화 (BUG-C/BUG-E 대응)

| 항목 | 웹 (M2 확정) | iOS (현재) | 갭 종류 |
|---|---|---|---|
| 마지막 즐겨찾기 삭제 후 | `shouldResetTagId` 판단 후 null 초기화 | **초기화 없음** | BUG-E (iOS 고유) |
| 구현 위치 | `handleRemoveFavorite` 내 `shouldResetTagId()` 호출 | `removeFavorite(url:)` 후 추가 처리 없음 | - |

**웹 기준 동작**: `removeFavorite` 완료 후 남은 favorites에 현재 selectedTagId 매칭 항목이 없으면 자동 nil 초기화.
**iOS 현재**: selectedTagId가 이전 값 유지 → 필터 결과 빈 배열 고정 → 빈 화면 먹통.

### 2-4. 새 즐겨찾기 추가 후 태그 갱신 (BUG-D)

| 항목 | 웹 (M2 확정) | iOS (현재) | 갭 종류 |
|---|---|---|---|
| 추가 후 태그 칩 갱신 | `favoritesStore.favorites` 반응성으로 자동 반영 | **`feature.tags`가 갱신되지 않음** | BUG-D (iOS 고유) |
| 원인 | `favTagIds = $derived(buildFavTagIds(favoritesStore.favorites))` — 파생이라 자동 | `hasLoaded` 가드로 `loadFavorites()` 재호출 차단 | - |

**iOS BUG-D 직접 원인**: `FavoritesFeature.swift:86`
```swift
func loadFavorites() async {
    guard !hasLoaded else { return }  // ← 추가 후 재호출 해도 tags 갱신 안 됨
    ...
}
```

`addFavorite()` 이후 `feature.tags`를 재계산하는 로직이 없다. 웹은 `favTagIds`가 `$derived`로 favorites 배열을 실시간 투영하므로 추가 즉시 반영된다.

### 2-5. 무한 스크롤

| 항목 | 웹 (M2 확정) | iOS (현재) | 갭 종류 |
|---|---|---|---|
| 구현 방식 | IntersectionObserver + loadMore() | **미구현** | 신규 기능 |
| offset 전략 | PageSize=20, nextOffset 누적 | - | M3에서 구현 필요 |

---

## 3. M3 범위 경계 명확화

### 3-1. M3 내 범위 (이번 마일스톤에서 반드시)

| 항목 | 근거 |
|---|---|
| BUG-D: `addFavorite` 후 `feature.tags` 갱신 | 로드맵 M3 버그 수정 |
| BUG-E: `removeFavorite` 후 `selectedTagId` nil 초기화 | 로드맵 M3 버그 수정 |
| BUG-F iOS: `WrongAnswerTagFilter.apply`에서 "favorites 미등록 오답 항상 포함" 제거 | 웹 M2 정책과 동기화 |
| 오답 탭 칩 소스를 `feature.tags` → 오답 매핑 태그로 분리 | 웹 `wrongAnswerFilterTags` 동등 구현 |
| 좋아요/즐겨찾기 UX 재설계 (iOS 네이티브 컴포넌트) | 로드맵 M3 + M2 기준 싱크 |
| 무한 스크롤 (`onAppear` 또는 `LazyVStack`) | 로드맵 M3 + M2 기준 싱크 |

### 3-2. M3 이후 보완 범위 (현 마일스톤 제외)

| 항목 | 이유 |
|---|---|
| `hasLoaded` 가드 전반 리팩토링 | BUG-D 수정은 `addFavorite` 후 tags 로컬 갱신으로 국소 처리 가능. 전반 리팩토링은 별도 MVP |
| realClient.ts branches 커버리지 부채 | M2에서 이미 deferred로 분류, 별도 태스크로 이관 |
| WrongAnswer 모델 tagId 필드 추가 | 서버 스키마 변경 없이 클라이언트 tagMap으로 처리하는 것이 기술 결정 (로드맵:81) |
| favorites 없는 오답의 "전체" 탭 동작 재정의 | M2 정책 확정(selectedTagId=null 시 전체 반환)으로 충분, 추가 설계 불필요 |

---

## 4. 로드맵 라벨 정정 권고

`_roadmap.md:60`:
> BUG-D: 즐겨찾기 추가 이벤트 시 스크랩 탭 태그 목록 재조회 (웹 BUG-C 대응 기능)

**정정 필요**: BUG-D는 iOS 고유 버그(추가 후 태그 미갱신). 웹 BUG-C의 대응은 iOS **BUG-E**(삭제 후 먹통)이다.

정확한 대응 관계:
- 웹 BUG-C(삭제 후 먹통) ↔ iOS **BUG-E** (동일 로직)
- iOS **BUG-D** (추가 후 태그 미생성) = iOS 고유 버그 (웹은 `$derived` 반응성으로 자동 해결됨)

---

## 5. A/B/C 개선안

### A안 — 최소 갭 메우기 (권장)

**전략**: M3 내 범위(3-1)만 정확히 구현. `WrongAnswerTagFilter.apply`의 "미등록 항상 포함" 한 줄 제거 + `addFavorite` 후 tags 로컬 갱신 + `removeFavorite` 후 `selectedTagId` nil 초기화 + 오답 탭 전용 computed tag 추가.

**장점**: 최소 변경, 기존 테스트 수정 범위 명확, 웹 정책과 완전 동기화  
**단점**: hasLoaded 가드 부채는 다음 MVP로 이월  
**권장 이유**: 로드맵 원칙("M2 기능 범위 기준 싱크")에 가장 충실. M3 범위 over-engineering 방지.

### B안 — FavoritesFeature hasLoaded 가드 제거 + 전면 재로드

**전략**: `hasLoaded` 가드를 제거하고 addFavorite/removeFavorite 이후 항상 `loadFavorites()` 재호출. tags는 loadFavorites 내에서 자동 갱신.

**장점**: BUG-D, BUG-E 원인(hasLoaded 가드)을 근본 제거  
**단점**: 매 변이마다 API 재호출 → 네트워크 비용 증가. `hasLoaded`는 빈 즐겨찾기 보호도 겸하므로 제거 시 side-effect 검토 필요. M3 범위 초과.

### C안 — WrongAnswer 모델에 tagId 필드 추가 (서버 확장)

**전략**: 서버 `quiz_wrong_answers` 테이블에 `tag_id` 컬럼 추가. WrongAnswer 모델이 tagId를 직접 보유해 favorites 매핑 불필요.

**장점**: favorites 삭제 후에도 오답의 태그 정보 유지  
**단점**: 서버 스키마 변경 + 마이그레이션 + iOS/웹 모델 변경 필요. 로드맵 기술 결정 기록("클라이언트 사이드 B안")에 위배. M3 범위 완전 초과.

---

## 결론

**M3 착수 조건**: 명확히 충족. 웹(M2) 기준 정책이 확정되어 있고, iOS 갭 4개가 구체적 코드 위치까지 특정됨.

**핵심 구현 순서**:
1. `WrongAnswerTagFilter.apply`의 "미등록 항상 포함" 줄 제거 (BUG-F, 테스트도 업데이트)
2. 오답 탭 전용 `wrongAnswerTags` computed 추가 (FavoritesView에 `wrongAnswerTagMap` 확장)
3. `tagChipBar`를 기사/오답 탭별 다른 tags 소스로 분기
4. `addFavorite` 후 `feature.tags` 로컬 갱신 (BUG-D)
5. `removeFavorite` 후 `selectedTagId` nil 초기화 로직 추가 (BUG-E)
6. UX 재설계 (좋아요/즐겨찾기 구분)
7. 무한 스크롤 (onAppear 기반)

**개선안 채택**: **A안** (최소 갭 메우기)
