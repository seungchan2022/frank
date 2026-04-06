# M4: Apple 로그인 통합

> 상태: 📋 기획 완료
> 범위: 서버 + 웹 + iOS
> 의존: M2, M3
> 참조: wip/apple-login-draft 브랜치 (커밋 2fbcedc)

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
- **상태**: [ ]
- **설명**: App ID에 Sign in with Apple capability, Service ID (웹용), 리다이렉트 URL
- **의존**: 없음

### ST-2: Supabase 대시보드 Apple Provider 설정
- **유형**: chore
- **상태**: [ ]
- **설명**: Supabase Auth → Apple provider 활성화, Client ID/Secret 등록
- **의존**: ST-1

### ST-3: iOS Apple 로그인 구현
- **유형**: feature
- **상태**: [ ]
- **설명**: wip/apple-login-draft 참고하여 재구현. LoginView에 SignInWithAppleButton 추가.
- **산출물**: LoginView.swift, AppleSignInHelper.swift 수정
- **의존**: ST-2

### ST-4: 웹 Apple 로그인 구현
- **유형**: feature
- **상태**: [ ]
- **설명**: 로그인 페이지에 Apple 로그인 버튼 추가. Supabase OAuth flow 사용.
- **산출물**: `web/src/routes/login/+page.svelte` 수정
- **의존**: ST-2

### ST-5: 크로스 플랫폼 테스트
- **유형**: test
- **상태**: [ ]
- **설명**: iOS 시뮬레이터 + 웹 브라우저에서 Apple 로그인 동작 확인
- **산출물**: 테스트 결과 스크린샷
- **의존**: ST-3, ST-4

## 의존 그래프

```
ST-1 (Developer 설정) → ST-2 (Supabase 설정) → ST-3 (iOS)  → ST-5 (테스트)
                                              → ST-4 (웹)   ↗
```

## 완료 기준

- [ ] iOS 시뮬레이터에서 Apple 로그인 동작
- [ ] 웹 브라우저에서 Apple 로그인 동작
- [ ] 동일 계정으로 웹/iOS 모두 로그인 가능
- [ ] 기존 이메일 로그인 영향 없음
