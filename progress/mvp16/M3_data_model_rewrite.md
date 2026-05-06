# M3: 데이터 모델 + 재작성 정합

> 프로젝트: Frank MVP16
> 상태: in-progress
> 예상 기간: 3~4일
> 의존성: M2 완료 (occupation 의미 변경 정합 후 진입)

## 목표

occupation 삭제 정합 + 재작성 본문-직업 불일치 UX 해소. 데이터 구조는 **덮어쓰기 유지, `rewrite_occupation` 컬럼 추가**로 단순하게 처리.

## 배경 (`_review_notes.md` §2 서버: 데이터 모델·재작성 정합 + §3 C2-bug 처리 방향)

실사용 피드백 중 다음 2건이 이 마일스톤에 포함된다 — D1(occupation 삭제 불가), C2-bug(직업 바꿔도 본문 안 바뀜).

> **D2(오답 노트 원문 URL 미저장)는 M4로 재배치됨.** 서버는 `quiz_wrong_answers.article_url` 컬럼·저장 페이로드를 이미 보유 (`server/src/api/quiz_wrong_answers.rs:32`, `domain/models.rs:76`). 따라서 D2의 실제 작업은 클라이언트 원문 이동 UI뿐이라 M4로 이동.

C2-bug 처리 방향: **덮어쓰기 유지 + `rewrite_occupation` 컬럼 추가**. 다중 시점 누적 저장(JSONB/별도 테이블)은 채택하지 않음. 직업 변경 시 버튼 라벨로 상태를 표현하고, 재작성 시 덮어씀.

## 포함 항목

### D1. occupation 삭제 불가 (DEBT-MVP16-02)

- **상황**: 사용자가 설정에서 직업 입력 후, 빈 값으로 다시 저장
- **현재**: 빈 값이 "변경 없음"으로 처리되어 이전 직업 유지. PATCH 핸들러 `Option<String>` 패턴이 null/empty 구분 불가.
- **기대**: null 저장 시 occupation 실제 삭제. 빈 문자열(`""`)은 클라이언트에서 null로 변환 후 전송.
- **수정 결**:
  - `UpdateProfileRequest.occupation: Option<Option<String>>` 전환 + 커스텀 deserializer (`serde(default, deserialize_with)`) 적용 — serde_with 크레이트 추가 없이 직접 구현
  - `DbPort.update_profile` 시그니처 → `occupation: Option<Option<String>>` (None=변경없음, Some(None)=삭제, Some(Some(s))=설정)
  - 빈 문자열 처리: 클라이언트에서 빈 입력 → `null` 변환 후 전송. 서버 기존 `""` → None 로직 유지 (서버 도달 시 `""` = 변경 없음)
- **캐스케이드**: occupation NULL 저장 시 `DbPort`의 신규 복합 메서드 `delete_occupation_with_cascade(user_id)` 호출 → 단일 DB 트랜잭션으로 `profiles.occupation = NULL` + `favorites.rewrite = NULL` + `favorites.rewrite_occupation = NULL` + `favorites.insight = NULL` 동시 처리. 중간 실패 시 전체 롤백.
  - **insight 포함 이유**: M2 C3 백필은 forward-only (기존 occupation-specific insight 보존). 직업 삭제 시 과거 occupation 시각 insight가 그대로 남으면 레이블-본문 불일치 재발. CASCADE에 `insight = NULL` 포함해야 다음 summarize 호출에서 M2 C3 범용 insight로 자동 재생성.
  - **COALESCE 보호 우회 의도**: `update_favorite_summary`의 `COALESCE($4, insight)` 보호 로직은 일반 요약 업데이트용. 삭제 캐스케이드는 이를 명시적으로 우회 (의도된 행위).

### C2-bug. 직업 바꿔도 본문 안 바뀜 — rewrite_occupation 컬럼 추가

- **상황**: iOS 개발자로 설정 → 어떤 기사 재작성 → 저장 → 설정에서 프론트엔드 개발자로 변경 → 같은 기사 다시 보기
- **현재**: 본문은 iOS 시각 그대로. 헤더 라벨만 "프론트엔드 개발자 시각으로 재작성"으로 바뀜. 레이블-본문 불일치.
- **기대**: 직업 변경 후 기사 열면 새 직업으로 재작성 가능한 상태로 표시. 재작성하면 덮어씀.
- **수정 결**: `favorites.rewrite_occupation TEXT` 컬럼 추가. `rewrite_occupation` ≠ 현재 직업이면 버튼 재활성화. `favorites.rewrite TEXT` 구조는 그대로 유지(덮어쓰기).
- **이번 MVP 미포함**: 다중 시점 UI(시점 칩, 즉석 전환) — 다음 MVP 후보

## UX 시나리오 (확정)

