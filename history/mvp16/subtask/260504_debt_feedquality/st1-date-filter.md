# ST-1: A-1 발행 날짜 필터

> 브랜치: feature/260504_feed-quality-debt-cleanup  
> 파일: server/src/api/feed.rs

## 요약
7일 이상 된 기사를 피드에서 제외. published_at=None은 통과.

## Feature List
<!-- size: 소형 | count: 6 | skip: true -->

### 기능
- [x] F-01 날짜 필터를 `state.feed_cache.set` 직전(items 최종 정제 후)에 적용 — HIT/MISS 양쪽 커버
  - 이유: 캐시 저장 전에 필터하면 이후 HIT 경로에서도 이미 필터된 items가 반환됨
  - 구현 위치: feed.rs `state.feed_cache.set(&cache_key, items.clone(), ttl)` 직전
- [x] F-02 published_at=None 기사는 날짜 알 수 없으므로 통과 처리

> ⚠️ **캐시 HIT 경로 주의**: `feed.rs:250-257` 캐시 HIT 시 `cached_items`를 그대로 반환.
> 필터를 MISS 경로 변환 직후에만 적용하면 HIT 경로가 우회됨.
> **캐시 TTL은 최대 30분**이므로 7일 경계를 넘을 수는 없으나, 기존 캐시 항목에 이미 7일+ 기사가
> 들어있을 수 있음. `feed_cache.set` 직전 필터링으로 저장/조회 양쪽을 동시에 해결.

### 엣지
- [x] E-01 경계값: `published_at <= Utc::now() - Duration::days(7)` 이면 제외 (정확히 7일 경과 포함)
  - `Duration::days(7)` 기준 이전(포함) = 7일 이상 된 기사 → 제외
  - 6일 23시간 59분 → 통과 / 7일 0시간 0분 → 제외

### 에러
- [x] R-01 날짜 파싱 오류 발생 시 None으로 처리하여 통과 (방어적)

### 테스트
- [x] T-01 8일 전 기사 제외, 6일 전 기사 통과, None 통과 단위 테스트 3개
- [x] T-02 정확히 7일 전(경계값) 기사 제외 검증
