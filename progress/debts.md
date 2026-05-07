# 기술 부채 목록

의도적으로 보류한 설계·구현 결정. 다음 MVP 기획 시 흡수 여부 판단.

> 최종 갱신: 2026-05-07 (DEBT-HOOK-02, 03, 04 신규 추가 — 훅 시스템 보완 계획 등록)

---

## [DEBT-01] 오답 태그 필터링 — DB 컬럼 방식(A안) 보류

**발생**: MVP12 BUG-F 처리 시점
**상태**: ✅ **RESOLVED** (260428 MVP13 M1+M2에서 해소)
**관련 버그**: BUG-003 (RESOLVED), BUG-F (RESOLVED)

### 현상

웹·iOS 오답노트에서 태그 필터가 제대로 동작하지 않음.
- favorites에 없는 기사의 오답은 태그 필터에서 아예 제외됨
- 태그 칩 자체가 안 나오는 경우 발생

### 근본 원인 (260428 코드 탐색 확인)

`quiz_wrong_answers` 테이블에 **tag_id 컬럼이 없음**.

오답 저장 시점에 어느 태그 기사인지 기록하지 않아서,
웹·iOS 모두 `favorites.tag_id`를 브릿지로 삼아 `article_url` 기준 간접 매핑으로 클라이언트 필터링 중.

```
현재(B안):
  서버 → 전체 오답 반환
  클라이언트 → favorites[url → tag_id] 맵 생성 → 필터링
  문제: favorites에 없는 기사 오답은 태그 정보 없음 → 필터 제외
```

**관련 파일**:
- DB: `supabase/migrations/20260412_mvp8_m1_schema.sql` (tag_id 컬럼 없음)
- 서버: `server/src/infra/postgres_quiz_wrong_answers.rs` (WHERE user_id 만 필터)
- 서버: `server/src/domain/models.rs` (QuizWrongAnswer, SaveWrongAnswerParams — tag_id 필드 없음)
- 웹: `web/src/lib/utils/favorites-filter.ts` (favorites 기반 클라이언트 필터)
- iOS: `ios/Frank/Frank/Sources/Features/Favorites/WrongAnswerTagFilter.swift` (favorites 기반 클라이언트 필터)

### 해결 방향 (A안)

1. **DB 마이그레이션**: `quiz_wrong_answers`에 `tag_id UUID` 컬럼 추가
2. **서버**: 오답 저장 시 tag_id 함께 저장, API에 `?tag_id=` 쿼리 파라미터 추가
3. **웹**: 클라이언트 필터 로직 제거 → 서버 필터 사용
4. **iOS**: 동일하게 클라이언트 필터 제거 → 서버 필터 사용

**흡수 조건**: MVP13 M1에서 구현

---

## [DEBT-02] iOS 피드 탭별 로딩 전략

**발생**: MVP12 M3 완료 시점
**상태**: ✅ **RESOLVED** (260428 코드 탐색 확인) — C안 적용 완료
**수정 내용**: "all" 탭 첫 페이지만 즉시 로드, 나머지 태그 탭은 첫 접근 시 lazy 로드 (`selectTag()` 캐시 미스 시 API 호출)

**잔여 관찰 사항 (신규 추적 필요 시)**: 웹은 "all" 단일 캐시 + 클라이언트 필터, iOS는 탭별 독립 캐시 + 서버 요청으로 구현 방식이 다름. 현재 기능상 문제는 없으나 웹 무한 스크롤 시 태그 탭 기사 희박 이슈 가능성 있음 → MVP13 피드 UX 마일스톤 시 재검토

---

## [DEBT-03] iOS 유닛 테스트 커버리지 수치 측정 자동화

**발생**: MVP12 종료 시점
**상태**: ✅ **RESOLVED** (260504 코드 탐색 확인) — `scripts/coverage.sh` 구축 완료

**실제 현황**: iOS 테스트 파일 28개. `scripts/coverage.sh`가 `xcodebuild test -enableCodeCoverage YES` + `xcrun xccov`로 Frank 타겟 `lineCoverage` 집계, 90% 미달 시 `exit 1` 처리됨.

---

## [DEBT-04] 피드 좋아요 버튼 터치포인트 — 디테일 화면으로 이동

**발생**: 2026-04-29 실사용 테스트 중
**상태**: ✅ **RESOLVED** (2026-04-30, MVP14 M3)
**플랫폼**: iOS
**현상**: 피드에서 좋아요 버튼을 누르면 버튼이 눌리지 않고 기사 디테일로 이동함. 터치 영역이 좁거나 이벤트가 상위 탭 제스처로 버블링되는 것으로 추정.
**해결**: FeedView.swift — NavigationLink 제거 (SwiftUI에서 NavigationLink 내부 Button이 탐색을 막지 못하는 근본 한계 확인). `ZStack(ArticleCardView + Button[like])` 구조로 전환 후 ZStack에 `.onTapGesture { navigationPath.append(item.id) }` 적용. like Button이 ZStack 최상위 레이어로서 탭을 독점 처리, 카드 영역 탭은 ZStack.onTapGesture가 받아 navigate.
**흡수 조건**: MVP14 M2

