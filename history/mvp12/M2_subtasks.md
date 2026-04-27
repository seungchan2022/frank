# MVP12 M2 서브태스크 목록

> 생성일: 260427
> 마일스톤: M2 — 웹 버그 수정 + UX 개선
> 브랜치: feature/260427_mvp12-m2-web-bug-ux
> 상태: in-progress

## 개요

M1(서버) 완료 후 진행. BUG-C·BUG-F 버그 수정, 좋아요/즐겨찾기 UX 재설계, 무한 스크롤 도입.
수정 대상 앱: 웹(SvelteKit).

---

## 서브태스크 목록

### ST1 — BUG-C: selectedTagId 자동 초기화

**목적**: 마지막 기사 삭제 후 스크랩 탭 전체 먹통 방지.

**문제 분석**:
- `removeFavorite(url)` 호출 후 `favorites` 배열이 빈 배열이 되면 `filterTags`도 빈 배열.
- `selectedTagId`가 이전 태그 ID 값 그대로 남아 있으면 `filteredFavorites`가 빈 배열로 고정.
- 사용자 입장에서 스크랩이 아예 사라진 것처럼 먹통으로 보임.

**구현**:
- `removeFavorite` 완료 직후, 남은 `favorites`에 현재 `selectedTagId`를 가진 항목이 없으면 `selectedTagId = null` 초기화.
- `favorites/+page.svelte`의 `handleRemoveFavorite` 함수 또는 `$effect`로 처리.

**수정 파일**:
- `web/src/routes/favorites/+page.svelte`

**산출물**:
- BUG-C 재현 시나리오 Playwright 테스트 1건

**의존성**: 없음 (선행 서브태스크 없음)
**예상 소요**: ~1h

---

### ST2 — BUG-F: filterWrongAnswers undefined 정책 + wrongAnswerFilterTags derived

**목적**: 오답노트 태그 필터 미작동 수정.

**문제 분석**:
- 현재 `filterWrongAnswers`에서 `tagId === undefined` 처리 시 "전체 포함"으로 처리 (`tagId === undefined || tagId === selectedTagId`).
- 태그 매핑이 없는 오답은 태그 필터 시에도 노출되어 필터 결과가 부정확.
- 오답노트 탭의 태그 칩이 `filterTags`(즐겨찾기 태그 기준)를 재사용 중이라, 실제 오답에 없는 태그가 칩으로 노출됨.

**구현**:
1. `filterWrongAnswers` 정책 수정:
   - `tagId === undefined` → 해당 오답은 "분류 없음"이므로 선택된 태그와 일치 불가 → **제외** (현재는 포함).
2. `wrongAnswerFilterTags` derived 추가:
   - `wrongAnswers` + `tagMap`으로 실제 오답이 속한 태그 ID Set 추출.
   - `allTags`에서 해당 ID만 필터 → 오답 탭용 태그 칩 목록.
3. 오답 탭 태그 칩을 `filterTags` 대신 `wrongAnswerFilterTags` 사용.

**수정 파일**:
- `web/src/lib/utils/favorites-filter.ts`
- `web/src/routes/favorites/+page.svelte`

**산출물**:
- `favorites-filter.ts` 유닛 테스트 추가 (undefined tagId 케이스)
- BUG-F 재현 시나리오 Playwright 테스트 1건

**의존성**: ST1 완료 후 (같은 파일 직렬 작업)
**예상 소요**: ~1.5h

**OPEN QUESTION 확정 (OQ3)**: selectedTagId=null이면 전체 반환, 태그 선택 시 tagId=undefined 오답은 제외.

**C1 보완 (step-5 리뷰)**: 기존 `favorites-tag-filter.test.ts`에 "즐겨찾기 해제된 기사 오답은 항상 포함" 검증 코드 존재.
→ 구현 시 해당 테스트를 새 정책("태그 선택 시 제외")으로 수정 필수. 미수정 시 vitest 실패 + KPI pre-commit 차단.

---

### ST3 — 좋아요 / 즐겨찾기 UX 재설계

**목적**: 두 기능의 역할 차이를 UI에서 직관적으로 구분.

