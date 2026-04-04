# DB 접근 전략 심층분석: Supabase REST -> sqlx

> 분석일: 260404
> 유형: performance + architecture

---

## 1. 현재 아키텍처 분석

### 1.1 의존 구조

```
api/ (핸들러) -> services/ (유스케이스) -> domain/ports.rs (DbPort trait) <- infra/supabase_db.rs (어댑터)
```

- **DbPort trait**: 8개 메서드 (`get_profile`, `update_profile_onboarding`, `list_tags`, `get_user_tags`, `set_user_tags`, `save_articles`, `get_user_articles`, `update_article_summary`)
- **SupabaseDbAdapter**: `reqwest::Client`로 Supabase REST API(PostgREST) 호출
- **FakeDbAdapter**: `Arc<Mutex<HashMap>>` 기반 인메모리 Mock (테스트용)
- **auth_token**: 모든 DbPort 메서드에 `auth_token: &str` 파라미터로 전달 -> RLS 적용을 위해 Bearer 토큰을 REST 헤더에 포함

### 1.2 현재 DB 접근 경로

```
Rust Server
  -> reqwest HTTP POST/GET/PATCH/DELETE
    -> Supabase Edge (Kong API Gateway)
      -> PostgREST (REST->SQL 변환)
        -> PostgreSQL (RLS 적용)
          -> 응답 JSON
        <- PostgREST JSON 직렬화
      <- Kong 응답 전달
    <- reqwest JSON 역직렬화
  -> serde 파싱 -> 도메인 모델
```

### 1.3 테이블 및 RLS 현황

| 테이블 | RLS | 정책 | 비고 |
|--------|-----|------|------|
| `profiles` | ON | SELECT/UPDATE/INSERT: `auth.uid() = id` | auth.users FK |
| `tags` | ON | SELECT: 모든 인증 사용자 | 읽기 전용 공용 데이터 |
| `user_tags` | ON | SELECT/INSERT/DELETE: `auth.uid() = user_id` | 복합 PK |
| `articles` | ON | SELECT/INSERT/DELETE/UPDATE: `auth.uid() = user_id` | UNIQUE(user_id, url) |

### 1.4 현재 코드 특성

- `auth_token`이 DbPort 인터페이스에 관통 파라미터로 존재 (8개 메서드 전부)
- `set_user_tags`: DELETE + INSERT 2회 HTTP 호출 (트랜잭션 미보장)
- `save_articles`: `on_conflict=user_id,url` + `resolution=ignore-duplicates` PostgREST 헤더 활용
- 모든 에러 처리: HTTP 응답 코드 + body 텍스트 기반

---

## 2. 성능 비교

### 2.1 네트워크 홉 수

| 경로 | 홉 수 | 구성 |
|------|-------|------|
| REST API | 4 | Server -> Kong -> PostgREST -> PostgreSQL |
| sqlx 직접 | 1 | Server -> PostgreSQL (TCP) |

### 2.2 직렬화/역직렬화 오버헤드

| 단계 | REST API | sqlx |
|------|----------|------|
| 요청 직렬화 | Rust struct -> JSON -> HTTP body | Rust value -> binary protocol |
| SQL 변환 | PostgREST가 URL 쿼리 파라미터 -> SQL | 직접 SQL 작성 |
| 응답 역직렬화 | PostgreSQL row -> PostgREST JSON -> HTTP body -> reqwest -> serde | PostgreSQL row -> binary -> sqlx `FromRow` |
| 추가 오버헤드 | HTTP 헤더 파싱, TLS (Kong) | 없음 |

**예상 오버헤드 차이**: 단일 쿼리당 직렬화/역직렬화에서 약 0.5~2ms 차이 발생.

### 2.3 연결 풀링

| 항목 | REST (reqwest) | sqlx (PgPool) |
|------|---------------|---------------|
| 풀링 방식 | HTTP keep-alive / connection reuse | TCP 커넥션 풀 (PgPool) |
| 연결 수립 비용 | TLS 핸드셰이크 (첫 연결) | TCP + PostgreSQL 인증 (첫 연결) |
| 풀 크기 제어 | reqwest Client 내부 관리 (제한적) | `PgPoolOptions::max_connections()` 명시적 제어 |
| 연결 재사용 | HTTP/1.1 keep-alive (Kong 설정 의존) | 완전 제어 (idle timeout, max lifetime) |
| 동시 요청 시 | HTTP 파이프라이닝 제한적 | 풀에서 즉시 커넥션 획득 |

### 2.4 레이턴시 추정

