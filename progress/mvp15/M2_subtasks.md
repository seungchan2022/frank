# MVP15 M2 서브태스크 분해

> 메인태스크: **M2-server-quantity-expansion** (피드 양 limit 5→20 + 무료 한도 자동 보호 인프라)
> 작성일: 2026-05-02
> 워크플로우 단계: step-5 완료 (조건부 승인 → 보강 후 step-6 진입 준비)
> 브랜치: `feature/mvp15-m2-quantity-expansion`
> 기획 문서: `progress/mvp15/M2_quantity_expansion.md`
> 적용 룰: `rules/0_CODEX_RULES.md` + `.claude/rules/rust-scope.md` (TDD, 포트 기반 mock, 90% 커버리지)

## 인터뷰 결정 요약 (입력)

### Q1~Q6 (step-1)
- **Q1 limit**: Tavily 5→20, Exa 5→max(실측 후 확정), Firecrawl 5 유지
- **Q2 카운터**: DB 테이블 `api_call_counters` (영속 저장)
- **Q3 셋 다 한도 시**: 빈 결과 + 안내 + 가장 빠른 `reset_at` min
- **Q4 Firecrawl**: 5 유지 (3순위 폴백)
- **Q5 가시성**: iMessage 자동 알림 (80%/100% 도달, 엔진별 1회)
- **Q6 개발 보호**: `FRANK_DEV_MOCK_SEARCH=1` 환경변수 토글

### H1~H7 (step-4)
- **H1 카운터 INC 위치**: B — `CountedSearchAdapter` 데코레이터 (각 어댑터 wrap)
- **H2 dedupe 영속 위치**: B — 별도 테이블 `api_alert_log` (시간 기반)
- **H3 월간 리셋 트리거**: A — lazy (INC 시점 비교)
- **H4 마이그레이션 위치**: `supabase/migrations/` (기존 위치)
- **H5 Exa max**: 구현 시 실측 (보통 10 추정)
- **H6 100% 엔진 skip 위치**: C — `CountedSearchAdapter`에 통합 (H1 동일 컴포넌트)
- **H7 TTL 자동 reverse**: yes (사용률 < 80% 시 5분 자동 복원)

### #1~#5 (step-5 조건부 승인 해소)
- **#1 R4 timeout 실현**: B — `spawn_blocking + tokio::timeout`. `NotificationPort::send` sync 시그니처 유지
- **#2 양 확대 계약**: A — 어댑터 생성자에 `max_cap` 주입, 내부에서 `min(requested, cap)` clamp. chain 인터페이스 그대로
- **#3 lazy reset + INC SQL**: 단일 CASE statement (race-free, 아래 S2 §SQL 참조)
- **#4 dedupe PK**: `api_alert_log (engine, threshold, period_start)` 3개 PK — 다음 주기 재알림 보장
- **#5 mock 모드 + 데코레이터**: `FRANK_DEV_MOCK_SEARCH=1` 시 `CountedSearchAdapter`는 그대로 wrapping하되 `CounterPort`도 `InMemoryCounter`로 주입 → DB 미오염 + 데코레이터 path 검증

### 보안 가드 (R2/R3/R4)
- **R2**: 토글 기본 OFF + 시작 로그 모드 명시
- **R3**: iMessage 페이로드에 user_id/쿼리/태그명 미포함, 엔진명·임계치·회복일만
- **R4**: spawn_blocking + 5초 timeout + 재시도 1회 + 크기 제한

## 분해 원칙

- **단일 책임**: 각 서브태스크는 하나의 목적·하나의 산출물
- **포트/어댑터 패턴 유지**: `CounterPort`/`NotificationPort`/`SearchPort` 모두 trait 추상화 → mock 주입
- **순수 로직 ↔ I/O 셸 분리**: 임계치 판정·페이로드 구성은 순수, DB UPSERT/iMessage Command는 셸로 격리
- **자기 보호 우선**: S6 (mock 토글) 가장 먼저

## 서브태스크 목록

