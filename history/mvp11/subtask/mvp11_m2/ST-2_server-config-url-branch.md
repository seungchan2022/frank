# ST-2: ServerConfig 시뮬레이터/실기기 URL 자동 분기 (BUG-002)

> MVP: 11 | 마일스톤: M2 | 브랜치: fix/mvp11-m2-ios-init

## 목표

`ServerConfig.swift`에 `#if targetEnvironment(simulator)` 컴파일 타임 분기를 추가하여
시뮬레이터는 `localhost:8080`, 실기기는 xcconfig/Secrets.plist 값을 자동으로 사용한다.
설정 오류 시 에러 타입으로 전파하고 사용자에게 알럿을 노출한다.

## 인터뷰 확정 사항 (step-5 리뷰 반영)

- URL 파싱: `guard let` + `ServerConfigError.invalidURL` throw (force-unwrap 금지)
- `live() throws` 함수로 전환 (결정 1-B: 근본 수정)
- `SupabaseAuthAdapter` 기본값 `= .live` 제거 — dead code, 호출부는 이미 명시적 전달
- 에러 처리: `AppDependencies.live()` catch → 폴백 URL 없음, `configError` 상태만 세팅 (결정 2-A)
- UI: `RootView`에 `.configError` case 추가 → 전용 에러 화면 (결정 3-B, FrankApp `.alert()` 사용 안 함)
- 변경 파일: `ServerConfig.swift`, `SupabaseAuthAdapter.swift`, `AppDependencies.swift`, `RootView.swift` + `ServerConfigTests.swift` 신규

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `ServerConfig.swift` | `make(urlString:) throws` 헬퍼 + `live() throws` 전환 + `#if simulator` 분기 |
| `SupabaseAuthAdapter.swift` | 기본값 `serverConfig: ServerConfig = .live` 제거 |
| `AppDependencies.swift` | `try ServerConfig.live()` catch → 로그 + `configError` 반환 (폴백 URL 없음) |
| `RootView.swift` | `.configError` case 추가 + 전용 에러 화면 렌더링 |
| `ServerConfigTests.swift` (신규) | 분기·에러 단위 테스트 |

## Feature List
<!-- size: 중형 | count: 20 | skip: false -->

### 기능
- [x] F-01 `ServerConfig.make(urlString:)` throws 헬퍼 함수 추가 및 `ServerConfigError.invalidURL` 정의
- [x] F-02 `ServerConfig.live()` 를 `throws` 함수로 전환
- [x] F-03 `#if targetEnvironment(simulator)` 분기로 시뮬레이터 시 `localhost:8080` 반환
- [x] F-04 실기기 경로: Info.plist → Secrets.plist → 에러 (폴백 URL 없음)
- [x] F-05 `AppDependencies.bootstrap()` — `AppBootstrap` enum 반환, catch → 로그 + `.configError`
- [x] F-06 `FrankApp`에 `ConfigErrorView` 추가 + `AppBootstrap` 분기 렌더링

### 엣지
- [x] E-01 시뮬레이터에서 `"http://localhost:8080"` 리터럴 파싱 실패 불가 확인 (dead path)
- [x] E-02 Info.plist `SERVER_URL` 키 없는 경우 Secrets.plist 경로로 폴오버
- [x] E-03 Secrets.plist 없는 경우 `invalidURL("")` 에러 발생
- [x] E-04 빈 문자열 `SERVER_URL` 값 → `invalidURL` 에러 발생 확인

### 에러
- [x] R-01 `invalidURL` 에러 발생 시 `Log.app.error` 로 에러 내용 기록
- [x] R-02 device catch 시 폴백 URL 없이 configError만 세팅 — API 어댑터 호출 안 됨
- [x] R-03 `.configError` 화면에서 앱이 빈 화면/빈 상태 없이 에러 화면 유지

### 테스트
- [x] T-01 시뮬레이터 빌드에서 `ServerConfig.live()` → `localhost:8080` 반환 확인 (6/6 통과)
- [x] T-02 빈 문자열로 `make(urlString:)` 호출 시 `invalidURL` throw 확인
- [x] T-03 기존 iOS 테스트 전체 회귀 없음 (227/227 통과)

### 플랫폼
- [x] P-01 `#if targetEnvironment(simulator)` 분기가 시뮬레이터 빌드에서 활성화 확인
- [x] P-02 실기기 빌드에서 simulator 분기 비활성화 확인 (xcconfig 경로 유지)
- [x] P-03 Tuist 재생성 완료 (신규 ServerConfigTests.swift 포함)

### 회귀
- [x] G-01 `ServerConfig`를 주입받는 모든 어댑터(`APITagAdapter` 등) 정상 동작 유지 (227/227)
