# Step-5 리뷰 결과: 260504_debt_feedquality

> 리뷰일: 2026-05-04  
> 대상: ST-1 ~ ST-6 (1차 구현 대상 ST-1~ST-5 + ST-6 별도 검토)

---

## Claude 리뷰 (문서 일치성)

| ST | 결과 | 주요 지적 |
|----|------|---------|
| ST-1 | 조건부 승인 | 캐시 HIT 경로 날짜 필터 미적용 — 수정 완료 |
| ST-2 | 조건부 승인 | as_label/Display/is_retryable arm 누락 위험 — 수정 완료 |
| ST-3 | 승인 | cd vs --manifest-path 표기 불일치 (경미, 동작 동일) |
| ST-4 | 조건부 승인 | 기존 Exa row 마이그레이션 미언급 — 수정 완료 |
| ST-5 | 승인 | clean_snippet 내 구현으로 통일, char::is_control() 사용 금지 명시 |
| ST-6 | 조건부 승인 | NotificationPort 공존 여부 미결정 — 수정 완료 (완전 교체로 확정) |

---

## critical-review 결과

### 치명 (1건, 수정 완료)

**C1 — ST-1: 캐시 HIT 경로 날짜 필터 미적용**

- 원인: `feed.rs:250-257` 캐시 HIT 시 cached_items 그대로 반환. MISS 경로 FeedItem 변환 직후에만 필터하면 HIT 우회.
- 수정: F-01을 `feed_cache.set` 직전 적용으로 변경 → HIT/MISS 양쪽 커버.
- 파일 수정: `st1-date-filter.md` F-01 재작성, T-02 경계값 테스트 추가.

### 중대 (2건, 수정 완료)

**M1 — ST-2: QuotaExhausted arm 누락 위험**

- 원인: 스펙에 as_label/Display/is_retryable arm 추가 명시 없음 → 구현자 누락 시 컴파일 에러.
- 수정: F-03(as_label/Display), F-04(is_retryable arm) 추가. T-02/T-03 테스트 추가.
- 파일 수정: `st2-retry-classifier.md`

**M2 — ST-4: 기존 Exa row 마이그레이션 미언급**

- 원인: 코드 변경만으로는 ON CONFLICT 분기에서 기존 row reset_at이 NULL로 바뀌지 않음.
- 수정: F-04로 마이그레이션 SQL(`UPDATE api_call_counters SET reset_at = NULL WHERE engine = 'exa'`) 추가.
- 파일 수정: `st4-exa-reset-at.md`

### 경미 (2건)

**T1 — ST-6: NotificationPort 공존 여부 → 완전 교체로 확정**

- AlertDispatcherPort가 내부에서 NotificationPort 소유. CountedSearchAdapter의 notifier 필드 완전 제거.
- 기존 테스트 fixture FakeNotification → FakeAlertDispatcher 교체 필요.
- 파일 수정: `st6-alert-dispatcher-port.md` F-03/F-05 재작성.

**T2 — ST-1: E-01 경계값 재서술**

- "< 아닌 <=" 표현이 F-01 "7일 이상 제외"와 논리 충돌 → `published_at <= Utc::now() - Duration::days(7)` 이면 제외로 명확화.
- 파일 수정: `st1-date-filter.md` E-01 재서술.

---

## 최종 결정

**승인** (문서 수정 완료 조건부 → 전원 수정 완료)

- ST-1~ST-5: 1차 병렬 구현 진입 가능
- ST-6: 별도 검토 완료, NotificationPort 완전 교체 확정 후 2차 구현 진입 가능
- ST-7~ST-9: 변경 없음

---

## 수정된 파일 목록

- `progress/subtask/260504_debt_feedquality/st1-date-filter.md` — F-01 재작성, E-01 경계값 명확화, T-02 추가
- `progress/subtask/260504_debt_feedquality/st2-retry-classifier.md` — F-03/F-04 추가, T-02/T-03 추가
- `progress/subtask/260504_debt_feedquality/st4-exa-reset-at.md` — F-04 마이그레이션 SQL 추가
- `progress/subtask/260504_debt_feedquality/st6-alert-dispatcher-port.md` — F-03/F-05 재작성 (완전 교체 확정)