| ID | 제목 | 유형 | 의존성 | 산출물 | 테스트 전략 | 예상 |
|----|------|------|--------|--------|-------------|------|
| S1 | 양 확대: 어댑터별 `max_cap` 주입 (Tavily=20, Exa=max 실측, Firecrawl=5) | feat | — | 각 어댑터 생성자에 `max_cap: usize` 인자 추가 + 내부 `min(requested, cap)` clamp. `main.rs:42-46` 와이어링 갱신. `feed.rs:216` 호출은 `chain.search(_, 20)`로 갱신 (실제 결과 수는 어댑터 cap에 의해 제한됨) | 단위: 어댑터별 cap clamp 함수 표 기반 테스트 (cap=20·요청=100 → 20). 실측: 활성 태그 1~2개 캐시 클리어 후 1회 새로고침 → ≥18건. Exa max 실측: 1회 호출로 numResults=100 응답 확인 | 1.5h |
| S2 | 카운터 인프라: 마이그레이션 + `CounterPort` trait + `CountedSearchAdapter` 데코레이터 (H1·H6 통합) + lazy 리셋 atomic SQL | feat | — | 1) 마이그레이션 `supabase/migrations/20260502_mvp15_m2_api_call_counters.sql` (table `api_call_counters` + `api_alert_log` PK 3개). 2) `domain/ports.rs` `CounterPort` trait. 3) `infra/postgres_counters.rs` (lazy reset CASE SQL). 4) `infra/in_memory_counter.rs` (mock용). 5) `infra/counted_search.rs` `CountedSearchAdapter`(SearchPort wrap, 호출 전 100% skip 결정, 호출 후 200 OK 시 INC) | 단위: MockCounter로 INC/snapshot/skip 결정적 검증. lazy reset 분기 (reset_at < now → calls=1, reset_at=다음달 1일). 데코레이터: skip 시 빈 결과·INC 0, 성공 시 INC +1, 실패 시 INC 0. 통합: 카운터 시드 → 재시작 후 보존 | 3h |
| S3 | 80% 도달 시 캐시 TTL 5분 → 30분 자동 전환 (엔진별 + reverse 자동) | feat | S2 | `api/feed.rs` chain 호출 직후 `CounterPort.snapshot(used_engine)` 조회 → 사용률 ≥ 80%이면 TTL 30분, 미만이면 5분. cache_key별 TTL 동적 결정 함수(순수) | 단위: 사용률(0/79/80/100)별 TTL 결정 함수. 통합: MockCounter 800/1000 시드 → `feed_cache.set` ttl 30분, 700/1000 → 5분 (reverse 자동) | 1h |
| S4 | 100% 차단 + iMessage 알림 (R3/R4 가드 + dedupe atomic) | feat | S2 | (a) 100% skip은 S2의 `CountedSearchAdapter`에서 자동 처리됨 (이 서브태스크는 알림만 신경). (b) `services/notification_service.rs` 임계 알림 헬퍼. (c) `api_alert_log` atomic INSERT(`ON CONFLICT (engine, threshold, period_start) DO NOTHING RETURNING id`) — RETURNING 행이 있어야만 발송. (d) 페이로드 `"{engine} {threshold}% 도달 — 회복 {reset_at}"` (user_id/쿼리/태그명 제외). (e) `tokio::task::spawn_blocking(send) + tokio::time::timeout(5s)` + 재시도 1회 + 크기 ≤200자 | 단위: 임계 판정·페이로드 빌더(순수). MockNotifier로 80%/100% 1회씩 검증. dedupe: 동일 (engine, threshold, period_start) 2회 진입 시 발송 1회. R3 검증: 페이로드 substring assert. R4: timeout/retry는 FailingNotifier mock | 2.5h |
| S5 | 셋 다 한도 시 사용자 안내 응답 (빈 결과 + reset_at min) | feat | S2, S4 | `api/feed.rs` 모든 엔진 차단/실패 시 `FeedResponse { items: [], notice: Some({ message, recovery_at }) }` 반환. `recovery_at = min(reset_at)` (NULL 엔진 제외) | 단위: `select_recovery_at(snapshots)` 순수 함수 표 테스트 (NULL 혼재). 통합: 셋 다 1000 시드 → items=[] + notice + recovery_at = 다음달 1일 | 1.5h |
| S6 | `FRANK_DEV_MOCK_SEARCH` 토글 + 시작 로그 모드 표시 + mock 시 `InMemoryCounter` 주입 (R2 가드, #5 반영) | feat | S2 | `main.rs` 분기: 환경변수 `==1` 시 `FakeSearchAdapter` + `InMemoryCounter` 주입(DB 미오염), `CountedSearchAdapter`는 그대로 wrap (path 검증). 시작 로그 `tracing::warn!("⚠️  MOCK SEARCH MODE")` / `tracing::info!("real search mode")`. `scripts/deploy.sh --mock-search` 플래그 | 단위: 분기 함수(env → adapter+counter 선택)를 헬퍼로 분리, 표 기반 테스트. 실측: 토글 ON/OFF 두 번 부팅 후 로그 grep | 1h |
| S7 | 통합 테스트 (한도 시뮬레이션 e2e) | test | S1~S6 | `tests/` 또는 `feed.rs mod tests`: (a) 800 시드 → TTL 30분, (b) 1000 시드 → CountedAdapter skip + iMessage 1회, (c) 셋 다 1000 → notice + recovery_at min, (d) 재시작 후 카운터 보존, (e) source_name == engine PK 일관성 assert | 모든 외부 의존 mock(MockCounter + FakeSearchAdapter + MockNotifier). 외부 API 호출 0회. KPI: cargo test 전체 + clippy `-D warnings` | 1.5h |

**합계**: 7개 서브태스크 ≈ 12h (1.5~2일)

## S2 핵심 SQL — lazy reset + INC atomic (#3)

```sql
-- 호출마다 1회 실행. CASE로 reset과 INC를 단일 statement로 묶어 race-free
INSERT INTO api_call_counters (engine, calls_this_month, reset_at)
VALUES ($1, 1, date_trunc('month', now()) + interval '1 month')
ON CONFLICT (engine) DO UPDATE SET
    calls_this_month = CASE
        WHEN api_call_counters.reset_at < now() THEN 1
        ELSE api_call_counters.calls_this_month + 1
    END,
    reset_at = CASE
        WHEN api_call_counters.reset_at < now() THEN date_trunc('month', now()) + interval '1 month'
        ELSE api_call_counters.reset_at
    END
RETURNING calls_this_month, reset_at;
```

## S2 마이그레이션 스키마 (#4 PK 반영)

```sql
CREATE TABLE api_call_counters (
    engine            TEXT PRIMARY KEY,
    calls_this_month  INT NOT NULL DEFAULT 0,
    reset_at          TIMESTAMPTZ -- Exa(크레딧형)는 NULL 허용
);

CREATE TABLE api_alert_log (
    engine        TEXT NOT NULL,
    threshold     INT NOT NULL,        -- 80 / 100
    period_start  TIMESTAMPTZ NOT NULL,-- 이번 주기의 시작 (date_trunc('month', now()))
    sent_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (engine, threshold, period_start)
);
```

## 의존성 DAG

```
S6 (자기 보호 — 가장 먼저)
   │
   ▼
┌─────────────┐
│ S1   │ S2  │ (S2가 foundation)
└─────────────┘
        │
        ├──────┬──────┐
        ▼      ▼      ▼
       S3    S4 ─→ S5
                    │
                    ▼
                   S7
```

## 권장 실행 순서

1. **S6** — 자기 보호. 이후 단계 토글 ON 진행 가능
2. **S2** — Foundation (마이그레이션 + CounterPort + 데코레이터)
3. **S1** — limit max_cap 주입 (S2와 병렬 가능)
4. **S3** — 80% TTL
5. **S4** — 100% 알림 (R3/R4 가드)
6. **S5** — 셋 다 안내 응답
7. **S7** — 통합 e2e

## 적용 룰 체크리스트 (rust-scope)

- [ ] `unwrap()` 금지 (테스트 제외) — `?` + `anyhow::Result`
- [ ] 외부 의존(DB, HTTP, iMessage Command, env)은 trait 추상화 → mock 주입
- [ ] `serde_json::Value` 외부 응답 파싱 외 금지 — typed 구조체 (`notice` 옵셔널 + 호환)
- [ ] 모든 trait는 `Send + Sync` 바운드
- [ ] 카운터 INC: lazy reset CASE 단일 statement (위 S2 SQL)
- [ ] dedupe atomic: `INSERT ... ON CONFLICT (engine, threshold, period_start) DO NOTHING RETURNING id`
- [ ] `cargo clippy -- -D warnings` + `cargo fmt --check` + `cargo test` 모두 통과
- [ ] 커버리지 90% 이상

## 보안/리스크 가드

| 리스크 | 영향 | 대응 |
|--------|------|------|
| 카운터 INC 동시성 (join_all 다중 INC race) | H | Postgres atomic CASE statement (위 S2 SQL) |
| iMessage 알림 race (80% 동시 도달 중복 알림) | M | `api_alert_log` atomic INSERT + `RETURNING id` 행이 있어야만 발송 |
| `FRANK_DEV_MOCK_SEARCH`가 프로덕션에 켜짐 | H | 기본 OFF, 시작 로그 명시(`⚠️  MOCK SEARCH MODE`), mock 시 `InMemoryCounter` 사용으로 DB 미오염 |
| iMessage 민감정보 노출 (R3) | M | 페이로드 엔진명·임계치·회복일로 제한 |
| iMessage timeout 미설정 (R4) | M | `spawn_blocking + tokio::timeout(5s)`, 재시도 1회, ≤200자 |
| 카운터 정확성 (재시작 리셋) | H | DB 영속 + S7 재시작 시뮬 테스트 |
| Tavily 432 카운팅 한계 | L | 200 OK일 때만 INC. 카운터는 보호적 추정치 (실제 한도와 100% 정확하지 않음 — 명시적 한계로 수용) |
| `source_name` ↔ `engine` PK 일치성 | M | S7에 일관성 assert 포함 (`tavily`/`exa`/`firecrawl` 리터럴 검증) |

## 다음 단계

→ `/step-6` (구현) — **완료 (2026-05-02)**
- 실행 순서: S2 foundation → S1 max_cap → S6 toggle → S3 TTL → S5 envelope/notice → S4 alert (S2 통합) → S7 integration
- 검증: cargo test 362 passed (10+ 신규), clippy `-D warnings` 통과, fmt 통과
- 변경 요약:
  - 신규: `domain/ports::CounterPort`, `infra/in_memory_counter.rs`, `infra/postgres_counters.rs`, `infra/counted_search.rs`
  - 마이그레이션: `supabase/migrations/20260502_mvp15_m2_api_call_counters.sql` (테이블 2개)
  - 수정: `tavily.rs`/`exa.rs`/`firecrawl.rs` (`with_max_cap` 빌더), `feed.rs` (envelope `FeedResponse` + TTL 결정 로직 + 셋다 안내), `services/notification_service.rs` (AlertDispatch + spawn_blocking+timeout dispatch)
  - main.rs: `FRANK_DEV_MOCK_SEARCH` 토글 + `CountedSearchAdapter` wrap (mock·real 모두)
  - scripts/deploy.sh: `--mock-search` 플래그
- 부채 (step-7 리팩토링에서 정리 예정):
  - `MONTHLY_LIMIT` 상수 3곳 중복 (counted_search 800/1000, notification_service 1000, feed 1000) → 단일 SSOT 통합
  - `재시작 후 카운터 보존` 통합 테스트는 InMemoryCounter 기반이라 본질 검증 한계 — Postgres CASE SQL 본격 검증은 수동 (psql) 또는 sqlx::test 도입 시 가능
- 클라이언트 영향:
  - **API 응답 shape 변경 (breaking)**: `Vec<FeedItemResponse>` → `FeedResponse { items, notice }`. 웹/iOS는 M3에서 envelope 디시리얼라이저로 전환 필요
