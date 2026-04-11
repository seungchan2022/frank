# M2: iOS UX 개선

> 프로젝트: Frank MVP4
> 상태: 완료
> 완료일: 260408
> 예상 기간: 1~2일
> 의존성: M1 완료 후
> 실행: `/workflow "MVP4 M2: iOS UX 개선"`

---

## 목표

iOS 앱의 두 가지 UX 결함을 해소한다.
요약 30s 이상 소요 시 타임아웃 UI를 표시하고 (전체 요약 기준, 단일 요약 전환 시 10~15s로 축소 예정), 로그인 에러를 alert 대신 인라인으로 표시한다.

---

## 서브태스크

| # | 서브태스크 | 유형 | 영향 범위 |
|---|-----------|------|---------|
| 1 | 요약 timeout UX — 타임아웃 감지 + 재시도 버튼 | feature | `FeedFeature`, `FeedView`, `APICollectAdapter` |
| 2 | 로그인 에러 인라인 표시 — alert → 폼 하단 텍스트 | feature | `LoginView`, `EmailSignInSheet` |

---

## 성공 기준 (DoD)

- [x] 요약이 30s 이상 걸릴 때 "요약이 오래 걸리고 있어요" + 재시도 버튼 표시
- [x] 재시도 버튼 탭 시 요약 재요청 동작
- [x] 로그인 실패 시 `.alert()` 대신 버튼 하단 인라인 에러 메시지 표시
- [x] 기존 iOS 테스트 155개 전체 통과
- [x] 신규 테스트 5개 추가 (isSummarizingTimeout 초기값, 소프트 타이머 발화, 타임아웃 이내 완료, retrySummarize, transport timedOut 분기)
- [x] UITest 2개 통과 (CrossFeatureFlow, OnboardingFlow)

---

## 서브태스크 상세

### ST-1: 요약 timeout UX

**현황**: 요약 60s+ 소요 시 "AI가 요약하고 있어요..." 상태가 무한 유지됨.

**구현 결과:**

1. `APICollectAdapter.triggerSummarize()` — `timeoutInterval = 60` 명시 (transport 하드 timeout)
2. `FeedFeature` — `isSummarizingTimeout: Bool` + `summarizeTimeoutSeconds: Double = 30` 추가
3. `FeedFeature.collectAndSummarize()` — triggerSummarize 호출 시 30s 소프트 타이머 병행, `URLError.timedOut` 별도 분기
4. `FeedView` — `isSummarizingTimeout == true`일 때 타임아웃 배너 + 재시도 버튼 표시
5. `FeedAction.retrySummarize` 추가 → `isSummarizingTimeout` 초기화 + `collectAndSummarize()` 재호출

**UX 임계값**: 30s 소프트(타이머 기반 UI 전환) + 60s 하드(URLSession transport timeout)
- 30s: "요약이 오래 걸리고 있어요" 배너 + [다시 시도] 버튼 표시
- 60s: URLError.timedOut → isSummarizingTimeout = true, errorMessage 없음 (배너가 이미 표시 중)

---

### ST-2: 로그인 에러 인라인 표시

**현황**: `LoginView` + `EmailSignInSheet` 모두 `.alert()` 방식.

**구현 방향 (Step-5 리뷰 반영):**

1. `LoginView` — `.alert()` 블록(L57~L66) 제거 + Apple Sign In 버튼 하단 인라인 에러 Text 추가
   ```swift
   if let errorMsg = feature.error?.localizedDescription {
       Text(errorMsg).foregroundStyle(.red).font(.footnote)
   }
   ```
2. `EmailSignInSheet` — `.alert()` 블록(L76~L85) 제거 + 폼 내부 하단 Section에 인라인 에러 Text 추가
3. `AuthFeature` 수정 불필요 — `feature.error: Error?` 이미 존재, `clearError()` 이미 존재
4. Apple Sign In 실패(`feature.send(error)`)도 동일 `feature.error` 경로 → 자동 처리됨

**테스트 계획**:
- `AuthFeatureTests`: 기존 로그인 실패 테스트 → `feature.error != nil` 이미 검증됨
- View 레벨 변경이므로 snapshot/render 테스트 추가 검토

---

## 리뷰 결과 (Step-5, 260408)

### Claude 리뷰 (문서 일치성)

- ST-1: 문서의 "URLSession timeoutIntervalForRequest" 구현 위치를 `APICollectAdapter`로 구체화. `FeedFeature`에 `isSummarizingTimeout` 상태 추가 필요 반영.
- ST-2: `LoginView` + `EmailSignInSheet` 둘 다 수정 범위 확인. `AuthFeature` 무수정 확인.
- 규칙: TDD 원칙 — 테스트 먼저 작성 필수. `MockCollectPort` 확장 필요.

### Codex 리뷰 (기술적 타당성)

- ST-1: timeout 위치는 어댑터(`APICollectAdapter`)가 적합. `Task.withTimeout`을 Feature에 넣는 건 transport concern 누출. URLError.timedOut 분기 + 재시도 버튼은 서버가 미요약 기사만 재처리하므로 중복 위험 낮음.
- ST-2: `LoginView.alert` 제거만으로는 부족, `EmailSignInSheet`도 병행 처리 필수. Apple Sign In 실패 경로도 동일 `feature.error` 사용 확인됨.
- 타임아웃 값: 30s 소프트(UX) + 60s 하드(transport) 분리 권장.

### 최종 결정: 조건부 승인

**조건**:
1. ST-1 구현 전 `MockCollectPort`에 delay/timeout 시뮬레이션 추가 후 테스트 먼저 작성
2. 30s 소프트 타이머와 60s 하드 transport timeout을 명확히 분리 구현
3. `EmailSignInSheet` alert 제거를 ST-2 범위에 명시적 포함 (기존 문서 누락)
