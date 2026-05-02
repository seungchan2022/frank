# 기술 부채 목록

의도적으로 보류한 설계·구현 결정. 다음 MVP 기획 시 흡수 여부 판단.

> 최종 갱신: 2026-04-29

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
**상태**: 🟡 **DEFERRED** (실질적 영향 낮음)

**실제 현황** (260428 코드 탐색 확인): iOS 테스트 파일 17개 확인됨. Adapter 4개, Feature 8개, Component 3개, 순수함수 2개. 커버리지 수치 자동 측정 스크립트 미구성.

**흡수 조건**: 커버리지 수치가 KPI 게이트로 필요해지는 시점에 `xccov` 연동

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
**상태**: 🟡 **OPEN** (새 스킬 구축 필요)
**현상**: 현재 기능 검증은 수동 테스트에 의존. Claude가 시나리오를 작성하고 직접 실행하며 결과를 체크하는 자동화 흐름이 없음.
**개선 방향**:
- 웹: Playwright MCP로 시나리오 직접 실행 + 스크린샷 검증
- iOS: XCUITest 시나리오 파일 자동 생성 → `xcodebuild test`로 실행
- `/e2e` 스킬: 마일스톤 기능 목록 → 시나리오 생성 → 실행 → 리포트 (누적 시 리그레션 베이스)
**흡수 조건**: MVP14 M3

---

## [DEBT-MVP15-01] 진단 retry_classifier 에러 카테고리 분류 — 4xx 세분화 부족

**발생**: 2026-05-02 MVP15 M1 진단 실행 결과
**상태**: 🟡 **OPEN** (중)
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
**상태**: 🟡 **OPEN** (낮)
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
**상태**: 🟡 **OPEN** (낮)
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
**상태**: 🟡 **OPEN** (낮)
**현상**: `infra/counted_search.rs`(데코레이터)가 `services::notification_service::dispatch_threshold_alert`를 직접 import. CLAUDE.md 명시 의존 방향 `api → services → domain ← infra` 위반. infra 데코레이터가 service orchestration까지 짊어진 구조.
**리스크**: 런타임 버그는 아님. 의존 그래프 정리 시 순환 가능성 + 신규 개발자 혼란.
**개선 방향 (택1)**:
- `CountedSearchAdapter`를 `services/` 로 이동 (orchestration 책임)
- `dispatch_threshold_alert` 동작을 별도 포트(`AlertDispatcherPort`)로 추상화 후 infra/는 포트만 호출
**흡수 조건**: 다음 MVP에서 카운터·알림 인프라 확장 시 또는 의존 그래프 정리 chore

---

## [DEBT-MVP15-07] 피드 응답 snippet에 invalid JSON escape sequence

**발생**: 2026-05-02 MVP15 M2 step-8 라이브 자동화 검증 중 발견 (M2 envelope 변경 이전부터 존재하는 기존 부채)
**상태**: 🟡 **OPEN** (중)
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
