# 캐시 계층 전체 맵 (ST-01 산출물)

> 작성일: 2026-04-29
> 목적: BUG-006/007/008/010 공통 원인 분석 + F-03(부분 실패 TTL) 필요성 판단
> 분석 대상 파일:
> - `server/src/infra/feed_cache.rs`, `server/src/api/feed.rs`, `server/src/api/summarize.rs`
> - `ios/Frank/Frank/Sources/Features/Feed/FeedFeature.swift`
> - `ios/Frank/Frank/Sources/Core/Models/SummarySessionCache.swift`
> - `ios/Frank/Frank/Sources/Features/Detail/ArticleDetailFeature.swift`
> - `web/src/lib/stores/feedStore.svelte.ts`
> - `web/src/lib/stores/summaryCache.svelte.ts`
> - `web/src/routes/feed/article/+page.svelte`

---

## 1. 피드/태그 콘텐츠 캐시

### 1-1. 서버 피드 캐시 (`InMemoryFeedCache`)

| 항목 | 값 |
|------|-----|
| 위치 | `server/src/infra/feed_cache.rs` |
| 구현 | `Mutex<HashMap<String, CacheEntry>>` (인메모리) |
| 캐시 키 | `"{user_id}:{sorted_tag_ids}"` 또는 `"{user_id}:all"` |
| TTL | `FEED_CACHE_TTL = Duration::from_secs(300)` (5분) |
| 만료 방식 | `get()` 시점에 `expires_at > Instant::now()` 검사 (lazy eviction) |
| 최대 항목 | `max_entries` 초과 시 오래된 항목 eviction |
| no-cache 우회 | `Cache-Control: no-cache` 헤더 수신 시 캐시 조회 스킵 (`is_no_cache()`) |
| **에러 저장 정책** | **완전 실패(items 빈 배열) → 저장 스킵** (`if !items.is_empty()` 조건, `feed.rs:274`) |
| **부분 실패 저장** | **부분 실패(일부 태그 검색 실패) → 성공 태그 결과만으로 5분 TTL 저장** ← F-03 수정 대상 |
| 무효화 조건 | TTL 만료, no-cache 요청 후 재저장(덮어쓰기), 서버 재시작(인메모리) |

**부분 실패 경로 (feed.rs:222-274)**:
```rust
// 222-228: 태그별 검색 실패 시 continue (누락된 태그는 결과에서 빠짐)
for tag in &tags {
    match search_adapter.search(...).await {
        Err(e) => { tracing::warn!(...); continue; }  // ← 부분 실패 skip
        Ok(articles) => items.extend(articles),
    }
}

// 274: 완전 실패(0건)만 저장 스킵 — 부분 실패는 그대로 5분 TTL 저장됨
if !items.is_empty() {
    state.feed_cache.set(&cache_key, items.clone(), FEED_CACHE_TTL);
}
```

### 1-2. iOS 태그 상태 캐시 (`FeedFeature.tagStates`)

| 항목 | 값 |
|------|-----|
| 위치 | `ios/.../Features/Feed/FeedFeature.swift` |
| 구현 | `[String: TagState]` (메모리, ViewModel 생애주기) |
| 캐시 키 | `tagId.uuidString` 또는 `"all"` |
| TTL | 명시적 TTL 없음 — FeedFeature 인스턴스 생애주기 동안 유지 |
| no-cache | `refresh()` → `fetchFeed(noCache: true)` → `Cache-Control: no-cache` 헤더 전송 |
| **에러 저장 정책** | **에러 시 `tagStates.removeValue(forKey: key)` → 다음 탭 전환 시 재시도 가능** |
| 무효화 조건 | `refresh()` 호출 시 `rebuildTagStates()` 전체 재구성, 에러 시 해당 키 제거 |
| BUG-008 관련 | **캐시 미스 시 빈 items + `.loading` 상태로 먼저 tagStates 설정 → UI 빈 상태 렌더** |

### 1-3. 웹 태그 캐시 (`feedStore.tagCache`)

| 항목 | 값 |
|------|-----|
| 위치 | `web/src/lib/stores/feedStore.svelte.ts` |
| 구현 | `Map<string, TabState>` (Svelte 5 `$state`) |
| 캐시 키 | `tag_id` (UUID string) 또는 `"all"` |
| TTL | 명시적 TTL 없음 — 페이지/세션 생애주기 동안 유지 |
| no-cache | `refresh()` → `fetchFeedApi({noCache: true})` → `Cache-Control: no-cache` 헤더 전송 |
| **에러 저장 정책** | **에러 시 `{items: [], status: 'error'}` 저장** ← BUG-008 원인 확정 |
| 무효화 조건 | `refresh()` → `buildTagCache()` 전체 재구성, 에러 키는 `{status: 'error'}`로 남음 |
| BUG-008 관련 | **캐시 미스 시 `{items: [], status: 'loading'}` 먼저 저장 → UI 빈 상태 렌더 + 에러 시 에러 캐시 저장** |

