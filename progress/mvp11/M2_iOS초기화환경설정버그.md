# M2: iOS 초기화·환경설정 버그 수정

> 프로젝트: Frank MVP11
> 상태: 대기
> 예상 기간: 1~2일
> 의존성: 없음

## 목표

앱 첫 실행 시 세션 복원 완료 전 API 요청으로 발생하는 태그 로딩 에러(BUG-001)와,
시뮬레이터/실기기 전환 시 서버 URL 고정으로 인한 연결 실패(BUG-002)를 수정한다.

## 성공 기준 (Definition of Done)

- [ ] 앱 첫 실행(콜드 스타트) 시 "태그를 불러오지 못했습니다" 에러 메시지 미노출
- [ ] `#if targetEnvironment(simulator)` 분기로 시뮬레이터는 `localhost:8080`, 실기기는 xcconfig 값 사용
- [ ] 시뮬레이터 → 실기기 전환 후 앱 정상 실행 확인 (연결 성공)
- [ ] 기존 iOS 테스트 전체 통과 (회귀 없음)

## 아이템

| # | 아이템 | 유형 | 실행 스킬 | 상태 |
|---|--------|------|----------|------|
| 1 | BUG-001: Supabase 세션 복원 완료 대기 후 API 요청 | feature | /workflow | 대기 |
| 2 | BUG-002: ServerConfig 시뮬레이터/실기기 URL 자동 분기 | feature | /workflow | 대기 |

## 수정 방향

### BUG-001 세션 초기화 순서

**현재 흐름**:
```
FrankApp.init() → AppDependencies.live() → SupabaseClient 생성
RootView.body → AuthFeature.checkSession() → client.auth.session (async)
   └─ [이 타이밍에 다른 곳에서 API 호출 시작] → 토큰 없어 401
```

**수정 옵션 A (권장)**: `SupabaseClient` 초기화 시 `emitLocalSessionAsInitialSession: true` 옵션 적용
- Supabase iOS SDK의 `AuthClientConfiguration` 또는 `SupabaseClientOptions`에 플래그 설정
- 세션이 즉시 동기적으로 복원되어 첫 요청부터 토큰 포함

**수정 옵션 B (폴백)**: `AuthFeature.checkSession()` 완료 신호를 기다린 후 태그/피드 로드 시작
- `RootView`에서 `.authenticated` 상태 전이 후에만 하위 뷰 생성

### BUG-002 URL 분기

**수정**: `ServerConfig.swift`에 컴파일 타임 분기 추가

```swift
static var live: ServerConfig {
    #if targetEnvironment(simulator)
    return ServerConfig(url: URL(string: "http://localhost:8080")!)
    #else
    // 기존 Info.plist → Secrets.plist → localhost 폴백 순서 유지
    ...
    #endif
}
```

## 리스크

| 리스크 | 영향(H/M/L) | 대응 |
|--------|------------|------|
| SDK 옵션명 변경 (버전별 차이) | M | 공식 문서 + context7 MCP로 확인 후 적용 |
| simulator 분기 시 실기기 테스트 경로 변경 | L | 실기기 xcconfig 경로 기존 유지, 분기만 추가 |

---

## KPI (M1)

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| M1 DoD 테스트 통과 | `xcodebuild test -workspace Frank.xcworkspace -scheme Frank -destination 'platform=iOS Simulator,name=iPhone 17 Pro'` | 전체 통과 | Hard | — |
| BUG-001 재현 0건 | 시뮬레이터 콜드 스타트 3회 수동 확인 | 0건 | Hard | 항상 재현 |
| BUG-002 재현 0건 | 시뮬레이터에서 API 연결 성공 수동 확인 | 0건 | Hard | 항상 재현 |
