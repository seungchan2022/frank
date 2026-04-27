# 토론: 무한 스크롤 도입 시 피드 prefetch 전략

> 날짜: 260427  
> 단계: MVP12 M2 (웹 버그 수정 + 무한 스크롤)  
> 참여: Claude (사용자 의도·문서) · Codex (기술적 타당성) · Serena (아키텍처)

---

## 주제

무한 스크롤 도입 시 feedStore prefetch 전략 선택.  
현황: Svelte 5 `$state` 기반 `tagCache Map`, 구독 태그 전체 병렬 프리패치. 서버 M1 `limit/offset` 지원 완료.

---

## 가설 비교

| 항목 | A안 (prefetch 폐지) | B안 (prefetch 유지 + 첫 페이지) | C안 (전체 탭만 즉시, 나머지 lazy) |
|---|---|---|---|
| 초기 API 요청 수 | 1 (all만) | N+1 (전체 탭 + 태그 N개) | 1 (all만) |
| 탭 전환 UX | 항상 로딩 | 항상 즉시 | 전체 탭 즉시, 나머지 첫 방문 시 로딩 |
| tagCache 구조 | 제거 | `Map<string, {items, offset, hasMore}>` | `Map<string, {items, offset, hasMore}>` |
| loadFeed() 복잡도 | 최저 | 최고 (Promise.allSettled 유지 + paged) | 낮음 (Promise.allSettled 제거) |
| 코드 단순성 | 1위 | 3위 | 2위 |
| 실사용 최적화 | 3위 | 2위 | 1위 |

---

## 각 관점 포지션

### Claude — C안

- **규칙 준수**: "코드 단순성 우선" 원칙상 B보다 C가 훨씬 가볍다. loadFeed에서 `Promise.allSettled` 제거, 초기 API 1회.
- **실사용 최적화**: 전체 탭은 진입 시 가장 많이 사용. 이것조차 on-demand면 매 진입마다 로딩 플리커.
- **A 탈락 이유**: 모든 탭 전환 로딩 = 현재 사용자가 누리는 즉시성 완전 소실.

### Codex — C안

- **구현 복잡도 순위**: A < C < B.
- **무한 스크롤 결합 시 안전 패턴**: `items + offset + hasMore`를 캐시 엔트리에 묶어 단일 진실 원천으로 관리. `feedItems`와 `tagCache` 이중 진실 원천은 offset drift 원인.
- **C 지지 근거**: paged cache 구조는 어차피 필요하나, eager fetch 범위를 `all` 1개로 제한해 fan-out·테스트 폭발 방지.

### Serena — C안 (조건부)

- **구조적 관찰**: `tagCache`는 내부 구현(외부 노출 없음). 소비자는 `feed/+page.svelte`가 전부.
- **최대 리스크**: `feedItems`(UI 바인딩)와 `tagCache`(내부 저장) 이중 진실 원천 → 무한 스크롤 append 시 반드시 동기화 어긋남 발생.
- **조건**: `feedItems`를 `tagCache` 엔트리의 투영값으로 리팩토링 시에만 C안 안전.

---

## 합의

**3자 만장일치: C안 채택**

---

## 구현 조건 (필수)

### 조건 1. `feedItems`를 tagCache의 투영값으로

현재처럼 `feedItems`와 `tagCache`를 독립적으로 할당하지 않는다.  
`activeTagId` 변경 시 `feedItems = tagCache.get(key)?.items ?? []` 방식으로 파생.  
`loadMore()` append 시에도 tagCache 엔트리만 갱신 → feedItems는 자동 동기화.

### 조건 2. `ApiClient.fetchFeed()` 인터페이스 확장

```typescript
// client.ts 변경
fetchFeed(
  tagId?: string,
  options?: { noCache?: boolean; limit?: number; offset?: number }
): Promise<FeedItem[]>;
```

Mock·Real 양쪽 모두 확장. `limit/offset` 미지정 시 전체 반환 (서버 기존 동작 유지).

### 조건 3. tagCache 값 타입 변경

```typescript
type CacheEntry = {
  items: FeedItem[];
  nextOffset: number;   // 다음 요청 시 사용할 offset
  hasMore: boolean;     // false면 IntersectionObserver 트리거 무시
};

let tagCache = $state(new Map<string, CacheEntry>());
```

Svelte 5 반응성: 엔트리 갱신 시 항상 `tagCache = new Map([...tagCache, [key, newEntry]])`.

---

## 탈락 이유 요약

| 안 | 탈락 이유 |
|---|---|
| A안 | 전체 탭 포함 모든 탭 전환 시 로딩 플리커. 실사용 UX 퇴보. tagCache 제거해도 무한 스크롤용 탭 상태는 어딘가에 남음. |
| B안 | 태그 N개 × 첫 페이지 초기 프리패치 유지 → 서버 fan-out, 코드 단순성 원칙 위반. 무한 스크롤 상태까지 더해지면 복잡도 최대. |

---

## 다음 액션 (M2 구현 시)

1. `client.ts` `fetchFeed()` 시그니처에 `limit?`, `offset?` 추가
2. `realClient.ts` `fetchFeed()` 쿼리스트링 조건부 추가
3. `feedStore.svelte.ts` 리팩토링:
   - `tagCache` 값 타입 → `CacheEntry`
   - `loadFeed()`: all 탭만 첫 페이지 프리패치 (`limit=20, offset=0`)
   - `selectTag()`: 캐시 히트 → `feedItems` 갱신, 미스 → fetch + cache 저장
   - `loadMore()` 신규: `hasMore` 확인 → `nextOffset`으로 fetch → items append → cache 갱신
   - `feedItems`를 tagCache 투영값으로 (이중 진실 원천 제거)
4. `feed/+page.svelte`: IntersectionObserver sentinel 추가 → `loadMore()` 호출