**단일 SELECT 쿼리 (profiles 1건 조회) 기준:**

| 지표 | REST API | sqlx | 개선율 |
|------|----------|------|--------|
| Avg latency | 15~40ms | 1~5ms | 3~10x |
| P50 | 20ms | 2ms | 10x |
| P99 | 80~200ms | 5~15ms | 10~15x |
| P99.9 (스파이크) | 300ms~1s+ | 10~30ms | 20~30x |

> **참고**: REST API 레이턴시는 Supabase 프로젝트 리전, Kong 프록시 상태, PostgREST 프로세스 웜업 등에 따라 편차가 크다. 특히 Supabase 무료/프로 플랜에서는 콜드 스타트 시 1초 이상 스파이크 발생 가능.

**복합 오퍼레이션 (`set_user_tags`: DELETE + INSERT):**

| 지표 | REST API (2회 HTTP) | sqlx (1 트랜잭션) | 개선율 |
|------|---------------------|-------------------|--------|
| Avg | 40~80ms | 3~8ms | 5~10x |
| 원자성 | 미보장 (중간 실패 시 불일치) | BEGIN..COMMIT 보장 | 무한대 |

### 2.5 트랜잭션 지원

| 기능 | REST API | sqlx |
|------|----------|------|
| 단일 쿼리 트랜잭션 | PostgREST가 자동 처리 | 자동 |
| 다중 쿼리 트랜잭션 | 불가능 (각 HTTP = 별도 트랜잭션) | `sqlx::Transaction` 완전 지원 |
| Savepoint | 불가능 | 지원 |
| Isolation level | 제어 불가 | SET TRANSACTION 가능 |

**현재 `set_user_tags`는 DELETE -> INSERT가 별도 HTTP 호출이므로, DELETE 성공 후 INSERT 실패 시 데이터 불일치 발생 가능.** sqlx 전환 시 트랜잭션으로 원자성 보장 가능.

---

## 3. RLS 전략 비교

### 3.1 A안: Server-level WHERE (RLS 바이패스)

**방식**: `service_role` 키로 PostgreSQL 연결 (RLS 무시), 모든 쿼리에 `WHERE user_id = $1` 추가.

```rust
// 예시
async fn get_profile(&self, user_id: Uuid) -> Result<Profile, AppError> {
    sqlx::query_as!(Profile,
        "SELECT id, display_name, onboarding_completed FROM profiles WHERE id = $1",
        user_id
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}
```

| 항목 | 평가 |
|------|------|
| 성능 | 최고. RLS 체크 오버헤드 없음 |
| 보안 | 서버 코드에 의존. WHERE 누락 시 전체 데이터 노출 위험 |
| 유지보수 | 쿼리마다 `WHERE user_id =` 작성 필수. 실수 가능성 존재 |
| 복잡도 | 낮음. 표준 sqlx 패턴 |
| DB 변경 | RLS 정책 유지하되, 서버 연결은 `service_role`로 바이패스 |
| auth_token | DbPort에서 제거 가능 (서버가 user_id만으로 접근 제어) |

**위험 완화**: `cargo clippy` 커스텀 린트 또는 코드 리뷰로 WHERE 누락 방지. 테스트에서 교차 사용자 접근 검증.

### 3.2 B안: RLS 유지 (SET LOCAL)

**방식**: 매 쿼리를 트랜잭션으로 감싸고, `SET LOCAL` 으로 JWT claims 설정.

```rust
async fn get_profile(&self, user_id: Uuid, auth_token: &str) -> Result<Profile, AppError> {
    let mut tx = self.pool.begin().await?;

    // JWT claims를 PostgreSQL 세션 변수로 설정
    let claims = json!({"sub": user_id.to_string(), "role": "authenticated"});
    sqlx::query(&format!("SET LOCAL request.jwt.claims = '{}'", claims))
        .execute(&mut *tx)
        .await?;
    sqlx::query("SET LOCAL role = 'authenticated'")
        .execute(&mut *tx)
        .await?;

    let profile = sqlx::query_as!(Profile,
        "SELECT id, display_name, onboarding_completed FROM profiles WHERE id = $1",
        user_id
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(profile)
}
```

| 항목 | 평가 |
|------|------|
| 성능 | 중간. 매 쿼리마다 SET LOCAL 2회 + BEGIN/COMMIT 오버헤드 |
| 보안 | 최고. PostgreSQL RLS가 최종 방어선 역할 |
| 유지보수 | 높은 보일러플레이트. 매 메서드마다 SET LOCAL 패턴 반복 |
| 복잡도 | 높음. 트랜잭션 관리 + role 전환 + claims 주입 |
| DB 변경 | RLS 정책 그대로 유지 |
| auth_token | 여전히 필요 (claims 구성용) 또는 user_id로 claims 재구성 |

