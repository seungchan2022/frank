# Rust 규칙 (자동 적용)

이 파일은 Rust 서버 코드 경로 작업 시 자동 로드된다.
프로젝트의 `.claude/rules/` 디렉토리에 복사하여 사용한다.

## 필수 검증 (커밋 전)

```bash
# 1. 린트 (warnings를 에러로 처리)
cargo clippy -- -D warnings

# 2. 포맷
cargo fmt --check

# 3. 빌드 (타입체크 포함)
cargo build

# 4. 테스트
cargo test
```

**네 가지 모두 통과해야 커밋 가능.**

## 타입 안전 (필수)

- `serde_json::Value` 사용 최소화 — 외부 API 응답 파싱 외에는 금지
- 페이로드는 typed enum으로 정의
- 문자열 enum 금지 — Rust enum 사용
- `dyn Any` 금지 — 구체 타입 또는 trait object 사용
- `unwrap()` 금지 (테스트 코드 제외) — `?` 연산자 또는 `anyhow::Result` 사용
- `clone()` 남용 금지 — 소유권/차용 규칙 준수

## TDD + 포트 인터페이스 기반 테스트 (필수)

- 외부 의존성(DB, API, 파일시스템)은 **trait(Port)로 추상화**
- 테스트는 Mock 구현체를 주입하여 결정적 테스트
- 커버리지 **90% 이상** 유지
- 새 기능 구현 시 **테스트 먼저 작성**

```
패턴:
  trait DbPort → PostgresClient (프로덕션) / MockDb (테스트)
  trait HttpPort → ReqwestClient (프로덕션) / MockHttp (테스트)
```

## 동시성 안전 (필수)

- 공유 상태는 `Mutex<T>` 또는 `RwLock<T>` 사용
- trait에 `Send + Sync` 바운드 필수
- `unsafe` 블록 최소화 — 필요 시 Safety 주석 필수

## 코드 스타일

- `#[instrument]` 매크로로 함수별 트레이싱 (tracing 사용 시)
- 에러 타입: `thiserror` (라이브러리) / `anyhow` (애플리케이션)
- 로깅: `tracing::info!`, `tracing::error!` (println 금지)
- 빌더 패턴으로 복잡한 구조체 생성

## 웹 서버 (Axum / Actix)

- 핸들러 함수는 얇게 유지: 파싱 + 서비스 호출 + 응답 변환만
- 미들웨어로 인증/로깅/에러 처리
- 라우터 구성은 모듈별 분리

## 디렉토리 구조 (권장)

```
src/
├── main.rs          # 진입점
├── lib.rs           # 모듈 루트
├── config/          # 환경변수 설정
├── api/             # HTTP 핸들러 (라우터)
├── domain/          # 비즈니스 로직, trait 정의
├── services/        # 유스케이스 오케스트레이션
├── infrastructure/  # DB, 외부 API 구현체
├── middleware/       # 인증, 로깅 미들웨어
└── error/           # 에러 타입 정의
```
