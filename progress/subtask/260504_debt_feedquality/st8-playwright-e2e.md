# ST-8: C-1 Playwright occupation→insight→rewrite→scrap E2E

> 브랜치: feature/260504_feed-quality-debt-cleanup  
> 파일: web/e2e/occupation-insight-rewrite.spec.ts (신규)

## 요약
occupation 설정 → AI 인사이트 포함 요약 → 직업 시각 재작성 → 스크랩 저장 플로우 Playwright E2E 검증.

## Feature List
<!-- size: 중형 | count: 15 | skip: false -->

### 기능
- [x] F-01 login helper로 test@test.com 로그인
- [x] F-02 프로필 설정에서 occupation 입력 및 저장
- [x] F-03 피드로 돌아가 첫 번째 기사 상세 진입
- [x] F-04 AI 요약 버튼 클릭 → insight 영역 텍스트 표시 확인
- [x] F-05 재작성 버튼 클릭 → 재작성 결과 텍스트 표시 확인
- [x] F-06 스크랩 저장 버튼 클릭 → 즐겨찾기 탭에서 기사 확인

### 엣지
- [x] E-01 occupation 미설정 상태에서 요약 시 insight 영역 없음 또는 일반 요약만
- [x] E-02 재작성 결과 로딩 중 버튼 비활성화 확인

### 에러
- [x] R-01 API 타임아웃 시 에러 메시지 표시 (빈 화면 아님)

### UI·UX
- [x] U-01 재작성 결과가 화면에 가시적으로 표시됨 (스크롤 없이 확인 가능)
- [x] U-02 스크랩 완료 후 버튼 상태 변경 (저장됨 표시)

### 테스트
- [x] T-01 전체 시나리오 통과 (npx playwright test occupation-insight-rewrite)
- [x] T-02 기존 E2E 테스트 회귀 없음 (feed-like, smoke 등)
- [x] T-03 헤드리스 모드에서 정상 실행
- [x] T-04 CI 환경 실행 가능 (환경변수 의존 없음)
