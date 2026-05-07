# ST-3: B-2 진단 바이너리 실행 스크립트

> 브랜치: feature/260504_feed-quality-debt-cleanup  
> 파일: scripts/run-diagnose.sh (신규)

## 요약
진단 바이너리 실행 환경(env var, cwd)을 표준화하는 셸 스크립트.

## Feature List
<!-- size: 소형 | count: 5 | skip: true -->

### 기능
- [x] F-01 필수 환경변수(EXA_API_KEY, TAVILY_API_KEY, DATABASE_URL) 체크 후 없으면 exit 1
- [x] F-02 server/.env 자동 소싱 후 cargo run --manifest-path server/Cargo.toml --bin diagnose 실행

### 엣지
- [x] E-01 환경변수 누락 시 어떤 변수가 없는지 명시적 안내 출력

### 에러
- [x] R-01 cargo 빌드/실행 실패 시 exit code 그대로 전파

### 테스트
- [x] T-01 -h 플래그로 도움말 출력 smoke test (실행 오류 없음 확인)
