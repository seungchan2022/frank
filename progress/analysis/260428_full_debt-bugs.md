# 심층 분석: 이관 부채(DEBT-01, DEBT-02) + 미해결 버그(BUG-001~004)

> 날짜: 260428  
> 분석 유형: full (코드베이스 직접 탐색)  
> 대상: DEBT-01, DEBT-02, BUG-001, BUG-002, BUG-003, BUG-004

---

## 1. 분석 목적

1. 각 항목의 실제 해결 상태 확인 (이미 수정됐는지 vs 아직 미해결인지)
2. DEBT 목록과 BUG 목록 간 중복 여부 파악
3. 버그들 간 상호 관련성·중복 파악

---

## 2. 실제 해결 상태 요약

| 항목 | bugs.md 상태 | 실제 코드 상태 | 비고 |
|---|---|---|---|
| BUG-001 | open | **해결됨** | MVP12 이전 수정 |
| BUG-002 | open | **해결됨** | MVP12 이전 수정 |
| BUG-003 | open | **해결됨** (부분) | BUG-F로 흡수, 클라이언트 필터 B안 |
| BUG-004 | open | **해결됨** | MVP12 M1 수정 |
| DEBT-01 | 이관 중 | **의도적 보류** | B안 선택, A안은 다음 MVP 검토 |
| DEBT-02 | 이관 중 | **C안 적용 완료** | 재결정은 사용 패턴 데이터 확보 후 |

---

## 3. BUG 항목별 상세

### BUG-001: iOS 첫 실행 시 세션 미복원 → API 401

**분류**: 해결됨 (수정 위치: `AppDependencies.swift`)

**수정 내용**:
```swift
// ios/Frank/Frank/Sources/App/AppDependencies.swift line 79
SupabaseClientOptions(
    auth: .init(emitLocalSessionAsInitialSession: true)
)
```

`emitLocalSessionAsInitialSession: true` 설정으로 앱 시작 시 로컬 저장 세션을 즉시 방출한다. `FrankApp.swift`에서 `AuthState.checkingSession` 상태일 때 `SplashView`를 표시해 세션 확인 완료 전까지 API 호출 진입을 차단하는 구조도 완비됐다.

**확인 파일**:
- `ios/Frank/Frank/Sources/App/AppDependencies.swift`
- `ios/Frank/Frank/Sources/App/FrankApp.swift`
- `ios/Frank/Frank/Sources/Core/Adapters/SupabaseAuthAdapter.swift`

---

### BUG-002: 시뮬레이터/실기기 SERVER_URL 분기 없음

**분류**: 해결됨 (수정 위치: `ServerConfig.swift`)

**수정 내용**:
```swift
// ios/Frank/Frank/Sources/Core/Config/ServerConfig.swift
#if targetEnvironment(simulator)
    return "http://localhost:8080"
#else
    // Info.plist → Secrets.plist 순서로 탐색
#endif
```

컴파일 타임 분기로 시뮬레이터는 항상 `localhost:8080`을 사용. `Config.xcconfig`도 `SERVER_URL = http://localhost:8080`으로 기본 설정됨.

**확인 파일**:
- `ios/Frank/Frank/Sources/Core/Config/ServerConfig.swift`
- `ios/Frank/Frank/Config.xcconfig`

---

### BUG-003: 즐겨찾기/오답 화면 태그 필터 없음

**분류**: 해결됨 (수정 위치: 웹 + iOS, MVP12 M2/M3)

BUG-003은 MVP12에서 **BUG-F**로 재분류·흡수됐다. 웹과 iOS 양쪽 모두 클라이언트 필터(B안)로 수정됐다.

**웹 수정 내용**:
- `web/src/lib/utils/favorites-filter.ts`: `filterWrongAnswers()` BUG-F 정책 수정, `undefined` 제외 조건 추가
- `web/src/routes/favorites/+page.svelte`: `filteredFavorites`, `filteredWrongAnswers`, `wrongAnswerFilterTags` `$derived` 변수 모두 구현

**iOS 수정 내용**:
- `WrongAnswerTagFilter.swift`: `buildTagMap(from:)` + `apply(items:tagMap:selectedTagId:)` 순수 함수
- `FavoritesView.swift`: `wrongAnswerTagMap`, `wrongAnswerTags`, `filteredWrongAnswers` computed 프로퍼티 전부 구현

