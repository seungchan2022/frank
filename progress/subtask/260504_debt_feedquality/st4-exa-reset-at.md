# ST-4: B-4 Exa reset_at NULL semantics

> 브랜치: feature/260504_feed-quality-debt-cleanup  
> 파일: server/src/infra/postgres_counters.rs

## 요약
Exa는 크레딧 선불형이라 월간 자동 reset 없음. 현재 첫 INSERT 시 다음 달 1일 reset_at이 세팅되는 버그 수정.

## Feature List
<!-- size: 중형 | count: 12 | skip: false -->

### 기능
- [x] F-01 engine="exa" INSERT 시 reset_at=NULL로 처리
- [x] F-02 engine="exa" ON CONFLICT 시에도 reset_at 변경 없음 (NULL 유지)
- [x] F-03 Tavily, Firecrawl은 기존 월간 reset_at 로직 그대로 유지
- [x] F-04 운영 DB 기존 Exa row 마이그레이션: `UPDATE api_call_counters SET reset_at = NULL WHERE engine = 'exa'`
  - 이유: 코드 변경만으로는 ON CONFLICT 분기에서 기존 row의 reset_at이 NULL로 바뀌지 않음
  - 배포 전 또는 직후 스크립트/migration으로 1회 실행 필요
  - 실행 방법: `scripts/run-diagnose.sh` 또는 DB 직접 접속 후 실행

### 엣지
- [x] E-01 engine name 비교는 소문자 정규화 후 수행
- [x] E-02 Exa 카운터 첫 INSERT 이후 재호출 시 월간 갱신 없음 확인

### 에러
- [x] R-01 NULL reset_at → CounterSnapshot 반환 시 reset_at 필드 None 처리
- [x] R-02 DB 연결 오류 시 상위로 AppError 전파

### 테스트
- [x] T-01 Exa engine 첫 INSERT → reset_at=NULL snapshot 반환 검증
- [x] T-02 Exa engine 재호출 → reset_at 여전히 NULL 검증
- [x] T-03 Tavily engine → reset_at 기존 월간 동작 회귀 방지
- [x] T-04 Mock DB 기반 단위 테스트로 실제 DB 의존 없음
- [x] T-05 cargo test 전체 통과
