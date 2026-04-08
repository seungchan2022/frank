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
| 1 | 요약 timeout UX — AbortController + 재시도 버튼 | feature | 수집/요약 API 프록시 라우트 |
| 2 | 에러/로딩 UX 통일 — ErrorBanner + Skeleton 공통화 | feature | 피드, 기사 상세, 설정 화면 |

---

## 성공 기준 (DoD)

- [ ] 요약 요청이 30s 이상 걸릴 때 타임아웃 UI + 재시도 버튼 표시
- [ ] `ErrorBanner.svelte` 공통 컴포넌트 생성 및 주요 3개 화면 적용
- [ ] `LoadingSkeleton.svelte` 공통화 + 주요 3개 화면 적용
- [ ] 기존 웹 테스트 89개 전체 통과
- [ ] 새 컴포넌트 테스트 추가

---

## 서브태스크 상세

### ST-1: 요약 timeout UX

**현황**: `fetch()`에 타임아웃 처리 없음.

**구현 방향:**
```ts
// AbortController + setTimeout(30000)
// timeout 시 에러 상태 전환 + 재시도 버튼 표시
```

---

### ST-2: 에러/로딩 UX 통일

**현황**: 화면별 에러 표시 방식 불일치, Spinner/Skeleton 혼용.

**구현 방향:**
- `ErrorBanner.svelte` — 인라인 에러 공통 컴포넌트
- `LoadingSkeleton.svelte` — 로딩 상태 공통 컴포넌트
- 적용 범위: 피드 / 기사 상세 / 설정 화면 (3개 고정, 크리프 금지)
