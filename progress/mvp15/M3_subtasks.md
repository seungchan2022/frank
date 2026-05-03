# M3 서브태스크 분해

> 메인태스크: 사용자 프로필(직업 한 줄)을 받아 iOS 개발자 시각의 인사이트를 생성·표시한다 (서버 + 웹 + iOS)
> 작성일: 2026-05-03
> 의존성: M2 done

---

## 서브태스크 목록

### ST-1: DB 마이그레이션 — profiles + favorites 테이블 컬럼 추가
- **유형**: infrastructure
- **산출물**: `supabase/migrations/20260503_mvp15_m3_profiles_occupation.sql`
- **범위**:
  - `ALTER TABLE profiles ADD COLUMN occupation TEXT NULL;`
  - `ALTER TABLE favorites ADD COLUMN rewrite TEXT NULL;` — C2 수정: rewrite 저장 컬럼 사전 추가
  - 기존 행은 NULL로 자동 채워짐
- **의존성**: 없음
- **크기**: 20분

---

### ST-2: 서버 — Profile 도메인 모델 + DbPort + FakeDbAdapter + PostgresDbAdapter occupation 필드 추가
- **유형**: feature (서버 인프라)
- **산출물**:
  - `server/src/domain/models.rs` — `Profile` 구조체에 `occupation: Option<String>` 추가
  - `server/src/domain/ports.rs` — `update_profile` 시그니처에 `occupation: Option<String>` 파라미터 추가
  - `server/src/api/profile.rs` — `UpdateProfileRequest`에 `occupation` 필드 추가, 핸들러에 검증(최대 50자) + occupation 처리 추가
  - `server/src/infra/fake_db.rs` — `seed_profile`, `update_profile` occupation 지원
  - `server/src/infra/postgres_db.rs` — `update_profile` SQL에 occupation SET 절 추가 + UPSERT 전환 (M2 수정: profiles row 없는 구버전 계정 404 방지)
  - 기존 `profile.rs` 테스트의 `Profile { id, display_name, onboarding_completed }` 직접 생성 4곳 → occupation 필드 추가 갱신 필수
  - 테스트: profile occupation PATCH 시나리오 3개 (설정, 빈값 400, 초과 400)
- **의존성**: ST-1 (DB 컬럼 존재해야 postgres 테스트 통과)
- **크기**: 1.5시간

---

### ST-3: 서버 — LlmPort에 `summarize_with_occupation` + `rewrite_with_occupation` 메서드 추가 + Groq 구현
- **유형**: feature (서버 도메인 + 인프라)
- **산출물**:
  - `server/src/domain/ports.rs` — 두 메서드 모두 **`Pin<Box<dyn Future<...> + Send + '_>>`** 시그니처 필수 (M4 수정: `impl Future` 사용 시 `Arc<dyn LlmPort>` 컴파일 오류)
    - `summarize_with_occupation(title, content, occupation: Option<&str>)` → `Result<LlmResponse, AppError>`
    - `rewrite_with_occupation(title, content, occupation: &str)` → `Result<String, AppError>` (M4에서 ST-4로 이동: 동일 포트 변경 한 번에 처리)
  - `server/src/domain/models.rs` — `LlmSummary.insight` 타입을 `Option<String>`으로 변경 + 영향받는 호출처 전체 갱신:
    - `update_favorite_summary`의 `insight: &'a str` 파라미터 → `insight: Option<&'a str>` 변경 (포트 + 어댑터 2곳)
    - `summary_service.rs` 결과 처리 갱신
    - `SummarizeResponse.insight` → `Option<String>` (ST-4에서 처리하나, 타입 변경 파급 범위 이 ST에서 사전 파악)
  - `server/src/infra/groq.rs` — occupation 있을 때 별도 프롬프트 분기, occupation 없을 때 기존 프롬프트
  - **프롬프트 상수 분리**: `groq.rs` 내 `SYSTEM_PROMPT_NO_OCCUPATION`, `SYSTEM_PROMPT_WITH_OCCUPATION`, `SYSTEM_PROMPT_REWRITE` 각각 정의
  - `server/src/infra/fake_llm.rs` — 두 메서드 구현
  - 테스트: occupation 있을 때 insight 포함, occupation None일 때 insight None 반환, rewrite 정상 반환
