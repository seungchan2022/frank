# MVP3 완료 회고 — 웹+iOS API 통합 + Apple 로그인

> 작성일: 2026-04-08
> 기간: 2026-04-06 ~ 2026-04-08 (3일)
> 상태: MVP3 완료
> 선행: MVP2 완료 회고 (`history/mvp2/260406_mvp2_completion_retro.md`)

---

## 숫자로 보는 MVP3

| 항목 | 수치 |
|------|------|
| 총 개발 기간 | 3일 (기획 반나절 + M1~M4 구현) |
| 총 커밋 (MVP3 기간) | 22개 (feat 8, docs 8, fix 4, chore 2) |
| 마일스톤 | 5개 완료 (M1, M1.5, M2, M3, M4) + hotfix 1개 |
| 병렬 worktree | 2개 (frank-m2-web, frank-m3-ios) |
| 웹 테스트 | 89개 (전체 통과) |
| iOS 테스트 | 155개 (전체 통과) |

### 프로젝트 전체 규모 (MVP1+1.5+2+3)

| 영역 | 소스 코드 | 테스트 코드 | 테스트 수 |
|------|----------|-----------|---------|
| 서버 (Rust) | 6,075줄 | 포함 | 139개 |
| 웹 (Svelte/TS) | 2,489줄 | 1,297줄 | 89개 |
| iOS (Swift) | 3,338줄 | 3,388줄 | 155개 |
| **합계** | **11,902줄** | | **383개** |

MVP2 완료 시점(9,999줄/333테스트) 대비 소스 +1,903줄, 테스트 +50개.

---

## 무엇을 만들었나

**Frank 웹+iOS API 통합** — 웹과 iOS가 각각 Supabase를 직접 호출하던 구조를 Rust API 서버를 통해 단일 진실의 원천으로 통합. Apple Sign In을 웹+iOS 동시 구현하여 크로스 플랫폼 계정 연동 완성.

### 마일스톤별 진행

| M# | 마일스톤 | 핵심 산출물 | 주요 발견 |
|----|---------|-----------|---------|
| M1 | API Contract | fetchArticle, updateProfile 엔드포인트 신설 + DTO 분리 | 기존 articles 페이지네이션 오프셋 방식 재활용 가능 |
| M1.5 | 병렬 준비 | API SPEC 문서, fixture JSON, 웹 MockClient, iOS MockAdapters | fixture 공유로 웹/iOS Mock 데이터 동기화 |
| M2 | 웹 전환 | @supabase/ssr + httpOnly 쿠키 + Rust API 직통 | 태그 stale 문제 발견 → 옵션B(즉시 반영) 채택 |
| M3 | iOS 전환 | APIArticleAdapter, APITagAdapter + MVP2 부채 흡수 | 요약 60s timeout UX 미해결 → MVP4 이월 |
| hotfix | OpenRouter reasoning | MiniMax M2.5 reasoning mandatory 400 fix | M2/M3 통합 검증 중 발견 |
| M4 | Apple 로그인 | OAuth PKCE(웹) + ASAuthorizationController(iOS) | 크로스 플랫폼 계정 자동 연동 확인 |

---

## 심층 분석

### 1. 병렬 worktree 전략 — 첫 실전 검증

M2(웹)와 M3(iOS)를 두 개의 git worktree에서 동시 진행했다. 결과:

| 항목 | 결과 |
|------|------|
| 코드 충돌 | 0건 (web/ vs ios/ 완전 분리) |
| 외부 자원 충돌 | 0건 (Mock-First로 서버/DB 비접촉) |
| 병렬 개발 시간 절약 | M2+M3 합산 시간 ≈ 단일 진행 대비 약 40% 단축 추정 |
| 머지 복잡도 | 낮음 (progress/ 공통 문서만 주의) |

**핵심 성공 요인**: fixture JSON을 공유하여 웹/iOS Mock 데이터를 동기화한 것. 양쪽이 동일한 기준 데이터로 개발하여 통합 시 스키마 불일치 0건.

**아쉬운 점**: progress/ 문서에서 각 탭이 자기 마일스톤 문서만 수정하는 정책이 있었지만, 로드맵 갱신 타이밍이 어색했다. 머지 시점에 메인 탭에서 일괄 갱신하는 흐름은 유효.

### 2. httpOnly 쿠키 전환 — 웹 보안 업그레이드

M2에서 `@supabase/ssr`로 전환하면서 인증 토큰이 localStorage → httpOnly 쿠키로 이동했다.

**이전**: `supabase.auth.getSession()` → localStorage에서 직접 읽음, XSS 취약
**이후**: `hooks.server.ts` → 모든 요청에서 서버 사이드 세션 검증, JS 접근 불가

부작용으로 Playwright 테스트에서 `document.cookie`로 세션 삭제 불가 → form POST `/logout` 방식으로 대응. 이는 올바른 방향이다.

### 3. Apple 로그인 — 두 플랫폼의 전혀 다른 구현

