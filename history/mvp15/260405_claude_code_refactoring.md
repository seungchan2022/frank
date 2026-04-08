# Claude Code 설정 리팩토링

> 생성일: 260405
> 상태: ✅ 완료 (17건 중 16건 완료, #14 취소 — 사용자 결정)

## 배경

MVP1~1.5 세션 분석을 통해 Claude Code 설정 파일(CLAUDE.md, rules/, agents/, skills/)의 문제점을 심층 진단하고, 리서치 기반으로 개선안을 확정함.

## 핵심 발견

1. CLAUDE.md와 rules/ 간 ~60% 중복 → 컨텍스트 낭비
2. 금지규칙을 "문서"로만 강제 → 압축 후 약화되어 무시됨
3. compactPrompt 미설정 → 장시간 세션에서 규칙 망각
4. mcp_integration.md 213줄 → 우선순위 체인은 Claude 자체 판단이 더 정확
5. 미설치 MCP 6개가 스킬에서 참조 → 실행 시 실패
6. 24개 스킬 중 13개 미사용
7. 회고 → 규칙 반영 루프 부재

## 확정 작업 목록

### A. Hook 기반 규칙 강제 (최우선)

세션 히스토리 분석 결과, 문서로만 강제한 규칙의 위반율:

| 규칙 | 위반 | 문서에 명시 | hook 존재 |
|------|------|:---------:|:--------:|
| 커밋 전 테스트 실행 | **26/43건 (60%) 미실행** | O (3곳) | X |
| main 직접 커밋 금지 | **43/43건 (100%) main 커밋** | O (3곳) | X |
| git add -A 금지 | **10건 사용** | X | X |
| Co-Authored-By 금지 | 0건 (지켜짐) | O (3곳) | X |
| git push 금지 | 0건 (지켜짐) | O (3곳) | X |

| # | 작업 | 유형 | hook 종류 | 우선순위 | 상태 |
|---|------|------|----------|---------|------|
| 1 | **커밋 전 테스트 강제** — `cargo test` + `npm run test` 통과 필수 | Git hook | `pre-commit` | P0 | 완료 |
| 2 | **main 직접 커밋 차단** — main 브랜치면 커밋 거부 | Git hook | `pre-commit` | P0 | 완료 |
| 3 | **위험 명령 deny 패턴** — settings.json deny로 기계적 차단 | settings.json | `permissions.deny` | P0 | 완료 |
| | — `git push*`, `git add -A*`, `git add .`, `rm -rf*`, `git reset --hard*`, `Edit(.env*)` | | | | |
| 4 | **Co-Authored-By 차단** — 커밋 메시지에서 패턴 감지 시 거부 | Git hook | `commit-msg` | P1 | 완료 |
| 5 | **커밋 메시지 형식 검증** — `tag: 한글제목` 패턴 필수 | Git hook | `commit-msg` | P1 | 완료 |
| 6 | **Stop hook 완료 검증 게이트** — Claude "완료" 전 린트/테스트 자동 확인 | Claude Code hook | `Stop` | P1 | 완료 |
| 7 | **PreCompact hook 대화 백업** — 압축 직전 전체 대화를 파일로 자동 백업 | Claude Code hook | `PreCompact` | P2 | 완료 |

### B. 컨텍스트 관리

| # | 작업 | 유형 | 상태 |
|---|------|------|------|
| 8 | settings.json에 compactPrompt 추가 (중간형: 규칙 보존 + 태스크 진행상황 + 파일 경로 + 사용자 피드백) | 설정 | 완료 |
| 9 | `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=70` 설정 — 1M 모델에서 조기 압축으로 품질 유지 | 환경변수 | 완료 |
| 10 | CLAUDE.md 중복 제거 + hook 강제 섹션 한줄 교체 (~135줄) | 수정 | 완료 |
| 11 | 금지규칙 CLAUDE.md 한줄화 + CODEX_RULES에서 중복 제거 | 수정 | 완료 |

### C. MCP 정리

| # | 작업 | 유형 | 상태 |
|---|------|------|------|
| 12 | mcp_integration.md 슬림화 (213줄 → ~60줄: 안전/비용 제한 + 핵심 체인 3개) | 수정 | 완료 |
| 13 | 미설치 MCP 5개(D2, Perplexity, Morph, Magic, Paper Search) 스킬 allowed-tools에서 제거 | 수정 | 완료 |
| 14 | ~~GitHub MCP 설치~~ | 설정 | 취소 (사용자 결정: 연결 안함) |

### D. 스킬 정리

| # | 작업 | 유형 | 상태 |
|---|------|------|------|
| 15 | 미사용 스킬 3개 삭제 (weekly-report, architecture, deploy-verify) | 삭제 | 완료 |
| 16 | progress-cleanup 스킬 프롬프트 보완 (실제 패턴: history/{마일스톤}/ 아카이빙 + INDEX.md 갱신) | 수정 | 완료 |
| 17 | /daily-retro 스킬에 "설정 반영 체크" 단계 추가 (회고 → 규칙/hook/스킬 반영 루프) | 수정 | 완료 |

## 인터뷰 결정 로그

| Q | 항목 | 결정 | 근거 |
|---|------|------|------|
| Q1 | CLAUDE.md 리팩토링 | 3계층 강제 체계 (hook + compactPrompt + CLAUDE.md 경량화) | 리서치: "CLAUDE.md에 말하고 settings/hooks에서 강제하라" |
| Q2 | MCP 전략 | Tier 1/2 분류 + 미설치 삭제 + GitHub MCP 추가 | 실사용 데이터 기반 (DevTools 143회, Playwright 112회, Supabase 40회) |
| Q3 | 9단계 워크플로우 | 현행 유지 | 사용자: "익숙해지고 있어" |
| Q4 | 인터뷰 프로세스 중복 | 현행 유지 | 각 스킬 독립 동작이 장점 |
| Q5 | session_scope / 언어별 스코프 | 전부 유지 | 향후 활용 가능성 |
| Q6 | 미사용 스킬 | 3개 삭제 + progress-cleanup 보완 | 호출 0회 + 스킬은 호출 시만 로드되므로 컨텍스트 비용 없음 |
| Q7 | 금지규칙 반복 | hook 강제 + CLAUDE.md 한줄 | compactPrompt로 생존율 확보 → 반복 불필요 |
| Q8 | mcp_integration.md | 안전/비용 제한 + 핵심 체인 3개 (~60줄) | 리서치: 우선순위 체인은 tool description이 지배, 규칙 비효과적 |
| Q9 | compactPrompt | 중간형 | 규칙 + 태스크 진행상황 + 파일 경로 + 사용자 피드백 보존 |
| Q10 | 커밋/브랜치 강제 | Git hooks + Claude Code hooks | 기계적 차단이 문서 강제보다 확실 |

## 리서치 인사이트

### 컨텍스트 관리
- CLAUDE.md는 압축 시 "제안" 수준으로 약화됨
- compactPrompt 설정으로 규칙 원문 보존 가능
- 500줄 초과 시 준수율 92% → 71% 급락

### MCP 도구 선택
- Claude는 tool description의 의미적 유사도로 도구 선택 → 규칙 문서의 우선순위 체인 비효과적
- 도구 30개 이하에서 선택 정확도 90%+, 100개+ 에서 13.6%
- Tool Search(deferred loading)가 토큰 95% 절감

### 규칙 강제
- "CLAUDE.md에 말하고, settings/hooks에서 강제하라"
- 5계층: settings.json → Hooks → CLAUDE.md → MEMORY.md → Skills/Agents

### 세션 히스토리 실증 분석
- 전 세션 43건 커밋 분석: 60%가 테스트 없이 커밋, 100%가 main 직접 커밋
- git add -A 10건 사용 (개별 파일 추가 35건 대비)
- Co-Authored-By, git push는 실제 위반 0건 → hook 우선순위 낮음
- **문서로만 강제한 규칙은 위반율이 높고, 실제 위반 0건인 규칙에 문서 반복이 집중된 역전 현상**

### MCP 실사용 데이터 (전 세션 합산)
- Chrome DevTools 143회, Playwright 112회, Supabase 40회 (Top 3)
- Mermaid 4회, Context7 4회, Firecrawl 2회
- Codex, Tavily, Serena, Sequential Thinking, Memory, Exa, arXiv: 전부 0회
- 추천 추가: GitHub MCP (PR/이슈 자동화)

### 사용자 피드백
1. 회고 → 규칙 반영 루프 부재 → /daily-retro에 설정 반영 체크 단계 추가로 해결
2. 회고 문서 품질은 매우 높음 — "의사결정 + 이유 + 인사이트" 구조 유지
3. 규칙을 문서로만 강제하지 말 것 → hook/settings 승격이 핵심 개선

### 고급 최적화 기법 (심층 리서치)
- **permissions.deny**: hook 없이 settings.json 한줄로 위험 명령 차단. hook보다 간단하고 우선순위 최상위
- **Stop hook**: Claude가 "완료"하기 전 린트/테스트 자동 검증 게이트. exit 2 반환 시 강제 속행
- **PreCompact hook**: 압축 직전 전체 대화 백업. 장시간 세션에서 "뭘 잊었는지" 추적 가능
- **CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=70**: 1M 모델에서도 일찍 압축하여 품질 유지 (기본 95%는 너무 늦음)
- **5계층 강제 체계**: settings.json(deny) → Hooks(PreToolUse/Stop/PreCompact) → CLAUDE.md → MEMORY.md → Skills
- **PreToolUse updatedInput**: 도구 입력을 런타임에 투명 교체 가능 (커밋 메시지 자동 수정 등)
- **SessionStart hook**: 세션 시작 시 최근 커밋/상태를 자동 주입
