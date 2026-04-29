# M2: 실사용 버그 수정

> 프로젝트: Frank MVP14
> 상태: done
> 예상 기간: 1주
> 의존성: M1 (E2E 인프라)

## 목표

실사용(260429) 중 발견된 버그 4건(BUG-006~008, BUG-010)을 수정한다. M1에서 구축한 E2E 인프라를 활용해 수정 결과를 검증한다. BUG-009는 외부 의존 한계로 M1 기획 중 문서화 완료.

## 성공 기준 (Definition of Done)

- [x] BUG-006: 에러 캐시 없음 확인 — iOS/웹 요약 캐시 에러 저장 안 함, 서버 부분 실패 1분 TTL 적용
- [x] BUG-007: pull-to-refresh 코드 정상 동작 확인 — noCache:true → Cache-Control:no-cache 전송
- [x] BUG-008: 태그 탭 전환 시 "기사가 없습니다" 깜빡임 제거 — iOS isLoading + 웹 selectTag 순서 수정
- [x] BUG-009: 외부 의존 한계 확인 — 동일 이벤트 보도자료 이미지 공유로 수정 불가, 문서화 완료
- [x] BUG-010: 태그 전환 기사 자동 변경 코드 탐색 완료 → 정상 동작(캐시 만료 후 갱신) 확인, 문서화
- [x] 서버 331 / 웹 267 / iOS 262+4 테스트 전체 통과

## 작업 순서 원칙

**서버/DB 먼저 → iOS + 웹 독립 병렬** 순서로 진행.
서버 수정이 확정되어야 클라이언트 구현 기준이 생기므로, 서버 관련 항목을 먼저 완료한 뒤 iOS·웹을 독립적으로 작업한다.

## 아이템

| # | 아이템 | 유형 | 플랫폼 | 순서 | 상태 |
|---|--------|------|--------|------|------|
| 0 | 캐시 계층 전체 맵 작성 (BUG-006/007/008/010 공통 원인 분석) | research | 서버 + iOS + Web | 1단계 | 대기 |
| 1 | BUG-006: 서버 에러 캐시 정책 확인 + 에러 캐시 제거 | feature | 서버 | 1단계 | 대기 |
| 2 | BUG-009: 외부 의존 한계 문서화 완료 | — | — | 완료 | ✅ |
| 3 | BUG-006: iOS 에러 캐시 제거 + 재시도 수정 | feature | iOS | 2단계 | 대기 |
| 4 | BUG-006: 웹 에러 캐시 제거 + 재시도 수정 | feature | Web | 2단계 (iOS와 병렬) | 대기 |
| 5 | BUG-007: iOS pull-to-refresh 캐시 무효화 수정 | feature | iOS | 2단계 | 대기 |
| 6 | BUG-008: iOS 탭 전환 빈 상태 깜빡임 제거 | feature | iOS | 2단계 | 대기 |
| 7 | BUG-008: 웹 탭 전환 빈 상태 깜빡임 제거 | feature | Web | 2단계 (iOS와 병렬) | 대기 |
| 8 | BUG-010: 태그 전환 자동 변경 동작 코드 탐색 + 이관 판단 | research | iOS + Web | 2단계 | 대기 |

## 워크플로우 진입점

```
/workflow "M2-버그 수정: BUG-006 에러 캐시 제거 + BUG-007 pull-to-refresh + BUG-008 깜빡임 + BUG-010 동작 조사"
```

**메인태스크**: 실사용 발견 버그 4건(BUG-006~008, BUG-010)을 서버 먼저 수정 후 iOS·웹 병렬로 수정하여 앱 체감 품질을 정상 수준으로 복원한다.

**작업 순서**:
1. **1단계 — 서버**: 캐시 계층 전체 맵 작성 + BUG-006 서버 측 에러 캐시 정책 확인/수정
2. **2단계 — iOS + 웹 병렬**: 서버 수정 완료 후 iOS·웹 각각 독립적으로 진행
   - iOS: BUG-006 에러 캐시, BUG-007 pull-to-refresh, BUG-008 깜빡임, BUG-010 탐색
   - 웹: BUG-006 에러 캐시, BUG-008 깜빡임 (iOS와 병렬)

