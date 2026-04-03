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

## 다음 단계

→ `/step-9` (커밋)
