# M1: API Contract 보강

> 상태: 📋 기획 완료
> 범위: 서버 (Rust)
> 의존: 없음

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
- **상태**: [ ]
- **현재**: `?limit=50` 만 지원
- **변경**: `?limit=20&offset=0&tag_id=<uuid>` 지원
- **수정 파일**: `server/src/api/articles.rs`, `server/src/domain/ports.rs`, `server/src/infra/postgres_db.rs`
- **의존**: 없음

### ST-2: GET /api/me/articles/:id (기사 단건 조회)
- **유형**: feature
- **상태**: [ ]
- **설명**: UUID로 단일 기사 조회. 본인 기사만 접근 가능 (user_id 검증)
- **응답**: `Article` (전체 필드)
- **에러**: 404 Not Found (기사 없음 또는 권한 없음)
- **수정 파일**: `server/src/api/articles.rs`, `server/src/lib.rs` (라우트 추가), `server/src/domain/ports.rs`, `server/src/infra/postgres_db.rs`
- **의존**: 없음

### ST-3: PUT /api/me/profile (프로필 수정)
- **유형**: feature
- **상태**: [ ]
- **설명**: onboarding_completed, display_name 수정
- **요청**: `{ "onboarding_completed": true, "display_name": "이름" }` (모두 optional)
- **응답**: 수정된 `Profile`
- **수정 파일**: `server/src/api/tags.rs` (또는 새 `api/profile.rs`), `server/src/lib.rs`, `server/src/domain/ports.rs`, `server/src/infra/postgres_db.rs`
- **의존**: 없음

### ST-4: 응답 필드 정리 (클라이언트 불필요 필드 제외)
- **유형**: chore
- **상태**: [ ]
- **설명**: articles 응답에서 `content`, `llm_model`, `prompt_tokens`, `completion_tokens` 제외 (서버 내부용)
- **방법**: ArticleResponse DTO 생성, 또는 `#[serde(skip_serializing)]` 적용
- **수정 파일**: `server/src/domain/models.rs` 또는 `server/src/api/articles.rs`
- **의존**: 없음

### ST-5: 웹 Article 타입에 title_ko 추가
- **유형**: chore
- **상태**: [ ]
- **설명**: `web/src/lib/types/article.ts`에 `title_ko: string | null` 필드 추가
- **수정 파일**: `web/src/lib/types/article.ts`
- **의존**: 없음

### ST-6: 테스트
- **유형**: test
- **상태**: [ ]
- **설명**: ST-1~ST-4 각각에 대한 단위 테스트 + 통합 테스트
- **수정 파일**: `server/src/api/` 테스트 모듈
- **의존**: ST-1, ST-2, ST-3, ST-4

## 의존 그래프

```
ST-1 (articles 확장) ──┐
ST-2 (articles/:id) ───┼→ ST-6 (테스트)
ST-3 (profile PUT) ────┘
ST-4 (응답 정리) ───────→ ST-6
ST-5 (웹 타입) ─────────→ (독립)
```

## 완료 기준

- [ ] `cargo clippy -- -D warnings` 통과
- [ ] `cargo fmt --check` 통과
- [ ] `cargo test` 전체 통과
- [ ] 새 엔드포인트 3개 동작 확인 (curl 또는 테스트)
