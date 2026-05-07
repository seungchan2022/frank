# ST-5: A2-b KR 파라미터/멀티패스 구현

> ⚠️ **통합됨**: 이 서브태스크는 ST-3으로 합산. `tavily.rs` 동일 파일 수정 충돌 방지.
> 구현은 ST-3 문서 "파트 2: 한국어 기사 노출" 섹션을 참조.

> 유형: feature | 의존: 없음 (ST-4 결과 참고) | 병렬: ST-2, ST-3, ST-4와 독립 | 예상: 2h (ST-3에 포함)

## 목적

Tavily API의 country/language 파라미터 또는 KR 쿼리 멀티패스로 한국어 기사를 일정 비율 확보한다.

## 파일

- `server/src/infra/tavily.rs` → `search()` 파라미터 또는 새 KR 전용 메서드
- `server/src/api/feed.rs` → 멀티패스 시 jobs 빌드 로직

## ST-4 결과에 따른 분기

### ST-4 방안 A 충분 (KR 기사 나옴)

ST-5 구현 불필요 — 스킵 처리 후 ST-7로.

### ST-4 방안 A 불충분 (KR 기사 여전히 0건)

아래 중 하나 구현:

#### 옵션 1: Tavily `country="kr"` 파라미터

```rust
let body_kr = serde_json::json!({
    "query": kr_query,   // 한국어 키워드만
    "country": "kr",
    "max_results": effective / 2,  // 전체 cap의 절반
    "topic": "news",
    ...
});
```

Tavily 문서 기준 `country` 파라미터가 지원되면 이 방법이 가장 깔끔.

#### 옵션 2: EN/KR 멀티패스 (별도 호출)

```rust
// EN 호출 (기존)
let en_results = chain.search(&en_query, max/2).await;
// KR 호출 (신규)
let kr_results = chain.search(&kr_query, max/2).await;
// 합치기 + 중복 제거
```

**비용 주의**: 호출 수 2배 → `progress/mvp16/cost_log.md` 업데이트 필수.

## 완료 기준

- [ ] Tavily country 파라미터 지원 여부 확인 (문서/실험)
- [ ] 선택 옵션 구현
- [ ] 멀티패스 선택 시 cost_log.md에 호출 수 증가 기록
- [ ] 테스트: KR 결과 포함 확인 (Mock 기반)
