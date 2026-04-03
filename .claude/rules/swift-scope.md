# Swift 규칙 (자동 적용)

이 파일은 Swift 코드 (iOS/macOS 앱) 경로 작업 시 자동 로드된다.
프로젝트의 `.claude/rules/` 디렉토리에 복사하여 사용한다.

## 코드 스타일

- Swift 공식 API Design Guidelines 준수
- `struct` 우선, `class`는 참조 시맨틱 필요 시에만
- Protocol-oriented programming 패턴 권장
- SwiftUI 선언적 UI 패턴 사용
- Combine/async-await 기반 비동기 처리

## 아키텍처

- MVVM 또는 TCA (The Composable Architecture) 패턴
- 의존성 주입: Protocol 기반 추상화
- 네트워크 레이어: URLSession 또는 Alamofire
- 데이터 모델: Codable 프로토콜 필수

## 필수 검증 (커밋 전)

```bash
# 1. 빌드
xcodebuild build -scheme {스킴명} -destination 'platform=iOS Simulator,name=iPhone 16'

# 2. 린트
swiftlint lint --strict

# 3. 테스트
swift test
# 또는
xcodebuild test -scheme {스킴명} -destination 'platform=iOS Simulator,name=iPhone 16'
```

**세 가지 모두 통과해야 커밋 가능.**

## 타입 안전 (필수)

- `Any`, `AnyObject` 사용 최소화 — Generic 또는 Protocol 활용
- Force unwrap (`!`) 금지 (테스트 코드 제외) — `guard let`, `if let`, `??` 사용
- Force cast (`as!`) 금지 — `as?`와 guard 조합
- `@objc` 최소화 — Swift native API 우선

## 테스트 규칙

- XCTest 또는 Swift Testing 프레임워크 사용
- Mock은 Protocol 기반으로 생성
- 네트워크 테스트: URLProtocol 서브클래싱으로 Mock
- UI 테스트: XCUITest 또는 ViewInspector
- 커버리지 **90% 이상** 유지

## 보안

- Keychain으로 민감 정보 저장 (UserDefaults 금지)
- ATS (App Transport Security) 예외 최소화
- 인증 토큰은 메모리에서만 관리, 디스크 캐시 금지

## SwiftUI 패턴

```swift
// 상태 관리
@State private var count = 0
@StateObject private var viewModel = MyViewModel()
@EnvironmentObject var settings: AppSettings

// 비동기 데이터 로딩
.task { await loadData() }
```

## 디렉토리 구조 (권장)

```
{App}/
├── App/               # App entry point
├── Features/          # 기능별 모듈
│   └── {Feature}/
│       ├── Views/     # SwiftUI Views
│       ├── ViewModels/ # MVVM ViewModel
│       └── Models/    # Data Models
├── Core/              # 공통 유틸, 확장
├── Services/          # 네트워크, 인증, 저장소
├── Resources/         # Assets, Localizable
└── Tests/             # 테스트
```
