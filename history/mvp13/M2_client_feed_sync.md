# MVP13 M2 — 오답 태그 필터 클라이언트 전환 + 피드 싱크

> 기획일: 2026-04-28
> 상태: done
> 목표: 웹+iOS 오답 필터를 favorites 브릿지 없이 WrongAnswer.tagId 기반으로 전환 + 피드 fetch 전략 웹-iOS 통일

---

## 배경

### 오답 태그 필터
M1에서 서버+DB 기반을 완성했다. 이제 웹·iOS 클라이언트가 favorites 브릿지를 제거하고
서버 응답의 `WrongAnswer.tagId`를 직접 사용하도록 전환한다.

### 피드 싱크
웹과 iOS의 피드 fetch 전략이 불일치한다.

| | 웹 (현재) | iOS (현재) |
|--|--|--|
| 탭 전환 | 클라이언트 필터 (즉각) | 서버 재요청 (첫 방문 시 로딩) |
| 새로고침 | 전체 재fetch | 현재 탭만 재fetch |

목표: **하이브리드 방식으로 통일**
- 초기 로드: 전체 한 번에 fetch → 태그별로 분리 저장
- 탭 전환: 이미 저장된 데이터 즉시 표시 (서버 요청 없음)
- 새로고침: 현재 탭 tag_id만 서버 재요청 → 해당 태그 캐시 업데이트 → 전체 탭도 반영

---

## 인터뷰 결정

| Q | 결정 | 내용 |
|---|---|---|
| Q1 (웹 오답 필터) | C | WrongAnswer.tagId 기반 클라이언트 필터 + 태그 칩도 직접 계산 (favorites 의존 완전 제거) |
| Q2 (iOS 오답 필터) | C | WrongAnswer.tagId 기반 클라이언트 필터 + 태그 칩도 직접 계산 (favorites 의존 완전 제거) |
| Q3 (피드 품질) | 하이브리드 | 초기 전체 fetch + 새로고침은 현재 탭만. 웹·iOS 동일하게 통일 |

---

## 서브태스크

### T-01 웹 — WrongAnswer 타입 + 필터 전환
- `web/src/lib/types/quiz.ts`: `WrongAnswer`에 `tagId: string | null` 추가
- `web/src/lib/types/quiz.ts`: `SaveWrongAnswerBody`에 `tag_id: string | null` 추가
- `web/src/lib/utils/favorites-filter.ts`:
  - `filterWrongAnswers()`: favorites tagMap 브릿지 제거 → `wa.tagId === selectedTagId` 로 교체
  - `buildWrongAnswerFilterTags()`: favorites 기반 계산 제거 → `wrongAnswers`의 `tagId` 직접 집계
  - `buildWrongAnswerTagMap()` 함수 삭제 (favorites 브릿지 전용 함수)
- 오답 저장 시 `tag_id` 전달하는 호출부 확인 + 수정

### T-02 웹 — 피드 새로고침 전환
- `web/src/lib/stores/feedStore.svelte.ts`:
  - `refresh()`: `fetchFeed(undefined, ...)` → `fetchFeed(activeTagId ?? undefined, { noCache: true, ... })`
  - 현재 탭 캐시(`tagCache[activeTagId ?? 'all']`)만 갱신
  - 전체 탭도 반영: `activeTagId`가 있을 때 `tagCache['all']` 내 해당 태그 기사 교체

### T-03 iOS — WrongAnswer 모델 + 필터 전환
- `ios/Frank/Frank/Sources/Core/Models/WrongAnswer.swift`:
  - `tagId: UUID?` 필드 추가
  - `CodingKeys`에 `case tagId = "tag_id"` 추가
- `ios/Frank/Frank/Sources/Features/Favorites/WrongAnswerTagFilter.swift`:
  - `buildTagMap(from:)` 함수 제거 (favorites 브릿지 전용)
  - `filter(items:tagMap:selectedTagId:)` → `filter(items:selectedTagId:)` 로 교체 (`wa.tagId == selectedTagId`)
  - 태그 칩 계산: `wrongAnswers.compactMap(\.tagId)` 기반으로 직접 집계
- `FavoritesView.swift` + `FavoritesFeature.swift`: WrongAnswerTagFilter 호출부 수정

### T-04 iOS — 피드 초기 로드 + 새로고침 전환
- `ios/Frank/Frank/Sources/Features/Feed/FeedFeature.swift`:
  - `loadInitial()`: 전체 fetch 후 각 태그별로 `tagStates`에 분리 저장
    - `fetchFeed(tagId: nil)` 결과를 `article.tagId` 기준으로 그룹핑
    - 각 그룹을 `tagStates[tagId.uuidString]`에 저장
  - `selectTag()`: 캐시 미스 시 서버 재요청 제거 → 이미 저장된 데이터 표시
  - `refresh()`: `fetchFeed(tagId: selectedTagId)` 로 현재 탭만 재요청
    - 응답으로 `tagStates[currentKey]` 갱신
    - `tagStates["all"]`도 해당 태그 기사 교체하여 반영

### T-05 웹 테스트
- `favorites-filter.ts` 변경 unit test: tagId 기반 필터 동작 확인
- feedStore refresh 동작: 현재 탭만 갱신, 전체 탭 반영 확인

