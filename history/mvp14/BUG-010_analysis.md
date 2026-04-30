# BUG-010 분석: 태그 전환 시 기사 자동 변경 동작

> 작성일: 2026-04-29
> ST-08 산출물
> 결론: **정상 동작** — 버그 아님, 수정 없이 문서화로 종료

---

## 증상

태그를 누르다 보면 기사 목록이 자동으로 바뀜. 의도된 동작인지 불명확.

## 코드 탐색 결과

### iOS (`FeedFeature.swift`)

`selectTag()` 흐름:

1. `tagStates[key] != nil` → 캐시 히트 → `selectedTagId = tagId`, 즉시 표시 (API 없음)
2. `tagStates[key] == nil` → 캐시 미스 → `fetchFeed(tagId:, noCache: false)` 서버 호출
   - 서버 캐시 HIT: 5분 내 이전 결과 반환
   - 서버 캐시 MISS: 검색 API 재호출 → 최신 기사 목록 반환

**`loadInitial()`**: `hasLoadedInitially` 플래그로 최초 1회만 실행. 이후 탭 전환은 `tagStates` 캐시 기반.

### 웹 (`feedStore.svelte.ts`)

동일 패턴:
- 캐시 히트(`tagCache.has(key)`) → `activeTagId = tagId`만 변경
- 캐시 미스 → `apiClient.fetchFeed(tagId, ...)` 서버 호출

---

## 기사 "자동 변경" 원인 분석

| 시나리오 | 원인 | 버그 여부 |
|---------|------|---------|
| 탭 전환 직후 기사 변경 | 해당 태그 `tagStates` 캐시 없음 → 서버 재요청 → 최신 기사 | 정상 (캐시 만료 후 탭 재방문 시 최신화) |
| 앱 재시작 후 기사 변경 | `tagStates` 메모리 캐시 초기화 → 재요청 → 새 결과 | 정상 |
| 같은 탭 여러 번 탭 전환 | 캐시 히트 → 변경 없음 | 정상 |
| 서버 5분 TTL 이후 탭 전환 | 서버 캐시 MISS → 새 검색 → 기사 업데이트 | 정상 (TTL 설계 의도) |

**결론**: 어떤 시나리오도 "사용자 명시적 탭 없이 기사가 변경"되는 경우가 없다.
태그 탭 선택 자체가 명시적 액션이며, 캐시 미스 시 최신 기사를 가져오는 것은 의도된 동작이다.

---

## 판단: 정상 동작

- "사용자 명시적 탭 없이 기사 변경 = 버그" 기준 → 해당 없음
- 탭 선택은 명시적 액션
- 캐시 미스 후 최신 기사 표시는 의도된 설계

**M2 내 추가 수정 없음. M3 이관도 불필요.**

---

## 코드 증거

`ios/Frank/Frank/Sources/Features/Feed/FeedFeature.swift`:
```swift
private func selectTag(_ tagId: UUID?) async {
    // 캐시 히트 → 즉시 표시, 재요청 없음
    if tagStates[key] != nil {
        selectedTagId = tagId
        return
    }
    // 캐시 미스 → 서버 재요청 (의도적 최신화)
    let items = try await article.fetchFeed(...)
    tagStates[key] = .firstPage(items: items, pageSize: PAGE_SIZE)
}
```

`web/src/lib/stores/feedStore.svelte.ts`:
```typescript
async function selectTag(tagId: string | null): Promise<void> {
    if (!tagCache.has(key)) {
        // 캐시 미스 → 서버 재요청 (의도적 최신화)
        const items = await apiClient.fetchFeed(...)
        tagCache = new Map([...tagCache, [key, { items, ... }]])
    }
    // 캐시 히트 → activeTagId만 변경
}
```
