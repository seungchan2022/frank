# 서브 룰북 인덱스

| # | 파일 | 설명 |
|---|---|---|
| 1 | agents.md | 에이전트 역할 + **MCP 서버 정책/안전/비용/우선순위/토론 프로토콜** (v2.0 머지본) |
| 2 | documentation.md | Progress 문서 형식 |
| 3 | git.md | Git 커밋 규칙 |
| 4 | session_scope.md | 멀티세션 스코프 격리 |
| 5 | sub_agent_usage.md | 서브에이전트 사용 규칙 (실제 등록 에이전트 기반) |
| 6 | workflow.md | 개발 사고 가이드 (0_CODEX_RULES.md §3의 5단계 내부 참고) |
| 7 | milestone.md | 마일스톤 플로우 (Discovery-First, KPI 선언, 상태 전이) |

## v2.0 변경 사항

- **삭제**: `mcp_integration.md` (agents.md에 머지), `mcp_tool_design.md` (미사용)
- **통합**: MCP 정책은 `agents.md` 하나로 통일 — MCP 목록·안전·비용·우선순위 체인 포함
- **재정의**: `workflow.md`는 별도 단계 시스템이 아닌 **사고 가이드**. 유일한 강제 워크플로우는 `0_CODEX_RULES.md §3`의 5단계 (Inspect → Specify → Implement → Verify → Report)
- **신규 강제**: `0_CODEX_RULES.md §3.5` 마일스톤 KPI 게이트 (Hard 지표 미달 시 커밋 차단)