| 상태 | 버튼 | 동작 |
|------|------|------|
| 직업 없음 | 재작성 버튼 안 보임 | — |
| 직업 있음, 재작성 안 함 | `"[직업명] 기준으로 재작성하기"` | 누르면 생성·저장 |
| 재작성 완료, 직업 동일 | 버튼 없음 (본문 표시) | — |
| 직업 변경 후 | `"[새 직업명] 기준으로 재작성하기"` 재등장 | 누르면 덮어씀 |
| 직업 삭제 | 버튼 사라짐 | `rewrite`, `rewrite_occupation` NULL 초기화 |

## 미결정 → 전부 확정

| 항목 | 결정 |
|------|------|
| D1 페이로드 패턴 | `Option<Option<String>>` (A안) |
| C2-bug 데이터 모델 | 덮어쓰기 유지 + `rewrite_occupation TEXT` 컬럼 추가 |
| C2-bug 기존 데이터 마이그레이션 | 컬럼 추가만이므로 기존 데이터 보존, `rewrite_occupation` NULL로 초기화 |
| C4 occupation NULL 상태 rewrite 정책 | 버튼 자체 안 보임. 서버 400은 안전망으로 유지 |

## occupation 의미 매트릭스

| 차원 | 마일스톤 | occupation NULL일 때 동작 |
|------|---------|--------------------------|
| 입력 정합 | M3 D1 | 명시적 NULL 저장 가능 |
| 인사이트 분기 | M2 C3 | 범용 인사이트 생성 (직업 무관) |
| rewrite 버튼 노출 | M3 C2-bug | 버튼 안 보임. rewrite/rewrite_occupation NULL |

## 성공 기준 (Definition of Done)

- [ ] D1: `UpdateProfileRequest.occupation` → `Option<Option<String>>` 변경 + DB SQL 명시적 NULL 처리
- [ ] D1: `fake_db.rs` mock 동기화 + 단위 테스트 (`"occupation": null` → 삭제 확인)
- [ ] D1: 클라이언트(웹/iOS) 페이로드 송신 정합 (빈 입력 → null 전송)
- [ ] D1: occupation 삭제 시 `favorites.rewrite`, `rewrite_occupation`, `insight` NULL 초기화 처리 (`delete_occupation_with_cascade` 단일 트랜잭션)
- [ ] C2-bug: `favorites` 테이블에 `rewrite_occupation TEXT` 컬럼 추가 마이그레이션 SQL
- [ ] C2-bug: 재작성 저장 시 `rewrite_occupation` 함께 저장
- [ ] C2-bug: 조회 시 `rewrite_occupation` 응답에 포함
- [ ] C2-bug: 단위 테스트 (직업 변경 전후 rewrite_occupation 값 확인)
- [ ] 클라이언트: 재작성 버튼 라벨 `"[직업명] 기준으로 재작성하기"` 동적 표시
- [ ] 클라이언트: `rewrite_occupation` ≠ 현재 직업이면 버튼 재활성화
- [ ] 클라이언트: 직업 없음 → 재작성 버튼 숨김
- [ ] 본인 직접 사용: 직업 입력 → 비우기 저장 → 재조회 시 occupation NULL 확인 (E2E)
- [ ] 본인 직접 사용: A 직업 재작성 → B로 변경 → 같은 기사 → 버튼 재활성화 확인 → 재작성 → 덮어씀 확인 (E2E)
- [ ] 본인 직접 사용: 직업 없는 상태 → 재작성 버튼 안 보임 확인 (E2E)
- [ ] 서버 단위·통합 테스트 통과 (`cargo test`)
- [ ] 비용 영향: rewrite 호출 빈도 측정 후 무료 한도 위반 0건 확인
- [ ] **이번 MVP 미포함 명시**: 다중 시점 UI(시점 칩, 즉석 전환) 작업 없음

## 아이템

| # | 아이템 | 유형 | 순서 | 상태 |
|---|--------|------|------|------|
| 1 | D1 occupation 삭제 정합 (서버 PATCH + DB + mock + 클라이언트) | feature | 1 | 완료 |
| 2 | C2-bug `rewrite_occupation` 컬럼 추가 마이그레이션 SQL | feature | 2 | 완료 |
| 3 | C2-bug 저장/조회 핸들러 `rewrite_occupation` 포함 + 단위 테스트 | feature | 3 | 완료 |
| 4 | 클라이언트 재작성 버튼 UX (라벨 동적 표시 + 재활성화 + 숨김) | feature | 4 | 완료 |
| 5 | E2E 시나리오 3종 통과 + 비용 영향 검토 | chore | 5 | ✅ done |

## Feature List
<!-- size: 중형 | count: 24 | skip: false -->