- BUG-006(에러 캐시) → 에러 상태 캐시 제거 (캐시 키에 에러 저장 금지 또는 TTL 0 설정)
- BUG-007(pull-to-refresh) → iOS FeedFeature pull-to-refresh 핸들러에 캐시 무효화 추가
- BUG-008(깜빡임) → 탭 전환 시 캐시 확인 후 렌더 (빈 상태 선렌더 제거)
- BUG-009(썸네일) → ✅ 외부 의존 한계 문서화 완료, 수정 없이 종료
- BUG-010(자동 변경) → 캐시 TTL·갱신 트리거 탐색 → **판단 분기**: 동일 캐시 영역이면 M2 내 수정, UI 제스처/렌더 영역이면 M3 이관, 별개 기능이면 별도 이관 기록

## KPI (M2)

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| 테스트 전체 통과 | cargo test + vitest + xcodebuild test | 전체 통과 | Hard | 서버 328 / 웹 265 |
| BUG-006~010 수정/조사 완료 | progress/bugs.md 상태 | 전체 RESOLVED 또는 조사 완료 기록 | Hard | 현재 4건 OPEN |
| 회귀 버그 0건 | E2E 인프라 + 수동 QA | 0건 | Hard | — |

## M2 완료 후 검증 체크포인트

> **실행 타이밍**: ST-09(전체 테스트 통과 + KPI 검증) 완료 직후, M3 진입 전
> **참고 문서**: `progress/mvp14/cache_map.md` (ST-01 산출물)

### 검증 목적

M2 버그 수정 과정에서 확보한 `cache_map.md`를 기반으로:
1. **F-03 필요성 확정** — 부분 실패 1분 TTL이 실제로 필요한지 판단
2. **캐시 전체 이해 세션** — 피드 로드 → 태그 필터 → 새로고침 → pull-to-refresh → 캐싱 → DB캐싱 전체 흐름을 처음부터 끝까지 설명

### 체크리스트

```
[ ] 1. cache_map.md 완성 여부 확인
       - 서버·iOS·웹 3개 플랫폼 캐시 키 목록 존재
       - TTL 설정값 기록됨
       - 에러 응답 저장 여부 기록됨

[ ] 2. 부분 실패 시나리오 실존 여부 확인 (ST-01 결과 기반)
       - 서버 코드에 부분 실패(1개+ 태그 에러) 캐시 저장 경로가 있는가?

[ ] 3. 판단 매트릭스 실행
```

### 판단 매트릭스

| ST-01 결과 | 사용자 영향 | 구현 복잡도 | 결정 |
|-----------|------------|------------|------|
| 부분 실패 캐시 코드 없음 | 없음 | — | M3 진행, F-03 폐기 |
| 부분 실패 캐시 코드 있음 | 있음 | 단순 (feed.rs 5줄 이내) | **M2.5 추가 후 바로 수정** |
| 부분 실패 캐시 코드 있음 | 있음 | 복잡 (신규 로직 50줄+) | debts.md DEBT 등록 후 M3 진행 |

### M2.5 추가 시 진입점

```
M2.5 마일스톤 신규 생성:
- 목표: BUG-006 부분 실패 1분 단축 TTL 적용
- 범위: server/src/api/feed.rs 단일 파일
- 예상 기간: 1~2일
- 의존성: M2 완료
```

### 이해 세션 진입점

M2 (+ M2.5 해당 시) 완료 후 아래 명령으로 설명 세션 시작:

```
"cache_map.md 기반으로 피드 로드 → 태그 필터 → 새로고침 → pull-to-refresh → 캐싱 → DB캐싱
전체 흐름을 서버-iOS-웹 순서로 처음부터 끝까지 설명해줘."
```

---

## 구현 결정 (인터뷰 확정)

- **캐시 맵 범위**: 코드 정적 분석 — 캐시 키 + TTL + 무효화 조건 + 에러 저장 여부 + 플랫폼별 비교표
- **BUG-006 서버 수정 방식**: 단축 TTL — 완전 성공 5분 / 부분 실패 1분 / 완전 실패 저장 스킵
- **클라이언트 캐시 무효화**: Cache-Control 헤더 기반 자동 재요청
- **BUG-010 판단 기준**: 사용자 명시적 탭 없이 기사 변경 = 버그

## 리스크

| 리스크 | 영향(H/M/L) | 대응 |
|--------|------------|------|
| BUG-010이 실제 버그로 판명될 경우 추가 수정 범위 발생 | M | 탐색 결과에 따라 M2 내 추가 태스크 또는 M3로 이관 |
| 캐시 무효화 수정이 성능 저하로 이어질 가능성 | L | 캐시 TTL 정책 유지하되 에러 상태/refresh 케이스만 선택적 무효화 |

## Feature List
<!-- size: 대형 | count: 31 | skip: false -->

