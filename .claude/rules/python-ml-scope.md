# Python ML 규칙 (자동 적용)

이 파일은 Python ML 서버 코드 경로 작업 시 자동 로드된다.
프로젝트의 `.claude/rules/` 디렉토리에 복사하여 사용한다.

## 필수 검증 (커밋 전)

```bash
# 1. 린트
ruff check .

# 2. 포맷
ruff format --check .

# 3. 타입체크
mypy . --strict
# 또는
pyright .

# 4. 테스트
pytest tests/ -v
```

**네 가지 모두 통과해야 커밋 가능.**

## 코드 스타일

- 타입 힌팅 필수 (공개 API 모두)
- f-string 사용 (% 포맷팅, .format() 금지)
- 함수는 단일 책임
- 매직 넘버 → 상수 정의
- 주석은 "왜"를 설명 (코드가 보여주는 "무엇"은 반복하지 않음)

## ML 관련 규칙

### 모델 관리
- 모델 파일은 Git에 커밋하지 않음 (Git LFS 또는 외부 스토리지)
- 모델 버전은 config/환경변수로 관리
- 모델 로딩은 앱 시작 시 1회 (요청마다 로딩 금지)
- 모델 fallback 전략 정의 필수

### 추론 파이프라인
- 전처리 → 추론 → 후처리 명확 분리
- 각 단계는 독립적으로 테스트 가능
- 배치 처리 지원 (가능한 경우)
- 타임아웃 설정 필수

### 데이터 처리
- pandas 대신 polars 고려 (대용량 데이터 시)
- numpy 연산은 벡터화 우선 (for 루프 금지)
- 중간 결과 로깅 (디버깅용)
- 입력 데이터 검증 필수 (shape, dtype, range)

## 프레임워크

### FastAPI 서버
- 핸들러 함수는 얇게: 파싱 + 서비스 호출 + 응답 변환
- Pydantic v2 모델로 요청/응답 정의
- 비동기 I/O: `async def` + `httpx` (requests 대신)
- 헬스체크 엔드포인트 필수: `GET /health`

### 의존성 관리
- `uv` 또는 `poetry` 사용 (pip install 직접 사용 금지)
- `pyproject.toml`에 의존성 명시
- pinned 버전 사용 (>=X.Y.Z 금지, ==X.Y.Z 사용)

## 테스트 규칙

- pytest + pytest-asyncio
- ML 모델 테스트: 결정적 시드 사용 (`torch.manual_seed`, `np.random.seed`)
- 통합 테스트: 실 모델 대신 Mock/Stub
- 성능 테스트: 추론 시간 regression 감지
- 커버리지 **90% 이상** 유지

## 보안

- API 키는 환경변수 (하드코딩 금지)
- 사용자 입력은 반드시 검증 (prompt injection 방지)
- 모델 응답은 후처리 후 반환 (raw 출력 노출 금지)
- 에러 메시지에 내부 정보 노출 금지

## 디렉토리 구조 (권장)

```
src/
├── api/               # FastAPI 라우터
├── services/          # 유스케이스 오케스트레이션
├── domain/            # 비즈니스 로직
│   ├── models/        # ML 모델 래퍼
│   └── processors/    # 전처리/후처리
├── infrastructure/    # 외부 의존 (DB, API, 스토리지)
├── config/            # 설정 (pydantic-settings)
└── schemas/           # Pydantic 요청/응답 모델
```

## Docker

```dockerfile
# 멀티스테이지 빌드 권장
FROM python:3.11-slim AS base
# GPU 모델 사용 시 nvidia/cuda 베이스 이미지

# 의존성 먼저 설치 (캐시 활용)
COPY pyproject.toml uv.lock ./
RUN pip install uv && uv sync --frozen

# 소스 복사
COPY src/ ./src/
```