---

## [DEBT-05] 태그 스와이프 탐색 + 스와이프 삭제 충돌

**발생**: 2026-04-29 실사용 테스트 중
**상태**: ✅ **RESOLVED** (2026-04-30, MVP14 M3)
**플랫폼**: iOS
**현상**: 태그 바를 옆으로 스와이프하면 인접 태그로 이동하면 더 자연스러운데, 스크랩/오답노트 탭에서는 스와이프 삭제 기능과 충돌함.
**결정 방향**: 태그 스와이프 탐색 우선 → 스와이프 삭제는 롱프레스 또는 편집 모드로 대체.
**해결 1단계**: FavoritesView.swift — 기사 탭과 오답 탭 양쪽의 `.swipeActions` → `.contextMenu` 전환. 롱프레스로 삭제 메뉴 표시.
**해결 2단계**: FeedView.swift + FavoritesView.swift — DragGesture 제거 → `TabView(selection:).tabViewStyle(.page(indexDisplayMode: .never))` 전환. iOS 네이티브 페이지 스와이프 물리 효과로 자연스러운 태그 탐색 UY 제공. FavoritesView는 클라이언트 필터링이므로 각 페이지가 즉시 올바른 데이터를 표시.
**흡수 조건**: MVP14 M2 → MVP14 M3에서 완전 해소

---

## [DEBT-06] 요약 후 상단 버튼 사용성 저하

**발생**: 2026-04-29 실사용 테스트 중
**상태**: ✅ **RESOLVED** (2026-04-30, MVP14 M3)
**플랫폼**: iOS + Web 공통
**현상**: 기사 디테일에서 요약하기 실행 시 요약/인사이트 콘텐츠가 화면 하단에 쌓여, 그 위의 버튼(퀴즈 등)에 대한 접근성이 사실상 사라짐.
**개선 방향**: 요약/인사이트를 스크롤 가능한 하단 영역 또는 Bottom Sheet로 분리. 버튼은 고정 위치 유지.
**해결**: iOS — ArticleDetailView.swift에서 actionButtons를 ScrollView 밖으로 분리, `.safeAreaInset(edge: .bottom)`으로 하단 고정. 웹 — +page.svelte에서 스크랩/퀴즈 버튼을 `fixed bottom-0` 패널로 분리, main에 `pb-36` 추가.
**흡수 조건**: MVP14 M2

---

## [DEBT-07] 기사 소개 vs 요약/인사이트 카드 구분 미흡

**발생**: 2026-04-29 실사용 테스트 중
**상태**: ✅ **RESOLVED** (2026-04-30, MVP14 M3)
**플랫폼**: iOS + Web 공통
**현상**: 기사 디테일에서 기사 소개(원문 요약)와 AI 요약/인사이트가 시각적으로 구분되지 않아 읽기 불편함.
**개선 방향**: 카드 컴포넌트로 분리. 배경색·테두리·헤더 레이블로 명확히 구분.
**해결**: iOS(ArticleDetailView.swift) — 기사 소개: `systemGray6` 배경 + `📰 기사 소개` Label. AI 요약+인사이트: `indigo.opacity(0.06)` 배경 + `✨ AI 요약 및 인사이트` Label(인디고). 웹(+page.svelte) — 기사 소개: `bg-gray-50` 배경. AI 요약+인사이트: `bg-indigo-50/50 border-indigo-100` 배경 + `✨ AI 요약 및 인사이트` 헤더(인디고).
**흡수 조건**: MVP14 M2

---

## [DEBT-08] E2E 시나리오 테스트 자동화 기반 미구축

**발생**: 2026-04-29 실사용 테스트 필요성 인식
**상태**: ✅ **RESOLVED** (260504 코드 탐색 확인)
**수정 내용**: `.claude/skills/e2e/SKILL.md` 존재 확인. `web/e2e/` 4개 파일(feed-like, feed-summary, tag-navigation, smoke), `ios/FrankUITests/` 5개 UITest 파일(LoginFlow, FeedRefresh, CrossFeatureFlow, M3UXImprovements, UITestHelpers) 모두 구현 완료.

**잔여 작업**: occupation → insight → rewrite → scrap 플로우 등 MVP15 신규 기능 시나리오 미작성 → `progress/tasks/260504_debt_feedquality.md` C-1, C-2로 이관. 해당 시나리오는 ST-8(웹 E2E), ST-9(iOS UITest)에서 추가 예정

---

## [DEBT-MVP15-01] 진단 retry_classifier 에러 카테고리 분류 — 4xx 세분화 부족

