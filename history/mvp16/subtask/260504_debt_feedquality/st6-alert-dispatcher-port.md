# ST-6: B-6 AlertDispatcherPort trait 신설

> 브랜치: feature/260504_feed-quality-debt-cleanup  
> 파일: domain/ports.rs, infra/counted_search.rs, services/notification_service.rs, AppState, infra/fake_alert_dispatcher.rs

## 요약
infra/counted_search.rs → services/notification_service.rs 역방향 레이어 의존 위반 해소.  
domain/ports.rs에 AlertDispatcherPort trait 신설, infra는 포트만 참조.

## Feature List
<!-- size: 중형 | count: 18 | skip: false -->

### 기능
- [ ] F-01 AlertDispatcherPort trait domain/ports.rs에 추가 (dispatch_threshold_alert 메서드)
- [ ] F-02 NotificationAlertDispatcher: AlertDispatcherPort impl을 services/notification_service.rs에 추가
- [ ] F-03 CountedSearchAdapter에서 `notifier: Arc<dyn NotificationPort>` 필드를 `alert_dispatcher: Arc<dyn AlertDispatcherPort>`로 **교체** (공존 아님, 완전 대체)
  - AlertDispatcherPort가 내부에서 NotificationPort를 소유 — 외부에서 NotificationPort 주입 불필요
- [ ] F-04 infra/counted_search.rs에서 `services::notification_service` 직접 import 제거 (use 라인 31 삭제)
- [ ] F-05 AppState에서 AlertDispatcherPort 구현체 wire-up 수정
  - 기존 `notifier: Arc<FakeNotification>` 주입하던 테스트 fixture를 `FakeAlertDispatcher`로 교체
- [ ] F-06 infra/fake_alert_dispatcher.rs 신규 생성 (테스트용 FakeAlertDispatcher)

### 엣지
- [ ] E-01 AlertDispatcherPort는 Send + Sync 바운드 필수
- [ ] E-02 dispatch 실패(오류) 시 counted_search 검색 결과에 영향 없음 (무시 처리)

### 에러
- [ ] R-01 AlertDispatcher dispatch 중 panic 시 counted_search 동작 보장
- [ ] R-02 AppState wire-up 오류 시 컴파일 타임 감지 (Arc 타입 미스매치)

### 회귀
- [ ] G-01 AppState 구성 변경으로 다른 서비스 회귀 없음
- [ ] G-02 기존 notification_service 알림 동작 변경 없음
- [ ] G-03 기존 counted_search 임계값 알림 동작 변경 없음

### 테스트
- [ ] T-01 FakeAlertDispatcher 기반 CountedSearchAdapter 단위 테스트 통과
- [ ] T-02 AlertDispatcherPort trait → NotificationAlertDispatcher 구현 검증
- [ ] T-03 cargo clippy -- -D warnings 통과 (infra → services import 없음 확인)
- [ ] T-04 cargo test 전체 통과
- [ ] T-05 의존 방향: infra → domain(port) ← services 검증
