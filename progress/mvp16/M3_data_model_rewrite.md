# M3: 데이터 모델 + 재작성 정합

> 프로젝트: Frank MVP16
> 상태: 대기
> 예상 기간: 4~6일
> 의존성: M2 (occupation 의미 변경이 M2 C3와 정합되어야 함)

## 목표

occupation·rewrite 데이터가 정확히 저장·표시되도록 데이터 모델 정합을 회복한다. 이번 사이클은 **데이터 레이어 변경 + 직업별 누적 저장 구조 도입**까지. 다중 시점 UI는 다음 MVP로 분리.

## 배경 (`_review_notes.md` §2 서버: 데이터 모델·재작성 정합 + §3 C2-bug 처리 방향)

실사용 피드백 중 다음 2건이 이 마일스톤에 포함된다 — D1(occupation 삭제 불가), C2-bug(직업 바꿔도 본문 안 바뀜).

> **D2(오답 노트 원문 URL 미저장)는 M4로 재배치됨.** 서버는 `quiz_wrong_answers.article_url` 컬럼·저장 페이로드를 이미 보유 (`server/src/api/quiz_wrong_answers.rs:32`, `domain/models.rs:76`). 따라서 D2의 실제 작업은 클라이언트 원문 이동 UI뿐이라 M4로 이동.

C2-bug는 **합의 처리 방향: (3) 직업별 누적 저장**. 다만 현재 rewrite 저장 실체는 `favorites.rewrite` 단일 컬럼 덮어쓰기이므로 (`services/rewrite_service.rs:69-77`), 직업 차원을 보존하려면 데이터 모델 자체를 변경해야 한다. 단순 "캐시 키 확장"으로는 처리 불가.

## 포함 항목

### D1. occupation 삭제 불가 (DEBT-MVP16-02)

- **상황**: 사용자가 설정에서 직업 입력 후, 빈 값으로 다시 저장
- **현재**: 빈 값이 "변경 없음"으로 처리되어 이전 직업 유지. PATCH 핸들러 `Option<String>` 패턴이 null/empty 구분 불가.
- **기대**: 빈 값/null 저장 시 occupation 실제 삭제
- **수정 결**: `Option<Option<String>>` 패턴 또는 명시적 `clear_occupation` 플래그. 서버 PATCH 핸들러 + 클라이언트 페이로드 정합. DB SQL `CASE WHEN $5 THEN $4 ELSE profiles.occupation END` 패턴.
- **연쇄 영향**: occupation NULL 상태에서 신규 기사 rewrite 호출 시 동작 정의 필요 (현재 `api/rewrite.rs:42-46`은 occupation 없으면 400 반환). C4 미결정 항목 참조.

### C2-bug. 직업 바꿔도 본문 안 바뀜 — favorites 다중 시점 보존 구조 도입

- **상황**: iOS 개발자로 설정 → 어떤 기사 재작성 → 저장 → 설정에서 프론트엔드 개발자로 변경 → 같은 기사 다시 보기
- **현재**: 본문은 iOS 시각 그대로. 헤더 라벨만 "프론트엔드 개발자 시각으로 재작성"으로 바뀜. **rewrite 저장은 캐시가 아니라 `favorites.rewrite` 단일 TEXT 컬럼 덮어쓰기**(`services/rewrite_service.rs:69-77`의 `update_favorite_rewrite`). 새 직업으로 호출하면 이전 직업 본문이 사라짐.
- **기대**: 직업 바꾸면 그 직업 시점의 본문 표시. 없으면 새로 생성. 이전 직업 시점은 DB에 보존(누적).
- **수정 결**: **데이터 모델 변경.** 두 가지 형태 중 선택 (step-1):
  - (a) `favorites.rewrite TEXT` → `favorites.rewrites JSONB`로 변경. `{ "iOS 개발자": "...", "프론트엔드 개발자": "..." }` 맵 구조.
  - (b) 별도 테이블 `favorite_rewrites (favorite_id, occupation, content, updated_at, PRIMARY KEY (favorite_id, occupation))` 신설.
  - 어느 쪽이든 MVP15 시대까지 저장된 기존 `favorites.rewrite` 단일 값의 마이그레이션 정책 필요 (현재 사용자 occupation에 매핑 vs NULL 시점으로 보존 vs 폐기) — step-1.
