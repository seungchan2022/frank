# 심층 분석: 피드 캐시 에러 처리 방식 선택

**날짜**: 2026-04-29  
**유형**: code-quality  
**대상**: `server/src/api/feed.rs` + `server/src/infra/feed_cache.rs`  
**질문**: 검색 에러 발생 시 A(에러 저장 스킵) vs B(TTL 0) vs C(구조 파악 후 결정)

---

## 1. 현재 구현 파악

### 캐시 구조

```
FeedCachePort (trait)
├── NoopFeedCache        — 테스트 전용, 항상 MISS
└── InMemoryFeedCache    — 프로덕션, TTL + max_entries + LRU eviction
```

- 캐시 키: `"{user_id}:{sorted_tag_ids}"` 또는 `"{user_id}:all"`
- 기본 TTL: `300초 (5분)`
- 호출 위치:
  - `GET /me/feed` — `.get()` / `.set()` (feed.rs L146, L277)
  - `PUT /me/tags` — `.invalidate_user()` (tags.rs L42)

### 현재 에러 처리 흐름 (feed.rs L222~278)

```rust
for (tag_id, tag_name, search_result) in results {
    let (search_items, source) = match search_result {
        Ok(pair) => pair,
        Err(e) => {
            tracing::warn!(..., "search failed for tag, skipping");  // ← 태그 단위 에러 삼킴
            continue;
        }
    };
    // ... items에 push
}

// URL 중복 제거 후
if !items.is_empty() {          // ← A 패턴이 이미 부분 적용됨
    state.feed_cache.set(&cache_key, items.clone(), FEED_CACHE_TTL);
}
```

**핵심 관찰**: `if !items.is_empty()` 가드로 완전 실패(0건)는 저장하지 않는다.  
그러나 **부분 실패**(3개 태그 중 1개 실패 → 2/3 결과)는 정상 TTL(5분)로 캐시한다.

---

## 2. A / B / C 후보 분석

### 방안 A — 에러 시 저장 스킵 (skip-on-error)

```rust
// 에러 없이 성공한 태그만 items에 축적 (현재와 동일)
// 단, 실패한 태그가 있으면 캐시에 저장하지 않음
let had_error = results.iter().any(|(_, _, r)| r.is_err());
if !items.is_empty() && !had_error {
    state.feed_cache.set(&cache_key, items.clone(), FEED_CACHE_TTL);
}
```

**장점**
- 가장 단순. 코드 추가 없이 가드 조건 1줄 변경.
- 캐시에 "불완전한" 데이터가 들어갈 가능성 차단.
- 다음 요청에서 자연스럽게 재시도.

**단점**
- 불안정한 외부 검색 API가 반복 실패하면 캐시 HIT이 없어 매 요청 API 호출.
- 태그가 많을수록 부분 실패 확률 증가 → 캐시 효과 감소.

**현업 채택률**: 가장 흔함. Redis, Caffeine, Django cache 등 대부분 프레임워크의 기본 정책.

---

### 방안 B — TTL 0 저장

```rust
let ttl = if had_error {
    Duration::from_secs(0)   // 즉시 만료 (= 저장 의미 없음)
} else {
    FEED_CACHE_TTL
};
state.feed_cache.set(&cache_key, items.clone(), ttl);
```

**장점**: 없음 — TTL 0은 즉시 만료되므로 다음 get()에서 MISS. 저장을 건너뛰는 것과 **기능적으로 동일**하면서 불필요한 HashMap insert + evict 연산 추가.

**단점**
- 저장하고 바로 만료되는 쓰레기 엔트리 생성.
- Negative caching(짧은 TTL로 에러 마커 저장)과 혼동될 수 있음 — 목적이 다름.

**결론**: 실무에서 이 용도로 사용하지 않음. **채택 불가**.

---

### 방안 C — 구조 파악 후 결정

방안 C는 선택지가 아닌 **분석 프로세스**다. 코드베이스 파악이 완료됐으므로 이미 소비됨.

---

## 3. 진짜 결정 포인트 — 부분 성공 캐싱 정책

advisor가 지적한 핵심: A/B/C 프레이밍보다 더 중요한 **부분 성공 캐싱** 문제.