### 기능
- [x] F-01 `UpdateProfileRequest.occupation` → `Option<Option<String>>` 전환 + 커스텀 deserializer (serde_with 없이 직접 구현)
- [x] F-02 `DbPort.update_profile` 시그니처 — occupation=None 시 no-op, 핸들러에서 Some(None) 분기 처리
- [x] F-03 `DbPort.clear_occupation(user_id)` + `FavoritesPort.clear_rewrites_for_user(user_id)` 분리 신설 — 포트 경계 준수를 위해 best-effort 순서 호출 방식 채택 (step-7 아키텍처 위반 수정). 핸들러에서 두 포트 순서 호출: profiles 먼저, favorites 후속. 실패 시 500 반환.
- [x] F-04 웹/iOS 클라이언트 빈 직업 입력 → `{"occupation": null}` 전송 (웹: JSON null, iOS: `encode(nil, forKey:)`)
  - 웹: `settings/+page.svelte` L107 이미 `occupation: trimmed.length > 0 ? trimmed : null` 처리됨. 확인 완료.
  - iOS: `ProfileAPI.swift`의 `UpdateProfileRequest.occupation: String??` 이미 구현. `SettingsFeature.swift`에서 빈 TextField → nil 변환 로직 확인 완료.
- [x] F-05 `favorites` 테이블 `rewrite_occupation TEXT` 컬럼 추가 (`supabase/migrations/20260506_mvp16_m3_rewrite_occupation.sql`)
- [x] F-06 `Favorite` 도메인 모델에 `rewrite_occupation: Option<String>` 필드 추가 (sqlx::FromRow 파생으로 자동)
- [x] F-07 `update_favorite_rewrite` 시그니처 확장 — `occupation: Option<&str>` 파라미터 추가 및 저장 (FavoritesPort trait + Postgres 구현체 + FakeFavoritesAdapter + rewrite_service.rs 호출 사이트 동기화)
- [x] F-08 즐겨찾기 조회 응답에 `rewrite_occupation` 포함 (sqlx::FromRow 자동 + 웹 타입/iOS CodingKeys 추가)
- [x] F-09 웹/iOS 재작성 버튼 라벨 `"[직업명] 기준으로 재작성하기"` 동적 표시
- [x] F-10 `rewrite_occupation` ≠ 현재 직업이면 버튼 재활성화 (웹: canRewrite() derived, iOS: loadUserProfile()에서 occupation 비교)
- [x] F-11 직업 없음(null) → 재작성 버튼 숨김 (기존 `{#if userProfile?.occupation}` 유지)

### 엣지
- [x] E-01 occupation `""` → 클라이언트에서 null 변환 전송, 서버 핸들러에서 빈 문자열 → `should_delete_occupation=true` 처리
- [x] E-02 occupation 이미 NULL인 상태에서 NULL 재저장 → idempotent (cascade 트랜잭션 no-op)
- [x] E-03 재작성 이력 없는 즐겨찾기 — `rewrite_occupation` NULL → 버튼 정상 표시
- [x] E-04 `rewrite_occupation` == 현재 직업 → 버튼 숨김 (canRewrite() false)

### 에러
- [x] R-01 occupation 삭제 + favorites NULL 초기화 중 DB 오류 → 500 반환 (best-effort: 각 포트 실패 시 즉시 전파)
- [x] R-02 마이그레이션 SQL `ADD COLUMN IF NOT EXISTS` → 재실행 안전
- [x] R-03 직업 없는 상태에서 rewrite API 직접 호출 → 서버 400 안전망 유지 (핸들러 occupation 검증 그대로)
- [x] R-04 레거시 `rewrite IS NOT NULL AND rewrite_occupation IS NULL` 행 → NULL 그대로 유지, 버튼 재활성화 허용 (R04 정책 준수)

### 테스트
- [x] T-01 `{"occupation": null}` → occupation NULL 단위 테스트 (`occupation_null_deletes_occupation`)
- [x] T-02 occupation 필드 없음 → 변경 없음 단위 테스트 (`occupation_not_sent_does_not_change_occupation`)
- [x] T-03 `{"occupation": ""}` → 서버 동일 삭제 경로로 처리 (`occupation_blank_string_treated_as_none` 기존 테스트 유지)
- [x] T-04 `update_favorite_rewrite` — rewrite_occupation 함께 저장 단위 테스트 (`update_rewrite_stores_occupation`, `update_rewrite_with_none_occupation_clears`)
- [x] T-05 `cargo test` 전체 통과 (418개)
- [x] T-06 `Option<Option<String>>` 커스텀 deserializer 3케이스 단위 테스트 (`deserializer_key_absent_is_none`, `deserializer_null_is_some_none`, `deserializer_value_is_some_some`)