**발생**: 2026-05-02 MVP15 M1 진단 실행 결과
**상태**: ✅ **RESOLVED** (260504 `quota_exhausted` 카테고리 신설, `retry_classifier.rs` 수정)
**현상**: Exa 무료 한도 소진 시 HTTP 402 Payment Required 반환되는데, `retry_classifier`가 이를 `network` 카테고리로 오분류. 운영상 결제 만료/한도 소진은 별도 카테고리로 식별돼야 운영 알람과 진단 보고서 정확.
**근본 원인**: `retry_classifier::categorize()`가 4xx 그룹 내에서 401/403만 `auth`로 분기하고 402/429 외 나머지는 일괄 처리. 402 Payment Required는 전용 카테고리 없어 fallback인 `network`로 잡힘.
**개선 방향**:
- `quota_exhausted` 카테고리 신설 (402 매핑)
- 진단 보고서 실패 표에 4xx 세분화 컬럼 추가
- 운영 코드(`feed.rs::SearchFallbackChain`)에서도 402 발생 시 즉시 알람
**흡수 조건**: MVP15 M2 진입 시 또는 별도 chore PR

---

## [DEBT-MVP15-02] 진단 바이너리 실행 환경 가정 강함

**발생**: 2026-05-02 MVP15 M1 진단 실행 시 cwd/`.env` 경로 문제
**상태**: ✅ **RESOLVED** (260504 `scripts/run-diagnose.sh` 추가, env 자동 소싱)
**현상**: `cargo run --bin diagnose_search`를 자연스럽게 실행하면 두 가지 에러 발생:
- `server/`에서 실행 → 출력 경로 `progress/mvp15/` 없음 (cwd가 server 기준)
- repo 루트에서 실행 → `DATABASE_URL` 미발견 (`.env`가 `server/.env`인데 dotenvy는 cwd 기준)
**임시 해결**: `cd <repo> && set -a && source server/.env && set +a && cargo run --manifest-path server/Cargo.toml --bin diagnose_search`
**개선 방향 (택1)**:
- 출력 경로·env 경로를 환경변수(`DIAGNOSE_OUTPUT_DIR`, `DIAGNOSE_ENV_PATH`) 또는 CLI 인자로 외부화
- `server/src/bin/diagnose/README.md`에 정확한 실행 명령 1줄 명시 + 실행 스크립트(`scripts/run-diagnose.sh`) 제공
**흡수 조건**: MVP15 후속 진단 사이클 진입 시

---

## [DEBT-MVP15-03] Tavily `effective_max` 동적 감지 부재

**발생**: 2026-05-02 MVP15 M1 진단에서 `requested=100, effective=20` 패턴 발견
**상태**: 🟡 **OPEN** (낮)
**현상**: Tavily 무료 티어가 `max_results` 상한 20으로 silent clamp. 진단 코드는 응답 결과 수에서 `effective`를 추정. Tavily가 무료 티어 정책을 변경(예: 30으로 상향)하면 코드 자동 추적 안 됨.
**개선 방향**: Tavily/Exa 응답 헤더(`X-Ratelimit-*`, `X-Plan-Limit` 등) 또는 첫 호출 응답 메타에서 plan limit 추출하여 진단 코드가 자동 갱신
**흡수 조건**: 진단 다음 사이클 또는 무료 티어 정책 변경 감지 시

---

## [DEBT-MVP15-04] Exa `reset_at` NULL semantics 미구현

**발생**: 2026-05-02 MVP15 M2 step-7 코드 리뷰 (Codex P2 지적)
**상태**: ✅ **RESOLVED** (260504 Exa `record_call` reset_at=NULL 적용, migration SQL 추가)
**현상**: `CounterPort` 설계 의도는 "Exa 등 크레딧형 엔진은 `reset_at = NULL`로 자동 리셋 없음"이나, 현재 Postgres/InMemory 구현 모두 첫 `record_call`에서 모든 엔진에 `date_trunc('month', now()) + 1 month`로 reset_at을 세팅. 알림 메시지 "수동 (크레딧 갱신)" 분기는 사실상 실행 경로 없음.
**리스크**: Exa 크레딧이 매월 자동 갱신되지 않는데도 카운터는 매월 0으로 reset됨 → 한도 보호 누락 가능성. 다만 현재 운영에서 Tavily 1순위·Exa 2순위로 호출 빈도 낮아 실제 위험 낮음.
**개선 방향 (택1)**:
- 엔진별 reset 정책 분리: `Engine::reset_policy()` enum (Monthly / CreditManual)
- Exa는 `record_call`에서 reset_at NULL 유지 → 알림에서 "크레딧 갱신" 분기 실효성
- 또는 설계 의도 폐기 후 모든 엔진 단일 month reset 정책 명문화 + 코멘트 정정
**흡수 조건**: Exa 한도 도달 운영 시점 또는 다음 카운터 리팩토링

