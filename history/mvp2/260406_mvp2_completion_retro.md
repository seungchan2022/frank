# MVP2 완료 회고 — iOS 네이티브 앱 하루 완주

> 작성일: 2026-04-06
> 기간: 2026-04-05 ~ 2026-04-06 (2일, 기획 반나절 + 구현 1일)
> 상태: MVP2 완료
> 선행: MVP1.5 완료 회고 (`history/mvp15/260405_mvp15_completion_retro.md`)

---

## 숫자로 보는 MVP2

| 항목 | 수치 |
|------|------|
| 총 개발 기간 | 2일 (기획+셋업 반나절 + 구현 1일) |
| 총 커밋 | 28개 |
| iOS 소스 코드 | 2,433줄 (37파일) |
| iOS 테스트 코드 | 2,268줄 (16파일) |
| 테스트 | 117개 (전체 통과) |
| 마일스톤 | 6개 완료 (M1~M6) |
| 포트(Protocol) | 4개 (Auth, Tag, Article, Collect) |
| Mock 어댑터 | 4개 (전 포트 테스트 커버) |
| 화면 수 | 7개 (Splash, Login, EmailSignIn, Onboarding, Feed, Detail, Settings) |
| 컴포넌트 | 5개 (ArticleCard, TagChip, TagChipBar, EmptyState, Shimmer) |
| 외부 SDK | 1개 (Supabase Swift) |
| 커밋 태그 분포 | docs 8, feat 7, chore 6, refactor 3, test 2, fix 2 |

### 프로젝트 전체 규모 (MVP1+1.5+2)

| 영역 | 코드 | 테스트 |
|------|------|--------|
| 서버 (Rust) | 5,327줄 | 139개 |
| 웹 (Svelte) | 2,239줄 | 77개 |
| iOS (Swift) | 2,433줄 | 117개 |
| **합계** | **9,999줄** | **333개** |

---

## 무엇을 만들었나

**Frank iOS** — MVP1 웹앱의 네이티브 iOS 포팅. 동일 백엔드(Rust 서버 + Supabase)를 공유.

### 아키텍처: Feature-Driven Unidirectional Flow

```
View(탭) → 클로저 → Feature.send(Action) → Reducer → 프로퍼티 갱신 → 서브뷰 리렌더
                         ↓
                    Port(Protocol) → Adapter (URLSession/Supabase SDK)
                         ↓
                    도메인 모델 → Component State
```

### 완성된 기능

| 기능 | 상세 |
|------|------|
| 인증 | Apple Sign-In + 이메일 회원가입/로그인, 세션 복원, Splash |
| 온보딩 | 12개 태그 FlowLayout 선택, 최소 1개 강제, user_metadata 저장 |
| 피드 | 키워드 상단 탭(SmartNews 스타일), 기사 카드, 수집/요약 트리거, 무한 스크롤 |
| 기사 상세 | AI 요약 카드, 인사이트, 원문 스니펫, Safari 원문 링크 |
| 설정 | 태그 관리(추가/해제), 로그아웃(확인 Alert), 피드 동기화 |

### 마일스톤별 진행

| M# | 마일스톤 | 핵심 산출물 | 발견된 이슈 |
|----|---------|-----------|------------|
| M1 | 프로젝트 셋업 | Tuist + Port/Adapter + DI 컨테이너 + 19 테스트 | Tuist MCP tools 미지원 |
| M2 | 인증 플로우 | Apple/Email 로그인 + 세션 복원 + 상태 머신 | signUp nil session 계약 위험, checkingSession 상태 추가 |
| M3 | 온보딩 | 태그 선택 + FlowLayout + 100% 커버리지 | tag data loss 버그, slug→category 스키마 불일치 |
| M4 | 피드 | 탭 필터 + 카드 리스트 + 수집/요약 + 페이지네이션 | Per-tag 캐시 설계, 클라이언트 필터링 필요 |
| M5 | 기사 상세 | DetailFeature + NavigationStack 연결 | 없음 |
| M6 | 설정 | SettingsFeature + TagManagement + 피드 동기화 | sheet 타이밍, "전체" 탭 필터 버그, onDismiss 순서 |

---

## 심층 분석

### 1. 아키텍처 스코어카드

