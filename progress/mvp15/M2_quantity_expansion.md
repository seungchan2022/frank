# M2: 양 확대 + 무료 한도 자동 보호 인프라 (서버)

> 프로젝트: Frank MVP15
> 상태: ✅ done (2026-05-02)
> 예상 기간: 3~5일 (실제 1일 — 자동화 가속)
> 의존성: M1 (진단 결과)
> 갱신: 2026-05-02 — step-1~step-9 완료, 라이브 자동화 검증 통과

## 목표

서버에서 피드 양 limit을 5→20 (M1 진단 권고)으로 확대하고, 무료 한도 안에서 안전하게 운영되도록 엔진별 카운터·임계 자동 동작·iMessage 알림·개발용 mock 토글을 구축한다.

## 배경

- **인터뷰 결정 (2026-05-02)**:
  - Q1 limit: Tavily 5→20, Exa 5→max(실측), Firecrawl 5 유지
  - Q2 카운터: DB 테이블 `api_call_counters` (영속 저장)
  - Q3 셋 다 한도 시 화면: 빈 결과 + 안내 + 가장 빠른 회복 날짜
  - Q5 가시성: iMessage 자동 알림 (80%·100% 도달 시)
  - Q6 개발 보호: `FRANK_DEV_MOCK_SEARCH=1` 환경변수 토글
- **비용 원칙**: $0 유지 (`project_api_cost_policy`)
- **운영 발견 (2026-04-22 로그)**: Tavily 432 한도 초과 + Exa 폴백 동작 → 한도 보호 인프라 시급

## M1 진단 결과별 분기

M1 결론 = **(A) limit 병목 (Tavily 풍부)** + Exa 한도 소진 발견.
→ limit Tavily 5→20 적용 + 한도 보호 인프라 동시 진행.

## 성공 기준 (DoD)

### 1. 양 확대
- [ ] `server/src/api/feed.rs:216` `chain.search(_, 5)` → `chain.search(_, 20)`
- [ ] Exa numResults: 5→max (구현 시 Exa 무료 max 실측 후 확정 — 보통 10 추정)
- [ ] Firecrawl은 5 유지 (3순위 폴백)

### 2. 카운터 인프라
- [ ] DB 테이블 `api_call_counters` 신규 sqlx 마이그레이션
  - 스키마(안): `engine` (text, PK), `calls_this_month` (int), `reset_at` (timestamp, nullable — Exa는 NULL)
- [ ] 호출 성공 시 엔진별 카운터 +1 (`tavily_/exa_/firecrawl_calls_this_month`)
- [ ] 매월 1일 자동 리셋 (월간 엔진의 `reset_at` 갱신, 카운터 0)
- [ ] 재시작 후 카운터 보존 검증

### 3. 한도 자동 보호
- [ ] 80% 도달 시 캐시 TTL 5분 → 30분 자동 전환 (엔진별)
- [ ] 100% 도달 시 해당 엔진 호출 자동 차단 (체인에서 skip), 다른 엔진/캐시 사용
- [ ] 셋 다 한도 시 응답: 빈 결과 + 안내 메시지 + 가장 빠른 회복 날짜 (`reset_at` min)

### 4. iMessage 알림 (R3/R4 가드 포함)
- [ ] 80% 도달 시 iMessage 1회: "{엔진} 80% 도달, 캐시 강화 적용"
- [ ] 100% 도달 시 iMessage 1회: "{엔진} 한도 종료, 차단 적용. 회복: {reset_at}"
- [ ] 동일 임계 중복 알림 방지 (월 1회만)
- [ ] **민감정보 미포함**: 메시지에 user_id·쿼리·태그명 제외, 엔진명·임계치·회복일만 (R3)
- [ ] **외부 호출 가드**: timeout 5s, 재시도 1회, 메시지 크기 제한 (R4)

### 5. 개발용 mock 토글 (R2 가드 포함)
- [ ] 환경변수 `FRANK_DEV_MOCK_SEARCH=1` 시 `SearchFallbackChain`에 `FakeSearchAdapter`만 주입
- [ ] **기본값 OFF** — 미설정 시 실제 어댑터 (R2: 프로덕션 안전)
- [ ] 서버 시작 로그에 모드 명시 ("⚠️ MOCK SEARCH MODE" / "real search mode")
- [ ] `scripts/deploy.sh --mock-search` 플래그 (옵션, 본인 워크플로우용)

### 6. 테스트
- [ ] 단위 테스트: 카운터 INC, 80% TTL 전환, 100% 차단 (mock, 외부 호출 0)
- [ ] 통합 테스트: 카운터 800 시드 → 캐시 TTL 30분 확인
- [ ] 통합 테스트: 카운터 1000 시드 → 엔진 차단 확인
- [ ] 통합 테스트: 셋 다 1000 시드 → 안내 메시지 + reset_at min 확인
- [ ] 통합 테스트: iMessage 알림 mock (80%·100%에서 1회씩)
- [ ] 실측 검증 (소량): 활성 태그 1~2개로 캐시 클리어 후 1회 새로고침 → limit 20 확인

## 아이템

