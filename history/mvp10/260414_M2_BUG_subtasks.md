# M2 버그수정 서브태스크 분해

브랜치: `feature/260414_m2_bug_keyword_weights_summary_error`  
작성일: 2026-04-14

---

## 메인태스크

M2 버그수정 두 가지:
- **ST-1**: `user_keyword_weights` 태그별 분리 — UNIQUE constraint `(user_id, keyword)` → `(user_id, tag_id, keyword)`
- **ST-2**: 요약 에러 메시지 개선 — 서버 에러 분류 + 웹 친절한 메시지 re-throw

---

## 서브태스크 목록

### ST-1: keyword_weights 태그별 분리

#### 1-A. DB 마이그레이션 (독립)
- **작업**: `user_keyword_weights`에 `tag_id UUID NOT NULL` 컬럼 추가
- **작업**: UNIQUE constraint `(user_id, keyword)` → `(user_id, tag_id, keyword)` 변경
- **작업**: 기존 레코드 전체 삭제 (fresh start)
- **산출물**: `supabase/migrations/20260414_m2_keyword_weights_tag_id.sql`
- **의존**: 없음

#### 1-B. 포트 시그니처 변경 (TDD)
- **작업**: `domain/ports.rs` — DbPort trait 변경
  - `increment_keyword_weights(user_id, tag_id: Uuid, keywords)` (tag_id 추가)
  - `get_top_keywords(user_id, tag_ids: Vec<Uuid>, limit)` (tag_ids 복수로 변경)
- **TDD**: 포트 변경 전 컴파일 오류로 확인 → 구현으로 통과
- **산출물**: `server/src/domain/ports.rs`
- **의존**: 없음 (1-A와 병렬)

#### 1-C. 인프라 어댑터 구현 변경 (TDD)
- **작업**: `infra/postgres_db.rs` — 새 시그니처로 SQL 변경
  - `increment_keyword_weights`: `tag_id` 바인딩 추가, UNIQUE key `(user_id, tag_id, keyword)`
  - `get_top_keywords`: `WHERE user_id = $1 AND tag_id = ANY($2)` 조건 추가
- **작업**: `infra/fake_db.rs` — 인메모리 구조 변경
  - `keyword_weights: HashMap<Uuid, HashMap<Uuid, HashMap<String, i32>>>` (user_id → tag_id → keyword → weight)
  - `increment_keyword_weights`, `get_top_keywords` 구현 업데이트
  - 기존 fake_db 테스트 수정
- **산출물**: `server/src/infra/postgres_db.rs`, `server/src/infra/fake_db.rs`
- **의존**: 1-B 완료 후

#### 1-D. 호출부 전파 (TDD)
- **작업**: `services/likes_service.rs`
  - `process_like`에 `tag_id: Option<Uuid>` 파라미터 추가
  - `tag_id`가 None이면 keyword_weights 저장 스킵 (태그 없는 좋아요 허용)
  - 테스트: tag_id Some/None 케이스 각각 검증
- **작업**: `api/likes.rs`
  - `LikeArticleRequest`에 `tag_id: Option<Uuid>` 필드 추가
  - `process_like` 호출 시 `tag_id` 전달
  - API 테스트 수정
- **작업**: `api/feed.rs`
  - `get_top_keywords(user.id, user_tag_ids, 3)` — user_tag_ids 전달 (이미 조회된 user_tags 활용)
  - 테스트 코드 수정 (`increment_keyword_weights` 직접 호출부에 tag_id 추가)
- **작업**: `api/related.rs`
  - user_tags 추가 조회 후 `get_top_keywords(user.id, user_tag_ids, 5)` 전달
  - 테스트 코드 수정
- **산출물**: 위 파일들
- **의존**: 1-C 완료 후

---

### ST-2: 요약 에러 메시지 개선

#### 2-A. 서버 AppError 확장 (TDD, 독립)
- **작업**: `domain/error.rs`
  - `UnprocessableEntity(422)` variant 추가
  - `IntoResponse` 구현 (422 UNPROCESSABLE_ENTITY)
  - 테스트: `unprocessable_entity_returns_422`
- **산출물**: `server/src/domain/error.rs`
- **의존**: 없음 (1-A와 병렬)

#### 2-B. summary_service 에러 분류 개선 (TDD)
- **작업**: `services/summary_service.rs`
  - crawl 실패 → `ServiceUnavailable(503)` (기존 Internal → 변경)
  - LLM 실패 → `ServiceUnavailable(503)` (기존 Internal → 변경)
  - 타임아웃 → `Timeout(504)` (기존 유지)
  - LLM 응답 파싱 실패 / 빈 응답 → `UnprocessableEntity(422)` (신규)
  - 테스트: `crawl_failure_returns_503`, `llm_failure_returns_503` (기존 500→503으로 변경)
- **산출물**: `server/src/services/summary_service.rs`
- **의존**: 2-A 완료 후

#### 2-C. 웹 realClient.ts summarize() 에러 메시지 분기 (TDD, 독립)
- **작업**: `web/src/lib/api/realClient.ts`
  - `summarize()` 내부에서 `try/catch` 후 `ApiError.status` 분기
  - 422 → `"요약할 수 없는 내용입니다. 다른 기사를 시도해주세요."`
  - 503 → `"요약 서비스가 일시적으로 사용 불가합니다. 잠시 후 다시 시도해주세요."`
  - 504 → `"요약 시간이 초과되었습니다. 다시 시도해주세요."`
  - 기타 → 원본 에러 re-throw
- **작업**: `web/src/lib/api/__tests__/realClient.test.ts`
  - summarize 에러 케이스 테스트 3개 추가 (422/503/504)
- **산출물**: `web/src/lib/api/realClient.ts`, `web/src/lib/api/__tests__/realClient.test.ts`
- **의존**: 없음 (서버와 병렬)

---

## 의존성 DAG

```
1-A (마이그레이션)
         \
          +-→ (독립, 병렬 가능)
         /
1-B (포트 시그니처)
         \
          +→ 1-C (어댑터)
                  \
                   +→ 1-D (호출부 전파)

2-A (AppError 확장) → 2-B (summary_service)

2-C (웹 클라이언트)  ← 독립
```

**실행 순서:**
1. `1-A`, `2-A`, `2-C` 병렬 시작
2. `1-B` (1-A와 병렬)
3. `2-B` (2-A 완료 후)
4. `1-C` (1-B 완료 후)
5. `1-D` (1-C 완료 후)

---

## 체크리스트

### ST-1
- [ ] 1-A: DB 마이그레이션 파일 작성
- [ ] 1-B: ports.rs 시그니처 변경 (TDD)
- [ ] 1-C: postgres_db.rs + fake_db.rs 구현 (TDD)
- [ ] 1-D: likes_service, likes API, feed, related 호출부 전파 (TDD)

### ST-2
- [ ] 2-A: AppError::UnprocessableEntity 추가 (TDD)
- [ ] 2-B: summary_service 에러 분류 변경 (TDD)
- [ ] 2-C: realClient.ts summarize() 에러 분기 + 테스트 (TDD)