### UI·UX
- [ ] U-01 웹: 직업 변경 후 즐겨찾기 기사 진입 → 버튼 라벨 새 직업명으로 변경 확인
- [ ] U-02 iOS: 동일 시나리오 시뮬레이터 확인

## KPI (M3)

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| 서버 테스트 통과 | `cargo test` | 전체 통과 | Hard | — |
| occupation null 저장 단위 테스트 | `cargo test profile::clear_occupation` (또는 동등 이름) | 통과 | Hard | — |
| rewrite_occupation 저장/조회 단위 테스트 | `cargo test rewrite::occupation_column` (또는 동등 이름) | 통과 | Hard | — |
| 마이그레이션 SQL 적용 검증 | `sqlx migrate run` 성공 + 기존 데이터 보존 | 통과 | Hard | — |
| occupation 삭제 E2E | 본인 E2E: 빈 값 저장 → 재조회 NULL | 통과 | Hard | 유지됨(버그) |
| 재작성 버튼 재활성화 E2E | 본인 E2E: 직업 변경 → 버튼 재등장 확인 | 통과 | Hard | 라벨만 바뀜(버그) |
| 무료 한도 위반 발생 0건 | `progress/mvp16/cost_log.md` 검토 | $0 유지 | Hard | — |

## 리스크

| 리스크 | 영향(H/M/L) | 대응 |
|--------|------------|------|
| occupation 삭제 시 rewrite NULL 초기화 누락 → 레이블-본문 불일치 재발 | H | D1 구현 시 NULL 초기화 로직 명시적으로 포함. 단위 테스트로 커버 |
| 클라이언트 버튼 재활성화 조건 누락 (웹/iOS 따로 구현) | M | 클라이언트 DoD 체크리스트 공유. E2E로 최종 확인 |
| 다중 시점 UI를 이번 MVP에 슬쩍 끌어옴 → 스코프 폭주 | H | DoD에 "이번 MVP 미포함" 명시. 발견 시 즉시 다음 MVP 후보로 이관 |

## 포트 분리 결정 (step-7 아키텍처 수정)

occupation 삭제 시 `profiles` + `favorites` 두 테이블을 정리해야 한다.

### 채택된 방식: best-effort 두 포트 순서 호출

```
핸들러 (api/profile.rs)
├── state.db.clear_occupation(user_id).await?       → profiles.occupation = NULL
└── state.favorites.clear_rewrites_for_user(user_id).await?  → favorites.rewrite/rewrite_occupation/insight = NULL
    (각 포트 실패 시 500 즉시 반환)
```

**왜 이 방식인가 (step-7 아키텍처 리뷰 후 수정):**
- **단일 DbPort 복합 메서드 기각**: `DbPort`가 favorites 테이블을 건드리면 포트 경계 위반. `DbPort`는 profiles/tags/user_keyword_weights만 담당하는 계약을 위반.
- **best-effort 채택**: 포트 분리를 지키면서 핸들러에서 두 포트를 순서대로 호출. profiles 먼저(occupation NULL) → favorites 후속(rewrite NULL). 두 DB UPDATE 모두 idempotent라 실패 시 재시도 가능.

## 참고

- 합의 노트: `progress/mvp16/_review_notes.md` §2.3 (서버: 데이터 모델·재작성 정합) + §3 (C2-bug 처리 방향)
- 부채: `DEBT-MVP16-02` (`progress/debts.md`)
- 코드: `server/src/services/rewrite_service.rs:21-77`, `server/src/api/rewrite.rs:42-46`, `server/src/api/profile.rs` (PUT /me/profile 핸들러)
- 관련 메모리: `project_api_cost_policy`

## step-7 리뷰 부채 (MVP 다음 사이클 이관)

| ID | 설명 | 우선순위 |
|---|---|---|
| DEBT-MVP16-FAKE-CASCADE | `FakeDbAdapter.delete_occupation_with_cascade`에서 favorites cascade no-op. Postgres 통합 테스트에서만 검증됨. 필요 시 `FakeFavoritesAdapter.reset_user_rewrites()` 추가 후 연동. | Low |
| DEBT-MVP16-PROFILE-SERVICE | `profile.rs` 핸들러가 occupation 3-상태 판단 + cascade 실행을 직접 오케스트레이션함. 서비스 계층 `profile_service.rs` 분리 검토 필요. 현재 volume은 허용 가능하나 로직 증가 시 분리 권장. | Low |
| DEBT-MVP16-REWRITE-OCC-OPTION | `FavoritesPort::update_favorite_rewrite` 의 `occupation: Option<&str>` — 현재 호출부에서 `Some(occupation)`만 전달되어 Option이 약함. 향후 서비스 레이어 분리 시 `&str`로 강화 검토. | Low |
