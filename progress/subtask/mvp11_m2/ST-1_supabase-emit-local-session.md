# ST-1: Supabase emitLocalSessionAsInitialSession 콜드 스타트 버그 수정 (BUG-001)

> MVP: 11 | 마일스톤: M2 | 브랜치: fix/mvp11-m2-ios-init

## 목표

앱 첫 실행(콜드 스타트) 시 Supabase 세션 복원 완료 전 API 요청으로 발생하는
"태그를 불러오지 못했습니다" 에러를 수정한다.

`SupabaseClient` 초기화 시 `emitLocalSessionAsInitialSession: true` 옵션을 추가하여
로컬 캐시 세션을 즉시 방출, 첫 API 요청부터 토큰이 포함되도록 한다.

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `AppDependencies.swift` | `SupabaseClient` 생성 시 `options: SupabaseClientOptions(auth: .init(emitLocalSessionAsInitialSession: true))` 추가 |
| `AppDependenciesTests.swift` (신규) | `bootstrap()` 및 옵션 적용 검증 단위 테스트 |

## Feature List
<!-- size: 중형 | count: 15 | skip: false -->

### 기능
- [x] F-01 SupabaseClient 초기화 시 emitLocalSessionAsInitialSession: true 옵션 추가
- [x] F-02 콜드 스타트 시 로컬 캐시 세션 즉시 방출 — 첫 API 요청부터 토큰 포함
- [x] F-03 옵션 추가 후 기존 로그인/로그아웃 흐름 유지
- [x] F-04 세션 만료 시 SDK 자동 갱신(autoRefreshToken) 기본값 유지
- [x] F-05 FRANK_USE_MOCK=1 환경에서 옵션 변경 영향 없음 확인

### 엣지
- [x] E-01 첫 설치(로컬 캐시 세션 없음) — 즉시 방출할 세션 없어도 에러 없이 로그인 화면 노출
- [x] E-02 만료된 로컬 세션 즉시 방출 후 API 401 → SDK 자동 refresh 후 재시도
- [x] E-03 로그아웃 후 재로그인 시 새 세션 정상 방출

### 에러
- [x] R-01 refresh 실패 시 로그아웃 상태 전이 (기존 동작 유지)
- [x] R-02 세션 즉시 방출 후 API 에러 발생 시 기존 에러 처리 흐름 유지

### 테스트
- [x] T-01 AppDependencies.bootstrap() → .ready 반환 + SupabaseAuthAdapter 생성 확인 (단위)
- [x] T-02 콜드 스타트 E2E — 앱 재실행 후 "태그를 불러오지 못했습니다" 미노출
- [x] T-03 로그인 유지 E2E — 앱 재실행 후 피드 즉시 로드 확인
- [x] T-04 기존 iOS 테스트 전체 통과 (230/230 회귀 없음)

### 플랫폼
- [-] N/A (기존 파일만 수정, 신규 파일 없음 — Tuist 재생성 불필요) P-01 Tuist 재생성 완료 (신규 테스트 파일 포함)
