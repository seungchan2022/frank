---
name: step-2
description: 룰즈 검증. Codex로 rules/ 준수 여부 확인.
context: fork
allowed-tools:
  - Read
  - mcp__codex__codex
  - mcp__codex__codex-reply
---

# Step 2: 룰즈 검증

## 수행 작업

1. **rules/ 로드**: 관련 규칙 파일 읽기
2. **Codex 리뷰**: Codex MCP로 메인태스크 문서 검증
3. **위반 사항 보고**: 규칙 위반 시 수정 요청

## 검증 대상 규칙

| 파일 | 검증 항목 |
|------|----------|
| `rules/0_CODEX_RULES.md` | 강제 규칙, TDD, 보안, 출력 요구사항 |
| `rules/sub/workflow.md` | 개발 워크플로우 절차 |
| `rules/sub/git.md` | Git 커밋 규칙 |

## 산출물

```markdown
## Codex 리뷰 결과 ({YYMMDD})

### 반영됨
- {반영된 피드백}

### 잔여 이슈
- {해결되지 않은 이슈}
```

## 다음 단계

→ `/step-3` (서브태스크 분리)
