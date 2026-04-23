# Discovery: Frank MVP11

> 생성일: 260423

## 컨텍스트 분석 (기존 프로젝트 진화)

### 버그 목록 (`progress/bugs.md` 기반)

| ID | 위치 | 심각도 | 원인 요약 |
|----|------|--------|----------|
| BUG-001 | iOS | 높음 | Supabase 세션 복원 완료 전 API 요청 → 401/토큰 없음 |
| BUG-002 | iOS | 높음 | xcconfig SERVER_URL 하드코딩 → 시뮬레이터에서 LAN IP 접근 실패 |
| BUG-003 | iOS+웹 | 중간 | 즐겨찾기·오답 화면에 태그 필터 UI 미적용 (API는 이미 태그 반환 중) |
| BUG-004 | 서버 | 중간 | Tavily `topic:news` 없음 + Exa `type:news`/날짜 필터 없음 → 인덱스 페이지 혼입 |

### 코드베이스 현황

**BUG-001 관련**:
- `SupabaseAuthAdapter.currentSession()` — `try? await client.auth.session` 호출
- `AuthFeature.checkSession()` — 앱 시작 시 세션 확인 후 `APITagAdapter` 등 바로 호출
- Supabase SDK 콘솔 경고: `emitLocalSessionAsInitialSession: true` 옵션 제안 중

**BUG-002 관련**:
- `Config.xcconfig`: `SERVER_URL = http://localhost:8080` (현재는 localhost로 수정된 상태)
- `ServerConfig.swift`: Info.plist → Secrets.plist → localhost 폴백 순서로 읽음
- `#if targetEnvironment(simulator)` 분기 없음

**BUG-003 관련**:
- `FavoritesView.swift`: 세그먼트 탭(기사/오답노트) 있지만 태그 필터 없음
- `FeedView.swift`: 태그 칩 필터 컴포넌트 이미 구현됨 → 재사용 가능
- API: `GET /me/favorites`, `GET /me/wrong-answers` 모두 태그 데이터 반환 중

**BUG-004 관련**:
- `infra/tavily.rs`: `topic` 파라미터 없음, `time_range:"week"` + `search_depth:"advanced"` 있음
- `infra/exa.rs`: `type:"news"` 없음, `startPublishedDate` 없음
- `infra/firecrawl.rs`: 뉴스/날짜 필터 API 자체 없음 → URL 패턴 후처리로만 보완
- `infra/search_chain.rs`: Tavily → Exa → Firecrawl 순 폴백 체인

## 수렴 결과

### 이번에 넣을 것 (In)

| # | 아이템 | 유형 | 실행 스킬 | 마일스톤 |
|---|--------|------|----------|---------|
| 1 | BUG-004 검색엔진 뉴스 파라미터 + 후처리 필터 | bug | /workflow | M1 |
| 2 | BUG-001 iOS 세션 초기화 순서 수정 | bug | /workflow | M2 |
| 3 | BUG-002 시뮬레이터/실기기 URL 자동 분기 | bug | /workflow | M2 |
| 4 | BUG-003 즐겨찾기·오답 태그 필터 UI (웹) | feature | /workflow | M3 |
| 5 | BUG-003 즐겨찾기·오답 태그 필터 UI (iOS) | feature | /workflow | M4 |

### 다음에 할 것 (Next)

| # | 아이템 | 메모 |
|---|--------|------|
| 1 | Firecrawl 완전 교체 | 뉴스 필터 API 없어 근본 해결 어려움. M3에서 후처리로만 보완 |

### 안 할 것 (Out)

| # | 아이템 | 사유 |
|---|--------|------|
| 1 | Supabase SDK 버전 업그레이드 | `emitLocalSessionAsInitialSession` 옵션이 현 SDK에서도 지원됨, 업그레이드 불필요 |
| 2 | 별도 검색엔진 추가(Perplexity 등) | API 비용 정책상 추가 엔진 도입 금지 |