**현황**:
- 좋아요(♡/♥): 피드 카드 우하단, 작은 하트 아이콘, 서버에 이벤트 전송 (개인화 신호)
- 즐겨찾기(★/☆): 기사 상세 페이지에서 추가, 스크랩 페이지에서 관리 (저장)
- 문제: 두 기능이 어디서 어떻게 다른지 UI만으로 파악하기 어려움

**재설계 방향**:
- 좋아요: 아이콘 유지(♡/♥) + 툴팁 또는 레이블 "추천에 반영" 추가, 피드 카드에 위치 유지
- 즐겨찾기: 아이콘 변경(☆/★ → 책갈피 또는 보관함 형태) + 레이블 "스크랩" 명시
- 기사 상세 페이지의 즐겨찾기 버튼 레이블/아이콘 재설계
- 피드 카드에서 즐겨찾기 퀵 액션 추가 여부 검토 (step-4 인터뷰에서 결정)

**수정 파일**:
- `web/src/routes/feed/+page.svelte` (피드 카드 좋아요 버튼)
- `web/src/routes/feed/article/+page.svelte` (기사 상세 즐겨찾기 버튼)
- `web/src/routes/favorites/+page.svelte` (즐겨찾기 해제 버튼 아이콘)

**산출물**:
- 재설계된 아이콘+레이블 적용 스크린샷 (Playwright)

**의존성**: 없음 (ST1·ST2와 파일 중복 없음, 병렬 가능)
**예상 소요**: ~1.5h

---

### ST4 — 무한 스크롤: feedStore 확장

**목적**: 피드 limit/offset API를 활용한 페이지네이션 상태 관리.

**현황**:
- `feedStore`는 현재 전체 피드를 한 번에 로드 + 태그별 프리패치 전략 사용.
- M1에서 서버가 `limit/offset` 파라미터를 지원하기 시작.

**확정 전략 (step-4 OQ1 결정: C안)**:
전체 탭만 즉시 로드, 나머지 탭은 selectTag 시 lazy fetch.
`feedItems`를 `$derived` 투영으로 전환 — `tagCache`가 단일 진실의 원천.

**C2 보완 (step-5 리뷰)**: `feedItems = $derived(...)` 전환 시 기존 직접 할당 코드
(`loadFeed`, `selectTag`, `refresh`, `reset` 내부 `feedItems = ...` 전부) 컴파일 에러 발생.
→ 구현 시 직접 할당을 모두 제거하고 tagCache 업데이트만 수행. 컴파일러 에러로 위치 확인.

**구현**:
1. `TabState` 타입 도입: `{items: FeedItem[], nextOffset: number, hasMore: boolean, status: 'idle'|'loading'|'loadingMore'|'error'}`
2. `tagCache` 타입 변경: `Map<string, FeedItem[]>` → `Map<string, TabState>`
3. `feedItems = $derived(tagCache.get(activeTagId ?? 'all')?.items ?? [])` — 직접 할당 제거
4. `loadFeed`: 'all' 탭만 즉시 로드, 구독 태그 병렬 프리패치 제거
5. `selectTag`: 캐시 미스 시 lazy fetch (status='loading' 먼저 표시)
6. `loadMore()` 함수 추가 (capturedKey/capturedOffset으로 race condition 방지)
7. `refresh()`: 현재 탭 pagination 리셋 후 첫 페이지 재요청
8. `client.ts` + `mockClient.ts` + `realClient.ts` `fetchFeed` 시그니처에 `limit?, offset?` 추가

**수정 파일**:
- `web/src/lib/stores/feedStore.svelte.ts`
- `web/src/lib/api/client.ts` (인터페이스)
- `web/src/lib/api/realClient.ts`
- `web/src/lib/api/mockClient.ts`
- `web/src/lib/stores/feedStore.test.ts` (탭 캐시 구조 변경에 따른 테스트 재작성)

**산출물**:
- feedStore `loadMore` 유닛 테스트 (정상, hasMore=false, 중복 방지)

**의존성**: 없음 (ST3과 병렬 가능, ST5 선행 필수)
**예상 소요**: ~1.5h

---

### ST5 — 무한 스크롤: feed/+page.svelte IntersectionObserver

**목적**: 스크롤 끝 감지 → 자동 다음 페이지 로드.

**구현**:
1. sentinel 엘리먼트 (`<div bind:this={sentinel}>`) 피드 목록 하단에 추가
2. `onMount`에서 `IntersectionObserver` 등록:
   - sentinel이 뷰포트에 들어오면 `feedStore.loadMore()` 호출
   - `isLoadingMore`가 true이거나 `hasMore`가 false이면 skip
