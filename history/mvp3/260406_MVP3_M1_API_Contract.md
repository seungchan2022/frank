# M1: API Contract 보강

> 상태: ✅ 완료 (2026-04-07)
> 범위: 서버 (Rust)
> 의존: 없음
> 브랜치: `feature/260407_m1_api_contract`

## 목표

웹/iOS가 Supabase 직접 호출 대신 Rust API만으로 모든 데이터 요청을 처리할 수 있도록 부족한 엔드포인트 추가 + 기존 엔드포인트 확장.

## 현재 API 현황

| 엔드포인트 | 상태 | 비고 |
|-----------|------|------|
| GET /health | ✅ | |
| GET /api/tags | ✅ | |
| GET /api/me/tags | ✅ | |
| POST /api/me/tags | ✅ | onboarding_completed 자동 설정 포함 |
| GET /api/me/profile | ✅ | |
| GET /api/me/articles | ✅ | limit만 지원 |
| POST /api/me/collect | ✅ | |
| POST /api/me/summarize | ✅ | |

## 서브태스크

### ST-1: GET /api/me/articles 확장 (offset + tag_id 필터)
- **유형**: feature
- **상태**: [x]
- **현재**: `?limit=50` 만 지원
- **변경**: `?limit=20&offset=0&tag_id=<uuid>` 지원 (모두 optional, 기존 호출 호환)
- **검증**: limit clamp(1..=100), offset >= 0, tag_id UUID 파싱 실패 시 400
- **포트**: `get_user_articles(user_id, limit, offset, tag_id: Option<Uuid>)` 단일 확장
- **수정 파일**: `server/src/api/articles.rs`, `server/src/domain/ports.rs`, `server/src/infra/postgres_db.rs`, `server/src/infra/fake_db.rs`
- **의존**: ST-4(DTO) 선행 권장

### ST-2: GET /api/me/articles/:id (기사 단건 조회)
- **유형**: feature
- **상태**: [x]
- **설명**: UUID로 단일 기사 조회. WHERE id=$1 AND user_id=$2 강제
- **응답**: `ArticleResponse` (ST-4 DTO 적용)
- **에러**: 타인 기사/없는 기사 모두 **404 통일** (정보 누설 방지)
- **포트**: `get_user_article_by_id(user_id, article_id) -> Option<Article>`
- **수정 파일**: `server/src/api/articles.rs`, `server/src/lib.rs` (라우트 추가), `server/src/domain/ports.rs`, `server/src/infra/postgres_db.rs`, `server/src/infra/fake_db.rs`
- **의존**: ST-4(DTO) 선행 권장

### ST-3: PUT /api/me/profile (프로필 수정)
- **유형**: feature
- **상태**: [x]
- **설명**: onboarding_completed, display_name 부분 수정
- **요청**: `{ "onboarding_completed": Option<bool>, "display_name": Option<String> }` — 빈 바디 `{}` 허용 (no-op, 현재 프로필 반환)
- **검증**: display_name trim, 최대 50자
- **응답**: 수정된 `Profile`
- **포트**: `update_profile(user_id, onboarding_completed: Option<bool>, display_name: Option<String>) -> Profile`
- **수정 파일**: `server/src/api/profile.rs` (**신설**), `server/src/lib.rs`, `server/src/domain/ports.rs`, `server/src/infra/postgres_db.rs`, `server/src/infra/fake_db.rs`
- **의존**: 없음

### ST-4: 응답 필드 정리 (클라이언트 불필요 필드 제외)
- **유형**: chore
- **상태**: [x]
- **설명**: articles 응답에서 `content`, `llm_model`, `prompt_tokens`, `completion_tokens` 제외 (서버 내부용)
- **방법**: `ArticleResponse` DTO 신설 + `From<Article>` impl. **도메인 모델 불변 유지**
- **수정 파일**: `server/src/api/articles.rs` (DTO 정의 위치 확정)
- **의존**: 없음 (선행)

### ST-5: 웹 Article 타입에 title_ko 추가
- **유형**: chore
- **상태**: [x]
- **설명**: `web/src/lib/types/article.ts`에 `title_ko: string | null` 필드 추가
- **수정 파일**: `web/src/lib/types/article.ts`
- **의존**: 없음

