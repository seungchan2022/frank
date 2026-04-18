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
[2] Claude + Codex 병렬 리뷰 (정합성·기술적 타당성)
       ↓
[3] (선택) 필요 시 /debate 명령으로 3자 토론 가능
       ↓
[3.5] ★ 구멍 찾기 리뷰 자동 호출 — /critical-review
       ↓
[4] 수정 사항 반영
       ↓
[5] 리뷰 결과 문서화
```

## 리뷰 체크리스트 (기본)

- [ ] 서브태스크 목표가 명확한가?
- [ ] 수정 대상 파일이 정확한가?
- [ ] 구현 방법이 규칙을 준수하는가?
- [ ] 테스트 계획이 충분한가?
- [ ] 기존 코드와의 충돌은 없는가?

## [3.5] 구멍 찾기 리뷰 (자동 호출)

Claude+Codex 병렬 리뷰가 "승인" 또는 "조건부 승인"으로 끝나도 **자동으로 `/critical-review` 호출**.

목적: 기술적으로 통과한 분리안이라도 **기획·구조의 숨은 함정**을 선제 발굴.

- 호출 대상: 방금 만든 서브태스크 분리 문서
- 적대적 시각으로 놓친 엣지케이스·구조 결함·전제 함정 발굴
- 결과는 **치명/중대/경미** 분류 + 각 구멍 대안 동반
- 치명/중대 발견 시 → **사용자에게 수정 여부 확인 후 [4]로 진행**
- 경미만 있으면 → 후속 과제로 기록하고 [4]로 진행

상세는 `.claude/skills/critical-review/SKILL.md` 참조.

> **왜 step-5에서?** 구현 전이라 구멍 수정 비용이 최저. step-7까지 기다리면 벽 부수는 비용.

## 산출물

```markdown
## 리뷰 결과

### Claude 리뷰 (문서 일치성)
### Codex 리뷰 (기술적 타당성)
### 구멍 찾기 리뷰 (critical-review)
  - 치명: N건
  - 중대: N건
  - 경미: N건 (각 구멍 요약 + 대안)
### 최종 결정: {승인/조건부 승인/반려}
```

## 다음 단계

→ `/step-6` (구현)
