---
name: daily-retro
description: "하루 회고 HTML 자동 생성. 일기 스타일로 의사결정의 '왜' + 인사이트 위주 정리. 트리거 키워드: 회고, 일일회고, daily retro, 오늘 회고."
context: fork
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - Write
  - Edit
  - Agent
  - mcp__sequential-thinking__sequentialthinking
  - mcp__mermaid__mermaid-mcp-app
  - mcp__gamma__*
  - mcp__playwright__browser_navigate
  - mcp__playwright__browser_take_screenshot
  - mcp__playwright__browser_close
---

# 하루 회고 PDF 생성 (/daily-retro)

> `/daily-retro` 또는 `/daily-retro 260404`로 호출.
> 인자 없으면 오늘 날짜 기준.

## 대상 독자

개인 기록 + 멘토/면접관에게 보여줄 수 있는 수준.

## 스타일

- **일기 스타일** — 딱딱하지 않게, 설명하듯 자연스럽게
- **핵심: 의사결정의 "왜"** — 왜 이것을 선택했는지, 다른 선택지와 비교해서 어떤 근거로 결정했는지
- **인사이트/피드백 위주** — 단순 기록이 아니라, 결정에서 얻은 교훈과 시사점
- 감정/느낌도 포함 (힘들었던 점, 뿌듯했던 점 등)

## 수행 절차

### Phase 1: 데이터 수집 (병렬)

1. **Git 히스토리**: 해당 날짜의 커밋 로그 수집
   ```bash
   git log --after="{날짜} 00:00" --before="{날짜} 23:59" --oneline --all
   git diff {첫커밋}^..{마지막커밋} --stat
   ```

2. **Progress 문서**: 해당 날짜의 progress/ 파일 확인
   ```bash
   ls progress/{YYMMDD}_*
   ```

3. **토론/플랜 로그**: debate/, plans/ 문서 확인

4. **메모리**: 해당 날짜에 저장된 memory 파일 확인

### Phase 2: 인터뷰 (필수)

사용자에게 **1개씩** 질문한다:

1. **오늘 가장 중요한 결정은 뭐였나요?**
2. **어려웠거나 고민했던 부분이 있었나요?**
3. **내일은 뭘 할 예정인가요?**

답변이 짧아도 OK. "패스"도 허용.

### Phase 3: 회고 초안 작성

다음 구조로 작성한다:

```markdown
# 🗓️ {날짜} 개발 회고

## 오늘 뭘 했나

{한 일을 시간순/중요도순으로 자연스럽게 서술}

## 핵심 의사결정과 그 이유

{오늘의 주요 결정들을 각각 다음 구조로:}
### 결정 1: {제목}
- **상황**: 어떤 문제/선택지가 있었는지
- **선택지들**: 비교한 옵션들과 각각의 장단점
- **결정**: 무엇을 선택했는지
- **왜**: 그 결정의 핵심 근거
- **인사이트**: 이 결정에서 얻은 교훈/시사점
{인포그래픽/다이어그램 포함}

## 기획/설계 과정

{어떻게 아이디어를 좁혀나갔는지, 인터뷰/토론 과정}

## 인사이트 & 피드백

{오늘 얻은 핵심 교훈들}
{다음에 비슷한 상황이 오면 어떻게 할 것인지}
{멘토/면접관이 보면 인상 깊을 만한 사고 과정}

## 배운 것

{기술적으로 새로 알게 된 것, 프로세스에서 배운 것}

## 느낀 점

{솔직한 감정, 힘들었던 것, 재밌었던 것}

## 내일 할 일

{다음 단계 계획}
```

### Phase 4: 인포그래픽 생성

회고 내용에서 시각화할 수 있는 것을 Mermaid로 생성:
- 의사결정 흐름도
- 아키텍처 다이어그램
- 비교표
- 타임라인

### Phase 5: HTML 생성

1. **Claude Sunset 테마** HTML 파일 생성 (모바일 최적화, max-width: 480px)
2. 폰트: 시스템 기본 폰트
3. 다이어그램: Mermaid JS CDN 임베드 (`<pre class="mermaid">`)

**Claude Sunset 테마 색상:**
```css
--bg: #FDF6F0;           /* 따뜻한 크림 배경 */
--text: #2D2926;         /* 짙은 갈색 텍스트 */
--accent: #D97706;       /* 선셋 오렌지 강조 */
--accent-light: #FEF3C7; /* 연한 옐로우 하이라이트 */
--heading: #92400E;      /* 갈색 헤딩 */
--border: #E5D5C5;       /* 베이지 보더 */
--code-bg: #FFF7ED;      /* 코드 블록 배경 */
--card-bg: #FFFFFF;      /* 카드 배경 */
```

**Mermaid 임베드 방법:**
```html
<script type="module">
  import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs';
  mermaid.initialize({ startOnLoad: true, theme: 'base', themeVariables: { ... } });
</script>
<pre class="mermaid">graph TB ...</pre>
```

### Phase 6: 설정 반영 체크 (회고 → 규칙 루프)

회고에서 발견한 문제 중 **반복될 수 있는 것**을 식별하고, 설정 파일 수정을 제안한다.

1. 오늘 회고에서 발견한 문제/인사이트를 수집
2. 각 문제를 분류:
   | 문제 유형 | 반영 대상 | 예시 |
   |----------|----------|------|
   | 반복적 실수 | Git hook 또는 settings.json deny | "테스트 없이 커밋" → pre-commit hook |
   | 워크플로우 이탈 | 스킬 프롬프트 수정 | "step-7 건너뜀" → /next에서 경고 추가 |
   | 새로운 규칙 필요 | CLAUDE.md 또는 rules/ 추가 | "에러 메시지 노출" → 보안 규칙 추가 |
   | 도구 부족 | MCP 추가/설정 | "시각 검증 누락" → Playwright 활용 강화 |
3. 반영 항목을 **A/B/C 선택지**로 사용자에게 제안
4. 승인된 항목은 즉시 반영 (또는 다음 세션 TODO로 기록)

**핵심 원칙**: "같은 문제가 두 번 발생하면, 문서가 아니라 기계(hook/deny/스킬)로 막는다."

### Phase 7: 저장 및 보고

- 마크다운: `history/{현재마일스톤}/{YYMMDD}_daily_retro.md`
- HTML: `history/{현재마일스톤}/{YYMMDD}_daily_retro.html`
- 사용자에게 파일 경로 안내
- 설정 반영 항목이 있으면 함께 보고

## 출력 규칙

- **5페이지 이상** (상세하게)
- 인포그래픽 최소 1개 이상 포함
- 코드 블록은 최소화, 다이어그램으로 대체
- 한국어 작성
- 이모지 적절히 사용 (회고 문서이므로 허용)