---

## [DEBT-MVP15-05] notification_service `spawn_blocking + timeout` task leak 가능성

**발생**: 2026-05-02 MVP15 M2 step-7 코드 리뷰 (advisor + Codex P3 지적)
**상태**: 🟡 **OPEN** (낮)
**현상**: `services/notification_service.rs::send_with_timeout_retry`는 `tokio::time::timeout(spawn_blocking(...))` 패턴. `tokio::timeout`은 future를 drop할 뿐, spawn_blocking 작업 자체는 cancel 불가. `osascript`이 hang하면 timeout 발화 후에도 blocking 스레드는 계속 점유되어 재시도 시 누적 가능.
**리스크**: 단일 사용자 + 알림 빈도 낮음 (월 1~2회) → 운영 위험 매우 낮음. 다만 blocking pool 크기(기본 512) 대비 leak 누적 시 알림 외 다른 blocking 작업까지 영향.
**개선 방향**: `osascript`을 `std::process::Command` + `tokio::process::Child::kill()`로 교체하여 timeout 시 OS 레벨에서 강제 종료 가능하도록 변경
**흡수 조건**: 운영에서 알림 hang 사례 발견 시 또는 알림 채널 다양화 시

---

## [DEBT-MVP15-06] `infra → services` 역방향 레이어 의존 위반

**발생**: 2026-05-02 MVP15 M2 step-7 코드 리뷰 (advisor + Codex P6 지적)
**상태**: ✅ **RESOLVED** (260504 `AlertDispatcherPort` trait 신설, infra→services 의존 제거)
**현상**: `infra/counted_search.rs`(데코레이터)가 `services::notification_service::dispatch_threshold_alert`를 직접 import. CLAUDE.md 명시 의존 방향 `api → services → domain ← infra` 위반. infra 데코레이터가 service orchestration까지 짊어진 구조.
**리스크**: 런타임 버그는 아님. 의존 그래프 정리 시 순환 가능성 + 신규 개발자 혼란.
**개선 방향 (택1)**:
- `CountedSearchAdapter`를 `services/` 로 이동 (orchestration 책임)
- `dispatch_threshold_alert` 동작을 별도 포트(`AlertDispatcherPort`)로 추상화 후 infra/는 포트만 호출
**흡수 조건**: 다음 MVP에서 카운터·알림 인프라 확장 시 또는 의존 그래프 정리 chore

---

## [DEBT-MVP16-01] 피드 검색 쿼리 영어 통일 — 언어 필터 미적용

**발생**: 2026-05-04 회고 중 발견
**상태**: 🟡 **OPEN** (중)

### 현상

`tag_search_keyword()`로 한국어 태그명 → 영어 검색 키워드 변환을 적용했으나, Tavily·Exa 모두 언어/지역 필터 파라미터 없이 영어 쿼리를 그대로 던지고 있음. 영어 쿼리 + 언어 필터 없음 = 영어 기사 위주로 반환될 가능성 있음.

### 근본 원인

`tag_search_keyword()` 신설이 트리거는 Tavily 복귀(Tavily는 영어 최적화)였으나, 실제 수정은 엔진 무관하게 쿼리 전체를 영어로 통일한 것. 언어/도메인 필터는 포함하지 않음.

- Tavily: `country` 파라미터 또는 `include_domains`로 한국 뉴스 소스 제한 가능
- Exa: `includeDomains`로 한국 언론사 화이트리스트 적용 가능

### 리스크

현재 앱이 한국어 기사를 주로 노출해야 하는 서비스라면, 영어 기사 위주 피드는 사용성 문제. 반대로 영어 기사도 허용하는 방향이라면 문제 없음.

### 개선 방향

- 엔진별 언어/도메인 필터 파라미터 추가 (Tavily `country:"KR"`, Exa `includeDomains`)
- 또는 결과 후처리에서 한국어 기사 우선 정렬
- 먼저 현재 피드 응답에서 영어 기사 비율 실측 후 방향 결정

**흡수 조건**: MVP16 마일스톤 설계 시 "키워드 매핑 정밀화" 작업에 포함

---

## [DEBT-MVP15-07] 피드 응답 snippet에 invalid JSON escape sequence