| 카테고리 | 점수 | 근거 |
|---------|------|------|
| 의존 방향 | 10/10 | View→Feature→Port←Adapter 완벽 준수. 역방향 의존 0건 |
| 포트/어댑터 | 10/10 | 4개 포트 전부 프로덕션 어댑터 + Mock 완비 |
| 코드 품질 | 9/10 | force unwrap 1건(justified literal), force cast 0, Any 0, TODO 0 |
| 테스트 | 8.5/10 | 117개 테스트, PortContract 21개, Feature별 테스트 완비 |
| 컴포넌트 재사용 | 9/10 | TagChipView 3곳, ArticleCardView 2곳 재사용 |
| 에러 처리 | 8.5/10 | 일관된 errorMessage 패턴, 한국어 메시지, 에러 삼킴 없음 |
| 상태 관리 | 8/10 | @Observable 프로퍼티 단위 추적, FeedFeature 복잡도 주의 |
| 네비게이션 | 8.5/10 | NavigationStack + 상태 기반, Router 미활용 |
| **종합** | **8.9/10** | **프로덕션 레디 iOS 앱** |

### 2. 포트/어댑터 패턴 — iOS에서의 검증

MVP1.5 회고에서 "iOS에서 과한 추상화인지는 실제 어댑터 구현 후 평가"라고 보류했던 판단:

**결론: 과하지 않았다. 정확히 맞았다.**

근거:
- Mock 기반 117개 테스트가 0.86초에 완료 — 네트워크 없이 전체 비즈니스 로직 검증
- PortContractTests 21개로 Mock/Production 동작 일관성 보장
- Feature 개발 시 Adapter 미구현 상태에서도 TDD 진행 가능 (M1에서 Port만으로 19개 테스트)
- 4개 Adapter 각각 80줄 내외 — 추상화 오버헤드 미미

**웹과의 비교:**
```
서버 (Rust)  : 5개 trait, Fake 5개 → 139 테스트 0.01초
웹 (Svelte)  : vi.mock 기반     → 77 테스트 0.05초
iOS (Swift)  : 4개 Protocol, Mock 4개 → 117 테스트 0.86초
```

3개 플랫폼 모두 동일 패턴으로 TDD가 가능해졌다. 컨텍스트 스위칭 비용이 거의 없다.

### 3. FeedFeature — 가장 복잡한 Feature의 해부

252줄로 전체 Feature 중 최대. 상태 변수 분석:

```swift
// 8개 독립 프로퍼티 + 3개 캐시 딕셔너리
var articles: [Article]           // 현재 표시 기사
var selectedTagId: UUID?          // 선택된 탭
var isLoading: Bool               // 초기 로딩
var isLoadingMore: Bool           // 페이지네이션
var isCollecting: Bool            // 수집 중
var isSummarizing: Bool           // 요약 중
var hasMore: Bool                 // 추가 페이지 존재
var errorMessage: String?         // 에러 배너

private var cache: [UUID?: [Article]]     // 탭별 캐시
private var offsets: [UUID?: Int]         // 탭별 오프셋
private var hasMoreMap: [UUID?: Bool]     // 탭별 추가 페이지
```

**문제**: 4개 Bool 플래그(isLoading, isLoadingMore, isCollecting, isSummarizing)가 동시에 true일 수 없는 상호 배타적 상태. 현재는 helper 함수(`beginLoading`, `beginCollect` 등)로 관리하지만, 상태 전이가 암시적.

**개선 제안 (MVP2.5)**:
```swift
enum LoadingPhase {
    case idle
    case loading
    case loadingMore
    case collecting
    case summarizing
}
```

### 4. 태그 로딩 중복 패턴

OnboardingFeature와 SettingsFeature에서 거의 동일한 태그 로딩 로직:

```swift
// OnboardingFeature (lines 54-65)
async let allTags = tag.fetchAllTags()
async let myIds = tag.fetchMyTagIds()
let (tags, ids) = try await (allTags, myIds)

// SettingsFeature (lines 50-64) — 거의 동일
async let allTags = tag.fetchAllTags()
async let myIds = tag.fetchMyTagIds()
let (tags, ids) = try await (allTags, myIds)
```

에러 처리 방식만 약간 다름. 공통 헬퍼 또는 Protocol extension으로 추출 가능.

### 5. 테스트 분포 분석