- **이번 MVP 미포함**: 다중 시점 UI(시점 칩, 즉석 전환, 새 시점 추가) — 다음 MVP 후보 (`_review_notes.md` §5)

## occupation 의미 매트릭스 (M2/M3 충돌 회피)

같은 `profiles.occupation` 컬럼이 이번 MVP에서 세 차원의 의미를 동시에 갖는다. step-1에서 매트릭스를 박제 후 진입.

| 차원 | 마일스톤 | occupation NULL일 때 동작 |
|------|---------|--------------------------|
| 입력 정합 | M3 D1 | 명시적 NULL 저장 가능 |
| 인사이트 분기 | M2 C3 | 범용 인사이트 생성 (직업 무관) |
| rewrite 저장 키 | M3 C2-bug | NULL 키 자체로 별도 시점 보존 OR rewrite 호출 자체 차단 (C4 결정) |

## 미결정 (워크플로우에서 결정)

| 항목 | 결정 시점 | 비고 |
|------|-----------|------|
| D1 페이로드 패턴 (`Option<Option<String>>` vs `clear_occupation` 플래그) | step-1 | |
| C2-bug 데이터 모델 (favorites JSONB vs 별도 테이블) | step-1 | 둘 다 마이그레이션 필요. 별도 테이블이 다중 시점 UI 확장 용이 |
| C2-bug 기존 favorites.rewrite 마이그레이션 정책 (사용자 occupation 매핑 / NULL 시점 보존 / 폐기) | step-1 | |
| **C4** occupation NULL 상태에서 rewrite 호출 정책 (400 유지 / 모달로 직업 입력 유도 / 원문 fallback / 범용 시각 생성) | step-1 | `api/rewrite.rs:42-46` 현재 400. M2 C3 범용 인사이트 정책과 정합 필요 |

## 성공 기준 (Definition of Done)

- [ ] D1: `UpdateProfileRequest.occupation` 페이로드 패턴 변경 + DB SQL 명시적 NULL 처리
- [ ] D1: `fake_db.rs` mock 동기화 + 단위 테스트 (`"occupation": null` → 삭제 확인)
- [ ] D1: 클라이언트(웹/iOS) 페이로드 송신 정합 (빈 입력 → null 또는 삭제 플래그)
- [ ] D1: occupation NULL 상태 rewrite 호출 정책 구현 (C4 결정 반영)
- [ ] C2-bug: 데이터 모델 변경 (favorites JSONB 또는 별도 테이블) + 마이그레이션 SQL
- [ ] C2-bug: 기존 `favorites.rewrite` 데이터 마이그레이션 (step-1 결정 정책 적용)
- [ ] C2-bug: 조회/저장 핸들러를 occupation 차원으로 분리 + 단위 테스트 (직업 변경 시 hit/miss)
- [ ] C2-bug: occupation NULL 시 rewrite 보존 키 처리 정책 적용
- [ ] 본인 직접 사용: 직업 입력 → 비우기 저장 → 재조회 시 NULL 확인 (E2E)
- [ ] 본인 직접 사용: A 직업 재작성 → B로 변경 → 같은 기사 → B 시점 본문 새로 생성 → 다시 A로 돌아오면 이전 본문 그대로 노출 확인 (E2E)
- [ ] 본인 직접 사용: occupation 비운 상태에서 신규 기사 → rewrite 정책 일관 동작 (C4 결정 반영) (E2E)
- [ ] 서버 단위·통합 테스트 통과 (`cargo test`)
- [ ] 비용 영향: rewrite 호출 빈도 측정 후 무료 한도 위반 0건 확인 (직업 변경 빈도 낮으므로 통제 가능)
- [ ] **이번 MVP 미포함 명시**: 다중 시점 UI(시점 칩, 즉석 전환) 작업 없음 — 다음 MVP 후보

## 워크플로우 진입점

```
/workflow "M3-data-model-rewrite"
```

**메인태스크**: occupation 삭제 정합 + favorites 다중 시점 보존 구조 도입 + occupation NULL 상태 rewrite 정책 (서버 + 클라이언트 페이로드).

## 아이템