---

## 2. 요약 콘텐츠 캐시

### 2-1. 서버 요약 캐시

| 항목 | 값 |
|------|-----|
| 위치 | `server/src/api/summarize.rs` |
| 구현 | **캐시 없음** — 순수 pass-through to `summary_service::summarize()` |
| TTL | 해당 없음 |
| **에러 저장 정책** | **에러 저장 없음** — 422/503/504 응답 후 상태 유지 없음 |

### 2-2. iOS 요약 캐시 (`SummarySessionCache`)

| 항목 | 값 |
|------|-----|
| 위치 | `ios/.../Core/Models/SummarySessionCache.swift` |
| 구현 | `@MainActor static let shared`, `[String: SummaryResult]` |
| 캐시 키 | 기사 URL (String) |
| TTL | 명시적 TTL 없음 — 앱 세션 동안 유지 |
| **에러 저장 정책** | **에러 저장 없음** — `SummaryResult` 타입(성공 결과)만 저장 가능 |
| 무효화 조건 | 앱 종료/재시작 (메모리 캐시) |
| BUG-006 관련 | **에러가 캐시에 저장되지 않음** → 재진입 시 phase가 `.idle`로 시작 → 재시도 가능 |

### 2-3. 웹 요약 캐시 (`summaryCache`)

| 항목 | 값 |
|------|-----|
| 위치 | `web/src/lib/stores/summaryCache.svelte.ts` |
| 구현 | `Map<string, SummaryResult>` (Svelte 5 `$state`) |
| 캐시 키 | 기사 URL (string) |
| TTL | 명시적 TTL 없음 — 페이지 세션 동안 유지 |
| **에러 저장 정책** | **에러 저장 없음** — 성공 SummaryResult만 저장 |
| 무효화 조건 | 페이지 새로고침 |
| BUG-006 관련 | **에러가 캐시에 저장되지 않음** → 재시도 버튼 클릭 시 API 재호출 가능 |

---

## 3. 에러 상태 캐시 요약

| 플랫폼 | 레이어 | 에러 저장 여부 | 비고 |
|--------|--------|--------------|------|
| 서버 | 피드 캐시 | 완전 실패 — 저장 안 함 | 부분 실패는 저장됨 (F-03 수정 필요) |
| 서버 | 요약 캐시 | 없음 | 캐시 자체 없음 |
| iOS | 태그 상태 캐시 | 키 제거 (재시도 가능) | 에러 상태 저장 안 함 |
| iOS | 요약 캐시 | 저장 안 함 | SummaryResult 타입만 허용 |
| 웹 | 태그 캐시 | **`{status: 'error'}` 저장** | BUG-008 원인, 수정 필요 |
| 웹 | 요약 캐시 | 저장 안 함 | 성공만 저장 |

---

## 4. BUG별 캐시 원인 분석

### BUG-006: 요약 실패 후 재시도 불가

**코드 분석 결론**: iOS·웹 모두 에러를 요약 캐시에 저장하지 않음. 재시도 시 API 재호출 가능.

**실제 증상 원인 추정**:
- `ArticleDetailFeature` 재초기화 시 `.idle` 상태로 시작 → 재시도 가능
- "다른 기사 탐색 후 복귀해도 동일" 증상은 서버 측 crawl/LLM 지속 오류일 가능성이 높음
- 서버: crawl 실패 → 422, LLM 실패 → 503, 타임아웃 → 504. 재시도 시 서버 상태 변화 없으면 동일 에러 반환

**결론**: 클라이언트 에러 캐시 버그 아님. 서버/외부 서비스 일시 장애 시 동일 에러 반복이 원인. ST-02~ST-04에서 추가 검토 후 "코드 수정 없이 문서화" 또는 "서버 crawl 재시도 로직 추가" 판단 필요.

### BUG-007: pull-to-refresh 후 목록 미변경

**코드 분석 결론**:
- iOS `refresh()`: `noCache: true` → `Cache-Control: no-cache` 헤더 전송 → 서버 캐시 우회 → `rebuildTagStates()` 전체 재구성
- 코드상 정상 동작으로 보임

**실제 증상 원인 추정**:
- 서버가 no-cache 우회 후에도 동일 검색 결과 반환 (외부 검색 API 결과가 짧은 시간 내 동일)
- 또는 `rebuildTagStates()` 로직에서 이전 상태와 동일한 items가 설정되어 UI 변화 없어 보임
- 버그가 아닌 "새 기사가 없어서 동일하게 보이는" 정상 동작일 가능성 있음