**주의**: `SET LOCAL`은 해당 트랜잭션 내에서만 유효. 트랜잭션 외부에서 실행하면 효과 없음. 커넥션 풀 반환 시 자동 리셋되나, 실수 시 role 누출 위험.

### 3.3 C안: 하이브리드 (읽기 바이패스 + 쓰기 RLS)

**방식**: 읽기는 `service_role` + WHERE, 쓰기는 SET LOCAL로 RLS 적용.

| 항목 | 평가 |
|------|------|
| 성능 | 읽기 최적, 쓰기 중간 |
| 보안 | 읽기에서 WHERE 누락 위험 존재 |
| 유지보수 | 가장 복잡. 읽기/쓰기 패턴이 다름 |
| 복잡도 | 최고. 두 패턴 혼용으로 인지 부하 증가 |

### 3.4 RLS 전략 추천

| 기준 | A안 (WHERE) | B안 (SET LOCAL) | C안 (하이브리드) |
|------|-------------|-----------------|------------------|
| 성능 | +++  | + | ++ |
| 보안 | ++ | +++ | ++ |
| 유지보수 | ++ | + | - |
| 복잡도 | 낮음 | 높음 | 최고 |
| **추천** | **추천** | 보안 최우선 시 | 비추천 |

**A안 추천 이유**:
1. 현재 서버가 이미 미들웨어에서 JWT 검증 + user_id 추출을 하고 있음 (`middleware/auth.rs`)
2. 서버가 인증 게이트웨이 역할을 하므로, DB 레벨 RLS는 이중 방어에 해당
3. `auth_token` 파라미터를 DbPort에서 제거할 수 있어 인터페이스가 깔끔해짐
4. 테스트에서 FakeDbAdapter도 `_auth_token` 파라미터를 사용하지 않으므로, 현재도 사실상 서버 레벨 접근 제어에 의존

---

## 4. 마이그레이션 영향도

### 4.1 변경 대상 파일

| 파일 | 변경 유형 | 영향도 |
|------|----------|--------|
| `Cargo.toml` | sqlx 의존성 추가, reqwest는 다른 어댑터에서 사용하므로 유지 | 소 |
| `config/mod.rs` | `database_url` 추가 | 소 |
| `domain/ports.rs` | `auth_token` 파라미터 제거 (A안 채택 시) | 중 |
| `domain/models.rs` | `sqlx::FromRow` derive 추가 | 소 |
| `infra/supabase_db.rs` | 전면 재작성 -> `infra/postgres_db.rs` | 대 |
| `infra/fake_db.rs` | `auth_token` 파라미터 제거 반영 | 소 |
| `infra/mod.rs` | 모듈 경로 변경 | 소 |
| `main.rs` | PgPool 초기화, 어댑터 교체 | 소 |
| `lib.rs` | `create_router` 시그니처 변경 가능성 | 소 |
| `services/collect_service.rs` | `auth_token` 전달 제거 | 소 |
| `services/summary_service.rs` | `auth_token` 전달 제거 | 소 |
| `services/tag_service.rs` | `auth_token` 전달 제거 | 소 |
| `api/tags.rs` | `auth_token` 전달 제거 | 소 |
| `api/articles.rs` | `auth_token` 전달 제거 | 소 |
| `middleware/auth.rs` | `AuthUser.token` 필드 제거 가능 | 소 |
| 테스트 전체 | `auth_token` 파라미터 제거 반영 | 중 |

### 4.2 변경 규모 요약

| 항목 | 수치 |
|------|------|
| 변경 파일 수 | ~15개 |
| 신규 파일 | 1개 (`infra/postgres_db.rs`) |
| 삭제 파일 | 1개 (`infra/supabase_db.rs`) |
| 전면 재작성 | 1개 (어댑터: ~260줄) |
| DbPort trait 메서드 시그니처 변경 | 8개 |
| 추정 작업량 | 2~3시간 |

### 4.3 위험 요소