| 테스트 영역 | 수 | 비율 | 평가 |
|------------|-----|------|------|
| FeedFeatureTests | 18 | 15% | 적절 (가장 복잡) |
| PortContractTests | 21 | 18% | 우수 (계약 검증) |
| OnboardingFeatureTests | 13 | 11% | 적절 |
| SettingsFeatureTests | 14 | 12% | 적절 |
| AuthFeatureTests | 14 | 12% | 적절 |
| DetailFeatureTests | 7 | 6% | 적절 (단순 Feature) |
| Component 테스트 | 20 | 17% | 적절 |
| Router + DI 테스트 | 6 | 5% | 최소 (구조 코드) |
| UI 테스트 | 4 | 3% | **부족** |

**관찰**: Feature/Port 테스트는 탄탄하지만, UI 테스트(AuthFlowUITest 4개)가 전체의 3%로 적다. E2E 시나리오 검증 부족.

### 6. MVP1→MVP2 공유 백엔드 검증

iOS가 동일 Rust 서버 + Supabase를 호출하면서 확인된 것:

| 검증 항목 | 결과 |
|----------|------|
| API 호환성 | ✅ 웹과 동일 엔드포인트 그대로 사용 |
| 인증 토큰 | ✅ Supabase JWT Bearer 동일 |
| RLS 정책 | ✅ user_id 기반 행 격리 iOS에서도 동작 |
| 수집/요약 트리거 | ✅ POST /api/me/collect, /api/me/summarize 동일 |
| 스키마 불일치 | ⚠️ Tag.slug → Tag.category 변경 필요했음 (M3에서 발견) |

**교훈**: API 스키마 문서화가 없어서 iOS 개발 중 DB 컬럼명을 직접 확인해야 했다. OpenAPI 스펙이 있었다면 방지 가능.

---

## 잘한 것 (Keep)

### 1. 하루 만에 6개 마일스톤 완주

M1~M6을 하루(+반나절 기획)에 완료했다. MVP1(웹)도 하루 구현이었으니, **동일한 비즈니스 로직을 새 플랫폼(iOS)으로 포팅하는 데 동일한 시간이 걸렸다.** 포트/어댑터 패턴 덕분에 "무엇을 만들어야 하는지"가 명확했고, Feature별 독립 구현이 가능했다.

### 2. 에코 서버 패턴의 3개 플랫폼 검증

서버(Rust trait), 웹(vi.mock), iOS(Protocol Mock) — 세 플랫폼 모두 동일한 추상화 원칙으로 동작한다. 특히 iOS에서 "과한 추상화 아닌가?"라는 우려를 완전히 해소했다. **패턴이 동일하니 컨텍스트 스위칭 비용이 거의 0.**

### 3. @Observable 프로퍼티 단위 렌더링 규칙 확립

SwiftUI의 @Observable이 프로퍼티별 추적을 하지만, body 단위로 적용되는 함정을 사전에 인식하고 규칙을 세웠다:
- Feature 프로퍼티를 서브뷰별로 분리
- 부모 View에서 직접 읽지 않고 Feature 객체를 전달
- 서브뷰 body에서 필요한 프로퍼티만 읽음

이 규칙이 FeedView의 성능을 보장한다 — tagBar 변경 시 cards는 리렌더되지 않는다.

### 4. PortContractTests — 계약 테스트의 가치

21개 계약 테스트가 Mock과 Production Adapter의 행동 일관성을 검증한다. Mock이 "실제와 다르게 동작"하는 문제를 사전에 차단. MVP1에서는 없었던 새로운 테스트 패턴.

### 5. Hook 강제 체계의 실전 효과

260405 일일 회고에서 "hook이 main 커밋을 차단했고, 그게 가장 만족스러운 순간이었다"라고 기록했다. MVP2 28개 커밋 중 main 직접 커밋 0건, 테스트 미통과 커밋 0건. **문서 강제 → 엔진 강제 전환이 완벽히 동작.**

---

## 아쉬운 것 (Problem)

### 1. FeedFeature 상태 복잡도 — Bool 4개의 함정

`isLoading`, `isLoadingMore`, `isCollecting`, `isSummarizing` 4개 Bool이 상호 배타적인데 독립 프로퍼티로 관리된다. 상태 전이가 helper 함수(`beginLoading`, `beginCollect`)에 숨어 있어 새로운 상태 추가 시 누락 위험이 있다.

OnboardingFeature는 enum(`loading/loaded`)으로 관리하여 더 안전한데, FeedFeature는 복잡도 때문에 Bool로 풀었다. **MVP2.5에서 LoadingPhase enum으로 통합 필요.**