**발생**: 2026-05-02 MVP15 M2 step-8 라이브 자동화 검증 중 발견 (M2 envelope 변경 이전부터 존재하는 기존 부채)
**상태**: ✅ **RESOLVED** (260504 `clean_snippet()` nul 바이트·제어문자 제거, 단위 테스트 추가)
**현상**: `GET /api/me/feed` 응답 raw JSON에 `\4`, `\_` 같은 invalid escape sequence 27건 출현(예: `"snippet":"k5:G DEJ=6lQ42C6E\4@=@..."`). `JSON.parse` / `serde_json::from_str` / `jq` 모두 파싱 실패(`Invalid \escape: line 1 column 9006`).
**리스크**: M3에서 클라이언트(웹 `realClient.ts` / iOS `APIArticleAdapter.swift`)가 envelope 디시리얼라이저로 전환할 때 일부 응답이 디시리얼라이즈 실패 → 피드 화면 빈 결과 + 사용자 영향. 현재 관측 시점의 32 items 중 1건이 문제.
**원인 후보**:
- Tavily/Exa/Firecrawl 응답의 snippet이 raw HTML/특수 인코딩 포함 → `clean_snippet()` 파이프라인이 backslash escape 처리 누락
- 또는 직렬화 시 `serde_json`의 `String` 타입에 비ASCII 제어문자가 들어가 있어 escape 처리 실패
**개선 방향**:
- `infra/exa.rs::clean_snippet` 같은 정제 함수에서 invalid backslash escape 제거 추가
- 또는 직렬화 직전 `serde_json::to_string` 에서 escape 표준화
- 단위 테스트: `\4`, `\_` 등 27가지 패턴 fixture로 클린업 검증
**흡수 조건**: M3 클라이언트 envelope 전환 시 같이 처리 권장 (디시리얼라이즈 견고성 + 사용자 영향 최소화)

---

## [DEBT-MVP16-02] occupation 삭제 불가 버그

**발생**: 2026-05-05 내부 동작 분석 중 발견
**상태**: ✅ **RESOLVED** (2026-05-06, MVP16 M3)

**현상**: 직업(occupation)을 한번 설정하면 다른 값으로 변경은 가능하나, 아예 삭제(공백으로 비우기)가 불가능하다. 저장 버튼을 눌러도 기존 직업이 그대로 유지된다.

**원인 — 3단계 연쇄**:

1. **클라이언트(웹/iOS)**: 삭제 의도 시 `"occupation": null` 또는 `""` 전송 (올바름)
2. **서버 `api/profile.rs`**: `UpdateProfileRequest.occupation` 타입이 `Option<String>` → JSON `null`(삭제 의도)과 "필드 없음"(변경 안 함)이 모두 `None`으로 붕괴되어 구별 불가
3. **DB `infra/postgres_db.rs`**: `occupation.is_none()` → 조기 반환 또는 `COALESCE($4, profiles.occupation)` SQL로 기존 값 유지 → **실제 NULL 저장 안 됨**

**관련 파일**:
- `server/src/api/profile.rs` — `UpdateProfileRequest.occupation: Option<String>` (문제 지점)
- `server/src/infra/postgres_db.rs` — `COALESCE($4, profiles.occupation)` SQL (문제 지점)
- `server/src/domain/ports.rs` — `update_profile` 시그니처

**개선 방향**:
- `occupation` 타입을 `Option<Option<String>>`으로 변경 (`#[serde(default)]` 사용)
  - outer `None` = 필드 미제공 (변경 없음)
  - `Some(None)` = 명시적 null (삭제)
  - `Some(Some(v))` = 값 설정
- DB SQL: `CASE WHEN $5 THEN $4 ELSE profiles.occupation END` 패턴으로 교체 (`$5` = occupation 제공 여부 bool)
- `fake_db.rs` mock 및 테스트도 동일하게 수정
- 테스트 추가: `"occupation": null` → 삭제 확인

**흡수 조건**: MVP16 마일스톤 또는 occupation 관련 기능 작업 시

---

## [DEBT-MVP16-03] occupation 없을 때 insight 미노출 버그

**발생**: 2026-05-05 내부 동작 분석 중 발견
**상태**: ✅ **RESOLVED** (2026-05-06, MVP16 M2 C3 — 범용 insight 생성으로 처리)

**현상**: 직업(occupation)을 설정하지 않은 사용자는 요약하기 기능에서 인사이트(insight)가 전혀 표시되지 않는다.

**원래 의도 vs 현재 구현**:
- **MVP9 이전 원래 구현** (`SYSTEM_PROMPT`): occupation 무관하게 `summary` + `insight` 항상 생성. insight = "이 기사가 왜 중요한지, 독자에게 어떤 의미인지 분석"
- **MVP15 M3 변경 후 (`bb8f517`)**: occupation 유무로 분기
  - occupation 없음 → `SYSTEM_PROMPT_NO_OCCUPATION` → summary만, insight = `None`
  - occupation 있음 → `SYSTEM_PROMPT_WITH_OCCUPATION` → summary + 직업 맞춤 insight

**MVP15 시드 원래 의도**: "내 커스텀 인사이트 = 프로필 기반 맞춤 시각으로 기존 요약/인사이트를 고도화"
→ 즉 기존 범용 insight를 직업 맞춤으로 **업그레이드**하는 게 의도였으나, 구현 시 occupation 없으면 insight 자체가 사라지는 방식으로 구현됨

