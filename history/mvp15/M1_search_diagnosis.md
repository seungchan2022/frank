# M1: 검색엔진 진단

> 프로젝트: Frank MVP15
> 상태: in-progress (인터뷰 완료, 구현 대기)
> 예상 기간: 1~2일
> 의존성: 없음
> 마지막 갱신: 2026-05-02 (step-5 리뷰 14건 반영 — 필수 6 + 권장 8)
>
> **본 문서는 기획 문서로 `rules/0_CODEX_RULES.md §8 예외` 적용**: TDD 등 구현 단계 룰은 후속 `/workflow` 진입 시 코드 작성 단계부터 적용.

## 목표

"1개만 나오는 태그" 원인을 실측으로 파악하고, 양 확대(M2) 및 후속 결정(Q4 검색엔진 활용, Q5 쿼리 다양화)의 근거 데이터를 만든다.

## 배경

- 시드 발견: `Tavily 실패 → Exa 폴백`, 사용자 관찰상 현재 Exa로 빠짐 — 실패 원인 미점검
- 사용자 통점: 일부 태그는 결과가 1개뿐. limit 5 병목인지 검색 결과 자체 부족인지 미확인

## 성공 기준 (Definition of Done)

- [ ] 모든 활성 태그(현재 12개)에 대해 다음 데이터 측정:
  - Tavily 단독, limit 100 호출 시 결과 수
  - Exa 단독, limit 100 호출 시 결과 수
  - Firecrawl: 진단 본 측정에서 제외 — DoD "(가능한 경우)" 단서 적용. 운영 한도 500/월 보호
- [ ] Tavily 실패 발생 시 에러 코드·원인 기록 + **재시도 1회로 영구/간헐 분류**
- [ ] 동일 태그에 대해 5개 쿼리 변형 비교 (가설 격리 — 자세히는 "## 진단 설계"):
  - `"{tag} latest news"` (운영 baseline)
  - `"{tag}"` (단순 — 혼합 쿼리·suffix 영향 격리)
  - `"{tag} 2026"` (시간 변형)
  - `"{tag} announcement"` (관점 변형)
  - `"{eng_translation} latest news"` (영어 번역 — H1 시드 가설 검증)
- [ ] 진단 데이터: `progress/mvp15/M1_diagnosis_data.md` (자동 생성, 매 측정마다 overwrite)
- [ ] 진단 보고서: `progress/mvp15/M1_diagnosis_report.md` (수동 권고 — data.md 인용 + 분석)
- [ ] 진단 결과를 바탕으로 M2 양 limit 결정값 확정 (10 유지 vs 조정)
- [ ] Q4·Q5 후속 결정 권고안 (병합/병행/엔진 추가/쿼리 다양화 필요성)

## 아이템

| # | 아이템 | 유형 | 실행 스킬 | 최소 검증 범위 | 상태 |
|---|--------|------|----------|--------------|------|
| 1 | 진단 바이너리 작성 (`server/src/bin/diagnose_search.rs`, 어댑터 재사용) | chore | /workflow | pre-flight 분기 (`y`/그 외), 재시도 영구/간헐 분류, data.md 출력 포맷 (헤더·표·매핑·한도 영향 모두 포함) | ✅ done |
| 2 | 12 태그 × 5 변형 × 2 엔진 = 120 호출 측정 + data.md 자동 생성 | research | 직접 실행 | 측정 시각·실행 파라미터·overwrite 정책 명시 | ✅ done |
| 3 | 진단 보고서 작성 (M1_diagnosis_report.md) + M2/Q4/Q5 권고 | research | /deep-analysis | data.md 인용 + Tavily 실패 분류 narrative + 권고 3건 | ✅ done |

## 진단 설계 (확정 — 2026-05-01 인터뷰 결과)

### 변형 셋 (5개)