### 2. 태그 로딩 중복

OnboardingFeature와 SettingsFeature가 거의 동일한 `fetchAllTags + fetchMyTagIds` 로직을 가진다. 현재는 2곳이라 관리 가능하지만, 태그 관련 Feature가 추가되면 문제.

### 3. UI 테스트 부족 (3%)

117개 테스트 중 UI 테스트는 4개(AuthFlowUITest)뿐. Feature/Port 로직은 탄탄하지만, 실제 화면 전환과 사용자 인터랙션 검증이 부족하다. 특히 피드→상세→뒤로, 설정→태그관리→저장→피드 동기화 같은 E2E 시나리오가 미검증.

### 4. iOS 커버리지 정량 미측정

CLAUDE.md에 "90% 이상" 규칙을 명시했지만, Swift Testing + Tuist 환경에서 커버리지 수치를 자동 측정하는 파이프라인이 없다. 테스트 수(117개)와 소스 대비 테스트 비율(2,268줄/2,433줄 ≈ 93%)로 간접 추정만 가능.

### 5. API 스키마 문서화 부재

Tag.slug → Tag.category 변경처럼 DB 스키마와 iOS 모델 간 불일치를 런타임에 발견했다. OpenAPI 스펙이나 공유 스키마 정의가 있었다면 컴파일 타임에 잡을 수 있었다.

---

## 놀랐던 것 (Surprise)

### 1. 테스트 코드가 소스 코드의 93%

iOS 소스 2,433줄에 테스트 2,268줄. 거의 1:1 비율이다. MVP1 서버(3,365줄 소스에 테스트 미측정)와 비교하면, TDD 원칙이 훨씬 잘 지켜졌다. **Hook 강제 + 명시적 TDD 워크플로우의 결과.**

### 2. 3개 플랫폼 합계 코드가 정확히 9,999줄

서버 5,327 + 웹 2,239 + iOS 2,433 = 9,999줄. 우연이지만, 만 줄 미만으로 6개 외부 API 통합 + 3개 플랫폼 앱을 완성한 것이 인상적. 포트/어댑터 패턴의 코드 효율성.

### 3. M6에서 가장 많은 실전 버그 발견 (3건)

M2~M5는 TDD로 진행하면서 실전 버그가 적었지만, M6(설정)에서 sheet 타이밍, "전체" 탭 필터, onDismiss 순서 문제 3건이 터졌다. **설정은 다른 Feature(FeedFeature)의 상태를 간접적으로 변경하는 Feature여서, 단위 테스트만으로는 타이밍 버그를 잡기 어렵다.** 크로스-Feature 통합 테스트의 필요성.

### 4. MVP1 회고의 "방향 감각" 인사이트가 재확인

MVP1 회고에서 "AI가 코드를 짜더라도 방향은 사람이 잡아야 한다"고 적었다. MVP2에서도 동일했다 — Feature-Driven Unidirectional Flow 패턴 선택, @Observable 프로퍼티 세분화 규칙, PortContractTests 도입 등 모든 핵심 결정은 내가 내렸다. **바이브 코딩의 본질은 "패턴을 알고 방향을 잡는 것".**

---

## MVP1 → MVP1.5 → MVP2 성장 비교

| 관점 | MVP1 | MVP1.5 | MVP2 |
|------|------|--------|------|
| 범위 | 웹 풀스택 | 기술 부채 해소 | iOS 네이티브 |
| 코드 | 5,426줄 | 7,566줄 | 9,999줄 |
| 테스트 | 135개 | 216개 | 333개 |
| 포트 패턴 | 5 trait (Rust) | 의존 위반 0건 복원 | 4 Protocol (Swift) 추가 |
| 아키텍처 | 포트/어댑터 도입 | 역방향 의존 해소 | 3개 플랫폼 패턴 통일 |
| 규칙 강제 | 문서만 (위반 60%) | Hook 도입 | 위반 0% 검증 |
| 핵심 교훈 | "방향 감각" | "안전장치 먼저" | "패턴이 동일하면 플랫폼은 장벽이 아니다" |

---

## MVP2.5를 위한 기술 부채 정리

MVP1 → MVP1.5 패턴처럼, MVP3 진행 전 해소해야 할 항목:

### 즉시 (MVP3 시작 전 — MVP2.5)

