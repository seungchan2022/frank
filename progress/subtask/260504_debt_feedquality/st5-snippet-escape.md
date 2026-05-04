# ST-5: B-5 snippet invalid JSON escape 제거

> 브랜치: feature/260504_feed-quality-debt-cleanup  
> 파일: server/src/infra/exa.rs

## 요약
Exa 응답 snippet에 포함된 nul 바이트 및 PostgreSQL이 거부하는 제어문자 제거.

## Feature List
<!-- size: 소형 | count: 5 | skip: true -->

### 기능
- [x] F-01 clean_snippet()에서 nul 바이트(\0) 및 제어 문자(U+0001~U+001F) 제거

### 엣지
- [x] E-01 \t, \n, \r(합법적 공백)은 제거하지 않음
- [x] E-02 이미 정상인 텍스트는 변형 없음

### 에러
- [x] R-01 빈 문자열 입력 시 빈 문자열 반환 (panic 없음)

### 테스트
- [x] T-01 nul 포함 입력 → nul 제거된 결과 검증
- [x] T-02 정상 UTF-8 텍스트 보존 검증