| # | 아이템 | 유형 | 실행 스킬 | 순서 | 상태 |
|---|--------|------|----------|------|------|
| 1 | limit 변경 (Tavily 5→20, Exa 5→max) + 단위 테스트 | feature | /workflow | 1 | 대기 |
| 2 | DB 테이블 + 카운터 INC 도메인/infra 어댑터 | feature | /workflow | 2 | 대기 |
| 3 | 한도 80% 캐시 TTL 자동 강화 | feature | /workflow | 3 | 대기 |
| 4 | 한도 100% 차단 + iMessage 알림 (R3/R4 가드) | feature | /workflow | 4 | 대기 |
| 5 | 셋 다 한도 시 사용자 안내 응답 (빈 결과 + reset_at min) | feature | /workflow | 5 | 대기 |
| 6 | FRANK_DEV_MOCK_SEARCH 토글 + 시작 로그 모드 표시 (R2 가드) | feature | /workflow | 6 | 대기 |
| 7 | 통합 테스트 (한도 시뮬레이션) | feature | /workflow | 7 | 대기 |

## 워크플로우 진입점

```
/workflow "M2-server-quantity-expansion"
```

**메인태스크**: 피드 양 limit을 M1 진단값(20)으로 확대하고 무료 한도 자동 보호 인프라(엔진별 카운터·임계 자동 동작·iMessage 알림·개발용 mock 토글)를 서버에 구현한다.

## KPI (M2)

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| feed.rs Tavily limit 값 | grep `chain.search` | = 20 | Hard | 현재 5 |
| feed.rs Exa limit 값 | 코드/실측 | Exa max | Hard | 현재 5 |
| 서버 테스트 통과 | `cargo test` | 전체 통과 | Hard | MVP14 카운트 |
| 서버 클리피 통과 | `cargo clippy -- -D warnings` | 통과 | Hard | — |
| `api_call_counters` 마이그레이션 | sqlx | exists | Hard | — |
| 카운터 영속 저장 | 재시작 후 보존 통합 테스트 | 통과 | Hard | — |
| 카운터 엔진별 분리 | 키 grep | tavily/exa/firecrawl 분리 | Hard | — |
| 80% 캐시 TTL 자동 전환 | 통합 테스트 (엔진별) | 통과 | Hard | — |
| 100% 차단 동작 | 통합 테스트 (엔진별) | 통과 | Hard | — |
| 셋 다 소진 안내 + reset_at min | 통합 테스트 | 통과 | Hard | — |
| iMessage 알림 트리거 | 통합 테스트 (mock) | 80%·100% 1회씩 | Hard | — |
| iMessage 민감정보 미포함 (R3) | 메시지 페이로드 검증 | user_id·쿼리·태그 없음 | Hard | — |
| FRANK_DEV_MOCK_SEARCH 토글 | ON/OFF 어댑터 차이 검증 | 통과 | Hard | — |
| 서버 시작 로그 모드 표시 | grep "MOCK"/"real search" | 둘 중 하나 | Hard | — |
| 양 20개 실측 | 본인 1회 사용 후 결과 수 | ≥ 18 (단일 결과 태그 제외) | Soft | — |

## 검증 전략

### (1) 보호 인프라 — 외부 API 호출 0회
- DB에 카운터 시드 INSERT → mock 호출 → 자동 동작 검증
- iMessage 알림은 mock 발송 검증

### (2) 실제 API 응답 — 소량 호출
- 활성 태그 1~2개 + 캐시 클리어 후 1회 새로고침 (~2 호출)
- limit 20 효과·Exa max 실측·폴백 체인 동작 확인

## 리스크

| 리스크 | 영향 | 대응 |
|--------|------|------|
| `FRANK_DEV_MOCK_SEARCH`가 프로덕션에 켜짐 | H | 기본 OFF, 시작 로그 명시, 운영 환경 가드 |
| iMessage 알림 민감정보 노출 | M | 페이로드를 엔진명·임계치·회복일로 제한 |
| iMessage 외부 호출 timeout 미설정 | M | 5초 timeout, 재시도 1회, 크기 제한 |
| 카운터 정확성 (재시작 리셋) | H | DB 영속 저장 |
| Tavily 432 응답 카운팅 정책 | M | 200 OK일 때만 +1 (실패 카운트 안 함) |
| Exa 무료 numResults max 미확정 | M | 구현 시 실측 후 적용 |

## 범위 제외

- **Exa 결제 / 새 무료 엔진 추가** — M2 밖. 다음 마일스톤에서 결정
- **클라이언트(웹/iOS) 작업** — M3 범위
- **사용자 안내 메시지 i18n** — 한국어만 (M4)

## 변경 이력

| 날짜 | 변경 내용 | 사유 |
|------|----------|------|
| 260501 | 초안 작성 | 시드 + Q1~Q5 결정 |
| 260502 | 인터뷰 결정 반영 (limit 5→20, DB 카운터, iMessage, FRANK_DEV_MOCK_SEARCH) + R2~R4 보안 가드 | step-2 룰즈 검증 결과 |
| 260502 | step-1~step-9 완료 | 7개 서브태스크(S1~S7) 구현 + 362 단위 테스트 통과 + 라이브 자동화 검증(envelope/limit 20/카운터 INC/lazy reset/mock 미오염) 통과. DEBT-MVP15-04~07 등록. |