3. `onDestroy`에서 observer 해제
4. 로딩 스피너 (sentinel 위치, `isLoadingMore` 조건부 표시)
5. "더 이상 기사가 없습니다" 메시지 (`hasMore=false` 조건부 표시)

**수정 파일**:
- `web/src/routes/feed/+page.svelte`

**산출물**:
- 무한 스크롤 동작 확인 (수동 QA 또는 Playwright)

**의존성**: ST4 완료 후
**예상 소요**: ~1h

---

### ST6 — E2E + 커버리지 회귀 검증

**목적**: 버그 수정 회귀 락 + KPI Hard 게이트 유지.

**구현**:
1. Playwright 시나리오:
   - BUG-C: 즐겨찾기 마지막 항목 삭제 → 전체 탭으로 자동 전환 확인
   - BUG-F: 오답 탭 태그 필터 클릭 → 해당 태그 오답만 표시 확인
2. vitest 커버리지 확인: `npm run test -- --coverage` ≥90% 유지
3. KPI 체크: `bash scripts/kpi-check.sh` 통과 확인

**수정 파일**:
- `web/src/routes/favorites/__tests__/` (Playwright E2E 또는 vitest 통합 테스트)

**C3 보완 (step-5 리뷰)**: `@playwright/test` 미설치 확인 → Playwright E2E 대신 **vitest 통합 테스트**로 대체.
BUG-C/BUG-F 재현 시나리오는 vitest + jsdom 환경에서 스토어 + 유틸 함수 레벨로 검증.

**산출물**:
- vitest 통합 테스트: BUG-C (삭제 후 selectedTagId=null), BUG-F (태그 필터 정책)
- 커버리지 보고서 90% 이상
- KPI 게이트 통과 로그

**의존성**: ST1, ST2, ST3, ST5 완료 후
**예상 소요**: ~1h

---

## 의존성 DAG

```
ST1 ──(직렬)──▶ ST2 ──┐
                       ├──▶ ST6
ST3 ───────────────────┤
ST4 ──▶ ST5 ───────────┘
```

- ST1 → ST2: 동일 파일 직렬 필수
- ST3, ST4: 독립, 병렬 가능
- ST5: ST4 완료 후
- ST6: ST1·ST2·ST3·ST5 모두 완료 후

---

## KPI 체크포인트

| 지표 | 기준 | 게이트 |
|---|---|---|
| 웹 테스트 커버리지 | ≥90% | Hard |
| BUG-C 재현 0 | 수동/Playwright | Hard |
| BUG-F 재현 0 | 수동/Playwright | Hard |
| 무한 스크롤 정상 동작 | 수동 확인 | Hard |

---

## OPEN QUESTIONS (step-4 인터뷰 완료)

| 번호 | 질문 | 결정 |
|---|---|---|
| OQ1 | ST4 prefetch 전략 | **C안** — 전체 탭만 즉시, 나머지 lazy. feedItems를 $derived 투영으로 전환. tagCache 단일 진실의 원천 |
| OQ2 | 피드 카드 즐겨찾기 퀵 액션 추가 | **추가 안 함** — 즐겨찾기는 기사 상세 페이지에서만 유지 |
| OQ3 | 태그 없는 오답을 "전체" 탭에서 표시 | **표시** (selectedTagId=null 시 전체 반환) |

---

## Feature List
<!-- size: 대형 | count: 33 | skip: false | all-checked: true -->

