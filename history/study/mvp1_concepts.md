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

### 자가 확인 기록

Q1. 웹 이메일 로그인에서 signInWithPassword()를 실행하는 주체는?
사용자 답: A (SvelteKit 웹)
정답: C (+page.server.ts)
포인트: 브라우저 JS는 httpOnly 쿠키를 설정할 수 없어서 인증 처리를 서버에서 해야 함

Q2. 로그인 후 JWT가 httpOnly 쿠키에 저장되는 이유는?
사용자 답: B
정답: B ✅
포인트: httpOnly로 JS 접근 차단 → XSS 방어. localStorage 대비 핵심 보안 이점

Q3. Rust API 호출 시 인증 정보를 전달하는 방식은?
사용자 답: B
정답: B ✅
포인트: 쿠키는 브라우저-웹서버 간에만 자동 전송. 외부 Rust API엔 Bearer 헤더로 명시적 전달 필요