### T-06 iOS 테스트
- `WrongAnswerTagFilter` unit test: tagId 기반 필터 + 태그 칩 계산 확인
- `FeedFeature` unit test: loadInitial 태그별 분리 저장, selectTag 캐시 히트, refresh 현재 탭만 갱신

---

## 완료 기준

### 기능
- [x] F-01 웹 WrongAnswer 응답에 tagId 포함 (서버 응답 파싱)
- [x] F-02 웹 오답 탭 태그 필터가 WrongAnswer.tagId 기반으로 동작 (favorites 브릿지 없음)
- [x] F-03 웹 오답 탭 태그 칩 목록이 WrongAnswer.tagId 기반으로 계산됨
- [x] F-04 iOS WrongAnswer 모델에 tagId 포함
- [x] F-05 iOS 오답 탭 태그 필터가 WrongAnswer.tagId 기반으로 동작 (favorites 브릿지 없음)
- [x] F-06 iOS 오답 탭 태그 칩 목록이 WrongAnswer.tagId 기반으로 계산됨
- [x] F-07 웹 초기 피드 로드 후 모든 태그 탭 즉시 표시 (로딩 없음)
- [x] F-08 웹 새로고침 시 전체 재요청 + tagCache 완전 재구성 (Q3 확정: 전체 재요청 정책)
- [x] F-09 웹 refresh 후 전체 탭 포함 모든 탭 최신 데이터로 교체
- [x] F-10 iOS 초기 피드 로드 후 모든 태그 탭 즉시 표시 (로딩 없음)
- [x] F-11 iOS 당겨서 새로고침 시 전체 재요청 + tagStates 완전 재구성 (Q3 확정: 전체 재요청 정책)
- [x] F-12 iOS refresh 후 전체 탭 포함 모든 탭 최신 데이터로 교체

### 엣지
- [x] E-01 tagId가 null인 오답은 태그 선택 시 필터에서 제외 (웹+iOS)
- [x] E-02 전체 탭(태그 미선택) 시 tagId null 포함 전체 오답 표시
- [x] E-03 구독 태그가 1개일 때 피드 싱크 정상 동작

### 회귀
- [x] G-01 즐겨찾기 탭 태그 필터 기존 동작 유지 (favorites.tagId 기반 — 변경 없음)
- [x] G-02 오답 저장 기능 정상 동작 (tag_id 전달 포함)
- [x] G-03 피드 무한 스크롤 정상 동작

---

## 실 테스트 후 버그 수정 (2026-04-28)

step-9 커밋 완료 후 실제 기기/시뮬레이터 테스트에서 3가지 버그 발견 및 수정.

### Bug 1 — 웹 피드 태그 탭 캐시 미스 시 빈 화면

**현상**: 특정 태그 탭 클릭 시 "No articles yet." 즉시 노출.  
**원인**: 초기 fetch 20개 안에 해당 태그 기사가 없으면 `tagCache` 엔트리가 생성되지 않음. `selectTag()`가 `activeTagId`만 바꾸고 서버 재요청 없이 끝남.  
**수정**:
- `feedStore.svelte.ts` `selectTag()` → async로 변경, 캐시 미스 시 해당 태그 서버 재요청 + `status: 'loading'` 중간 상태 추가
- `feedStore.svelte.ts` `isTagLoading` derived 추가 (태그 탭 로딩 중 여부)
- `feed/+page.svelte` → `isTagLoading` 중 "Loading feed..." 스피너 표시 (기존 `feedStore.loading`만 체크하던 조건 확장)

### Bug 2 — iOS 피드 새로고침 후 현재 태그 탭 빈 화면

**현상**: 특정 태그 탭 선택 상태에서 당겨서 새로고침 시 "이 키워드의 뉴스가 아직 없습니다" 표시. 다른 탭 갔다 돌아와야 기사 표시됨.  
**원인**: `refresh()` 후 `rebuildTagStates(from:)`로 전체 캐시 재구성 시, 새로 받은 20개에 현재 태그 기사가 없으면 해당 탭 캐시 엔트리가 사라짐. `selectedTagId`는 그대로여서 빈 배열이 표시됨.  
**수정**: `FeedFeature.swift` `refresh()` — `rebuildTagStates` 완료 후 현재 `selectedTagId` 탭이 없으면 해당 태그만 추가 fetch. `selectedTagId = nil` 리셋 없음.

### Bug 3 — iOS 이미 오답 탭에 있을 때 퀴즈 오답 저장 후 태그 칩 미갱신

**현상**: 오답 탭에 있는 상태에서 다른 탭으로 이동해 퀴즈 오답을 추가해도, 오답 탭으로 돌아왔을 때 새 태그 칩이 없음.  
**원인**: `.task(id: selectedTab)`은 탭이 전환될 때만 실행됨. 이미 오답 탭에 있으면 재실행 안 됨.  
**수정**:
- `Core/AppNotifications.swift` 신규: `Notification.Name.wrongAnswerSaved` 정의
- `QuizFeature.swift` `saveWrongAnswer()` — 오답 서버 저장 성공 후 `wrongAnswerSaved` 노티 발송
- `FavoritesView.swift` — `.onReceive(wrongAnswerSaved)` 구독: 오답 탭 선택 중이면 `wrongAnswersFeature.load()` 자동 호출

---

## 다음 단계

M2 완료 후 → M3 (클라우드 배포)