- **의존성**: ST-2 (Profile 모델 occupation 추가 후)
- **크기**: 2시간

---

### ST-4: 서버 — 요약하기 엔드포인트 occupation 연동 + 재작성 엔드포인트 신규 추가 + 응답 스키마 박제
- **유형**: feature (서버 API)
- **산출물**:
  - `server/src/api/summarize.rs` — `post_summarize` 핸들러 변경:
    - JWT → `db.get_profile(user_id).occupation` 조회. **조회 실패 시 `occupation=None`으로 degrade** (M3 수정: DB 장애가 요약 전체 outage 방지)
    - `summarize_with_occupation` 호출
  - `SummarizeResponse.insight` → `Option<String>` 변경
  - `server/src/api/rewrite.rs` (신규) — `POST /me/rewrite` 엔드포인트:
    - occupation 미설정 시 400 반환
    - DB에서 occupation 조회 → LLM `rewrite_with_occupation` 호출 → `{ rewrite: String }` 반환
    - 결과를 favorites `rewrite` 컬럼에 upsert (C2/C3 수정: 비즐겨찾기 기사는 favorites upsert로 자동 생성; 실패해도 200 반환)
    - `FavoritesPort`에 `update_favorite_rewrite` 메서드 추가 (포트 + postgres + fake 3곳)
  - 요약 결과도 favorites upsert 방향으로 정렬 (C3 수정: 비즐겨찾기 기사 요약 결과 유실 방지)
  - `server/src/lib.rs` — `POST /me/rewrite` 라우트 등록
  - **응답 스키마 SSOT 파일 최우선 생성**: `progress/mvp15/M3_response_schema.md` — `insight: string | null` 확정값 포함 (C1 수정: 병렬 클라이언트 작업 전 스키마 파일이 존재해야 함)
  - 통합 테스트: 재작성 occupation None → 400, 재작성 정상 → 200, DB fail → 요약 insight null 반환
- **의존성**: ST-3
- **크기**: 2.5시간

---

### ST-5: 웹 — 설정 페이지 직업 입력란 추가
- **유형**: feature (웹 UI)
- **전제조건**: `M3_response_schema.md`가 ST-4에서 먼저 생성되어야 함 (C1: 스키마 SSOT 선 확정)
- **산출물**:
  - `web/src/routes/settings/+page.svelte` — occupation 텍스트 입력란 추가
  - `PUT /api/me/profile` 호출로 저장 (참고: 라우터는 PUT, GET /me/profile은 tags.rs에 있음)
  - occupation 로드: onMount에서 `GET /api/me/profile` 호출
  - `web/src/lib/api.ts` — `fetchProfile`, `updateOccupation` 메서드 추가 (또는 기존 확장)
  - **occupation 저장 성공 시 summaryCache 전체 무효화** (M1 수정: 구 occupation 기반 인사이트 캐시 오염 방지)
  - 웹 타입 정의에서 `insight: string | null` 적용 (C1 수정: `M3_response_schema.md` 기준으로)
  - 테스트: occupation 저장 동작 vitest
- **의존성**: ST-4 (`M3_response_schema.md` 생성 완료 후)
- **크기**: 1.5시간

---

### ST-6: 웹 — 요약하기 + 재작성 버튼 UI (occupation 연동)
- **유형**: feature (웹 UI)
- **산출물**:
  - `web/src/routes/feed/[id]/+page.svelte` 또는 ArticleDetail 컴포넌트:
    - 요약하기: `{ summary, insight }` 표시. **`insight: string | null` 타입 적용** (C1 수정: non-null 단언 금지)
    - `{@html}` 렌더링 시 **`DOMPurify.sanitize()` wrapping 필수** (T3 수정: XSS 표면 확대 방지)
    - insight null 시 섹션 숨김, null 상태에서 레이아웃 깨짐 없음 확인 (vitest 컴포넌트 테스트)
  - 재작성 버튼: occupation 설정 시에만 렌더링. `POST /api/me/rewrite` 호출 → `{ rewrite }` 표시
  - favorites 저장: 서버 side에서 upsert 처리하므로 클라이언트는 별도 `POST /api/me/favorites` 불필요 (C3 반영)
  - 테스트: insight null 시 UI, occupation 미설정 시 재작성 버튼 숨김