**한계 (DEBT-01과 연결)**:
- 오답에 직접 `tag_id` 없음 → `favorites.tag_id`를 브릿지로 사용하는 간접 필터
- favorites에 등록되지 않은 오답은 태그 선택 시 제외됨 (의도된 정책)
- 서버 `wrong_answers.tag_id` 컬럼 추가(A안)는 DEBT-01로 보류

**확인 파일**:
- `web/src/lib/utils/favorites-filter.ts`
- `web/src/routes/favorites/+page.svelte`
- `ios/Frank/Frank/Sources/Features/Favorites/WrongAnswerTagFilter.swift`
- `ios/Frank/Frank/Sources/Features/Favorites/FavoritesView.swift`

---

### BUG-004: 뉴스 카테고리/인덱스 페이지가 피드에 수집됨

**분류**: 해결됨 (수정 위치: 서버, MVP12 M1)

**수정 내용**:
1. **`is_listing_url()`** 3규칙 구현 (`server/src/api/feed.rs`):
   - Rule 1: 마지막 URL 세그먼트에 listing 키워드 포함
   - Rule 2: 페이지 번호 + 선행 listing 경로
   - Rule 3: BBC topics 뒤 해시 세그먼트
2. **Tavily**: `"time_range": "week"`, `"topic": "news"` 파라미터 추가 (`server/src/infra/tavily.rs`)
3. **Exa**: `"category": "news"`, `"startPublishedDate"` 파라미터 추가 (`server/src/infra/exa.rs`)
4. Firecrawl 포함 모든 검색 결과는 `is_listing_url()` 통합 후처리로 커버

**확인 파일**:
- `server/src/api/feed.rs`
- `server/src/infra/tavily.rs`
- `server/src/infra/exa.rs`

---

## 4. DEBT 항목별 상세

### DEBT-01: `wrong_answers.tag_id` 서버 컬럼 추가 (A안 보류)

**분류**: 의도적 보류 (B안 선택으로 현재 기능 동작 중)

**선택 경과**:
- `history/mvp12/analysis_BUG-F-tag-filter.md`에서 A/B안 비교 분석 수행
- **A안** (서버 `tag_id` 컬럼 추가): 4레이어(DB 마이그레이션, Rust API, Svelte store, iOS adapter) 변경 필요, denormalization 부채 발생
- **B안** (클라이언트 필터, 채택): 서버 변경 없음, `favorites.tag_id`를 브릿지로 사용

**현재 동작 방식**:
```
quiz_wrong_answers (tag_id 없음)
         ↓ articleUrl로 조인
favorites (tag_id 있음)
         ↓
클라이언트 필터 (tagMap 기반)
```

**재검토 트리거**:
- 오답 페이지네이션 기능 추가 시 (서버에서 필터링 필요해짐)
- 오답이 favorites에 없는 케이스 증가 시 (필터 정확도 저하)
- 오답 전용 태그 배정 기능 요구 시

---

### DEBT-02: iOS 피드 Lazy vs Prefetch 전략 재결정

**분류**: C안 적용 완료, 재결정 보류

**선택 경과**:
- `history/mvp12/debate_M2_feed_prefetch_strategy.md`에서 3자 토론 (Claude/Codex/Serena 만장일치 C안)
- **A안**: 모든 탭 on-demand (초기 API 1회, 탭 전환마다 로딩)
- **B안**: 구독 태그 전체 prefetch (탭 전환 즉시, 초기 N+1 API)
- **C안** (채택): 전체 탭 첫 페이지만 즉시, 나머지 lazy

**현재 구현** (`ios/.../FeedFeature.swift`):
```swift
// loadInitial: "all" 탭만 첫 페이지
let items = try await article.fetchFeed(tagId: nil, noCache: false, limit: PAGE_SIZE, offset: 0)
tagStates["all"] = .firstPage(items: items, pageSize: PAGE_SIZE)

// selectTag: 캐시 히트 즉시 표시, 미스 시 lazy fetch
if tagStates[key] != nil {
    selectedTagId = tagId
    return
}
```

