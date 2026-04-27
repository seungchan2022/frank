# MVP12 회고

> 날짜: 2026-04-27
> 범위: M1(서버) + M2(웹) + M3(iOS)
> 브랜치: feature/260427_mvp12-m3-ios-bug-ux

---

## 무엇을 했나

| 마일스톤 | 내용 |
|---------|------|
| M1 서버 | BUG-A(BBC 토픽 URL 필터), BUG-B(snippet 마크다운 오염), 피드 limit/offset 페이지네이션 API |
| M2 웹 | BUG-C(마지막 즐겨찾기 삭제 후 탭 먹통), BUG-F(웹 오답 태그 필터), 좋아요/즐겨찾기 UX 재설계, 무한 스크롤 |
| M3 iOS | BUG-D(즐겨찾기 추가 후 태그 미생성), BUG-E(마지막 기사 삭제 후 탭 초기화), BUG-F(iOS 오답 필터 정책 웹 싱크), 좋아요/즐겨찾기 UX 재설계, 무한 스크롤 |

---

## KPI 결과

| 지표 | 목표 | 결과 | 비고 |
|------|------|------|------|
| 버그 6건 재현 0 | 0건 | BUG-A/B/C/D/E ✅, BUG-F ⏭️ | BUG-F 오답 태그 칩 근본 수정은 서버 스키마 변경 필요 — 다음 MVP |
| 피드 무한 스크롤 | 정상 동작 | ✅ 웹+iOS | offset 페이지네이션, sentinel 패턴 |
| MVP 회고 | history/mvp12/retro.md | ✅ | 이 파일 |

---

## 의사결정 기록

### 1. BUG-F iOS — 웹과 싱크 (오답 태그 칩 근본 수정 defer)

**결정**: 웹 현재 상태(즐겨찾기 미등록 오답 → 필터 시 제외)와 동일하게 iOS 정책 맞춤. 태그 칩 자체가 비는 근본 문제(즐겨찾기 해제 후 tagMap 매핑 소실)는 서버 스키마 변경 없이 불가 → 다음 MVP로 defer.

**이유**: MVP12 범위 내에서 가능한 수정은 필터 정책 통일까지. 태그 칩 소스 문제는 `wrong_answers` 테이블에 `tag_id` 컬럼 추가 or 조인 구조 변경이 필요하며, 4개 레이어(DB 스키마 + 서버 API + 웹 + iOS) 동시 변경 규모라 다음 MVP 단독 기획이 적절.

### 2. iOS 피드 태그 탭 Lazy Loading 유지

**결정**: 탭 첫 방문 시 fetch (Lazy), 재방문은 TagState 캐시 히트. 프리패치(Prefetch) 방식은 무한 스크롤과 조합 시 복잡도가 급증해 현재 유지.

**이유**: 무한 스크롤 도입으로 각 탭이 독립 페이지네이션 상태(TagState)를 가짐. 프리패치 시 "어디까지 프리패치할지", "캐시 무효화 타이밍" 등 추가 판단이 필요. 사용 패턴 데이터 확보 후 다음 MVP에서 재결정.

**트레이드오프**:
- 프리패치: 탭 전환 즉시 표시(쾌적) vs 초기 네트워크 비용 + 구현 복잡도
- Lazy: 초기 빠름 + 단순 vs 첫 탭 방문 시 공백 딜레이

### 3. FeedFeature TagState read-modify-write 패턴

**결정**: `@Observable` + `Dictionary<String, TagState>` 조합에서 `tagStates[key].items.append()`가 반응성을 트리거하지 않음. 값 타입 struct를 꺼내서 수정 후 재할당하는 read-modify-write 패턴 필수 적용.

**이유**: Swift의 `@Observable`은 프로퍼티 할당(`=`)을 감지. Dictionary 내부 struct 필드 직접 변이는 감지 못함. 이 패턴은 FeedFeature `loadMore`, `loadInitial`, `selectTag` 전체에 일관 적용.

### 4. recomputeTags — fetchAllTags 재호출 없이 allTagsCache 재사용

**결정**: 즐겨찾기 추가/삭제 시 서버 태그 목록을 다시 fetch하지 않고, 최초 로드 시 캐싱한 `allTagsCache`와 현재 `items`의 tagId Set 교집합으로 `tags`를 재계산.

**이유**: 태그 목록 자체는 즐겨찾기 조작으로 변하지 않음. 불필요한 API 호출 제거, 오프라인/에러 상황에서도 태그 칩 갱신 가능.

---

## 부채 (다음 MVP 이관)

| ID | 내용 | 이유 |
|----|------|------|
| DEBT-01 | BUG-F 오답 태그 칩 근본 수정 | 서버 스키마 변경 필요 (wrong_answers.tag_id) |
| DEBT-02 | iOS 피드 Lazy vs Prefetch 전략 재검토 | 사용 패턴 데이터 확보 후 결정 |
| ~~DEBT-03~~ | ~~M3 iOS 유닛 테스트 (T-01~T-09)~~ | ✅ 260427 해소 — T-01~T-08 유닛 작성 완료, T-09 U-01 E2E 대체 |