- **의존성**: ST-5 (occupation 정보 접근 가능 후)
- **크기**: 1.5시간

---

### ST-7: iOS — 설정 화면 직업 입력 필드 추가
- **유형**: feature (iOS UI)
- **전제조건**: `M3_response_schema.md`가 ST-4에서 먼저 생성되어야 함 (C1)
- **산출물**:
  - `ios/Frank/Frank/Sources/Features/Settings/SettingsFeature.swift` — occupation 상태 + PUT 액션 추가
  - `ios/Frank/Frank/Sources/Features/Settings/SettingsView.swift` — occupation TextField 추가
  - API 클라이언트에 `updateOccupation` 메서드 추가 (참고: HTTP 메서드는 PUT `/me/profile`)
  - occupation 로드: onAppear에서 GET /me/profile 호출
  - iOS `SummaryResult` 모델에 `let insight: String?` (optional 적용, C1 수정: non-optional 사용 시 occupation 미설정 디코드 오류)
  - **occupation 저장 성공 시 SessionCache summary 무효화** (M1 수정)
  - 테스트: SettingsFeatureTests occupation 설정 시나리오
- **의존성**: ST-4 (`M3_response_schema.md` 생성 완료 후)
- **크기**: 1.5시간

---

### ST-8: iOS — ArticleDetail 요약하기 + 재작성 버튼 UI (occupation 연동)
- **유형**: feature (iOS UI)
- **산출물**:
  - `ios/Frank/Frank/Sources/Features/Detail/ArticleDetailFeature.swift` — summarize/rewrite 액션에 occupation 조회 통합
  - `ios/Frank/Frank/Sources/Features/Detail/ArticleDetailView.swift`:
    - insight nil 시 섹션 숨김, nil 상태에서 레이아웃 깨짐 없음 확인
    - 재작성 버튼 occupation 미설정 시 숨김, rewrite 결과 표시
    - `SummaryResult.insight` — `String?` (optional) 처리 (C1 반영)
  - favorites 저장: 서버 side upsert 처리로 별도 favorites 저장 API 호출 불필요 (C3 반영)
  - 테스트: ArticleDetailFeatureTests — occupation nil/non-nil 시나리오
- **의존성**: ST-7 (occupation 상태 접근 가능 후)
- **크기**: 2시간

---

## 의존성 DAG

```
ST-1 (DB migration)
  └─→ ST-2 (서버 Profile + DbPort occupation)
        └─→ ST-3 (LlmPort summarize_with_occupation)
              └─→ ST-4 (summarize 연동 + rewrite 엔드포인트 + 스키마 박제)
                    ├─→ ST-5 (웹 설정 occupation 입력)
                    │     └─→ ST-6 (웹 요약/재작성 버튼)
                    └─→ ST-7 (iOS 설정 occupation 입력)
                          └─→ ST-8 (iOS 요약/재작성 버튼)
```

**ST-5 ~ ST-8은 ST-4 완료 후 두 갈래(웹/iOS) 병렬 실행.**

---

## 실행 순서 요약

| 순서 | 서브태스크 | 실행 방식 | 예상 시간 |
|------|-----------|----------|---------|
| 1 | ST-1 DB 마이그레이션 (profiles + favorites) | 단독 | 20분 |
| 2 | ST-2 서버 Profile + DbPort + 기존 테스트 갱신 | 단독 | 1.5시간 |
| 3 | ST-3 LlmPort 두 메서드 + Pin<Box> 패턴 | 단독 | 2시간 |
| 4 | ST-4 API 엔드포인트 + **M3_response_schema.md 최우선** | 단독 | 2.5시간 |
| 5a | ST-5 웹 설정 occupation + 캐시 무효화 | 병렬 (웹) | 1.5시간 |
| 5b | ST-7 iOS 설정 occupation + 캐시 무효화 | 병렬 (iOS) | 1.5시간 |
| 6a | ST-6 웹 요약/재작성 + DOMPurify | ST-5 후 | 1.5시간 |
| 6b | ST-8 iOS 요약/재작성 + optional 처리 | ST-7 후 | 2시간 |

