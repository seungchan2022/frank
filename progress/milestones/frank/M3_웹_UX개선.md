# M3: 웹 UX 개선

> 프로젝트: Frank MVP4
> 상태: 대기
> 예상 기간: 1~2일
> 의존성: M1 완료 후 (M2와 병렬 가능)
> 실행: `/workflow "MVP4 M3: 웹 UX 개선"`

---

## 목표

웹의 두 가지 UX 결함을 해소한다.
요약 타임아웃 감지를 추가하고, 에러/로딩 상태를 전 화면에서 일관되게 통일한다.

---

## 서브태스크

| # | 서브태스크 | 유형 | 영향 범위 |
|---|-----------|------|---------|
| 1 | 요약 timeout UX — AbortController + 재시도 버튼 | feature | `+page.svelte` handleSummarize(), `realClient.ts` request() |

> ST-2 ErrorBanner/Skeleton 공통화는 MVP5로 이동. 새 화면 추가 시 같이 구현하는 게 자연스러움.

---

## 성공 기준 (DoD)

- [ ] 요약 요청이 30s 이상 걸릴 때 타임아웃 UI + 재시도 버튼 표시
- [ ] 기존 웹 테스트 89개 전체 통과
- [ ] 새 타임아웃 관련 테스트 추가

---

## 서브태스크 상세

### ST-1: 요약 timeout UX

**현황**: `fetch()`에 타임아웃 처리 없음. `realClient.ts`의 `request()` 함수도 signal 파라미터 없음.

**구현 방향 (Step-5 리뷰 반영):**

1. `realClient.ts` — `request()` 함수에 optional `signal?: AbortSignal` 파라미터 추가 (공통 로직 최소 변경)
2. `ApiClient` 인터페이스(`client.ts`) — `summarizeArticles(signal?: AbortSignal): Promise<number>` 시그니처 확장
3. `mockClient.ts` — 인터페이스 변경에 맞춰 signal 파라미터 추가 (무시해도 됨)
4. `+page.svelte handleSummarize()` — AbortController 소유 + 30s setTimeout 설정:
   ```ts
   async function handleSummarize() {
       const controller = new AbortController()
       const timerId = setTimeout(() => controller.abort(), 30_000)
       summarizing = true
       summarizingTimeout = false
       error = null
       try {
           await apiClient.summarizeArticles(controller.signal)
           // 성공 후 기사 재fetch
       } catch (e) {
           if (e instanceof DOMException && e.name === 'AbortError') {
               summarizingTimeout = true
           } else {
               error = e instanceof Error ? e.message : 'Failed to summarize'
           }
       } finally {
           clearTimeout(timerId)
           summarizing = false
       }
   }
   ```
5. `+page.svelte` 상태 — `summarizingTimeout = $state(false)` 추가
6. UI — `summarizingTimeout == true`일 때 "요약이 오래 걸리고 있어요." + 재시도 버튼 렌더

**Svelte 5 반응성 주의**: `summarizingTimeout` 리셋은 handleSummarize 진입 시 원자적으로 처리.

**테스트 계획**:
- `realClient.test.ts`: signal abort 시 fetch cancel 동작 확인
- `+page.svelte` 컴포넌트 테스트: summarizingTimeout 상태 전환 확인

---

## 리뷰 결과 (Step-5, 260408)

### Claude 리뷰 (문서 일치성)

- ST-1: `realClient.ts`에 signal 파라미터 없음 확인. `ApiClient` 인터페이스 + `mockClient.ts`도 함께 수정 범위.
- `+page.svelte`에 `summarizingTimeout` 상태 변수 추가 필요 (기존 문서 미명시).
- 규칙: Svelte 5 반응성 — `summarizingTimeout` 리셋 원자성 주의.

### Codex 리뷰 (기술적 타당성)

- AbortController 소유 위치: `handleSummarize()` (page.svelte)가 적합. `request()` 공통화는 과함 — 전 엔드포인트에 적용되면 부작용 우려.
- `request()` 함수는 signal만 전달받는 최소 확장으로 충분.
- 타임아웃 값 30s 고정(상수): 환경변수화는 현 단계에서 과함.

### 최종 결정: 조건부 승인

**조건**:
1. `ApiClient` 인터페이스 변경 시 `mockClient.ts` + 관련 테스트 동시 업데이트 필수
2. `summarizingTimeout` 상태를 handleSummarize 진입 시 원자적으로 리셋
3. 테스트 먼저 작성 (TDD) — signal abort mock 포함