**재검토 트리거**:
- 사용자 태그 탭 전환 패턴 데이터 수집 후
- 구독 태그 수 증가로 B안 fan-out 비용이 실측으로 허용 가능한 경우

---

## 5. 중복·관계 분석

### 5-1. BUG-003 ≡ BUG-F(MVP12) ≡ DEBT-01 관계

세 가지는 동일한 이슈의 표현 계층이 다를 뿐이다:

```
bugs.md BUG-003 (증상 레벨)
    = mvp12 BUG-F (버그 분류 레벨)
    ⊂ DEBT-01 (아키텍처 결정 레벨 — 완전 해결 여부)
```

BUG-003/BUG-F는 "필터 기능이 없음" 문제로 클라이언트 필터(B안)로 해결됐다.  
DEBT-01은 "B안이 근본 해결인지, A안으로 가야 하는지"라는 아키텍처 질문이다.

### 5-2. progress/bugs/BUG-004.md ≠ bugs.md의 BUG-004

파일명만 같고 내용은 완전히 다른 버그:

| | bugs.md BUG-004 | progress/bugs/BUG-004.md |
|---|---|---|
| 내용 | 서버 listing URL 혼입 | 즐겨찾기 삭제 후 고아 selectedTagId 빈 화면 |
| 해결 | MVP12 M1 (`is_listing_url()`) | MVP12 BUG-C(웹)/BUG-E(iOS)로 흡수 |

파일명 충돌로 혼동 가능성 있음 → 파일명 변경 또는 bugs.md 항목 번호 재정리 권장.

### 5-3. BUG-001 + BUG-002 독립성

두 버그는 모두 "앱 첫 실행 실패" 증상으로 묶이지만 원인은 독립적이다:
- BUG-001: Supabase 세션 복원 타이밍 (SDK 옵션 설정)
- BUG-002: API 엔드포인트 URL 분기 (컴파일 타임 조건)

같이 발견됐지만 각각 별개로 수정해야 한다.

---

## 6. 개선안

### 안 A: bugs.md 즉시 정리

모든 BUG 항목을 `resolved` 상태로 업데이트한다. DEBT-01은 다음 MVP 기획 시 포함 여부 명시.

- 장점: bugs.md가 현재 코드 상태와 일치
- 단점: 히스토리 손실 가능성

### 안 B: 진단 파일 충돌 해소 + 번호 재정리

`progress/bugs/BUG-004.md`를 `BUG-004_orphan-selectedTagId.md`로 파일명 변경. bugs.md 번호 체계 재검토.

- 장점: 파일명으로 내용 구분 가능
- 단점: 기존 레퍼런스 링크 깨짐

### 안 C: DEBT-01을 다음 MVP 기획에 명시적 포함

다음 MVP 로드맵에서 "DEBT-01 재검토" 마일스톤을 명시적으로 포함하거나 보류 이유를 기록한다. 오답 페이지네이션 기능과 묶어 한 번에 처리하는 방안이 효율적이다.

- 장점: 부채가 다시 묻히지 않음
- 단점: MVP 범위 확장 가능성

---

## 7. 권장 액션

| 우선순위 | 액션 | 난이도 | 효과 |
|---|---|---|---|
| 1 | bugs.md 항목 resolved 표시 업데이트 | 낮음 | 현재 상태 정확성 |
| 2 | `progress/bugs/BUG-004.md` 파일명 변경 | 낮음 | 혼동 방지 |
| 3 | DEBT-01을 다음 MVP 기획에 명시적 포함 | 중간 | 부채 추적 |
| 4 | DEBT-02 사용 패턴 데이터 수집 (분석 로그 추가) | 중간 | 재결정 근거 확보 |

---

## 8. 결론

**bugs.md는 현재 코드 현실과 불일치한다.** BUG-001~004 모두 코드에서 이미 수정됐으나 bugs.md에는 여전히 open으로 남아있다. DEBT-01/02는 의도적 보류이며 각각의 재검토 트리거가 명확히 정의됐다. 즉각적인 개발 액션이 필요한 항목은 없으며, 문서 정리(bugs.md 업데이트, 파일명 충돌 해소)가 가장 시급한 작업이다.
