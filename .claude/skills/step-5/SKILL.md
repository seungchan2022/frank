---
name: step-5
description: 서브태스크 리뷰. Claude + Codex 병렬 리뷰.
context: fork
allowed-tools:
  - Read
  - Write
  - mcp__codex__codex
  - mcp__codex__codex-reply
  - mcp__serena__find_symbol
  - mcp__serena__get_symbols_overview
  - mcp__serena__find_referencing_symbols
  - mcp__serena__read_file
  - mcp__serena__list_dir
  - mcp__serena__search_for_pattern
  - mcp__context7__resolve-library-id
  - mcp__context7__query-docs
  - mcp__exa__web_search_exa
  - Task
---

# Step 5: 서브태스크 리뷰

## 수행 작업

```
[1] 서브태스크 문서 로드
       ↓
[2] Claude + Codex 병렬 리뷰
       ↓
[3] (선택) 필요 시 /debate 명령으로 3자 토론 가능
       ↓
[4] 수정 사항 반영
       ↓
[5] 리뷰 결과 문서화
```

## 리뷰 체크리스트

- [ ] 서브태스크 목표가 명확한가?
- [ ] 수정 대상 파일이 정확한가?
- [ ] 구현 방법이 규칙을 준수하는가?
- [ ] 테스트 계획이 충분한가?
- [ ] 기존 코드와의 충돌은 없는가?

## 산출물

```markdown
## 리뷰 결과

### Claude 리뷰 (문서 일치성)
### Codex 리뷰 (기술적 타당성)
### 최종 결정: {승인/조건부 승인/반려}
```

## 다음 단계

→ `/step-6` (구현)