### ST-6: 테스트
- **유형**: test
- **상태**: [x]
- **설명**: ST-1~ST-4 단위 + 통합 + 회귀 테스트
- **케이스**:
  - ST-1: limit/offset/tag_id 조합 4종, 음수/과대값 400, 잘못된 UUID 400
  - ST-2: 본인 기사 200, 타인 기사 404, 없는 기사 404
  - ST-3: 부분 업데이트, 빈 바디 no-op, display_name trim/길이
  - ST-4: 응답 JSON에 `content`/`llm_model`/토큰 필드 부재 검증
  - 공통: 인증 401 회귀
- **수정 파일**: `server/src/api/` 테스트 모듈
- **의존**: ST-1, ST-2, ST-3, ST-4

## 의존 그래프

```
ST-5 (웹 타입) ─────────→ (완전 독립, 언제든 실행)

ST-4 (DTO 선행) → ST-1 → ST-2 → ST-3 → ST-6 (통합/회귀 검증)
```

- **실행 순서 (확정)**: ST-5 → **ST-4 (DTO 선행)** → ST-1 → ST-2 → ST-3 → ST-6 (**순차 실행**)
- **순차 실행 이유**: ST-1/2/3이 모두 `ports.rs`, `postgres_db.rs`, `fake_db.rs`를 공유 수정하므로 물리적 병렬 시 머지 충돌 발생. 논리적 병렬 ≠ 물리적 병렬.
- **TDD 정책**: ST-1~ST-4는 각자 단위 테스트를 먼저 작성(TDD)한 뒤 구현. ST-6은 "엔드포인트 통합/curl 스모크 + 회귀 검증" 성격.

## 서브태스크 요약표

| ID | 이름 | 유형 | 크기 | 독립성 | 산출물 |
|----|------|------|------|--------|--------|
| ST-1 | articles 확장 (offset+tag_id) | feature | 1-2h | ST-4 선행 권장 | 쿼리 파라미터 3종 동작 |
| ST-2 | articles/:id 단건 조회 | feature | 1-2h | ST-4 선행 권장 | 새 엔드포인트 + 권한 404 |
| ST-3 | profile PUT | feature | 1-2h | 완전 독립 | 새 엔드포인트 + Profile 반환 |
| ST-4 | 응답 DTO 분리 | chore | 30m | 완전 독립 | `ArticleResponse` 타입 |
| ST-5 | 웹 title_ko 추가 | chore | 10m | 완전 독립 | `article.ts` 필드 추가 |
| ST-6 | 통합/회귀 검증 | test | 1h | ST-1~ST-4 완료 후 | curl 스모크 + 회귀 통과 |

## 완료 기준

- [x] `cargo clippy -- -D warnings` 통과
- [x] `cargo fmt --check` 통과
- [x] `cargo test` 전체 통과 (157 passed, +17 신규)
- [x] 새 엔드포인트 3개 동작 확인 (TestServer 통합 + 401 회귀 테스트)
- [x] web `npm run check` / vitest 통과 (384 files / 77 tests)

## 완료 요약 (2026-04-07)

| 영역 | 결과 |
|------|------|
| 신규 엔드포인트 | `PUT /api/me/profile`, `GET /api/me/articles/:id` |
| 확장 엔드포인트 | `GET /api/me/articles` (offset/tag_id) |
| DTO 분리 | `ArticleResponse` (내부 4필드 차단) |
| 웹 타입 | `Article.title_ko` 추가 |
| 신규 테스트 | +17건 (단위 14 + 401 회귀 3) |
| 부수효과 | `ARTICLE_COLUMNS` 상수 추출, `seed_article` 헬퍼 |

### Followup (M2/M3에서 흡수)
1. `PUT /me/profile` tri-state — `display_name` 명시적 null 초기화 (`Option<Option<String>>`)
2. `get_my_profile` 핸들러를 `api/tags.rs` → `api/profile.rs`로 이관 (응집도)
3. OFFSET 페이지네이션 → cursor 기반 (큰 offset 성능)
