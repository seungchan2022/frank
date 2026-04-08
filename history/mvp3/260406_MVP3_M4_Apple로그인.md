# M4: Apple 로그인 통합

> 상태: ✅ 완료 (260408)
> 범위: 웹 + iOS (서버 변경 없음)
> 의존: M2, M3
> 브랜치: feature/260408_m4_apple_login

## 목표

Apple Sign In을 서버+웹+iOS 3개 플랫폼에서 동시 구현.

## 플랫폼별 구현

### iOS (참조 커밋 활용)
- ASAuthorizationController → idToken + nonce 획득
- Supabase Auth SDK: `signInWithIdToken(OpenIDConnect)` 호출
- **참조**: `git diff 2fbcedc~1..2fbcedc` (LoginView 에러 핸들링, Config 변경 등)

### 웹
- Supabase Auth SDK: OAuth PKCE flow (provider: "apple")
- `supabase.auth.signInWithOAuth({ provider: 'apple' })` 또는 ID 토큰 방식
- 콜백 URL 처리

### 서버
- 추가 작업 없음 — JWT 검증 로직은 동일 (Supabase Auth가 발급한 JWT)
- Apple Developer 설정: Service ID, 리다이렉트 URL 등록

## 서브태스크

### ST-1: Apple Developer 설정
- **유형**: chore
- **상태**: [x]
- **설명**: Service ID(com.frank.web) 생성, 리다이렉트 URL 등록, .p8 Key 발급
- **의존**: 없음

### ST-2: Supabase 대시보드 Apple Provider 설정
- **유형**: chore
- **상태**: [x]
- **설명**: Client IDs: `com.frank.web,dev.frank.app` (순서 중요), Secret Key JWT 등록
- **참고**: `scripts/generate_apple_secret.js` 로 JWT 생성 (6개월 만료)
- **의존**: ST-1

### ST-3: iOS Apple 로그인 에러 핸들링 강화
- **유형**: feature
- **상태**: [x]
- **설명**: AuthFeature.send(_ error:), LoginView canceled 분기, EmailSignInSheet alert, FrankApp .authenticating → LoginView
- **산출물**: AuthFeature.swift, LoginView.swift, EmailSignInSheet.swift, FrankApp.swift
- **의존**: ST-2

### ST-4: 웹 Apple 로그인 구현
- **유형**: feature
- **상태**: [x]
- **설명**: appleOAuth form action, /auth/callback +server.ts, Apple 버튼 non-enhanced form, PUBLIC_PATHS 추가
- **산출물**: +page.server.ts, +page.svelte, auth/callback/+server.ts, +layout.server.ts
- **결정사항**: use:enhance 바깥 별도 form으로 분리 (OAuth redirect와 enhanced fetch 충돌 방지)
- **의존**: ST-2

### ST-5: 크로스 플랫폼 테스트
- **유형**: test
- **상태**: [x]
- **설명**: 웹 Playwright 동작 확인, iOS 시뮬레이터 동작 확인, 크로스 플랫폼 계정 연동 확인
- **결과**: 웹/iOS 동일 Apple ID → 동일 Supabase 사용자, 태그/프로필 자동 공유
- **의존**: ST-3, ST-4

## 의존 그래프

```
ST-1 (Developer 설정) → ST-2 (Supabase 설정) → ST-3 (iOS)  → ST-5 (테스트)
                                              → ST-4 (웹)   ↗
```

## 완료 기준

- [x] iOS 시뮬레이터에서 Apple 로그인 동작
- [x] 웹 브라우저에서 Apple 로그인 동작
- [x] 동일 계정으로 웹/iOS 모두 로그인 가능
- [x] 기존 이메일 로그인 영향 없음

## MVP4 이월 부채

| 항목 | 이유 |
|------|------|
| alert → 인라인 에러 UX | 현재 alert 방식 동작에 문제 없음, UX 개선은 별도 |
| Manual Linking (이메일+Apple 동일 계정) | Supabase Manual Linking API가 beta → 졸업 후 |
