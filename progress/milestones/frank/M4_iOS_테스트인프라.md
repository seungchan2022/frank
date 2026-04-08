# M4: iOS 테스트 인프라

> 프로젝트: Frank MVP4
> 상태: 대기
> 예상 기간: 2~3일
> 의존성: M2 완료 후 (LoginView 변경 안정화 후 TC-01 작성 가능)
> 실행: `/workflow "MVP4 M4: iOS 테스트 인프라"`

---

## 목표

iOS 테스트 인프라를 구축한다.
커버리지 측정을 자동화하고, 핵심 E2E 시나리오 4개를 XCUITest로 자동화한다.

---

## 서브태스크

| # | 서브태스크 | 유형 | 비고 |
|---|-----------|------|------|
| 1 | iOS 커버리지 측정 파이프라인 | chore | scripts/coverage.sh |
| 2 | UITest TC-01: 이메일 로그인 → 피드 진입 | feature | M2 완료 후 안정적 작성 가능 |
| 3 | UITest TC-02: 온보딩 플로우 (신규 사용자) | feature | |
| 4 | UITest TC-03: 기사 상세 → 요약 요청 | feature | timeout UI 포함 |
| 5 | UITest TC-04: 설정 → 태그 관리 → 로그아웃 | feature | |

---

## 성공 기준 (DoD)

- [ ] `scripts/coverage.sh` 실행으로 iOS 커버리지 수치 출력 (목표 90% 이상)
- [ ] TC-01~04 XCUITest 시나리오 전체 통과 (시뮬레이터 기준)
- [ ] 기존 iOS 테스트 155개+ 전체 통과

---

## 서브태스크 상세

### ST-1: 커버리지 파이프라인

```bash
# scripts/coverage.sh
# xcodebuild test -enableCodeCoverage YES
# xcrun xccov view --report --json → 수치 파싱 + 90% 미만 시 경고
```

---

### ST-2~5: UITest 시나리오

```
TC-01: 앱 시작 → 로그인 화면 → 이메일/비밀번호 입력 → 피드 표시 확인
TC-02: 태그 선택 화면 → 최소 1개 선택 → 피드 진입
TC-03: 피드에서 기사 탭 → 상세 화면 → 요약 버튼 → 요약 표시 또는 timeout UI
TC-04: 설정 탭 → 태그 추가/삭제 → 로그아웃 → 로그인 화면 복귀
```

> Apple 로그인은 XCUITest Mock 어려움 → 전 시나리오 이메일 로그인 기준.