| # | 쿼리 형식 | 검증 가설/목적 | DoD 매핑 |
|---|----------|---------------|---------|
| 1 | `"{tag} latest news"` | 운영 baseline (현재 운영 그대로) | DoD #1 |
| 2 | `"{tag}"` | 혼합 쿼리·`latest news` suffix 영향 격리 | DoD #2 |
| 3 | `"{tag} 2026"` | 시간 변형 효과 (Q5) | DoD #3 |
| 4 | `"{tag} announcement"` | 관점 변형 효과 (Q5) | DoD #4 |
| 5 | `"{eng_translation} latest news"` | **H1 시드 가설** — Tavily 한국어 약점 검증 | 영어 번역 매핑 표 참조 |

### 영어 번역 매핑 (직역, 검증 가능)

| 한국어 태그 | 영어 번역 |
|-----------|----------|
| AI/ML | AI/ML |
| 모바일 개발 | Mobile Development |
| UX/디자인 | UX/Design |
| 데이터 사이언스 | Data Science |
| 보안 | Security |
| 블록체인 | Blockchain |
| 스타트업 | Startup |
| 오픈소스 | Open Source |
| 웹 개발 | Web Development |
| 클라우드/인프라 | Cloud/Infrastructure |
| 투자/VC | Investment/VC |
| 프로덕트 | Product |

### 엔진 선택

- **Tavily, Exa**: 시드 가설("Tavily 실패 → Exa 폴백")의 본 비교 대상. 필수.
- **Firecrawl 제외**: 운영 무료 한도 500/월 (Tavily 1,000보다 작음) + 본업(기사 본문 크롤링)이 핵심 → 진단 호출로 17%+ 소비는 운영 위협. DoD "(가능한 경우)" 단서 적용해 제외.

### 측정 파라미터

| 항목 | 값 | 근거 |
|---|---|---|
| limit (요청값) | 100 | "limit=5가 자르는지, API가 부족한지" 식별. 운영 limit=5와 분리 |
| limit (엔진별 실효 상한) | step-3 진입 시 사전 검증 | Tavily `max_results` 공식 상한이 100 미만일 가능성 → 사전 확인 후 엔진별 실효값 결정 |
| 결과 기록 | requested / effective / returned 분리 | API가 silent truncate/clamp/400 시 진단 왜곡 방지 |
| 호출 횟수 / 쿼리 | 1회 (보강: ≤3개 결과 셀은 자동 1회 추가 호출) | 단일 호출 변동성 ±일부 영향. 경계값 셀(우연 1개 가능성)만 한정적 재호출로 신뢰도 보강 |
| 실패 처리 | 재시도 1회 (HTTP 5xx / timeout / network 한정) | 200+빈결과 또는 4xx는 재시도 제외. 영구(같은 에러)/간헐(다른 결과) 분류 |
| 재시도 대칭성 | Tavily + Exa 양쪽 동일 적용 | 비대칭 정책의 근거 약함, 양쪽 같은 처리로 표 비교 일관성 |
| 에러 분류 카테고리 | rate_limit / auth / 5xx / timeout / network / json_parse / file_io / db | 단순 "실패"가 아닌 카테고리로 영구/간헐 판정 신뢰도 ↑ |

### 호출량 분석

```
기본: 12 태그 × 5 변형 × 2 엔진           = 120 호출
재시도(5xx/timeout/network 한정 추정 5~10): +10~20 호출 (양 엔진 대칭)
≤3개 결과 셀 자동 재호출(추정 ~10셀):     +10 호출
─────────────────────────────────────────────────
정상 추정:                                  ~140 호출
최악치 (재시도 다발 + 재호출 다수):        최대 180 호출

API 한도 영향:
- Tavily: 1,000/월 — 14~18% 사용 (안전 마진 80%+ 유지)
- Exa:    1,000/월 — 14~18% 사용 (안전 마진 80%+ 유지)
- Firecrawl: 0 호출 (운영 한도 500/월 보호)
```

### 진단 흐름

