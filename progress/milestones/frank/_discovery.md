# Discovery: Frank MVP4

> 생성일: 260408
> 진입점: MVP3 완료 회고 기반 (외부 리서치 대신 내부 부채 분석)

---

## 부채 분석 (영감 보드 대체)

MVP3 완료 회고(`history/mvp3/260408_mvp3_completion_retro.md`)와
부채 체크리스트(`history/260408_개념정리.md` 섹션13) 기반.

### 이월 부채 전체 목록

| # | 이슈 | 출처 | 우선순위 | 유형 |
|---|------|------|----------|------|
| 1 | iOS 요약 60s timeout — 클라이언트 타임아웃 감지 + 재시도 버튼 | MVP3 M3 회고 | High | feature |
| 2 | Apple Client Secret 갱신 알림 — 6개월 만료 2026-10-08경 | MVP3 완료 회고 | High | chore |
| 3 | Supabase Manual Linking — 이메일+Apple 계정 병합 (Beta 상태) | MVP3 완료 회고 | Medium | research |
| 4 | alert → 인라인 에러 UX 개선 (iOS 로그인 에러 표시) | MVP3 완료 회고 | Low | feature |
| 5 | iOS UITest/E2E 보강 | MVP2 이월 | Medium | feature |
| 6 | iOS 커버리지 측정 파이프라인 | MVP2 이월 | Medium | chore |
| 7 | 태그 stale article 이슈 (태그 변경 후 구 기사 잔류) | MVP3 M2 이월 | Medium | feature |
| 8 | 웹 UX 개선 — 에러 표시, 로딩 상태 일관성 | MVP3 완료 회고 | Low | feature |

---

### 접근 방식 분석 (아이디어 풀 역할)

#### 요약 60s timeout (부채 #1)

| 옵션 | 설명 | 노력 | 임팩트 |
|------|------|------|--------|
| A. 클라이언트 타임아웃 감지 | URLSession/fetch timeout 설정 + 재시도 버튼 | Low | High |
| B. 서버 SSE 스트리밍 | 서버 응답을 chunk 단위로 점진적 전달 | High | High |
| C. 비동기 job + polling | job_id 반환 후 주기적 상태 조회 | Very High | High |

**결론**: MVP4 취지(부채 해소, 최소 변경)에서 **옵션 A** 채택. 서버 변경 없이 클라이언트만 수정. 웹도 동시 적용.

#### Apple Secret 갱신 알림 (부채 #2)

| 옵션 | 설명 | 노력 |
|------|------|------|
| A. GitHub Actions cron | 매월 만료 30일 전 이슈 자동 생성 | Medium |
| B. scripts/ 만료 확인 스크립트 + 문서 | check_apple_secret.sh + README 가이드 | Low |
| C. .env 만료일 기록 + 시작 시 경고 | 서버 시작 시 만료 임박 경고 로그 | Low |

**결론**: **B + C 조합** — scripts/에 스크립트 추가 + 서버 시작 시 만료 경고.

#### 태그 stale 이슈 (부채 #7)

| 옵션 | 설명 | 노력 |
|------|------|------|
| A. 태그 변경 시 자동 재수집 트리거 | 서버 webhook/event 처리 | High |
| B. 태그 삭제 시 관련 article 소프트 삭제 | DB 레벨 cascade | Medium |
| C. 클라이언트 stale 필터링 | 태그와 기사 매핑 검증 후 표시 | Low |
| D. 수동 재수집 버튼 (설정 화면) | 사용자가 직접 트리거 | Low |

**결론**: 범위 확인 후 결정. M2 research 단계에서 재현 + 원인 분석 후 구현 방향 확정.

---

## 수렴 결과

### 이번에 넣을 것 (In)

| # | 아이템 | 유형 | 실행 스킬 | 마일스톤 |
|---|--------|------|----------|---------|
| 0 | LLM 모델 MiniMax → Qwen 복귀 | chore | 직접 수정 (10분) | M1 |
| 1 | 요약 timeout UX — iOS + 웹 클라이언트 타임아웃 + 재시도 | feature | /workflow | M1 |
| 2 | Apple Client Secret 만료 관리 체계 | chore | /workflow | M1 |
| 3 | iOS 로그인 에러 → 인라인 UX 개선 | feature | /workflow | M1 |
| 4 | 웹 UX 개선 (에러 표시 + 로딩 상태 일관성) | feature | /workflow | M1 |
| 5 | iOS 커버리지 측정 파이프라인 | chore | 직접 실행 | M2 |
| 6 | iOS UITest/E2E 보강 | feature | /workflow | M2 |
| 7 | 태그 stale article 이슈 해소 | feature | /workflow | M2 |
| 8 | Supabase Manual Linking 현황 조사 + 구현 가능성 판단 | research | /deep-analysis | M2 |

### 다음에 할 것 (Next — MVP5 진입 후)

| # | 아이템 | 메모 |
|---|--------|------|
| 1 | 스크랩 기능 | MVP5 학습 기능 첫 번째 |
| 2 | 퀴즈 자동 생성 | MVP5 학습 기능 두 번째 |
| 3 | 리포트 대시보드 | MVP5 학습 기능 세 번째 |

### 안 할 것 (Out)

| # | 아이템 | 사유 |
|---|--------|------|
| 1 | Supabase Manual Linking 구현 | Beta 미졸업, 외부 의존 — research로 현황만 파악 |
| 2 | 서버 SSE 스트리밍 (요약 스트리밍) | MVP4 범위 과다, MVP5에서 재검토 |
| 3 | 비동기 job + polling 구조 | 서버 대규모 리팩토링 필요, MVP5 이후 |