| # | 이슈 | 영향 | 예상 작업량 | 우선순위 |
|---|------|------|-----------|---------|
| 1 | FeedFeature Bool 4개 → LoadingPhase enum 통합 | 상태 전이 안전성 | 소 | High |
| 2 | 태그 로딩 중복 추출 (Onboarding/Settings) | 유지보수성 | 소 | Medium |
| 3 | UI 테스트 보강 (피드→상세, 설정→피드 동기화 E2E) | 크로스-Feature 검증 | 중 | High |
| 4 | iOS 커버리지 측정 파이프라인 구축 (xcodebuild + xcresult) | 90% 게이트 | 중 | Medium |
| 5 | SettingsFeature.tagsChanged 중복 제거 (computed로 충분) | 코드 정리 | 소 | Low |
| 6 | DetailFeature 요약 폴링 — summary==nil 시 3초 간격 폴링 (최대 10회/30초), 타임아웃 시 실패 상태 + 재시도 버튼 | UX (영원히 "요약 중..." 표시) | 소 | High |

### 계획적 (MVP3 마일스톤에 포함)

| # | 이슈 | 연관 기능 | 우선순위 |
|---|------|----------|---------|
| 6 | API 스키마 공유 (OpenAPI 또는 공유 타입 정의) | 멀티 플랫폼 안정성 | Medium |
| 7 | Router 패턴 활용 (딥링크, 복잡 네비게이션) | 새 Feature 추가 | Low |
| 8 | Feature base protocol (에러/로딩 상태 공통화) | 코드 중복 | Low |
| 9 | 에러 메시지 세분화 (네트워크/인증/서버 구분) | UX 개선 | Low |
| 10 | saveMyTags 원자적 RPC 전환 (현재 delete→insert) | 데이터 무결성 | Medium |

### MVP1/1.5 잔여 부채 현황 (12건 → 현재)

MVP1.5 회고에서 남겨둔 12건 중:

| 원래 이슈 | MVP2 상태 | 비고 |
|----------|----------|------|
| feed 컴포넌트 분리 (283줄) | 해당 없음 | iOS는 148줄로 적절 분리 |
| 태그 선택 UI 공통 컴포넌트화 | ✅ 해결 | iOS TagChipView 3곳 재사용 |
| i18n 체계 도입 | 미해결 | 웹만 해당, iOS는 한국어 단일 |
| API 프록시 라우트 중복 | 미해결 | 웹만 해당 |
| $effect race condition | 미해결 | 웹만 해당 |
| 나머지 7건 (Minor) | 미해결 | 웹 기능 추가 시 점진적 해소 |

---

## MVP3 방향 제언

### 1. "학습" 레이어 착수

MVP1 회고에서 제안한 방향이 여전히 유효:
- 스크랩 보관함 (북마크)
- 기사 기반 퀴즈/플래시카드
- 주간 학습 리포트

### 2. 멀티 플랫폼 동기화

iOS + 웹이 동일 백엔드를 공유하므로:
- 읽음 상태 동기화
- 북마크 크로스 디바이스
- 푸시 알림 (APNs)

### 3. 개인화 강화

- 읽은 기사 기반 추천
- 사용자 정의 검색 쿼리
- 수집 주기 자동화 (크론)

---

## 회고를 마치며

MVP2는 "같은 패턴이면 새 플랫폼도 하루면 된다"는 것을 증명한 프로젝트다.

iOS 개발자가 Rust + Svelte를 거쳐 다시 iOS로 돌아왔는데, 전혀 다른 기술 스택임에도 포트/어댑터 패턴이라는 공통 언어 덕분에 아키텍처 설계에 시간을 쓰지 않았다. Feature → Port → Adapter 구조가 세 플랫폼에서 동일하게 동작하고, TDD가 동일하게 적용되며, 테스트가 동일하게 빠르다.

9,999줄 코드, 333개 테스트, 3개 플랫폼. MVP1에서 시작한 "에코 서버 패턴"이 프로젝트 전체를 관통하는 설계 원칙으로 자리잡았다.

다음은 MVP2.5다. M6에서 드러난 크로스-Feature 버그 패턴과 FeedFeature 상태 복잡도를 정리하고, MVP3에서 "학습" 기능을 올릴 준비를 한다.

> *"패턴이 동일하면 플랫폼은 장벽이 아니다" — 2026.04.06, MVP2 완료*