| # | 아이템 | 유형 | 순서 | 상태 |
|---|--------|------|------|------|
| 1 | occupation 의미 매트릭스 박제 + 미결정 4종 결정 (D1 페이로드, C2-bug 모델, C2-bug 마이그레이션, C4 NULL 정책) | decision | 1 | 대기 |
| 2 | D1 occupation 삭제 정합 (서버 PATCH + DB + mock) | feature | 2 | 대기 |
| 3 | C2-bug 데이터 모델 변경 + 마이그레이션 SQL + 기존 데이터 처리 | feature | 3 | 대기 |
| 4 | C2-bug 조회/저장 핸들러 occupation 차원 분리 + 단위 테스트 | feature | 4 | 대기 |
| 5 | C4 occupation NULL 상태 rewrite 정책 구현 (서버 + 클라이언트 페이로드) | feature | 5 | 대기 |
| 6 | E2E 시나리오 3종 통과 + 비용 영향 검토 | chore | 6 | 대기 |

## KPI (M3)

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| 서버 테스트 통과 | `cargo test` | 전체 통과 | Hard | — |
| occupation null 저장 단위 테스트 | `cargo test profile::clear_occupation` | 통과 | Hard | — |
| favorites 다중 시점 저장 단위 테스트 | `cargo test rewrite::occupation_dimension` (또는 동등 이름) | 통과 | Hard | — |
| 마이그레이션 SQL 적용 검증 | `sqlx migrate run` 성공 + 기존 데이터 보존 검사 | 통과 | Hard | — |
| occupation NULL rewrite 정책 단위 테스트 | `cargo test rewrite::null_occupation_policy` | 통과 | Hard | — |
| occupation 삭제 회귀 차단 | 본인 E2E: 빈 값 저장 → 재조회 NULL | 통과 | Hard | 유지됨(버그) |
| 직업별 본문 누적 저장 회귀 차단 | 본인 E2E: A→B→A 시나리오에서 A 시점 본문 보존 | 통과 | Hard | 같은 본문(버그) |
| 무료 한도 위반 발생 0건 | `progress/mvp16/cost_log.md` 검토 | $0 유지 | Hard | — |

## 리스크

| 리스크 | 영향(H/M/L) | 대응 |
|--------|------------|------|
| occupation 의미 변경(M2 C3 범용 insight + M3 D1 삭제 + C2-bug 다중 시점)이 동시에 occupation을 만져 충돌 | H | M2 → M3 순서 고정. M3 step-1에서 occupation 의미 매트릭스 박제 (위 §occupation 의미 매트릭스 참조) |
| favorites 데이터 모델 변경이 마이그레이션 실수로 기존 rewrite 데이터 손실 | H | step-1에서 마이그레이션 정책 결정 후 dry-run 백업 확보. 단계적 마이그레이션 (새 컬럼/테이블 신설 → 백필 → 기존 컬럼 deprecate) 권장 |
| 별도 테이블 vs JSONB 결정이 앞으로의 다중 시점 UI 확장과 충돌 | M | step-1에서 다음 MVP의 다중 시점 UI 시나리오를 미리 그려보고 결정. 별도 테이블이 `updated_at`·인덱싱·삭제 처리에 유리 |
| C4 NULL 정책이 M2 C3 범용 인사이트 정책과 모순 (인사이트는 범용 노출되는데 rewrite는 400) | M | step-1에서 두 정책의 일관성 매트릭스 작성. 사용자 모델 = "직업 없어도 기본 동작 가능"으로 정합 권장 |
| 다중 시점 UI를 이번 MVP에 슬쩍 끌어옴 → 스코프 폭주 | H | DoD에 "이번 MVP 미포함" 명시. 발견 시 즉시 다음 MVP 후보로 이관 (`_review_notes.md` §5) |

## 참고

- 합의 노트: `progress/mvp16/_review_notes.md` §2.3 (서버: 데이터 모델·재작성 정합) + §3 (C2-bug 처리 방향)
- 부채: `DEBT-MVP16-02` (`progress/debts.md`)
- 코드: `server/src/services/rewrite_service.rs:21-77`, `server/src/api/rewrite.rs:42-46`, `server/src/api/profiles.rs` (PATCH 핸들러)
- 관련 메모리: `project_api_cost_policy`, `feedback_feed_ephemeral`, `user_app_motivation`