**관련 파일**:
- `server/src/infra/groq.rs` — `SYSTEM_PROMPT_NO_OCCUPATION` (insight 필드 없음), `SYSTEM_PROMPT_WITH_OCCUPATION` (직업 맞춤 insight)
- `server/src/api/summarize.rs` — `SummarizeResponse.insight: Option<String>`

**개선 방향**:
- occupation 없을 때도 범용 insight 표시 (MVP9 시대 프롬프트 복원)
- occupation 있을 때 → 직업 맞춤 insight로 업그레이드
- 즉: 항상 insight 생성, occupation 유무가 insight의 **시각**만 결정

**흡수 조건**: MVP16 마일스톤 또는 요약 기능 개선 작업 시

---

## [DEBT-HOOK-01] step-2,3,5,6,7,8 active_step.txt 미갱신

**발생**: 2026-05-05 훅 시스템 분석 중 발견
**상태**: 🔴 **OPEN**

**현상**: workflow/SKILL.md에 "각 step SKILL이 진입 시 자기 step으로 갱신할 책임"이라고 명시돼 있으나, step-1/step-4/step-9만 `active_step.txt`를 실제로 갱신함. step-2, 3, 5, 6, 7, 8은 구현 누락.

**영향**:
- 장치2(주입 훅)가 step-3에서도 "📝 활성 step: step-1"처럼 부정확한 값을 주입 → Claude가 stale 값을 컨텍스트로 받음
- step-9 SKILL.md에 cleanup 코드(`echo "none"`)가 이미 있었으나 실행 안 됨 → "지시가 있어도 Claude가 빠뜨린다"는 문제 확인됨

**해결 방향 (Phase 1, DEBT-HOOK-02와 세트)**:
1. A안(백업): 각 SKILL.md 진입부에 `echo "step-N" > progress/active_step.txt` 추가 (대상: step-2, 3, 5, 6, 7, 8)
2. B안(근본): PostToolUse 훅으로 Skill 도구 호출 시 자동 갱신 (DEBT-HOOK-02)
- B안이 Claude 의존 제거라 더 안정적. 두 가지 병행 권장.

**흡수 조건**: DEBT-HOOK-02와 함께 훅 시스템 보완 작업 시

---

## [DEBT-HOOK-02] PostToolUse 훅 미구현 — Skill 호출 시 active_step.txt 자동 갱신

**발생**: 2026-05-07 훅 설계 분석 중 도출
**상태**: 🔴 **OPEN**

**현상**: DEBT-HOOK-01의 근본 원인은 "SKILL에 지시가 있어도 Claude가 빠뜨린다"는 것. A안(지시 추가)만으로는 step-9에서 이미 실패한 방식의 반복임. B안으로 Claude 의존 없이 훅 레벨에서 강제 갱신 필요.

**설계**:
```
사용자: /step-3 호출
→ Claude: Skill("step-3") 도구 호출
→ PostToolUse 훅 발화
→ tool_input.skill 값 파싱
→ "step-[1-9]" 패턴 매칭 시에만 active_step.txt 갱신
→ /next, /workflow 등 다른 Skill은 필터로 제외
```

**사전 검증 필수**: 훅에서 `tool_input.skill` 값을 실제로 읽을 수 있는지 먼저 테스트 훅으로 확인 후 구현.
```bash
# 검증용 임시 훅 (hook.log로 tool_input 구조 확인)
echo "$HOOK_INPUT" >> /tmp/hook.log
```

**구현 범위**:
1. `.claude/hooks/update-active-step.sh` 신규 생성
2. `.claude/settings.json` PostToolUse 훅 등록
3. DEBT-HOOK-01 A안(각 SKILL.md 갱신 지시) 병행

**흡수 조건**: 다음 워크플로우 킥오프 전 또는 훅 시스템 보완 작업 시

---

## [DEBT-HOOK-03] /next 스킬 자동 흐름 미구현

**발생**: 2026-05-07 /next 스킬 설계 검토 중 도출
**상태**: 🔴 **OPEN** (Phase 2 — DEBT-HOOK-02 완료 후 착수)

**현상**: 현재 /next 스킬은 step-4~step-9 서브태스크 사이클만 정의. step-1~step-3 구간 포함 안 됨. 또한 "사용자 결정 불필요한 step은 자동 진행, 결정 필요한 step에서 멈춤" 로직이 없어 매 step마다 수동 진입 필요.

**원하는 흐름**:
```
/next 호출
→ active_step.txt로 현재 단계 확인
→ 다음 step 실행
→ 완료 후 판단:
   자동 진행 가능 → 다음 step도 실행
   사용자 결정 필요 → 멈추고 질문
```