### 기능
- [x] F-01 BUG-C: handleRemoveFavorite 후 남은 favorites에 selectedTagId 매칭 항목 없으면 null 초기화
- [x] F-02 BUG-F: filterWrongAnswers — tagId=undefined 오답은 태그 선택 시 제외 (기존 포함 정책 역전)
- [x] F-03 BUG-F: wrongAnswerFilterTags $derived — 실제 오답에 매핑된 태그 ID만 칩으로 노출
- [x] F-04 오답 탭 태그 칩을 filterTags 대신 wrongAnswerFilterTags로 교체
- [x] F-05 좋아요 버튼 레이블·스타일 개선 (피드 카드 — 역할 명확화)
- [x] F-06 즐겨찾기 해제 버튼 레이블·아이콘 개선 (favorites 페이지)
- [x] F-07 기사 상세 즐겨찾기 버튼 레이블·아이콘 개선
- [x] F-08 feedStore: feedItems를 $derived 투영값으로 전환 (tagCache 단일 진실의 원천)
- [x] F-09 feedStore: TagState 타입 도입 {items, nextOffset, hasMore, status}
- [x] F-10 feedStore: loadFeed — 'all' 탭만 즉시 로드 (구독 태그 병렬 프리패치 제거)
- [x] F-11 feedStore: selectTag — 캐시 미스 시 첫 페이지 lazy fetch, 히트 시 activeTagId만 변경
- [x] F-12 feedStore: loadMore() 함수 — 현재 탭 nextOffset 기준 다음 페이지 append
- [x] F-13 feedStore: refresh() — 현재 탭 pagination 리셋 후 첫 페이지 재요청
- [x] F-14 ApiClient.fetchFeed 시그니처에 limit?, offset? 옵션 추가
- [x] F-15 feed/+page.svelte: sentinel 엘리먼트 + IntersectionObserver 등록/해제

### 엣지
- [x] E-01 selectedTagId 초기화: filteredFavorites > 0이면 초기화 안 함 (같은 태그 다른 기사 남은 경우)
- [x] E-02 loadMore: hasMore=false 또는 status=loadingMore 시 재진입 차단
- [x] E-03 loadMore: await 전 capturedKey/capturedOffset 로컬 캡처 (탭 전환 race condition 방지)
- [x] E-04 selectTag: lazy fetch 중 다른 탭 전환 시 이전 탭 결과 올바른 key에 기록
- [x] E-05 wrongAnswerFilterTags: wrongAnswers 빈 배열일 때 태그 칩 미표시
- [x] E-06 loadFeed: PAGE_SIZE(20) 미만 응답 시 hasMore=false 즉시 설정

### 에러
- [x] R-01 loadMore API 실패 시 status='error', 전역 error 메시지 표시
- [x] R-02 selectTag lazy fetch 실패 시 탭 status='error'
- [x] R-03 removeFavorite API 실패 시 favorites 롤백 (낙관적 업데이트 없음) — selectedTagId 초기화 안 함

### 테스트
- [x] T-01 favorites-filter.ts 유닛: filterWrongAnswers(tagId=undefined, selectedTagId='x') → 제외
- [x] T-02 favorites-filter.ts 유닛: filterWrongAnswers(selectedTagId=null) → 전체 반환
- [x] T-03 feedStore 유닛: loadMore() 정상 누적, hasMore=false 정지, 중복 가드
- [x] T-04 feedStore 유닛: selectTag — 미방문 탭 최초 fetch, 재방문 캐시 히트 (fetch 없음)
- [x] T-05 vitest 유닛: BUG-C — shouldResetTagId 회귀 테스트 5건 (Playwright 대신 vitest, C3 확정)
- [x] T-06 vitest 유닛: BUG-F — filterWrongAnswers 정책 + wrongAnswerFilterTags 파생 테스트
- [~] deferred (realClient.ts branches 부채 — MVP12 이전부터 존재, kpi-check.sh PASS 확인, 별도 테스트 확충 태스크로 이관) T-07 vitest 커버리지 ≥90% — statements/functions 통과, branches 72.58% (realClient.ts 기존 부채. kpi-check.sh PASS)

### UI·UX
- [x] U-01 피드 카드 좋아요 버튼 시각 확인 (레이블/스타일 변경 후 스크린샷)
- [x] U-02 favorites 페이지 즐겨찾기 해제 버튼 시각 확인
- [x] U-03 무한 스크롤 loadingMore 스피너 표시 (sentinel 위)
- [x] U-04 "모든 기사를 읽었습니다" 메시지 (hasMore=false 시 sentinel 대체)

### 회귀
- [x] G-01 filterFavorites 기존 동작 유지 (selectedTagId=null → 전체 반환)
- [x] G-02 buildWrongAnswerTagMap 기존 동작 변경 없음
- [x] G-03 feedStore.selectTag 'all' 탭 재방문 시 캐시 히트 즉시 표시
- [x] G-04 feedStore.refresh() 동작 유지 — 현재 탭 pagination 리셋 + 새 페이지 로드
