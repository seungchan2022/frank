# E2E 격리 전략 (DB 초기화)

> 작성일: 2026-04-29
> 현황: M1에서 문서화만. 실제 구현은 M4에서 진행.

## 목표

각 E2E 시나리오 실행 시 DB 상태가 이전 시나리오에 의해 오염되지 않도록 보장한다.

---

## 1. 격리 수준 (계층적 선택)

| 수준 | 방법 | 장점 | 단점 | 권장 시점 |
|------|------|------|------|----------|
| L0 | 테스트 시작 시 특정 데이터 직접 삽입 (no cleanup) | 구현 간단 | 순서 의존성 발생 | smoke 더미 단계 |
| L1 | beforeEach/afterEach에서 테스트 전용 데이터 삽입+삭제 | 빠른 실행 | 삭제 실패 시 오염 | M4 초기 |
| L2 | 테스트 전용 Supabase 브랜치 또는 schema | 완전 격리 | 인프라 비용 | M4 이후 |
| L3 | 트랜잭션 롤백 (Rust 서버 테스트 헬퍼) | 가장 안전 | 서버 코드 수정 필요 | 장기 |

**M4 기본 전략: L1 (beforeEach 데이터 삽입 + afterEach 삭제)**

---

## 2. 웹 E2E (Playwright)

### 현재 (M1 ~ M3): 더미 테스트만, DB 격리 불필요

```typescript
// smoke.spec.ts — 백엔드 연결 없음
test('smoke', async ({ page }) => {
  await page.goto('about:blank');
  expect(1).toBe(1);
});
```

### M4 이후: 실제 시나리오 격리 방법

```typescript
// e2e/helpers/db-reset.ts (M4에서 구현)
export async function resetTestUser(email: string) {
  // API 호출: DELETE /api/test/users/:email
  // 또는 Supabase Admin API 직접 호출
}

test.beforeEach(async ({ request }) => {
  await request.delete(`${BASE_URL}/api/test/seed`);
});

test.afterEach(async ({ request }) => {
  await request.delete(`${BASE_URL}/api/test/cleanup`);
});
```

### 환경 분리 원칙

- E2E 테스트는 반드시 **개발/스테이징 환경**에서만 실행
- 프로덕션 DB 대상 실행 절대 금지
- `BASE_URL` 환경변수로 환경 구분
- 테스트 전용 계정: `test@test.com` (기존 Frank 테스트 계정 재활용)

---

## 3. iOS UITest (XCUITest)

### 현재 (M1 ~ M3): 기존 4개 UITest 현황 파악 + 구조 정리

### M4 이후: iOS E2E 격리 방법

```swift
// E2ETestHelper.swift (M4에서 구현)
class E2ETestHelper {
    static func resetAppState() {
        // UserDefaults 초기화
        // Keychain 초기화
        // 서버 테스트 헬퍼 API 호출
    }
}

// 각 UITest의 setUp()에서 호출
override func setUp() {
    super.setUp()
    continueAfterFailure = false
    E2ETestHelper.resetAppState()
    app.launch()
}
```

### 시뮬레이터 상태 초기화 (기존 방법)

```bash
# 시뮬레이터 앱 데이터 초기화 (M4 스크립트화 예정)
xcrun simctl uninstall booted dev.frank.app
# 재설치는 xcodebuild test가 자동 처리
```

---

## 4. 서버 테스트 헬퍼 API (M4 이후 구현 예정)

E2E 시나리오에서 DB를 직접 조작하기 위한 **개발 전용** API 엔드포인트.

```
POST /api/test/seed      — 테스트 데이터 삽입
DELETE /api/test/cleanup — 테스트 데이터 전체 삭제
```

**보안 원칙:**
- `FRANK_ENV=test` 환경변수 설정 시에만 라우터 활성화
- 프로덕션 빌드에서는 라우터 완전 제거 (feature flag 아닌 compile-time 조건)

---

## 5. 현재 상태 (M1 완료 시점)

| 항목 | 상태 |
|------|------|
| 전략 문서화 | ✅ 완료 |
| L1 격리 구현 | ⏸ M4 예정 |
| 서버 테스트 헬퍼 API | ⏸ M4 예정 |
| iOS E2ETestHelper | ⏸ M4 예정 |
| 시뮬레이터 초기화 스크립트 | ⏸ M4 예정 |