| 항목 | 웹 | iOS |
|------|-----|-----|
| 방식 | OAuth PKCE (redirect) | ID Token (ASAuthorizationController) |
| Supabase API | `signInWithOAuth({ provider: 'apple' })` | `signInWithIdToken(OpenIDConnect)` |
| 흐름 | 브라우저 Apple 페이지 → callback → 세션 교환 | 네이티브 시트 → idToken + nonce → Supabase |
| 핵심 트러블 | use:enhance가 redirect를 fetch로 가로챔 | canceled 분기 누락 |

두 방식이 동일 Supabase user_id를 생성하므로 크로스 플랫폼 계정 자동 연동. 웹에서 온보딩 완료 → iOS 로그인 시 온보딩 스킵이 자동으로 동작함을 실제로 확인.

### 4. Client IDs 순서 함정

Supabase Apple Provider 설정에서 `Client IDs` 필드의 첫 번째 값이 OAuth `client_id`로 사용된다. Bundle ID(`dev.frank.app`)를 먼저 적으면 Apple이 Service ID로 인식하지 못해 `invalid_client` 오류.

```
❌ dev.frank.app,com.frank.web   → Bundle ID가 OAuth client_id로 사용됨
✅ com.frank.web,dev.frank.app   → Service ID가 OAuth client_id로 사용됨
```

문서에 명시된 내용이 아니라 트러블슈팅으로 발견. **Apple Service ID와 Bundle ID의 역할 분리 이해 필수.**

### 5. 테스트 성장 분석

| MVP 시점 | 웹 | iOS | 서버 | 합계 |
|---------|-----|-----|------|------|
| MVP2 완료 | 77개 | 117개 | 139개 | 333개 |
| MVP3 완료 | 89개 | 155개 | 139개 | 383개 |
| 증가 | +12개 | +38개 | 0개 | +50개 |

iOS가 가장 많이 증가했다. MVP3에서 AuthFeatureTests 보강 + AppleSignInHelper 단위 테스트 + API 어댑터 테스트가 추가됐기 때문. 서버는 M1에서 엔드포인트 추가 시 테스트를 이미 작성했고, MVP3에서 서버 로직 변경이 없었으므로 그대로.

---

## 잘한 것 (Keep)

### 1. Mock-First 병렬 개발 패턴 확립

"외부 의존 0, 충돌 0"을 목표로 Mock으로 UI 완성 후 실 어댑터로 swap하는 패턴이 완벽하게 동작했다. fixture JSON이 웹/iOS Mock의 공통 기준 데이터 역할을 했고, 통합 시 스키마 불일치가 전혀 없었다.

이 패턴은 향후 MVP4에서도 새 기능 추가 시 기본 흐름이 된다.

### 2. generate_apple_secret.js — 인프라 스크립트화

Apple OAuth Client Secret JWT 생성을 Node.js 스크립트로 자동화했다. 6개월 만료마다 재실행하면 되고, 의존성 없이 Node.js built-in `crypto`만 사용. **비개발자도 실행 가능한 수준으로 추상화.**

### 3. scripts/deploy.sh — 통합 배포 원클릭화

iOS 시뮬레이터 + Rust API (Docker) + 웹 프론트 (Docker)를 단일 스크립트로 배포. `--target` 옵션으로 선택적 배포도 지원. MVP3 전체 동작 확인을 단 한 번의 명령으로 수행.

### 4. 크로스 플랫폼 계정 연동 자동 검증

기획 시 "동일 계정으로 웹/iOS 모두 로그인 가능"이라는 완료 기준을 세웠는데, 실제로 웹에서 Apple 로그인 후 iOS에서 로그인 시 태그/프로필이 그대로 연동되는 것을 확인했다. Supabase 단일 백엔드의 가장 큰 장점이 실증됐다.

### 5. 회고 주도 아키텍처 개선

MVP2 회고에서 지적한 `LoadingPhase Bool 4개 → enum` 개선이 M3에서 실제로 흡수됐다. 회고에 적은 부채가 다음 MVP에서 해소되는 선순환이 작동하고 있다.

---

## 아쉬운 것 (Problem)

### 1. iOS 요약 60s timeout UX 미해결

M3 전환 시 요약 중 60초가 넘으면 클라이언트가 타임아웃을 표시하지 못하고 "요약 중..." 상태가 유지된다. 서버는 정상 처리하지만 클라이언트 UX가 깨진다. MVP2 부채로 인식하고 MVP3에 흡수하려 했지만 시간 상 MVP4로 이월.

### 2. Supabase Manual Linking — Beta 미졸업

이메일로 가입한 계정과 Apple로 로그인한 계정을 병합하는 Manual Linking이 Supabase Beta 상태. 현재는 Supabase 자동 링크(`account_linking_enabled`)에만 의존. 동일 이메일이면 자동 병합되지만, privaterelay 이메일과 실제 이메일은 다르게 처리된다.

### 3. Apple Secret 갱신 알림 체계 없음

`generate_apple_secret.js`로 생성한 JWT는 6개월 만료다. 만료일(2026-10-08경)에 자동 알림이 없어, 잊으면 Apple 로그인이 전체 중단된다. 캘린더 등록이나 cron 기반 만료 감지 체계가 필요.