| 시나리오 | 현재 동작 | 사용자 영향 |
|---|---|---|
| 태그 3개, 전부 성공 | 캐시 SET (5분 TTL) | 정상 |
| 태그 3개, 1개 실패 | 2/3 결과를 5분 캐시 | **5분간 불완전 피드 노출** |
| 태그 3개, 전부 실패 | 저장 스킵 (items empty) | 매 요청 재시도 |

---

## 4. 최종 추천 — 개선안 A / B / C (개선 방향)

### 개선안 A (추천 ★) — 부분 성공 시 단축 TTL

```rust
let (ttl, cache_label) = if had_error {
    (Duration::from_secs(60), "partial")  // 1분 TTL
} else {
    (FEED_CACHE_TTL, "full")              // 5분 TTL
};
if !items.is_empty() {
    tracing::info!(cache_key = %cache_key, ttl_secs = ttl.as_secs(), %cache_label, "feed cache SET");
    state.feed_cache.set(&cache_key, items.clone(), ttl);
}
```

- 부분 성공 → 짧은 TTL(60초)로 저장: 사용자에게 불완전하더라도 빠른 응답 제공 + 1분 후 자동 재시도
- 완전 성공 → 기존 5분 TTL 유지
- `had_error` 플래그는 `results` 이터레이션 시 `bool` 하나로 추적 가능

**왜 이게 가장 좋은가**: 검색 API가 일시적 오류인 경우 1분 후 자연히 회복. 완전 실패 대비 응답 품질 저하 최소화. 코드 변경 최소.

---

### 개선안 B (차선) — 에러 태그 있으면 저장 완전 스킵 (원래 A)

```rust
let had_error = results.iter().any(|(_, _, r)| r.is_err());
if !items.is_empty() && !had_error {
    state.feed_cache.set(&cache_key, items.clone(), FEED_CACHE_TTL);
}
```

- 구현 가장 단순, 정합성 보장
- 단점: 외부 API 불안정 시 캐시 효과 0 → 매 요청 외부 API 호출

**채택 기준**: 사용자 수 적고 외부 API 안정성이 높을 때.

---

### 개선안 C (보류) — Stale-while-revalidate

기존 캐시 데이터를 반환하면서 백그라운드에서 refresh하는 패턴.  
MEMORY에 `project_mvp6_feed_perf.md`로 이미 기록됨.  
에러 처리보다 성능 최적화 목적에 가까워 이번 범위에서 제외. 별도 마일스톤 기획 권장.

---

## 5. 프로젝트 룰즈 적합성 평가

| 기준 | 개선안 A (단축 TTL) | 개선안 B (스킵) |
|---|---|---|
| `unwrap()` 금지 | 변화 없음 | 변화 없음 |
| trait 추상화 유지 | `FeedCachePort::set()` 시그니처 무변경 | 동일 |
| 테스트 커버리지 | `had_error=true` 케이스 추가 테스트 필요 | `had_error=true` 케이스 추가 테스트 필요 |
| 대규모 리팩토링 없음 | L274 가드 + ttl 변수 추가 | L274 가드 조건 변경만 |

**두 방안 모두 rust-scope.md 위반 없음.** 테스트 케이스 1개 추가 필요.

---

## 6. 현업 패턴 요약

| 패턴 | 사용 조건 | 이 프로젝트 적용 |
|---|---|---|
| Skip-on-error | 기본값, API 안정적 | 개선안 B |
| Partial + short TTL | 부분 실패 허용, 가용성 우선 | **개선안 A (추천)** |
| Negative caching (짧은 TTL) | Thundering herd 방지, high QPS | 현재 QPS에서 불필요 |
| Stale-while-revalidate | 응답 속도 최우선 | MVP 별도 기획 |
| TTL=0 저장 | **없음** — anti-pattern | 채택 불가 |

---

## 결론

**B(TTL 0)는 안티패턴, C는 프로세스 단계** → 실질 선택지는 A(스킵)와 단축 TTL.

**추천: 개선안 A (부분 성공 시 단축 TTL 60초)**

구현 비용이 `had_error` 변수 추적 + `ttl` 분기 2줄이며, 외부 검색 API의 일시적 장애 시에도 사용자에게 부분 피드를 보여주면서 1분 후 자연 회복된다. 완전 스킵(개선안 B)보다 가용성이 높고, TTL 0(방안 B) 안티패턴을 피한다.

**즉시 구현 범위**: `feed.rs` L274 블록 수정 + 테스트 케이스 1건 추가 (커밋 단위 분리 권장: `fix:` + `test:`).