| 위험 | 심각도 | 완화 방안 |
|------|--------|----------|
| DB 연결 문자열 노출 | 높음 | 환경변수만 사용, .env gitignore 확인 |
| `set_user_tags` 트랜잭션 미적용 | 중간 (현재도 동일) | 마이그레이션 시 트랜잭션으로 개선 |
| `sqlx::FromRow` derive 매크로 컴파일 타임 체크 | 낮음 | DB 스키마와 모델 불일치 시 컴파일 에러로 조기 발견 |
| Supabase Auth API 의존 유지 | 정보 | 인증은 여전히 Supabase Auth REST API 사용 (별도 관심사) |
| 로컬 개발 환경 | 중간 | Supabase CLI(`supabase start`)로 로컬 PostgreSQL 사용 가능 |

---

## 5. 개선안

### A안: 보수적 (어댑터만 교체)

- `SupabaseDbAdapter` -> `PostgresDbAdapter`로 교체
- DbPort trait 시그니처 유지 (`auth_token` 파라미터 유지하되 미사용)
- RLS: A안 (service_role + WHERE)
- 영향 범위: `infra/` + `main.rs` + `Cargo.toml`만 변경

| 장점 | 단점 |
|------|------|
| 변경 최소 | `auth_token` 사용하지 않는 파라미터가 인터페이스에 잔류 |
| 점진적 전환 가능 | 기술 부채 발생 |
| 테스트 수정 불필요 | |

**추정 작업량**: 1~2시간

### B안: 추천 (인터페이스 정리 + 어댑터 교체)

- DbPort trait에서 `auth_token` 파라미터 제거
- `SupabaseDbAdapter` -> `PostgresDbAdapter`로 전면 교체
- RLS: A안 (service_role + WHERE)
- `set_user_tags`를 트랜잭션으로 감싸 원자성 확보
- 도메인 모델에 `sqlx::FromRow` derive 추가

| 장점 | 단점 |
|------|------|
| 깔끔한 인터페이스 | 변경 파일 수 ~15개 |
| 트랜잭션 원자성 확보 | 전체 테스트 수정 필요 |
| 컴파일 타임 쿼리 검증 | sqlx CLI 설정 필요 |
| 레이턴시 3~10x 개선 | |

**추정 작업량**: 2~3시간

### C안: 공격적 (전면 리팩토링)

- B안 전체 + 추가 개선
- 인증 미들웨어도 JWT 로컬 검증으로 전환 (Supabase Auth API 호출 제거)
- `sqlx::query_as!` 매크로로 컴파일 타임 SQL 검증
- 커넥션 풀 헬스체크 + 메트릭 추가
- DB 마이그레이션을 sqlx-cli로 관리

| 장점 | 단점 |
|------|------|
| 외부 HTTP 의존 최소화 | 대규모 변경 |
| 인증 레이턴시도 개선 | JWT 로컬 검증 로직 구현 필요 |
| 운영 가시성 확보 | 작업량 증가 |

**추정 작업량**: 4~6시간

---

## 6. 결론 및 추천

### 추천: B안 (인터페이스 정리 + 어댑터 교체)

**이유**:

1. **성능 개선 확실**: 단일 쿼리 기준 평균 레이턴시 3~10x 개선, P99 스파이크 10~30x 개선
2. **트랜잭션 원자성**: `set_user_tags`의 DELETE+INSERT 불일치 버그 해소
3. **포트/어댑터 패턴 유지**: DbPort trait의 어댑터만 교체하므로 아키텍처 원칙 준수
4. **인터페이스 개선**: 불필요한 `auth_token` 파라미터 제거로 코드 간결화
5. **적정 작업량**: 2~3시간으로 ROI 우수
6. **기존 테스트 활용**: FakeDbAdapter는 시그니처만 변경하면 동일 테스트 유지

### 실행 순서 (B안)

```
1. Cargo.toml에 sqlx 의존성 추가
2. domain/models.rs에 sqlx::FromRow derive 추가
3. domain/ports.rs에서 auth_token 파라미터 제거
4. infra/postgres_db.rs 신규 작성 (PgPool 기반)
5. infra/fake_db.rs 시그니처 업데이트
6. services/ 3개 파일에서 auth_token 전달 제거
7. api/ 핸들러에서 auth_token 전달 제거
8. main.rs에서 PgPool 초기화 + 어댑터 교체
9. config/mod.rs에 database_url 추가
10. 전체 테스트 실행 + 커버리지 확인
```

### Cargo.toml 추가 의존성

```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
```

### 향후 고려사항

- **C안 점진적 적용**: B안 완료 후, 인증 미들웨어 JWT 로컬 검증 전환은 별도 태스크로 분리 가능
- **모니터링**: sqlx PgPool 메트릭 (active/idle connections) 로깅 추가 권장
- **마이그레이션 도구**: sqlx-cli 도입 시 `supabase/migrations/` 와의 관계 정리 필요
