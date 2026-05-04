# MVP1 학습 기록

> 학습 시작: 2026-04-22
> 현재 완료 단계: 1단계

---

## 1단계: 로그인 (인증)

> 학습 일시: 2026-05-04
> 상태: ✅ 완료

### 핵심 개념 요약

- **+page.server.ts** — 로그인 폼 POST를 받아 signInWithPassword()/signInWithOAuth() 실행하는 서버 액션
- **hooks.server.ts** — 모든 요청을 가로채는 문지기. safeGetSession()으로 쿠키 JWT 검증, locals.supabase 초기화
- **locals.supabase** — hooks.server.ts가 쿠키 처리 설정으로 만든 SSR 클라이언트. 인증 함수 호출 시 JWT를 httpOnly 쿠키에 자동 저장
- **JWT** — 로그인 성공 시 Supabase Auth가 발급하는 서명된 토큰. 서버가 DB 조회 없이 유저를 검증할 수 있게 해줌
- **httpOnly 쿠키** — JS 접근 차단 → XSS 공격으로 토큰을 훔칠 수 없음. 서버만 읽고 쓸 수 있음
- **safeGetSession** — hooks.server.ts에 정의된 커스텀 헬퍼. getSession(쿠키에서 JWT 추출) + getUser(Supabase 재검증) 묶음
- **Bearer 토큰** — SvelteKit 서버가 쿠키에서 JWT를 꺼내 Authorization 헤더에 담아 Rust API로 전달하는 방식
- **Rust require_auth** — Axum Extension으로 구현. Bearer 토큰을 Supabase에 재검증해서 유효한 유저만 통과시킴
- **iOS Keychain** — iOS SDK가 JWT를 저장하는 암호화 보관함. 웹의 httpOnly 쿠키와 같은 역할
- **Apple OAuth** — 브라우저/iOS 시스템이 Apple 서버로 이동해서 인증 후 authorization code를 받아오는 흐름. 이메일 로그인과 달리 중간에 외부 이동이 끼어 있음

### 흐름 + 개념 상세

상세 내용은 `history/study/mvp1_stage1_notes.md` 및 `history/study/login_flow/index.html` 참조.

**커버한 흐름:**
- 웹 이메일 로그인 (① ~ ⑭)
- 웹 Apple OAuth 로그인 (① ~ ⑮)
- iOS 이메일 로그인 (① ~ ⑪)
- iOS Apple 로그인 (① ~ ⑫)

**핵심 차이점:**
- 이메일 로그인: +page.server.ts → Supabase 직접 요청 → JWT 반환 → 쿠키 저장
- Apple OAuth: +page.server.ts → Apple URL로 redirect → 브라우저가 Apple로 이동 → /auth/callback으로 돌아와 code 교환 → JWT → 쿠키 저장
- iOS: 서버 레이어 없음. SDK가 토큰 저장·갱신·헤더 첨부 자동 처리
- iOS Apple: iOS 시스템(Face ID 팝업)이 직접 인증 처리 → SDK가 idToken을 Supabase JWT로 교환

### 자가 확인 기록 (2차, 7문제)

Q1. hooks.server.ts와 +page.server.ts의 역할 차이는?
사용자 답: B ✅
정답: B
포인트: hooks는 모든 요청 문지기, +page.server.ts는 특정 페이지 폼 제출 담당

Q2. signInWithPassword()를 실행하는 주체는?
사용자 답: C ✅
정답: C (+page.server.ts)
포인트: 브라우저 JS는 httpOnly 쿠키 설정 불가 → 반드시 서버에서 처리

Q3. safeGetSession()이 하는 일은?
사용자 답: B ✅
정답: B
포인트: getSession(JWT 추출) + getUser(Supabase 재검증) 두 단계를 묶은 커스텀 헬퍼

Q4. 이메일 로그인과 Apple OAuth의 핵심 차이는?
사용자 답: C ✅
정답: C
포인트: Apple OAuth는 브라우저가 Apple 서버로 직접 이동하는 과정이 중간에 끼어 있음

Q5. Rust API 호출 시 인증 정보 전달 방식은?
사용자 답: C ✅
정답: C (Bearer 헤더)
포인트: 쿠키는 브라우저-웹서버 간에만 전송. 외부 Rust API엔 Bearer 헤더로 명시적 전달

Q6. iOS Keychain과 웹 httpOnly 쿠키의 공통점은?
사용자 답: B ✅
정답: B
포인트: 둘 다 앱/브라우저 코드가 직접 접근할 수 없는 보안 저장소

Q7. locals.supabase가 JWT를 httpOnly 쿠키에 저장할 수 있는 이유는?
사용자 답: 모름
정답: B
포인트: hooks.server.ts에서 createServerClient()로 SSR 클라이언트 생성 시 쿠키 읽기/쓰기 핸들러를 설정함. 인증 함수 호출 시 이 핸들러가 자동으로 쿠키에 저장