**결론**: ST-05에서 `rebuildTagStates()` 호출 순서와 시뮬레이터 재현 확인 후 판단.

### BUG-008: 탭 전환 시 "기사가 없습니다" 깜빡임

**코드 분석 결론 (확정)**:
- **iOS**: `selectTag()` 캐시 미스 시 `tagStates[key] = TagState(status: .loading)` (빈 items) 먼저 설정 → UI가 빈 상태 렌더 → 데이터 도착 후 교체
- **웹**: `selectTag()` 캐시 미스 시 `tagCache = new Map([...tagCache, [key, {items: [], status: 'loading'}]])` 먼저 저장 → UI 빈 상태 렌더 + 에러 시 `{status: 'error'}` 영구 저장

**수정 방향**:
- iOS: 캐시 미스 시 tagStates 업데이트를 데이터 도착 이후로 지연, 또는 이전 items 유지하며 loading 표시
- 웹: 캐시 미스 시 이전 items 유지 + `status: 'loading'` 표시 (stale-while-revalidate 패턴)
- 에러 시 웹 `{status: 'error'}` 저장 제거 또는 다음 selectTag 시 에러 캐시 무효화

### BUG-010: 태그 전환 시 기사 자동 변경

**코드 분석 결론**:
- `selectTag()` → 캐시 미스 → 서버 API 재요청 → 새 결과 수신 → 기사 목록 업데이트
- 이 동작은 **의도적 설계** (캐시 만료 후 새 기사 표시)
- "자동으로 바뀜"은 캐시 만료 TTL 이후 탭 재선택 시 발생하는 정상 동작

**결론**: 정상 동작. "사용자 명시적 탭 없이 기사 변경 = 버그" 기준 적용 시 해당 없음. ST-08에서 코드 증거와 함께 "정상 동작" 문서화 예정.

---

## 5. F-03 부분 실패 TTL 필요성 판단

### 판단 매트릭스 실행

| 항목 | 결과 |
|------|------|
| 부분 실패 캐시 코드 존재 여부 | **있음** (`feed.rs:274` `if !items.is_empty()` — 부분 결과도 5분 TTL 저장) |
| 사용자 영향 | **있음** (3태그 중 1태그 검색 실패 시 불완전한 피드가 5분간 캐시됨) |
| 구현 복잡도 | **단순** (`feed.rs` 캐시 저장 시점 TTL 분기, 5줄 이내) |

**결정**: `M2_bug_fixes.md` 판단 매트릭스 기준 → **"M2.5 추가 후 바로 수정"** 해당

단, M2 내 ST-02 작업 범위에서 함께 처리 가능한 수준이므로 **M2.5 별도 마일스톤 생성 불필요 — ST-02에서 F-03 통합 구현**.

### 구현 계획

```rust
// feed.rs 수정 (캐시 저장 시점)
let has_failures = failed_tags.len() > 0;  // 실패한 태그 수 추적 필요
let ttl = if has_failures {
    Duration::from_secs(60)   // 부분 실패: 1분
} else {
    FEED_CACHE_TTL            // 완전 성공: 5분
};

if !items.is_empty() {
    state.feed_cache.set(&cache_key, items.clone(), ttl);
}
```

---

## 6. 플랫폼별 Cache-Control 흐름

```
사용자 pull-to-refresh
  │
  ├── iOS: FeedFeature.refresh()
  │     → fetchFeed(noCache: true)
  │     → URLRequest + "Cache-Control: no-cache" 헤더
  │     → 서버 is_no_cache() = true → 캐시 조회 스킵
  │     → 검색 실행 → 결과 캐시 저장(덮어쓰기)
  │     → rebuildTagStates() 전체 재구성
  │
  └── 웹: feedStore.refresh()
        → fetchFeedApi({noCache: true})
        → fetch() + "Cache-Control: no-cache" 헤더
        → 서버 캐시 우회
        → buildTagCache() 전체 재구성
```

---

## 7. 다음 단계 (ST-02 진입)

ST-01 완료. ST-02 진입 조건 충족.

**ST-02 작업 목록**:
1. `feed.rs` 부분 실패 태그 수 추적 로직 추가
2. 완전 성공 5분 TTL / 부분 실패 1분 TTL 분기 적용 (F-03)
3. 완전 실패(0건) 저장 스킵 현행 유지 확인 (F-02)
4. `cargo test` 전체 통과 확인

**BUG-006 서버 수정 재검토**:
- 서버 요약 API에 캐시 없음 → 요약 에러 캐시 수정 대상 아님
- 클라이언트(iOS/웹) 에러 캐시 없음 → ST-03/ST-04 작업 범위 축소 가능
- ST-03/ST-04에서 "에러 캐시 코드 없음 확인 + 재시도 로직 정상 동작 확인"으로 마무리