총 예상: 서버 6.5시간 + 클라이언트 병렬 3.5시간 = **약 10시간**

---

## 핵심 설계 결정 (인터뷰 확정값 + step-5 리뷰 반영)

1. **occupation 조회**: JWT → `db.get_profile(user_id).occupation` (URL 노출 없음). 조회 실패 시 `occupation=None`으로 degrade (insight null 반환)
2. **캐시 전략**: 없음 — 버튼 트리거 + `phase.done` 상태로 중복 호출 방지. **단, occupation 변경 시 summaryCache 전체 무효화 필수** (M1)
3. **출력 언어**: 항상 한국어
4. **rewrite 저장 방식**: favorites `rewrite TEXT NULL` 컬럼 upsert. 비즐겨찾기 기사도 자동 row 생성 (C2/C3)
5. **occupation 미설정 시**: insight = null, 재작성 버튼 숨김. 클라이언트 타입: `insight: string | null` (웹), `let insight: String?` (iOS) (C1)
6. **안전 장치**: Groq timeout 10s, 1회 retry, occupation 최대 50자 검증 (T1)
7. **응답 스키마 SSOT**: `progress/mvp15/M3_response_schema.md` — ST-4에서 **클라이언트 병렬 작업 전 최우선 생성** (C1)
8. **LlmPort 신규 메서드**: 반드시 `Pin<Box<dyn Future<...> + Send + '_>>` 반환 (M4)
9. **HTML 렌더링**: insight/rewrite `{@html}` 사용 시 `DOMPurify.sanitize()` wrapping 필수 (T3)

---

## step-5 리뷰 결과 (2026-05-03)

### Claude 리뷰 (문서 일치성)
- feature-list F-02 (favorites.rewrite ALTER)와 F-10 (rewrite 저장)이 M3_subtasks.md에 미매핑 → ST-1 + ST-4에 반영 완료
- GET /me/profile 엔드포인트 존재 확인 (`api/tags.rs:46`)
- PUT 엔드포인트가 아닌 PATCH로 서술된 부분 → 실제 라우터 확인 결과 PUT 사용 (`lib.rs:79`) → ST-5/ST-7 수정

### Codex 리뷰 (기술적 타당성)
- `LlmPort` 신규 메서드는 `Pin<Box<dyn Future...>>` 필수 (M4) — ST-3에 명시
- `LlmSummary.insight: String → Option<String>` 파급 범위 (호출처 4곳: FavoritesPort, postgres_favorites, summary_service, SummarizeResponse) — ST-3에 체크리스트 추가
- `rewrite_with_occupation`을 ST-4에서 ST-3으로 이동 — 동일 LlmPort 변경 한 번에 처리

### 구멍 찾기 리뷰 (critical-review)
- 치명 3건, 중대 4건, 경미 3건 발견 → 모두 서브태스크에 반영 완료:
  - C1 insight null 계약 파괴 → ST-4 스키마 최우선 생성, ST-5/ST-7 타입 수정 명시
  - C2 rewrite 저장 컬럼 없음 → ST-1 favorites ALTER 추가
  - C3 비즐겨찾기 기사 결과 유실 → ST-4 favorites upsert 방향 확정
  - M1 occupation 변경 시 캐시 오염 → ST-5/ST-7 캐시 무효화 명시
  - M2 profiles row 없는 계정 404 → ST-2 UPSERT 전환 명시
  - M3 DB 장애 시 요약 outage → ST-4 degrade path 명시
  - M4 Pin<Box> 패턴 → ST-3 명시
  - T1 occupation 검증 코드 부재 → ST-2에 이미 명시, 재확인
  - T2 설정 저장 부분 성공 → 태그/occupation 독립 버튼으로 설계 유지
  - T3 XSS DOMPurify → ST-6 명시

### 최종 결정: 조건부 승인 (수정 완료 후 승인)
- 치명/중대 구멍 10건 모두 서브태스크 문서에 반영 완료
- 총 예상 시간 8~9시간 → 10시간으로 상향 조정
- step-6 진입 가능