**자동 진행 / 멈춤 기준**:
| 전환 | 구분 | 이유 |
|------|------|------|
| step-1 → 2 | ✅ 자동 | 룰즈 검증은 판단 불필요 |
| step-2 → 3 | ✅ 자동 | 서브태스크 분리 바로 진행 |
| step-3 → 4 | 🛑 멈춤 | 인터뷰 = 사용자 답변 필요 |
| step-4 → 5 | 🛑 멈춤 | 인터뷰 답변 후 검토 필요 |
| step-5 → 6 | ✅ 자동 | 합의 완료 후 구현 바로 진행 |
| step-6 → 7 | ✅ 자동 | 구현 후 리팩토링 바로 진행 |
| step-7 → 8 | ✅ 자동 | 리팩토링 후 테스트 바로 진행 |
| step-8 → 9 | 🛑 멈춤 | 커밋 = 명시적 사용자 허락 필수 |

**설계 리스크 (착수 전 해결 필요)**:
- step-6(구현) → 7(리팩) → 8(테스트) chunk: 구현이 수십 turn에 걸칠 수 있어 "완료" 시그널 정의 모호. 모델이 자의적으로 넘어갈 위험.
- 초기 구현은 step-1→2→3 자동 진행 chunk부터 시작, step-6→7→8 자동 진행은 step 완료 시그널 정의 후 2차 확장.

**선행 조건**: DEBT-HOOK-02 완료 (active_step.txt가 정확해야 /next 로직이 의미있음)

**흡수 조건**: DEBT-HOOK-02 완료 후 별도 워크플로우 개선 마일스톤

---

## [DEBT-HOOK-04] step-8에서 자동화 가능 테스트를 사용자에게 위임

**발생**: 2026-05-07 MVP16 워크플로우 진행 중 발견
**상태**: 🔴 **OPEN**

**현상**: step-8 SKILL.md에 "자동화 가능한 항목(린트, 빌드, 단위 테스트, E2E)은 Claude가 직접 실행 후 [x] 처리, 시각 확인만 사용자에게 요청"이라고 명시돼 있음. 그러나 실제 진행 시 Claude가 이 구분 없이 모든 항목을 사용자에게 물어봄.

**영향**: step-8마다 사용자가 직접 CLI 명령을 실행하고 결과를 보고해야 함. 자동화된 검증의 의미가 퇴색.

**수정 방향**:
- step-8 SKILL.md의 자동화 가능 항목 목록을 더 명시적으로 재작성
- "다음 명령을 직접 실행하세요"가 아니라 Claude가 Bash 도구로 직접 실행 후 결과 보고
- 실패 시 자동으로 해당 step(6 또는 7)으로 복귀 안내
- 시각 확인(UI 렌더링, 레이아웃)만 사용자 요청

**흡수 조건**: 훅 시스템 보완 작업 시 또는 다음 워크플로우 킥오프 전

---

## [DEBT-MVP16-04] Tavily `include_domains` IT 전문 도메인 화이트리스트

**발생**: 2026-05-06 MVP16 M1 E2E 검증 중 발견
**상태**: 🟡 **OPEN** (중)

**현상**: E2E에서 "오픈소스" 탭에 광업 기사, "웹 개발" 탭에 NFL 기사 혼입 확인. `country` 파라미터 제거 및 `tag_search_keyword()` 정밀화만으로는 Tavily 인덱스 전체 범위에서 검색하므로 무관 도메인 기사를 근본 차단하지 못함.

**개선 방향**: Tavily `include_domains` 파라미터로 IT 전문 뉴스 도메인 30개 화이트리스트 적용.
- 후보 도메인: techcrunch.com, theverge.com, wired.com, arstechnica.com, zdnet.com, venturebeat.com, github.blog, devto.medium.com 등
- 도메인 리스트는 태그별로 다를 수 있음 (예: "투자/VC" 탭 → techcrunch.com + bloomberg.com)
- 구현 위치: `infra/tavily.rs::search()` body JSON에 `"include_domains": [...]` 추가

**리스크**: 도메인 수 제한이 너무 좁으면 특정 태그 결과 0건 발생 가능. 초기 적용 후 E2E로 부작용 확인 필요.

**흡수 조건**: MVP17 피드 품질 개선 작업 시 또는 무관 기사 혼입 재발 시

---

## [DEBT-MVP16-05] 한국어 기사 노출 — Naver News API 어댑터

**발생**: 2026-05-06 MVP16 M1 E2E 검증 중 확인
**상태**: 🟡 **OPEN** (중)

**현상**: `tag_search_keyword()`에 한국어 키워드를 추가했으나 E2E에서 한국어 기사 0건. Tavily API 인덱스가 영어 뉴스 중심이며 `language` 파라미터를 제공하지 않음. `country:"south korea"` 파라미터는 한국 뉴스 집계 사이트 우선화 부작용(스포츠/광업 혼입)만 발생시키고 언어 필터로는 동작하지 않음.