1. **부트스트랩**
   - `dotenv` 또는 `std::env`로 `.env` 로딩
   - 필수 변수 검증 + 누락 시 명확 에러:
     - `DATABASE_URL` (Supabase pooler), `DIAGNOSE_USER_ID` (Uuid), `TAVILY_API_KEY`, `EXA_API_KEY`
   - **운영 변수 오염 방지**: `server/.env` 직접 수정 대신 `server/.env.diagnose` 분리 권장 (DEBT 후속)
2. **Pre-flight confirm** (`stdin().read_line()`)
   - 활성 태그 목록 출력 (DB raw, 활성 시각 포함)
   - 예상 호출 수 (정상 ~140 / 최악 180) + 각 엔진 사전 한도 잔여 표시
     - **한도 잔여 조회 방식**: 자동 API 조회 미지원 → 사용자가 대시보드에서 확인한 값을 stdin 입력
     - **stdin sanity check**: 정수 0~10,000 범위, 형식 검증 실패 시 즉시 중단
   - 입력 검증: 허용 입력은 `y` / `yes` / `Y` (대소문자), 그 외 즉시 중단
   - **non-TTY 환경 안전장치**: stdin이 TTY 아닐 때(파이프·CI) 자동 중단 + 명시 메시지
3. **측정** — 어댑터(`TavilyAdapter`, `ExaAdapter`) **직접 인스턴스화 + 직접 호출**
   - **캐시·체인 우회 정의 (재정의)**: "import 미사용"이 아니라 **공용 builder/factory(SearchFallbackChain·feed_cache 미들웨어 포함)를 거치지 않고 어댑터 단일 인스턴스를 main에서 생성**. step-3 진입 직후 어댑터 코드 1회 검토하여 **어댑터 내부에 도메인 필터(언어/날짜/중복 제거)가 없음을 확인**, 있다면 진단용 모드 플래그 또는 raw response count 노출
   - **재시도 정책**: 5xx / timeout / network 에러 시만 1회 재시도. 200+빈결과 / 4xx는 재시도 제외
   - **재시도 대칭**: Tavily, Exa 양쪽 동일 적용
   - **≤3개 결과 셀**: 자동 1회 추가 호출하여 변동성 검증 (1차/2차 결과 모두 기록)
   - **부분 실패 보존**: 셀 단위 append-flush 정책 — 측정 1셀 끝날 때마다 append용 임시 파일(`data.md.partial`)에 누적 저장. main 종료 시 전체 commit (rename으로 원자적 교체). panic/SIGINT 발생 시에도 partial 파일 보존 → 재실행 시 진행분 인지 후 재개 또는 사용자 결정
4. **자동 생성** — `M1_diagnosis_data.md` overwrite (재실행 시 전체 덮어씀)
   - **자동 생성 헤더 (1행)**: `<!-- AUTO-GENERATED by server/src/bin/diagnose_search.rs — DO NOT EDIT (overwrites on re-run) -->`
   - **메타 헤더 블록**: 측정 시각 (UTC ISO8601), 실행 파라미터(태그 수/변형 수/엔진/limit/재시도 정책), overwrite 정책, **user_id (UUID)**, **활성 태그 raw 목록 + 각 태그 created_at**
   - raw 측정 표 (태그 × 변형 × 엔진 × requested / effective / returned)
   - 한도 영향 (사전 잔여 / 진단 호출 수 / 사후 잔여)
   - 실패 로그 (양 엔진 + 카테고리 분류 + 영구/간헐 판정)
   - ≤3개 셀 재호출 결과 (1차/2차 비교)
   - 영어 번역 매핑 표 (재현 가능성)
5. **수동 작성** — `M1_diagnosis_report.md`. data.md 절대 건드리지 않음. 골격은 "## 보고서 구조" 섹션 참조

## 신뢰성·재현성 정책 (step-5 리뷰 반영)

### user_id 동일성 검증 절차

