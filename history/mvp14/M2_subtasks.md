# M2 서브태스크 목록

> 메인태스크: 실사용 발견 버그 4건(BUG-006~008, BUG-010) 수정
> 생성일: 2026-04-29
> 브랜치: feature/260429_m2_bug_fixes

## 서브태스크 목록

| ID | 서브태스크 | 플랫폼 | 단계 | 산출물 | 상태 |
|----|-----------|--------|------|--------|------|
| ST-01 | 캐시 계층 전체 맵 작성 | 서버+iOS+웹 | 1단계 | progress/mvp14/cache_map.md | 대기 |
| ST-02 | BUG-006: 서버 에러 캐시 정책 확인 + 제거 | 서버(Rust) | 1단계 | 에러 캐시 제거 코드 + cargo test 통과 | 대기 |
| ST-03 | BUG-006: iOS 에러 캐시 제거 + 재시도 수정 | iOS(Swift) | 2단계 | 수정 코드 + xcodebuild test 통과 | 대기 |
| ST-04 | BUG-006: 웹 에러 캐시 제거 + 재시도 수정 | 웹(Svelte) | 2단계 | 수정 코드 + vitest 통과 | 대기 |
| ST-05 | BUG-007: iOS pull-to-refresh 캐시 무효화 | iOS(Swift) | 2단계 | 캐시 무효화 추가 코드 + UITest 확인 | 대기 |
| ST-06 | BUG-008: iOS 탭 전환 깜빡임 제거 | iOS(Swift) | 2단계 | 캐시 선확인 후 렌더 Swift 코드 | 대기 |
| ST-07 | BUG-008: 웹 탭 전환 깜빡임 제거 | 웹(Svelte) | 2단계 | 수정된 Svelte 코드 | 대기 |
| ST-08 | BUG-010: 태그 전환 자동 변경 코드 탐색 + 판단 | iOS+웹 | 2단계 | 판단 기록 문서 (정상/버그/이관 결정) | 대기 |
| ST-09 | 전체 테스트 통과 + KPI 검증 | 전체 | 최종 | 테스트 통과 로그 + bugs.md RESOLVED | 대기 |

## 의존성 DAG

```
ST-01 (캐시 계층 맵)
  └── ST-02 (서버 BUG-006) ─┐
  └── ST-03 (iOS BUG-006)   ├── ST-09 (전체 검증)
  └── ST-04 (웹 BUG-006)    │
  └── ST-05 (iOS BUG-007)  ─┤
  └── ST-06 (iOS BUG-008)  ─┤
  └── ST-07 (웹 BUG-008)   ─┤
  └── ST-08 (BUG-010 탐색) ─┘
```

**실행 원칙**: ST-01 완료 → ST-02 우선 완료 (서버 먼저) → ST-03~ST-08 병렬 → ST-09

## 병렬 그룹

| 그룹 | 서브태스크 | 실행 조건 |
|------|-----------|---------|
| A | ST-01 | 즉시 시작 |
| B | ST-02 | ST-01 완료 후 |
| C | ST-03, ST-04, ST-05, ST-06, ST-07, ST-08 | ST-01 완료 후 (ST-02와 병렬 가능, 단 서버 우선 원칙상 ST-02 완료 후 권장) |
| D | ST-09 | 그룹 C 전체 완료 후 |

## 서브태스크 상세

### ST-01: 캐시 계층 전체 맵 작성

**목적**: BUG-006/007/008/010의 공통 원인인 캐시 동작 전체 파악  
**탐색 대상**:
- `server/src/` — 캐시 모듈 (Redis? 인메모리 HashMap? 미들웨어?)
- `ios/Frank/` — FeedFeature, SummaryFeature 캐시/저장소 레이어
- `web/src/` — stores, fetch 래퍼, SWR 패턴 여부

**산출물**: `progress/mvp14/cache_map.md`
- 각 플랫폼별 캐시 키 목록
- TTL 설정값
- 무효화(invalidation) 조건
- 에러 응답 저장 여부

