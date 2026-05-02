# MVP15 M1 검색엔진 진단 바이너리

**SSOT**: `progress/mvp15/M1_search_diagnosis.md`

활성 태그 × 5변형 × 2엔진 격자를 측정해 `progress/mvp15/M1_diagnosis_data.md` 를
자동 생성한다. 후속 `M1_diagnosis_report.md` 는 사용자가 수동 작성.

## 구조

| 모듈 | 책임 |
|------|------|
| `env_loader` | `.env.diagnose` / `.env` 로딩 + 4개 필수 변수 검증 |
| `mapping_resolver` | 한국어 태그 → 영어 직역 매핑 (12개 사전 등록) |
| `variant_generator` | 5변형 쿼리 생성 (baseline/simple/year2026/announcement/english_baseline) |
| `retry_classifier` | 5xx/timeout/network 한정 재시도 + 영구/간헐 판정 |
| `markdown_formatter` | data.md 헤더·표·매핑 출력 (snapshot 테스트) |
| `runner` | 측정 루프 + append-flush + 원자적 rename |
| `diagnose_search.rs` (main) | env→DB→pre-flight→어댑터 직접 인스턴스화→runner 위임 |

## 실행

```bash
# 1) 진단 전용 환경변수 파일 준비 (운영 .env 오염 방지)
cp server/.env.example server/.env.diagnose   # 또는 직접 작성
# 다음 4개 키 설정 필수:
#   DATABASE_URL, DIAGNOSE_USER_ID(UUID), TAVILY_API_KEY, EXA_API_KEY

# 2) 저장소 루트에서 실행 (CWD가 progress/mvp15/ 의 부모여야 함)
cd /path/to/frank
cargo run --bin diagnose_search --manifest-path server/Cargo.toml
```

Pre-flight 단계에서 다음 입력 요구:
- Tavily 사전 한도 잔여 (대시보드 확인 후 정수)
- Exa 사전 한도 잔여 (정수, 0~10000)
- `y` / `yes` / `Y` 로 확정

## 산출물

- `progress/mvp15/M1_diagnosis_data.md` — 자동 생성 (재실행 시 overwrite)
- `progress/mvp15/M1_diagnosis_data.md.partial` — 측정 중 셀 단위 누적, 정상 완료 시 자동 삭제
- panic / Ctrl+C 발생 시 partial 파일이 보존되어 재실행 시 정황 파악 가능

## 운영 격리 (SSOT 캐시·체인 우회 정의)

- `feed_cache` / `SearchFallbackChain` import 안 함 (코드 차원 우회)
- `TavilyAdapter::new` / `ExaAdapter::new` 직접 호출 (공용 builder/factory 미경유)
- 어댑터 내부의 도메인 필터(`time_range=week`, `startPublishedDate=now-7d`)는 **운영 충실 차원에서 유지**.
  data.md 메타에 `engine_filter_notes` 로 명시.