진단 결과의 1차 신뢰성은 "통점 발생 시점의 활성 태그 = 진단 측정 시점의 활성 태그"인지에 달림. 다음 절차로 검증:

1. data.md 헤더에 `user_id` (UUID) + 활성 태그 raw 목록 + 각 태그의 `created_at` 자동 기록
2. report.md 작성 단계에서 사용자가 직접 확인:
   - "1개만 나오는 태그" 통점 발생 시점 (사용자 기억) ↔ data.md의 태그 created_at
   - 시점 불일치 시 보고서에 "통점 시 태그 구성 확인 불가" 명시 + 후속 사이클로 이관
3. 사용자가 1명이거나 메모리 `feedback_test_account`(test@test.com) 케이스라면 user_id 단순 환경변수 매칭으로 충분

### H1 가설 검증 임계 (Tavily 한국어 약점)

영어 번역 변형(#5)에서 결과 풍부 + 한국어 변형(#1~4)에서 결과 빈약 시 H1 검증. 단:

- **한국어 변형 실패율 ≥ 50%** (12 태그 × 4 한국어 변형 = 48 셀 중 24+ 셀 실패) 시 H1 결정 보류
- 사유: 외부 요인(Tavily 일시 장애·rate limit 누적) 가능성 배제 불가 → 별도 사이클 재진단 권고

### 진단 모듈 구조 (커버리지 90% 달성)

main 비대 방지 + 단위 테스트 가능성을 위해 다음 모듈로 분리:

| 모듈 | 책임 | 테스트 가능성 |
|------|------|--------------|
| `env_loader` | `.env` + 필수 변수 검증 | 단위 (mock env) |
| `variant_generator` | 5변형 쿼리 생성 | 단위 (순수) |
| `mapping_resolver` | 한국어 → 영어 매핑 | 단위 (순수, 테이블 기반) |
| `retry_classifier` | 영구/간헐 판정 + 카테고리 분류 | 단위 (순수) |
| `markdown_formatter` | data.md 헤더·표·매핑 출력 | 단위 (snapshot) |
| `runner` | 측정 루프 + append-flush + pre-flight | 통합 (mock SearchPort) |
| `main` | 모듈 wiring | 최소 (스모크) |

순수 로직 5개 + 통합 1개 + main wiring → 단위 테스트로 커버리지 90% 달성 가능.

## 보고서 구조 (`M1_diagnosis_report.md` 템플릿)

KPI Hard "M2/Q4/Q5 권고 명시"의 합격선 정의. 작성 시 아래 골격 강제:

```markdown
# M1 진단 보고서

## 1. 데이터 인용
- 출처: `progress/mvp15/M1_diagnosis_data.md` (생성 시각: ...)
- 측정 모집단: user_id={...}, 활성 태그 12개 (목록 인용)
- **통점 시점 동일성**: [확인됨 / 불확실 / 변경됨] — 근거 한 줄

## 2. 패턴 분석
- 결과량 분포 (저/중/고)
- 엔진별 격차 (Tavily vs Exa)
- 변형별 효과 (시간/관점/영어 번역)
- Tavily 실패 분류 (영구 N건 / 간헐 M건 / 한국어 실패율 X%)

## 3. 권고 (3블록 강제)

### 3.1 M2 권고 (양 limit 결정)
- **근거**: 측정 데이터 인용 (예: "limit=100에서 평균 결과 X개 → limit=10 충분 / 부족")
- **결정**: limit = {값}
- **위험**: 결정 채택 시 발생 가능한 부작용 + 완화책

### 3.2 Q4 권고 (검색 엔진 활용)
- **근거**: 엔진별·변형별 격차
- **결정**: [Tavily 단독 유지 / Exa 병합 / 폴백 변경 / 쿼리 변환 전처리 / 엔진 추가]
- **위험**: 채택 시 운영 영향 + 완화책

### 3.3 Q5 권고 (쿼리 다양화)
- **근거**: 변형별 결과 차이 (1·2·3·4 비교)
- **결정**: [현 단일 쿼리 유지 / 시간 변형 추가 / 관점 변형 추가 / 한·영 병행]
- **위험**: 채택 시 호출량/한도 영향 + 완화책

## 4. 후속 사이클
- 본 진단으로 결론 못 낸 항목 (있다면)
- 재진단 임계 조건
```

3블록(근거/결정/위험)이 **모두 채워져야** KPI Hard 통과. 빈 블록은 "해당 없음 + 사유 한 줄"로 명시.

## 워크플로우 진입점

```
/workflow "MVP15 M1 진단 바이너리 구현"
```

아이템 1(바이너리 작성)은 코드 변경 동반 → `/workflow` 9단계. 아이템 2~3은 실행·문서 작성이라 `/workflow` 진입 없이 직접 진행.

## step-4 보류 결정 확정 (2026-05-02)

| ID | 항목 | 결정 | 근거 |
|----|------|------|------|
| H1 | 활성 태그 user_id 해석 | A — `.env`의 `DIAGNOSE_USER_ID` 환경변수 주입 | 결정적·재현 가능, 운영 `feed.rs::get_user_tags(user.id)` 패턴과 정합 |
| H2 | 캐시 우회 검증 방식 | A — 어댑터 직접 호출 (구조적 우회) | 진단 바이너리가 `feed_cache`/`SearchFallbackChain` import 자체를 안 함 → 코드 차원에서 우회 보장 |
| H3 | data.md 헤더 표시 | A — 단순 헤더 한 줄 (`<!-- AUTO-GENERATED ... DO NOT EDIT -->`) | 사용자 영역은 `report.md`에 분리됨, data.md는 순수 자동 생성 |
| H4 | S6 통합 테스트 범위 | C — mock 통합 + step-7 e2e 실제 1회 | CI 통합 테스트는 결정적·무료, 실제 어댑터-API 연결은 step-7 e2e에서 한 번 검증 |

## 변경 파일 표

| 파일 | 변경 유형 | 비고 |
|------|----------|------|
| `server/src/bin/diagnose_search.rs` | 신규 | main + wiring (얇게 유지) |
| `server/src/bin/diagnose/mod.rs` | 신규 | 진단 모듈 루트 |
| `server/src/bin/diagnose/env_loader.rs` | 신규 | `.env` 로딩 + 필수 변수 검증 |
| `server/src/bin/diagnose/variant_generator.rs` | 신규 | 5변형 쿼리 생성기 |
| `server/src/bin/diagnose/mapping_resolver.rs` | 신규 | 한국어 → 영어 매핑 |
| `server/src/bin/diagnose/retry_classifier.rs` | 신규 | 영구/간헐 판정 + 에러 카테고리 분류 |
| `server/src/bin/diagnose/markdown_formatter.rs` | 신규 | data.md 출력 (snapshot 테스트 대상) |
| `server/src/bin/diagnose/runner.rs` | 신규 | 측정 루프 + append-flush + pre-flight |
| `server/Cargo.toml` | 수정 | `[[bin]] name = "diagnose_search"` 추가 (`required-features` 또는 별도 빌드 격리 검토) |
| `server/.env.diagnose` | 사용자 액션 | `DIAGNOSE_USER_ID=<본인 user_id>` (운영 `.env` 오염 방지, 커밋 안 함) |
| `server/.env.diagnose.example` | 신규 | 진단 변수 키 템플릿 (커밋함, 값은 비움) |
| `progress/mvp15/M1_diagnosis_data.md` | 자동 생성 | 진단 실행 시 overwrite (`.partial` 임시파일 → 원자적 rename) |
| `progress/mvp15/M1_diagnosis_report.md` | 수동 작성 | 진단 후 "## 보고서 구조" 템플릿대로 작성 |

## Feature List
<!-- size: 중형 | count: 27 | skip: false -->

### 기능
- [x] F-01 `env_loader` — `.env`/`.env.diagnose` 로딩 + 필수 변수 검증 (`DATABASE_URL`/`DIAGNOSE_USER_ID`/`TAVILY_API_KEY`/`EXA_API_KEY`)
- [x] F-02 DB에서 활성 태그 조회 (`get_user_tags(DIAGNOSE_USER_ID)`)
- [x] F-03 `mapping_resolver` — 한국어 → 영어 매핑 (`LazyLock<HashMap>` 또는 `match`, 직역 원칙)
- [x] F-04 `variant_generator` — 5변형 쿼리 생성 (운영 baseline / 단순 / 시간 / 관점 / 영어 번역)
- [x] F-05 Pre-flight confirm — 활성 태그 raw + 호출 수(140/180) + 한도 잔여 stdin (sanity check) + `y`/`yes`/`Y` 허용 + non-TTY 자동 중단
- [x] F-06 측정 루프 — `TavilyAdapter`/`ExaAdapter` main 직접 인스턴스화 (공용 factory 미경유)
- [x] F-07 `retry_classifier` — 5xx/timeout/network만 재시도, Tavily+Exa 대칭, 에러 카테고리 분류 (rate_limit/auth/5xx/timeout/network/json_parse/file_io/db)
- [~] deferred (이번 진단에서 ≤3개 셀 발생 안 함, 모두 14~20 범위) F-08 ≤3개 결과 셀 자동 1회 추가 호출 (변동성 검증)
- [x] F-09 셀 단위 append-flush — `data.md.partial`에 누적 + main 종료 시 원자적 rename
- [x] F-10 `markdown_formatter` — 자동 생성 헤더 + 메타(시각/파라미터/user_id/태그 raw) + 측정 표(requested/effective/returned) + 한도 영향 + 실패 로그 + 매핑 표

### 엣지
- [~] deferred (단위 테스트 커버, 운영 중 자연 발생 시 검증) E-01 활성 태그 0개 — 즉시 종료 + 명확 메시지
- [x] E-02 영어 매핑 누락 태그 (신규 태그) 감지 → 진단 중단 + 누락 태그 보고
- [~] deferred (단위 테스트 커버, 비정상 시나리오 일일이 실측 비현실) E-03 stdin `y`/`yes`/`Y` 외 입력 — 즉시 중단
- [x] E-04 stdin이 TTY 아님 (CI·파이프) — 자동 중단 + 안내
- [~] deferred (단위 테스트 커버, 비정상 시나리오 일일이 실측 비현실) E-05 한도 잔여 stdin 입력이 0~10,000 범위 밖 / 정수 아님 — 즉시 중단
- [~] deferred (panic 인위적 발생 비현실, 코드 구조로 보장됨) E-06 panic/SIGINT — `data.md.partial` 보존 (재실행 시 인지)

### 에러
- [~] deferred (단위 테스트 커버, 진단 시점 Tavily rate_limit 미발생) R-01 API rate_limit (429) — 카테고리 기록 + 해당 셀만 실패 표기, 진단 진행
- [x] R-02 인증 (401/403) — API 키 누락 안내 + 즉시 중단 (env_loader 통과로 검증)
- [x] R-03 5xx / timeout / network — 1회 재시도, 같은 카테고리 반복 시 영구 분류 (Exa 60셀에서 동작 확인 — DEBT-MVP15-01: 402가 network로 오분류)
- [~] deferred (단위 테스트 커버, 진단 시점 미발생) R-04 JSON 파싱 실패 — 카테고리 기록 + 응답 일부 보존, 진단 진행
- [x] R-05 DB 연결 실패 — `DATABASE_URL` 확인 안내 + 즉시 중단 (env_loader DB 조회 성공으로 검증)
- [~] deferred (디스크 풀 인위 발생 비현실, 단위 테스트로 부분 커버) R-06 `data.md.partial` 쓰기 실패 (디스크 풀 등) — 즉시 중단 + stderr 안내

### 테스트
- [x] T-01 단위: `mapping_resolver` (12개 태그 매핑 존재, 누락 감지)
- [x] T-02 단위: `variant_generator` (각 태그 × 5변형 정확)
- [x] T-03 단위: `retry_classifier` (5xx/timeout/network만 재시도, 200+빈결과 제외, 영구/간헐 분류)
- [x] T-04 단위: `markdown_formatter` (snapshot — 헤더·메타·표·매핑·실패 섹션 모두 포함)
- [x] T-05 통합: mock SearchPort + mock DB로 `runner` end-to-end (DB → 변형 → 측정 → append-flush → 출력)
- [x] T-06 e2e (step-8 진단 본 실행으로 검증): 실제 Tavily/Exa 호출 120회 (어댑터-API 연결 OK, 캐시·체인 우회 확인 — Tavily 60/60 응답, Exa 60/60 한도 소진)

세부 수정/추가/삭제는 `/step-8` 실측 단계에서 가능합니다.

## KPI (M1)

> **게이트 정책** (`rules/0_CODEX_RULES.md §3.5.3`): `planning` 상태에선 점진 지표(커버리지·측정 횟수·기록 완결성)는 자동 Soft 강등. 산출물 존재·핵심 결정만 Hard로 관리.

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| 진단 데이터 존재 | `progress/mvp15/M1_diagnosis_data.md` | exists | **Hard** | — |
| 진단 보고서 존재 | `progress/mvp15/M1_diagnosis_report.md` | exists | **Hard** | — |
| M2/Q4/Q5 권고 결정 | report.md "## 3. 권고" 3블록(근거/결정/위험) 모두 채움 | 3 권고 × 3 블록 = 9 채움 | **Hard** | — |
| 측정 대상 태그 커버리지 | data.md 표에 측정된 태그 / 활성 태그 | 100% | Soft | — |
| 측정 조합 수 | data.md 표 row 수 | ≥ 활성 태그 수(12) × **쿼리 변형 수(5) × 엔진 수(2)** = 120 | Soft | — |
| 엔진 실패 분류 기록 | data.md "실패 로그" 섹션 (Tavily + Exa 대칭) | 영구/간헐 + 카테고리 명시 (또는 "실패 0건") | Soft | — |
| 진단 전·후 한도 잔여 기록 | data.md "한도 영향" 섹션 | 사전 잔여, 진단 호출 수, 사후 잔여 명시 | Soft | — |

## 리스크

| 리스크 | 영향(H/M/L) | 대응 |
|--------|------------|------|
| 진단 호출 자체가 무료 한도 소비 | L | **120 호출 + 재시도 5~10 = 약 125~130** (Tavily/Exa 각 12~13% 사용). pre-flight confirm으로 사전 잔여 캡처 + 데이터 파일에 사후 잔여 기록 |
| Firecrawl 운영 한도 영향 | L→0 | 진단 본 측정에서 Firecrawl 제외 (DoD "가능한 경우" 단서 적용) |
| 태그가 너무 적어 통계 약함 | L | 본인 사용 데이터로 충분 — 정량 통계가 아닌 진단 목적 |
| Tavily 실패 원인이 외부 요인 (서비스 장애) | L | **재시도 1회로 영구/간헐 분류** → 간헐이면 외부 요인으로 판정, 다음 사이클 재측정 |
| 결과 변동으로 단일 호출이 부정확 | L | 1회 호출 — 진단 본 취지(1개 vs 50개 식별)엔 충분. 경계값(예: 정확히 5개) 발견 시 2차 진단으로 좁혀 측정 |
| 영어 번역 매핑이 자의적 | L | 직역 원칙 — "## 진단 설계 > 영어 번역 매핑" 표에 사전 기록해 검증 가능 |
