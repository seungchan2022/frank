# 알려진 버그 목록

발견된 버그를 기록. 다음 MVP 기능 구현 전에 수정 후 진행.

---

## [BUG-001] 앱 첫 실행 시 태그/데이터 로딩 실패

**발견**: 2026-04-22 실기기 테스트 중
**재현**: 시뮬레이터·실기기 공통, 항상 재현됨
**증상**: 앱 시작 직후 "태그를 불러오지 못했습니다" 에러 표시 → 다시 시도 누르면 정상 동작
**원인**: Supabase 세션 복원이 완료되기 전에 API 요청이 먼저 나감 → Bearer 토큰 없음 → 401/연결 실패

콘솔 경고:
```
Initial session emitted after attempting to refresh the local stored session.
To opt-in to the new behavior now, set `emitLocalSessionAsInitialSession: true`
```

**수정 방법**: Supabase AuthClient 설정에 `emitLocalSessionAsInitialSession: true` 추가,
또는 세션 준비 완료 이벤트를 기다린 후 데이터 요청 시작하도록 초기화 순서 수정

**우선순위**: 다음 MVP 시작 전 수정

---

## [BUG-002] 시뮬레이터/실기기 SERVER_URL 분리 안 됨

**발견**: 2026-04-23 시뮬레이터 테스트 중
**재현**: 실기기 테스트 후 시뮬레이터 전환 시 항상 재현
**증상**: `Config.xcconfig`의 `SERVER_URL`이 실기기용 LAN IP로 고정돼 있어 시뮬레이터에서 API 연결 실패 → 스플래시 화면 무한 대기

**원인**: `Config.xcconfig`에 `SERVER_URL = http://192.168.x.x:8080` 하드코딩.
시뮬레이터는 `localhost:8080` 접근 가능하지만 LAN IP로 연결 시도 → stall timeout

**수정 방법**:
- `ServerConfig.swift`에 `#if targetEnvironment(simulator)` 분기 추가
  - 시뮬레이터: `localhost:8080` 컴파일 타임 고정
  - 실기기: xcconfig 값 사용
- (선택) `deploy.sh --target=ios`에서 현재 Mac LAN IP를 자동 감지해 xcconfig 업데이트

**우선순위**: 다음 MVP 시작 전 수정

---

## [BUG-003] 즐겨찾기/오답 노트 태그 필터 없음

**발견**: 2026-04-23
**증상**: 즐겨찾기·스크랩·오답 화면이 전체 목록만 표시. 태그/키워드 필터 없어 콘텐츠 누적 시 탐색 불편
**원인**: 피드에는 태그 칩 필터가 있지만 즐겨찾기·오답 화면에 동일 컴포넌트 미적용
**수정 방법**: API는 이미 태그 데이터 반환 중. iOS + 웹 각 화면에 태그 필터 UI 추가 (피드와 동일 컴포넌트 재사용)
**우선순위**: MVP11 기능 개선 항목

---

## [BUG-004] 기사 목록 인덱스 페이지가 기사로 수집됨

**발견**: 2026-04-23 웹 테스트 계정 확인 중
**증상**: 개별 기사 대신 뉴스 카테고리/태그 인덱스 페이지가 피드에 노출됨.
기사 소개글이 Sentry JS 코드·내비게이션 텍스트·기사 제목 나열로 표시됨
예: "Data science recent news | AI Business", "data science News & Articles - IEEE Spectrum"

**폴백 체인 (실제)**: Tavily → Exa → Firecrawl (3개, `server/src/main.rs` 기준)

**원인 (엔진별)**:
- **Tavily** (`infra/tavily.rs`): `topic` 파라미터 없음 → 일반 웹 검색 모드 → listing 페이지 혼입
  - 현재 파라미터: `query`, `max_results`, `search_depth:"advanced"`, `include_answer:false`, `time_range:"week"`
- **Exa** (`infra/exa.rs`): `type:"news"` + `startPublishedDate` 없음 → 일반 신경망 검색 → 날짜·뉴스 구분 없음
  - 현재 파라미터: `query`, `numResults`, `contents.highlights(numSentences:3, highlightsPerUrl:1)`
- **Firecrawl** (`infra/firecrawl.rs`): 뉴스/날짜 필터 API 자체 없음. `published_at`·`image_url` 미반환
  - 현재 파라미터: `query`, `limit` 전부

**수정 방법**: 엔진별 뉴스 필터 파라미터 추가 후 실제 수집 결과 비교 검증 필수
  - **Tavily**: `"topic": "news"` 추가. `time_range:"week"`과 조합 동작 확인 필요 (공식 문서 상 조합 지원 여부 불명)
  - **Exa**: `"type": "news"` + `"startPublishedDate"` 추가 (현재 날짜 기준 7일 전 ISO8601 동적 생성)
  - **Firecrawl**: `/v1/search` API에 뉴스·날짜 필터 없음. 후처리 URL 패턴 필터로만 보완 가능
  - (공통 후처리) URL 패턴 필터: `/tag/`, `/category/`, `/topics/` 등 listing 경로 제외
**우선순위**: MVP11 기능 개선 항목
