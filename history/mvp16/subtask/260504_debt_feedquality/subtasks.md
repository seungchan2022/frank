# 서브태스크 분리: 부채 해소 + 피드 품질 개선

> 생성일: 2026-05-04  
> 브랜치: feature/260504_feed-quality-debt-cleanup  
> 태스크 원문: progress/tasks/260504_debt_feedquality.md

---

## 인터뷰 확정 결정사항

| 항목 | 결정 |
|------|------|
| B-6 AlertDispatcherPort | A안: domain/ports.rs에 trait 신설, counted_search.rs dispatch_threshold_alert 직접 호출 제거 |
| B-3 Tavily effective_max | 스킵 |
| A-2 소스 다양성 | 스킵 |
| A-1 날짜 필터 기준 | 7일 이상 기사 제외, published_at=None은 통과 |
| C-1, C-2 E2E | 이번 워크플로우 안에서 반드시 완료 |

---

## 서브태스크 목록

### ST-1: A-1 발행 날짜 필터
- **목적**: 7일 이상 된 기사를 피드에서 제외하여 최신성 보장
- **파일**: `server/src/api/feed.rs`
- **구현**: SearchResult → FeedItem 변환 직후 published_at이 Some이고 Utc::now() - 7일 이전이면 skip. published_at이 None이면 통과
- **테스트**: 8일 전 기사 제외, 6일 전 기사 통과, None 통과 — 단위 테스트 3개
- **산출물**: `server/src/api/feed.rs` 수정, 테스트 3개 추가
- **의존**: 없음 (독립)

### ST-2: B-1 retry_classifier 402 추가
- **목적**: Exa 쿼터 초과 시 반환하는 402를 quota_exhausted 카테고리로 분류 (현재 Network 폴백으로 재시도 시도됨)
- **파일**: `server/src/bin/diagnose/retry_classifier.rs`
- **구현**: ErrorCategory enum에 QuotaExhausted 추가. classify() 함수에 "402" 또는 "payment required" 패턴 추가. is_retryable()에서 QuotaExhausted는 false 반환
- **테스트**: classify 분기 테스트 1개 추가
- **산출물**: `retry_classifier.rs` 수정
- **의존**: 없음 (독립)

### ST-3: B-2 진단 바이너리 실행 스크립트
- **목적**: cargo run --bin diagnose 실행 환경과 필수 환경변수를 문서화한 셸 스크립트 제공
- **파일**: `scripts/run-diagnose.sh` (신규)
- **구현**: 환경변수 체크(EXA_API_KEY, TAVILY_API_KEY, DATABASE_URL) → missing 시 안내 후 exit 1. 도움말(-h) 지원. cd server && cargo run --bin diagnose -- "$@"
- **산출물**: `scripts/run-diagnose.sh`
- **의존**: 없음 (독립)

### ST-4: B-4 Exa reset_at NULL semantics
- **목적**: Exa는 크레딧 선불형이라 월간 reset이 없음. 현재 첫 INSERT 시 항상 다음 달 1일 reset_at 세팅됨 → NULL로 처리
- **파일**: `server/src/infra/postgres_counters.rs`
- **구현**: record_call() SQL에서 engine이 "exa"이면 INSERT reset_at=NULL, ON CONFLICT 시에도 reset_at 변경 안 함
- **테스트**: exa reset_at=None 시 snapshot 반환값 검증
- **산출물**: `postgres_counters.rs` 수정
- **의존**: 없음 (독립)

### ST-5: B-5 snippet invalid JSON escape 제거
- **목적**: Exa highlights에 nul 바이트 등 PostgreSQL이 거부하는 문자가 포함되면 파싱/저장 오류 발생
- **파일**: `server/src/infra/exa.rs`
- **구현**: clean_snippet() 내 또는 highlights 파싱 직후 nul 바이트(\0) 및 제어 문자 제거
- **테스트**: nul 포함 입력 → 제거 검증 테스트 1개
- **산출물**: `exa.rs` 수정, 테스트 1개 추가
- **의존**: 없음 (독립)

### ST-6: B-6 AlertDispatcherPort trait 신설 (레이어 의존 해소)
- **목적**: infra/counted_search.rs가 services/notification_service.rs를 직접 use → infra → services 의존 위반. domain/ports.rs에 AlertDispatcherPort trait 신설하여 해소
- **파일**: `server/src/domain/ports.rs`, `server/src/infra/counted_search.rs`, `server/src/services/notification_service.rs`, AppState wire-up
- **구현**:
  1. domain/ports.rs에 AlertDispatcherPort trait 추가 (dispatch 메서드)
  2. services/notification_service.rs에 AlertDispatcherService impl 추가
  3. infra/counted_search.rs의 CountedSearchAdapter가 Arc<dyn AlertDispatcherPort>를 받도록 변경
  4. AppState에서 wire-up 수정
  5. infra/fake_alert_dispatcher.rs 신규 생성