---

### ST-02: BUG-006 서버 에러 캐시 제거

**목적**: 요약 실패 응답이 캐시에 저장되어 재시도 불가한 문제 수정  
**작업**:
1. 서버 캐시 구현 위치 확인 (ST-01 결과 활용)
2. 에러 응답(4xx/5xx 또는 None 값) 캐시 저장 조건 제거
3. 에러 TTL 0 설정 또는 저장 skip 로직 추가

**검증**: `cargo test` 전체 통과

---

### ST-03: BUG-006 iOS 에러 캐시 + 재시도 수정

**목적**: iOS에서 요약 실패 후 재시도 시 실제 API 호출 발생하도록 수정  
**작업**:
1. SummaryFeature / FeedFeature에서 에러 상태 캐시 저장 코드 탐색
2. 에러 캐시 저장 제거
3. 재시도 시 캐시 우회하여 API 호출하는 로직 확인/추가

**검증**: `xcodebuild test`

---

### ST-04: BUG-006 웹 에러 캐시 + 재시도 수정

**목적**: 웹에서 요약 실패 후 재시도 시 실제 API 호출 발생하도록 수정  
**작업**:
1. web/src/ stores, fetch 래퍼에서 에러 응답 캐시 저장 코드 탐색
2. 에러 캐시 저장 제거

**검증**: `npm run test`

---

### ST-05: BUG-007 iOS pull-to-refresh 캐시 무효화

**목적**: 당겨서 새로고침 시 캐시가 초기화되어 실제 기사 목록이 갱신되도록 수정  
**작업**:
1. FeedFeature pull-to-refresh 핸들러 위치 확인
2. 핸들러에 캐시 무효화(cache clear or force-refresh) 추가

**검증**: 시뮬레이터 UITest 또는 수동 확인

---

### ST-06: BUG-008 iOS 탭 전환 깜빡임 제거

**목적**: 태그 탭 전환 시 "기사가 없습니다" 빈 상태 일시 노출(깜빡임) 제거  
**작업**:
1. 태그 탭 전환 → 기사 목록 로딩 상태 전이 코드 탐색
2. 캐시된 데이터가 있으면 빈 상태 없이 바로 렌더, 없으면 로딩 스피너 표시 로직 구현

**검증**: 시뮬레이터 확인 + xcodebuild test

---

### ST-07: BUG-008 웹 탭 전환 깜빡임 제거

**목적**: 웹에서 태그 탭 전환 시 빈 상태 노출 제거  
**작업**:
1. web/src/ 태그 탭 전환 로직 탐색
2. 이전 데이터 유지하며 새 데이터 로딩 후 교체하는 stale-while-revalidate 패턴 적용

**검증**: npm run test + Playwright 확인

---

### ST-08: BUG-010 태그 전환 자동 변경 동작 탐색 + 판단

**목적**: 태그 탭 전환 시 현재 기사가 자동으로 바뀌는 동작이 정상인지 버그인지 판단  
**작업**:
1. iOS + 웹 태그 전환 시 기사 자동 변경 트리거 코드 탐색
2. 캐시 TTL 만료? UI 제스처? 렌더 영역? 원인 분류
3. 판단: 정상 → 문서화 후 종료 / 버그 → M2 내 수정 또는 M3 이관

**산출물**: `progress/mvp14/BUG-010_analysis.md`

---

### ST-09: 전체 테스트 통과 + KPI 검증

**목적**: 수정된 모든 버그가 회귀 없이 동작하며 M2 KPI를 충족하는지 확인  
**작업**:
1. `cd server && cargo test`
2. `cd web && npm run test`
3. `xcodebuild test -scheme Frank -destination 'platform=iOS Simulator,name=iPhone 17 Pro'`
4. E2E 시나리오 확인 (M1 인프라 활용)
5. `bash scripts/kpi-check.sh`
6. `progress/bugs.md` BUG-006~008, BUG-010 상태 → RESOLVED 갱신

**검증**: kpi-check.sh exit 0
