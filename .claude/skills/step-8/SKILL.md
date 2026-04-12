---
name: step-8
description: 테스트. 프로젝트 타입별 자동 감지.
context: fork
allowed-tools:
  - Bash
  - Read
---

# Step 8: 테스트

## 프로젝트 타입 자동 감지

```bash
git diff --cached --name-only 2>/dev/null || git diff --name-only HEAD
```

변경된 파일 경로를 기반으로 어떤 테스트를 실행할지 자동 결정한다.

## 테스트 명령어

프로젝트에 맞는 테스트 명령어를 실행한다:

### Swift (iOS/macOS)
```bash
swift test                          # 단위 테스트
swiftlint lint                      # 린트
```

### Svelte (프론트엔드)
```bash
cd {프론트엔드_경로}
npm run lint                        # ESLint
npm run check                       # svelte-check
npm run test                        # Vitest
```

### Rust (서버)
```bash
cargo clippy -- -D warnings         # 린트
cargo test                          # 테스트
```

### Python ML (서버)
```bash
ruff check .                        # 린트
mypy .                              # 타입체크
pytest tests/ -v                    # 테스트
```

## 통과 기준

| 테스트 | 기준 |
|--------|------|
| 린트 | 0 errors |
| 타입체크 | 0 errors |
| 단위 테스트 | 100% pass |

## 자동화 테스트 통과 후 — 배포 + 수동 테스트 (필수)

자동화 테스트가 모두 통과하면 **반드시** 아래 순서로 진행한다.
건너뛰지 않는다.

### 1. 배포 스크립트 실행

```bash
scripts/deploy.sh
```

- Docker 미실행 시 `y` 입력하여 네이티브 모드로 실행
- 브라우저(`http://localhost:5173`)와 iOS 시뮬레이터가 자동으로 열린다

### 2. 수동 테스트 시나리오 안내

배포 완료 후 구현한 기능의 테스트 시나리오를 사용자에게 안내한다:

- **골든 패스**: 정상 흐름 전체 (로그인 → 기능 사용 → 결과 확인)
- **엣지 케이스**: 권한 없음, 데이터 없음, 실패 상황 등
- **회귀 확인**: 기존 기능이 깨지지 않았는지 주요 화면 확인

### 3. 사용자 확인 대기

테스트 결과를 사용자로부터 받은 후 Step-9로 넘어간다.
사용자 확인 없이 Step-9를 진행하지 않는다.

## 다음 단계

→ 사용자 테스트 확인 후 `/step-9` (커밋)