- **테스트**: 기존 counted_search.rs 테스트 전체 통과 확인
- **산출물**: `ports.rs`, `counted_search.rs`, `notification_service.rs`, AppState, `infra/fake_alert_dispatcher.rs`
- **의존**: ST-4, ST-5 완료 후 (counted_search 수정 충돌 방지)

### ST-7: B-7 debts.md DEBT-08 RESOLVED 처리
- **목적**: DEBT-08(E2E 자동화)이 이미 구현됨 → debts.md에 반영
- **파일**: `progress/debts.md`
- **구현**: DEBT-08 상태를 RESOLVED로 변경, 해소 내용 기술
- **산출물**: `progress/debts.md` 수정
- **의존**: 없음 (문서, 언제든 가능)

### ST-8: C-1 Playwright occupation→insight→rewrite→scrap E2E
- **목적**: MVP15 M3 핵심 흐름(occupation 설정 → AI 인사이트 포함 요약 → 직업 시각 재작성 → 스크랩 저장) Playwright E2E 검증
- **파일**: `web/e2e/occupation-insight-rewrite.spec.ts` (신규)
- **구현**:
  1. login helper 사용 → 프로필에서 occupation 설정
  2. 피드로 돌아가 기사 상세 진입
  3. AI 요약 버튼 클릭 → insight 영역 표시 확인
  4. 재작성 버튼 클릭 → 재작성 결과 표시 확인
  5. 스크랩 저장 버튼 클릭 → 즐겨찾기 추가 확인
- **산출물**: `web/e2e/occupation-insight-rewrite.spec.ts` 신규
- **의존**: 서버 실행 환경 필요. ST-1 이후 권장

### ST-9: C-2 XCUITest rewrite 버튼→결과 표시
- **목적**: iOS SwiftUI에서 재작성(rewrite) 버튼 탭 후 결과 화면 표시를 XCUITest로 검증
- **파일**: `ios/Frank/FrankUITests/RewriteFlowUITest.swift` (신규)
- **구현**:
  1. UITestHelpers.swift 패턴으로 로그인
  2. 피드 기사 탭 → 상세 화면
  3. 요약 버튼 탭 → 완료 대기
  4. 재작성 버튼 탭 → 결과 텍스트 뷰 표시 확인 (accessibilityIdentifier 기반)
- **산출물**: `ios/Frank/FrankUITests/RewriteFlowUITest.swift` 신규
- **의존**: 시뮬레이터 실행 환경 필요. ST-8 이후 권장

---

## 의존성 DAG (텍스트)

```
ST-1 (A-1 날짜 필터)      ─┐
ST-2 (B-1 402 분류)        │
ST-3 (B-2 진단 스크립트)   ├──→ ST-6 (B-6 AlertDispatcherPort) ──→ ST-7 (B-7 docs)
ST-4 (B-4 Exa NULL)      ─┤
ST-5 (B-5 snippet)        ─┘

ST-8 (C-1 Playwright) ──→ ST-9 (C-2 XCUITest)
```

- ST-1 ~ ST-5: 완전 독립, 병렬 구현 가능
- ST-6: ST-4, ST-5 완료 후 (counted_search 수정 충돌 방지)
- ST-7: 언제든 가능 (문서)
- ST-8 ~ ST-9: 서버+앱 실행 환경에서 순차 실행

---

## 구현 우선순위

| 단계 | 서브태스크 | 특이사항 |
|------|-----------|---------|
| 1차 (병렬 가능) | ST-1, ST-2, ST-3, ST-4, ST-5 | 독립, 어떤 순서든 가능 |
| 2차 | ST-6 | ST-4, ST-5 이후 |
| 3차 | ST-7 | ST-6 이후 (또는 독립으로 먼저 가능) |
| 4차 | ST-8, ST-9 | 실행 환경 필요 |

---

## 완료 조건

- [ ] ST-1 ~ ST-5 단위 테스트 통과
- [ ] ST-6 레이어 위반 해소 + 기존 counted_search 테스트 전체 통과
- [ ] ST-7 debts.md 갱신
- [ ] ST-8 Playwright 실행 통과
- [ ] ST-9 XCUITest 실행 통과
- [ ] cargo clippy -- -D warnings && cargo test 전체 통과
- [ ] npm run validate 전체 통과
