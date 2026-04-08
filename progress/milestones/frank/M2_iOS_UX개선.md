# M2: iOS UX 개선

> 프로젝트: Frank MVP4
> 상태: 대기
> 예상 기간: 1~2일
> 의존성: M1 완료 후
> 실행: `/workflow "MVP4 M2: iOS UX 개선"`

---

## 목표

iOS 앱의 두 가지 UX 결함을 해소한다.
요약 60s 이상 소요 시 타임아웃 UI를 표시하고, 로그인 에러를 alert 대신 인라인으로 표시한다.

---

## 서브태스크

| # | 서브태스크 | 유형 | 영향 범위 |
|---|-----------|------|---------|
| 1 | 요약 timeout UX — 타임아웃 감지 + 재시도 버튼 | feature | `SummarizeFeature` |
| 2 | 로그인 에러 인라인 표시 — alert → 폼 하단 텍스트 | feature | `AuthFeature`, `LoginView`, `EmailSignInSheet` |

---

## 성공 기준 (DoD)

- [ ] 요약이 30s(또는 60s) 이상 걸릴 때 "요약이 오래 걸리고 있어요." + 재시도 버튼 표시
- [ ] 재시도 버튼 탭 시 요약 재요청 동작
- [ ] 로그인 실패 시 `.alert()` 대신 폼 하단 인라인 에러 메시지 표시
- [ ] 기존 iOS 테스트 155개 전체 통과
- [ ] 새 기능 관련 테스트 추가

---

## 서브태스크 상세

### ST-1: 요약 timeout UX

**현황**: 요약 60s+ 소요 시 "요약 중..." 상태가 무한 유지됨.

**구현 방향:**
```swift
// URLSession timeoutIntervalForRequest 설정 (60s 기본, 샘플링 후 조정)
// SummarizeFeature: URLError.timedOut 분기
// → "요약이 오래 걸리고 있어요. 재시도할까요?" + 재시도 버튼
```

---

### ST-2: 로그인 에러 인라인 표시

**현황**: `LoginView` + `EmailSignInSheet` 모두 `.alert()` 방식.

**구현 방향:**
```swift
// LoginView: @State var errorMessage = ""
// 로그인 폼 하단 Text(errorMessage).foregroundStyle(.red)
// AuthFeature 에러 상태 직접 바인딩
```