**근본 원인**: Tavily가 구조적으로 한국어 기사 검색에 적합하지 않음. 한국어 기사는 별도 소스 어댑터가 필요.

**개선 방향**:
- Naver News Search API 어댑터 추가 (`infra/naver_news.rs`)
- `SearchPort` trait 구현, `SearchFallbackChain`에 한국어 소스로 병렬 합산 또는 별도 레인으로 추가
- 또는 한국 주요 IT 언론사(ZDNet Korea, IT조선, 디지털데일리 등) RSS 피드 어댑터

**비용 영향**: Naver News API 무료 쿼터 25,000건/일 — 현재 사용 패턴 대비 충분.

**흡수 조건**: MVP17 또는 다국어 피드 기능 작업 시

---

## [DEBT-MVP16-06] C1 한자 혼입 — LLM 모델 교체 평가

**발생**: 2026-05-06 MVP16 M2 E2E 검증 중 확인
**상태**: 🟡 **OPEN** (중)

**현상**: `SYSTEM_PROMPT_SUMMARY`에 "Output in Korean only. Do not use Chinese characters (漢字)" 제약을 추가했으나, llama-3.3-70b-versatile이 한국어 요약 중 `不断`, `变化`, `框架` 등 한자를 혼입함. 프롬프트 제약만으로는 모델 행동 통제 불충분.

**근본 원인**: llama-3.3-70b-versatile의 한국어-중국어 혼합 출력 경향. retry 방식은 비용·응답시간 2배, strip 방식은 문맥 파괴.

**개선 방향**: Groq에서 한국어 instruction following이 더 강한 모델로 교체 평가.
- 후보: `gemma2-9b-it` (Google, 경량·한국어 강함), `llama-3.1-70b-versatile`
- **주의**: `GROQ_MODEL` 상수를 `summarize_with_occupation`과 `rewrite_with_occupation`이 공유함 → 모델 교체 시 두 기능 모두 E2E 검증 필요
- 평가 항목: 한자 혼입 빈도, 요약 품질, 재작성 품질, 비용, 응답속도

**흡수 조건**: MVP17 또는 LLM 품질 개선 마일스톤에서 전용으로 다룰 것

---

## [DEBT-MVP16-07] B1 보일러플레이트 snippet — 검색 소스 품질

**발생**: 2026-05-06 MVP16 M2 E2E 검증 중 확인
**상태**: 🟡 **OPEN** (하)

**현상**: Forbes 등 일부 사이트에서 Tavily가 기사 본문 대신 footer/navigation 텍스트(Privacy Statement, reCAPTCHA, Subscribe 안내 등)를 snippet으로 반환. 마크다운 제거 후에도 의미 없는 텍스트가 그대로 노출됨.

**근본 원인**: Tavily가 JavaScript 중심 사이트에서 실제 기사 본문 추출 실패. snippet 내용 자체가 garbage.

**개선 방향**: snippet 품질 점수 기반 필터링 (예: 특정 boilerplate 키워드 감지 시 snippet=None 처리) 또는 Tavily `include_domains` 화이트리스트 적용(DEBT-MVP16-04와 연계).

**흡수 조건**: MVP17 피드 품질 개선 작업 시

---

## [DEBT-MVP16-08] FakeDbAdapter cascade no-op — favorites 연동 미검증

**발생**: 2026-05-06 MVP16 M3 step-7 코드리뷰 중 발견
**상태**: 🟡 **OPEN** (하)

**현상**: `FakeDbAdapter.clear_occupation`은 profiles만 변경하고 favorites를 건드리지 않음. 단위 테스트에서 occupation 삭제 시 favorites.rewrite NULL 초기화 여부가 검증되지 않는다. Postgres 통합 테스트에서만 실제 cascade 동작 확인 가능.

**개선 방향**: `FakeFavoritesAdapter`에 `reset_user_rewrites()` 유틸 추가 후 `profile.rs` 통합 테스트에서 두 어댑터 연동 시나리오 커버.

**흡수 조건**: 테스트 커버리지 강화 마일스톤 또는 MVP16 M4 진입 전

---

## [DEBT-MVP16-09] ProfileService 미분리 — 핸들러 오케스트레이션 비대

**발생**: 2026-05-06 MVP16 M3 step-7 코드리뷰 중 발견
**상태**: 🟡 **OPEN** (하)

**현상**: `api/profile.rs` 핸들러가 occupation 3-상태 판단 + `DbPort` + `FavoritesPort` 순서 호출을 직접 오케스트레이션함. 현재 볼륨은 허용 가능하나 로직 추가 시 핸들러 비대화 위험.

**개선 방향**: `services/profile_service.rs` 신설 후 오케스트레이션 위임. 핸들러는 파싱 + 서비스 호출 + 응답 변환만 담당.

**흡수 조건**: 서비스 레이어 정비 마일스톤 또는 핸들러 로직 증가 시
