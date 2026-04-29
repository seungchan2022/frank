# MVP13 M1 — DEBT-01 오답 태그 필터 (서버+DB)

> 기획일: 2026-04-28
> 상태: done
> 목표: quiz_wrong_answers에 tag_id 저장 + 서버 필터 API 지원 (M2 웹+iOS 전환의 기반)

---

## 배경

`quiz_wrong_answers` 테이블에 `tag_id` 컬럼이 없어 웹·iOS 모두 favorites 브릿지로 간접 필터링 중.
Favorites에 없는 기사의 오답은 필터에서 제외되는 버그 (DEBT-01).

M1에서 서버+DB 기반을 만들고, M2에서 웹·iOS 클라이언트가 이 API를 소비한다.

---

## 인터뷰 결정

| Q | 결정 | 내용 |
|---|---|---|
| Q1 (step-1) | A | 클라이언트가 tag_id를 요청 body에 직접 전달 |
| Q2 (step-4) | A | 잘못된 UUID 형식 → 400 Bad Request |
| Q3 (step-4) | C | 기존 오답 전체 삭제 (테스트 데이터, 실사용 전) |
| Q4 (step-4) | A | Supabase MCP apply_migration으로 적용 |

---

## 서브태스크

### T-01 DB 마이그레이션
- 마이그레이션 SQL 작성: `quiz_wrong_answers`에 `tag_id UUID NULL` 컬럼 추가
- FK: `REFERENCES tags(id) ON DELETE SET NULL` (태그 삭제 시 고아 데이터 방지)
- 인덱스: `(user_id, tag_id)` 복합 인덱스 추가
- 기존 오답 전체 삭제 (`DELETE FROM quiz_wrong_answers` — 테스트 데이터)
- Supabase MCP `apply_migration`으로 적용

### T-02 서버 — 모델 + Adapter
- `server/src/domain/models.rs`:
  - `SaveWrongAnswerParams`에 `tag_id: Option<Uuid>` 추가
  - `QuizWrongAnswer`에 `tag_id: Option<Uuid>` 추가
  - `WrongAnswerResponse`에 `tag_id: Option<Uuid>` 추가 + `TryFrom` 매핑 수정 (C1)
- `server/src/infra/postgres_quiz_wrong_answers.rs`:
  - INSERT 쿼리 + `DO UPDATE SET tag_id = EXCLUDED.tag_id` 명시
  - SELECT 쿼리에 tag_id 포함
- `server/src/infra/fake_quiz_wrong_answers.rs`: fake adapter `list()` tag_id 필터 구현

### T-03 서버 — API 핸들러
- `server/src/api/quiz_wrong_answers.rs`:
  - POST: `tag_id` 수신 + UUID 형식 검증 → 실패 시 400 (axum rejection → JSON 규약 일치)
  - GET: `?tag_id=` 쿼리 파라미터 추가 (없으면 전체 반환, 잘못된 형식 → 400)

### T-04 서버 테스트
- `postgres_quiz_wrong_answers` unit test: tag_id 저장/조회 케이스 추가
- API 핸들러 test: tag_id 포함 저장, ?tag_id= 필터, 잘못된 UUID 형식 → 400 케이스

---

## 완료 기준

- [x] 오답 저장 시 tag_id가 DB에 기록됨
- [x] GET ?tag_id={uuid} 로 필터된 오답만 반환됨
- [x] 잘못된 UUID 형식 → 400 반환
- [x] 기존 오답 (tag_id=NULL) 조회 정상 동작
- [x] 서버 테스트 전체 통과

---

## KPI

| 지표 | 목표 | 게이트 |
|---|---|---|
| 서버 테스트 커버리지 | 90% 이상 유지 | Hard |
| 서버 테스트 통과 | 전체 통과 | Hard |

---

## Feature List
<!-- size: 중형 | count: 22 | skip: false -->

### 기능
- [x] F-01 quiz_wrong_answers에 tag_id UUID NULL 컬럼 + FK + 인덱스 마이그레이션 적용
- [x] F-02 기존 오답 전체 삭제 (DELETE FROM quiz_wrong_answers, 테스트 데이터)
- [x] F-03 SaveWrongAnswerParams에 tag_id: Option<Uuid> 필드 추가
- [x] F-04 QuizWrongAnswer 도메인 모델에 tag_id: Option<Uuid> 필드 추가
- [x] F-05 WrongAnswerResponse에 tag_id: Option<Uuid> 추가 + TryFrom 매핑 수정
- [x] F-06 postgres INSERT 쿼리에 tag_id 포함 + ON CONFLICT DO UPDATE SET tag_id 명시
- [x] F-07 postgres SELECT 쿼리에 tag_id 포함
- [x] F-08 POST /api/me/quiz/wrong-answers 핸들러에서 tag_id 수신 및 저장
- [x] F-09 GET /api/me/quiz/wrong-answers?tag_id= 필터 파라미터 지원

### 엣지
- [x] E-01 ?tag_id= 파라미터 없을 때 전체 오답 반환
- [x] E-02 클라이언트가 tag_id 미전달 시 NULL로 저장 (Optional 처리)
- [x] E-03 ?tag_id= 필터 시 tag_id=NULL 행 제외 (SQL NULL 비교 동작)
- [x] E-04 tag_id 동일한 오답 여러 개 조회 시 전체 반환

### 에러
- [x] R-01 GET ?tag_id= 잘못된 UUID 형식 → 400 Bad Request (JSON 규약 일치)
- [x] R-02 POST body tag_id 잘못된 UUID 형식 → 400 Bad Request
- [~] deferred (M3 실서버 환경에서 검증 — fake adapter로 커버 불가, T-07 E2E 검증 시 실서버 DB 연결 끊음 테스트로 확인) R-03 DB 연결 실패 시 500 반환

### 테스트
- [x] T-01 tag_id 포함 오답 저장 unit test (postgres adapter)
- [x] T-02 ?tag_id= 필터 조회 unit test (NULL 행 제외 검증 포함)
- [x] T-03 잘못된 UUID 형식 → 400 API handler test
- [x] T-04 fake adapter tag_id 저장/조회/필터 unit test

### 회귀
- [x] G-01 tag_id 미포함 오답 저장 요청 (기존 클라이언트 호환) 정상 동작
- [x] G-02 ?tag_id= 없는 전체 조회 API 기존 동작 유지

---

## 다음 단계

M1 완료 후 → M2 (웹+iOS 클라이언트 전환 + 피드 품질 개선)