### 4. wip 브랜치 검증 없이 방치 후 재구현 비용

`wip/apple-login-draft`가 미검증 상태로 방치됐다가 M4에서 다시 TDD로 재구현했다. 분기 당시 검증을 마쳤거나 WIP 메모를 남겼다면 재구현 비용을 줄일 수 있었다. **WIP 브랜치는 반드시 상태 메모 + 검증 여부 기록 필요.**

---

## 놀랐던 것 (Surprise)

### 1. Hide My Email이 자동 적용

Apple 로그인 시 "이메일 숨기기" 선택지가 표시될 것으로 예상했지만, Supabase OAuth 방식에서는 자동으로 `@privaterelay.appleid.com` 릴레이 주소가 적용됐다. 경고 메시지 UI(ST-W3)를 기획했지만 불필요하다는 결론으로 삭제. **사용자 경험이 예상보다 단순하고 깨끗했다.**

### 2. iOS 테스트 코드가 소스 코드를 넘어섬

iOS 소스 3,338줄 vs 테스트 3,388줄 — 테스트가 소스를 50줄 초과. MVP2에서 93% 비율이었는데 MVP3에서 역전됐다. AuthFeature 테스트 보강 + API 어댑터 테스트 추가의 결과. **TDD 원칙이 누적될수록 테스트 코드가 소스 코드와 동등한 비중을 가지게 된다.**

### 3. 크로스 플랫폼 계정 연동이 "공짜"

웹과 iOS가 동일 Supabase 프로젝트를 쓰면 계정 연동은 별도 구현 없이 자동이다. 실제로 웹에서 Apple 로그인 → 태그 선택 → iOS에서 Apple 로그인 → 온보딩 없이 바로 피드 진입. 이것이 Supabase 단일 백엔드 전략의 핵심 ROI다.

---

## MVP2 → MVP3 성장 비교

| 관점 | MVP2 | MVP3 |
|------|------|------|
| 핵심 목표 | iOS 네이티브 앱 | 웹+iOS 통합 + Apple 로그인 |
| 코드 (누적) | 9,999줄 | 11,902줄 |
| 테스트 (누적) | 333개 | 383개 |
| 병렬 개발 | 미적용 | worktree 2개 병렬 (M2/M3) |
| 배포 | 수동 | scripts/deploy.sh 원클릭 |
| 인증 | Supabase SDK (웹/iOS 별도) | Apple 로그인 + 크로스 플랫폼 계정 연동 |
| 핵심 교훈 | "패턴이 동일하면 플랫폼은 장벽이 아니다" | "Mock-First 병렬 개발이 통합 비용을 낮춘다" |

---

## MVP4 부채 및 방향 제언

### 이월 부채

| # | 이슈 | 우선순위 |
|---|------|----------|
| 1 | iOS 요약 60s timeout — 클라이언트 타임아웃 + 재시도 버튼 | High |
| 2 | Apple Secret 갱신 알림 (6개월 만료 2026-10-08경) | High |
| 3 | Supabase Manual Linking — 이메일+Apple 계정 병합 (Beta 졸업 후) | Medium |
| 4 | alert → 인라인 에러 UX 개선 (iOS 로그인 에러 표시) | Low |

### MVP4 방향

**MVP4는 새 기능 추가가 아니라 부채 해소 + 품질 개선이다.**

MVP1~MVP3를 거치며 쌓인 기술 부채와 품질 이슈를 먼저 해소하고, 이후 서버-웹-앱 추가 연동(MVP5+)의 단단한 기반을 만드는 것이 목표다.

이월 부채 외 추가 검토 항목:
- iOS UITest/E2E 보강 (MVP2 이월)
- iOS 커버리지 측정 파이프라인 (MVP2 이월)
- 태그 stale article 이슈 (MVP3 M2 이월)
- 웹 UX 개선 (에러 표시, 로딩 상태 등)

학습 기능(스크랩, 퀴즈, 리포트 등)은 MVP5 이후 서버-웹-앱 연동 기반 위에서 진행.

---

## 회고를 마치며

MVP3는 "만든 것들을 하나로 묶는" 프로젝트였다. MVP1의 웹, MVP2의 iOS가 각자 Supabase를 직접 부르던 구조를 Rust API 서버 하나로 통일했고, 그 위에 Apple 로그인으로 두 플랫폼이 동일 계정을 공유하게 됐다.

병렬 worktree 전략이 처음 실전 투입됐고, Mock-First 흐름이 이론에서 실증으로 전환됐다. 두 개의 탭에서 웹과 iOS가 동시에 개발되면서 충돌이 0건이었던 것은 "외부 의존을 Mock으로 격리하면 병렬 작업이 가능하다"는 원칙이 실제로 동작함을 증명한다.

11,902줄 코드, 383개 테스트, 3개 플랫폼. 에코 서버 패턴 + Mock-First 병렬 개발이 Frank의 개발 방법론으로 완전히 자리잡았다.

다음은 MVP4다. "읽는 앱"에서 "배우는 앱"으로의 전환.

> *"Mock-First 병렬 개발이 통합 비용을 낮춘다" — 2026.04.08, MVP3 완료*