### 기능
- [x] F-01 ST-01: 서버·iOS·웹 캐시 계층 맵 (progress/mvp14/cache_map.md) 작성 완료
- [x] F-02 ST-02: BUG-006 서버 — 완전 실패(0건) 시 캐시 저장 스킵 유지 확인
- [x] F-03 ST-02: BUG-006 서버 — 부분 실패(1개+ 태그 에러) 시 1분 단축 TTL 저장
- [x] F-04 ST-02: BUG-006 서버 — 완전 성공 시 5분 TTL 기존 동작 유지
- [x] F-05 ST-03: BUG-006 iOS — 에러 캐시 없음 확인 (SummarySessionCache 성공만 저장), 재시도 가능
- [x] F-06 ST-04: BUG-006 웹 — 에러 캐시 없음 확인 (summaryCache 성공만 저장) + feedStore 에러 키 제거
- [x] F-07 ST-05: BUG-007 iOS — pull-to-refresh noCache:true → Cache-Control:no-cache 헤더 전송 확인
- [x] F-08 ST-06: BUG-008 iOS — isLoading에 tagStates[currentKey]?.status==.loading 조건 추가
- [x] F-09 ST-07: BUG-008 웹 — selectTag tagCache loading 먼저 / activeTagId 변경 후 / 에러 시 키 제거
- [x] F-10 ST-08: BUG-010 — 정상 동작 확인, BUG-010_analysis.md 문서화 완료

### 엣지
- [x] E-01 BUG-006 서버: 3개 태그 중 2개 실패 시 1분 TTL 정상 적용 (feed_cache_partial_failure_uses_1min_ttl 테스트)
- [x] E-02 BUG-006 서버: 모든 태그 실패 시 캐시 저장 없이 에러 응답 (기존 !items.is_empty() 가드 유지)
- [x] E-03 BUG-007 iOS: pull-to-refresh 연속 실행 방지 (refresh() guard phase == .idle)
- [x] E-04 BUG-008: 캐시 만료 직전 탭 전환 시 loading 상태 표시로 깜빡임 방지
- [x] E-05 BUG-010: 탐색 결과 "정상 동작"으로 확인, 코드 수정 없이 BUG-010_analysis.md 문서화로 종료

### 에러
- [x] R-01 BUG-006 서버: tracing::warn!만 기록, 에러 응답에 내부 세부정보 미노출
- [x] R-02 BUG-007 iOS: refresh 실패 시 tagStates 유지 (rebuildTagStates 미호출) + failRefresh("새로고침에 실패했습니다.")
- [-] N/A (Cache-Control 미지원 환경 없음 — fetchFeed는 plain fetch 사용, 헤더 미지원 시 서버는 no-cache 스킵 후 정상 처리) R-03 BUG-006 웹: Cache-Control 미지원 환경 fallback
- [x] R-04 BUG-010: BUG-010_analysis.md에 코드 증거(selectTag 흐름) 명시 후 결론 도출

### 테스트
- [x] T-01 cargo test 전체 통과 — 331개 (기준선 328+3 신규)
- [x] T-02 vitest 전체 통과 — 267개 (기준선 265+2 신규)
- [x] T-03 xcodebuild test 전체 통과 — 단위 262개 + UITests 4개
- [x] T-04 BUG-006 서버 단위테스트: feed_cache_partial_failure_uses_1min_ttl 통과
- [-] N/A (BUG-007 코드 탐색 결과 정상 동작 확인 — UITest 추가 불필요) T-05 BUG-007 iOS UITest
- [x] T-06 BUG-008 웹 테스트: 탭 전환 캐시 미스 깜빡임 방지 + 에러 캐시 저장 방지 2건 추가
- [x] T-07 회귀 테스트: 서버 331 / 웹 267 / iOS 266 전체 통과로 회귀 없음 확인

### 플랫폼
- [x] P-01 xcodebuild build — 에러 없이 통과 (iOS 단위 262 + UITest 4 통과)
- [x] P-02 iOS Simulator iPhone 17 Pro 동작 확인 (UITests 통과)
- [x] P-03 Tuist generate 후 빌드 정상 (Project generated. Total time: 0.8s)

### 회귀
- [x] G-01 캐시 TTL 변경 후 정상 피드 로드 동작 유지 (서버 331 통과, 완전 성공 TTL 5분 기존 동작 유지)
- [x] G-02 기존 피드 로드 iOS UITest LoginFlow/OnboardingFlow 통과
- [x] G-03 웹 feedStore 수정 후 기존 267 테스트 전체 통과, API 프록시 라우트 회귀 없음
