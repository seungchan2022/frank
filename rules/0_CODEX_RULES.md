# CODEX 에이전트 룰북 (v1.0)

이 문서는 Codex가 이 저장소에서 작업할 때 따라야 하는 **강제 규칙(Hard Constraints)**과 **필수 워크플로우**를 정의한다.
본 룰은 코드의 정확성, 보안성, 테스트 가능성, 유지보수성을 보장하기 위해 존재한다.

---

## 1. 역할과 목표 (Role & Goal)

너는 **코딩 에이전트**로서, **최소한의 리스크와 높은 신뢰성**을 유지하며 변경을 수행해야 한다.

주요 목표:

* 올바른 동작 보장
* 회귀(regression) 발생 방지
* 테스트로 검증된 변경
* 기본적으로 안전한 구현(Secure-by-default)

---

## 2. 강제 규칙 (MUST / MUST NOT)

### 2.1 일반 규칙

* **MUST** 기존 프로젝트 구조와 패턴을 따른다.
* **MUST** 명시적인 요청이 없는 한, 변경 범위를 최소화하고 국소적으로 수정한다.
* **MUST NOT** 명시적 요청 없이 대규모 리팩터링을 수행한다.
* **MUST NOT** 사용되지 않는 코드, 파일, 의존성을 추가한다.
* **MUST** 프로젝트에서 정의한 코드 스타일과 아키텍처 패턴을 준수한다.
* **MUST** 로깅 시 민감정보 마스킹을 보장한다.
* **MUST** 에러를 명시적으로 처리하고, 적절한 에러 타입/코드를 사용한다.
* **MUST** 외부 HTTP 호출 시 timeout/retry/크기 제한을 적용한다.

### 2.2 테스트 및 완료 조건 (TDD)

* **MUST** 동작을 추가하거나 수정할 때 테스트 우선(TDD) 방식으로 작업한다.
* **MUST** 구현 전에 실패하는 테스트를 추가하거나 수정하고, 구현 후 테스트가 통과해야 한다.
* **MUST NOT** 모든 관련 테스트가 통과되지 않은 상태에서 작업을 완료로 표시한다.
* **MUST** 실행한 테스트 명령어와 결과를 보고한다.
* **MUST** 프로젝트에서 정의한 테스트 커버리지 기준을 따른다.

#### 스모크 테스트 (Smoke Tests)

* **SHOULD** 다음에 영향을 주는 변경의 경우 기본적인 스모크 테스트를 추가한다:

  * 애플리케이션 기동
  * 핵심 API 엔드포인트
  * 주요 사용자 플로우

스모크 테스트란 다음을 최소한으로 검증하는 것이다:

* 애플리케이션이 오류 없이 시작되는지
* 기본 엔드포인트 또는 헬스 체크가 정상 응답하는지

---

## 3. 필수 워크플로우 (Required Workflow — 단일 소스)

비단순 변경(non-trivial change)은 다음 **5단계**를 반드시 따른다. 이 5단계가 본 저장소의 **유일한 강제 워크플로우**이며, `rules/sub/workflow.md`는 각 단계 내부에서 참고하는 사고 가이드이지 별도의 단계 시스템이 아니다.

1. **Inspect (분석)** — 관련 코드·테스트·문서를 파악하고 문제를 한 문장으로 정리한다.
2. **Specify (테스트 우선 정의)** — 기대 동작을 설명하는 실패하는 테스트를 식별·생성한다.
3. **Implement (구현)** — 테스트를 통과시키는 최소한의 코드를 작성한다.
4. **Verify (검증)** — 관련 테스트 + 필요 시 스모크 테스트를 실행한다. **active MVP의 Hard KPI 지표가 있다면 해당 지표도 같이 확인한다.**
5. **Report (보고)** — 변경 요약, 실행 명령, 결과, 남은 리스크를 보고한다.

세부 사고 가이드는 `rules/sub/workflow.md`를 참조한다 (자동화 우선 원칙, 리뷰 체크리스트, 중단 조건 등).

---

## 3.5 2층 KPI 게이트 (마일스톤 + MVP 최종)

본 저장소는 KPI를 **두 층위**로 관리한다:
- **마일스톤 KPI**: 각 `M{X}_*.md`에 `## KPI` — 해당 마일스톤 DoD 중심
- **MVP 최종 KPI**: `_roadmap.md`에 `## KPI` — MVP 전체 통합 품질

### 3.5.1 상태 파일 2개

| 파일 | 포맷 | 예 |
|---|---|---|
| `progress/active_mvp.txt` | `{N}:{state}` | `11:in-progress` |
| `progress/active_milestone.txt` | `M{X}:{state}` 또는 `none` | `M2:in-progress` |

상태: `planning | in-progress | completing | done`

### 3.5.2 검증 대상 결정 (scripts/kpi-check.sh)

1. `MVP:completing` → **MVP 최종 KPI** 검증 (`_roadmap.md`)
2. 그 외 → **현재 활성 마일스톤 KPI** 검증 (`M{X}_*.md`)
3. 마일스톤 문서 없으면 → MVP 기획 문서 폴백

### 3.5.3 상태별 Hard 게이트 적용

| 상태 | 점진 지표(커버리지·회고·debt·성능·manual) | 회귀형 지표(테스트·E2E) |
|---|---|---|
| `planning` | Soft 강등 | Soft 강등 |
| `in-progress` | Soft 강등 | Hard 유지 |
| `completing` | 선언 그대로 | 선언 그대로 |
| `done` | 게이트 없음 | 게이트 없음 |

측정값이 `-`/`missing`이고 상태가 `completing`이 **아닌** 경우 자동 Soft 강등 (캐시 부재 함정 방지).

### 3.5.4 전이 규칙 (실사용 흐름 기반)

전이 주체는 **`/workflow` 자동 처리** + **MVP 사이클 경계의 수동 확인** 두 가지.

| 이벤트 | 주체 | 변경 |
|---|---|---|
| `/milestone` 로드맵 확정 | `/milestone` 스킬 | MVP `planning → in-progress`, `M1:planning` 생성, 각 M·로드맵에 KPI 자동 삽입 |
| `/workflow "M{X}-..."` 호출 | `/workflow` 스킬 [0]단계 | 이전 `M{Y}:in-progress`가 있으면 사용자에게 수동 E2E 통과 확인 → `y` 시 자동 `M{Y}:done`. 새 `M{X}:in-progress`로 전이 |
| `/workflow` step-9 커밋 성공 | `/workflow` 스킬 | `_roadmap.md` 기준 모든 M이 done이면 MVP 완료 프로세스 제안 (`MVP:completing`으로 전이할지 확인) |
| MVP 완료 승인 | `/workflow` 또는 수동 | `MVP:completing` → MVP 최종 KPI 엄격 검증 → Pass 시 `MVP:done` → `history/mvp{N}/` 이동 → `active_mvp.txt = {N+1}:planning`, `active_milestone.txt = none` |
| 다음 MVP 방향성 | `/milestone-review` | MVP 사이클 경계에서 한 번 호출. 새 `/milestone` 전 회고·부채·다음 방향성 정리 |

**핵심**: 사용자는 **마일스톤 단위 작업 → 다음 `/workflow` 호출 → MVP 완료 커밋** 3가지만 기억하면 된다. 상태 전이는 시스템이 자동 처리하거나 명시적 확인만 요청한다.

### 3.5.5 강제 사항

* **MUST** 각 `M{X}_*.md`에 해당 마일스톤 `## KPI` 섹션, `_roadmap.md`에 MVP 최종 `## KPI` 섹션 포함
* **MUST NOT** 적용 중인 Hard 지표 미달 상태로 `feat`/`fix`/`test` 커밋을 하지 않는다 (pre-commit `scripts/kpi-check.sh`가 차단)
* **MUST** 상태 전이는 `/milestone`·`/milestone-review`·`/workflow`에서 명시적으로 수행
* **우회**: `KPI_BYPASS=1 git commit ...` + `progress/kpi/bypass.log`에 사유 필수 기록
* 문서 전용 커밋(`docs:`/`chore:`)은 자동 스킵
* Soft 지표는 경고만 출력

---

## 4. 보안 규칙 (강제)

### 4.1 비밀 정보 및 민감 데이터

* **MUST NOT** 비밀 정보(API 키, 토큰, 비밀번호, 개인 키, DB URL)를 코드, 테스트, 로그, 출력에 하드코딩하거나 노출한다.
* **MUST** 예제 및 테스트에서는 명확한 더미 값만 사용한다 (예: `test_token`, `example.invalid`).
* **MUST NOT** `.env`, 자격 증명 파일, 개인 키, 시크릿 파일을 코드나 출력에 포함한다.

### 4.2 인증 및 인가(Authentication / Authorization)

* **MUST NOT** 인증/인가를 약화시키거나 우회한다.
* **MUST** 인증/인가 변경 시, 권한 없음/금지 상태를 검증하는 부정 테스트를 포함한다.
* **MUST NOT** 임시 인증 우회나 디버그 플래그를 코드에 남긴다.

### 4.3 입력 검증 및 오류 처리

* **MUST** 모든 외부 입력(요청 바디, 쿼리, 헤더, 환경 변수)을 검증한다.
* **MUST NOT** 응답이나 로그에 내부 정보(스택 트레이스, SQL, 파일 경로, 시크릿)를 노출한다.
* **MUST** 오류를 명시적으로 처리하고, 클라이언트에는 최소한의 안전한 메시지만 반환한다.

### 4.4 위험한 작업

* **MUST NOT** 사용자 입력을 기반으로 OS 명령을 실행한다.
* **MUST** 파일 처리 시 경로 탐색 공격을 방지한다 (`..` 사용 금지, 절대 경로 금지).
* **MUST NOT** 명확한 SSRF 방어(allowlist) 없이 URL fetch/프록시 기능을 구현한다.

### 4.5 의존성 및 공급망

* **MUST NOT** 필요하지 않은 새로운 의존성을 추가한다.
* **MUST** 새로운 의존성 추가 시, 최종 보고서에 목적과 대안을 간단히 설명한다.
* **MUST NOT** 의존성 변경 목적 없이 lockfile을 수정한다.

---

## 5. 도구 및 명령어 (Tooling & Commands)

* 프로젝트에서 정의한 표준 명령어를 사용한다:

  * 테스트: `CLAUDE.md`에 정의된 테스트 명령
  * 린트: `CLAUDE.md`에 정의된 린트 명령
  * 타입체크: `CLAUDE.md`에 정의된 타입체크 명령
  * 빌드/실행: `CLAUDE.md`에 정의된 빌드/실행 명령

확실하지 않은 경우 `CLAUDE.md`, `README`, `Makefile`, `pyproject.toml`, `package.json`, CI 설정을 먼저 확인한다.

---

## 6. 출력 요구사항 (Mandatory)

모든 작업 완료 시 반드시 다음 내용을 포함한 최종 보고서를 제공해야 한다:

1. **변경 요약**

   * 무엇을, 왜 변경했는지 (1~3문장)
2. **수정된 파일 목록**

   * 파일 경로와 간단한 설명
3. **테스트**

   * 추가/수정된 테스트
   * 실행한 테스트 명령어
   * 결과 (성공/실패)
4. **스모크 테스트**

   * 수행 여부: 예/아니오
   * 수행하지 않은 경우 사유
5. **리스크 / 후속 작업**

   * 알려진 한계나 다음 단계 제안

---

## 7. 서브 룰북 (Sub Rulebooks)

다음 문서들은 본 룰의 세부 실행 기준을 담은 **서브 룰북**이다:

* `rules/sub/agents.md` — 에이전트 역할 + **MCP 서버 정책·안전·비용·우선순위 체인** (이전 `mcp_integration.md` 통합)
* `rules/sub/documentation.md` — progress/ 문서 구조 및 서브태스크 문서 형식
* `rules/sub/git.md` — Git 커밋 형식, 푸시 금지, Co-Authored-By 태그 금지
* `rules/sub/session_scope.md` — 멀티세션 스코프 격리
* `rules/sub/sub_agent_usage.md` — 서브에이전트 사용 규칙
* `rules/sub/workflow.md` — 개발 사고 가이드 (§3의 5단계 내부에서 참고)
* `rules/sub/milestone.md` — 마일스톤 플로우 규칙 (Discovery-First, 상태 전이, KPI 선언)

인덱스: `rules/sub/INDEX.md`

---

## 8. 예외 사항 (Exceptions)

다음 경우에 한해 TDD 및 테스트 요구사항을 생략할 수 있다:

* 문서만 수정하는 경우
* 포맷팅 또는 주석만 변경하는 경우

이 경우에도 **생략 사유를 반드시 최종 보고서에 명시해야 한다.**

---

## 9. 원칙 (Principle)

이 룰들은 단순한 문서가 아니다.
이들은 **행동을 제한하는 규칙**이다.

의심스러울 때는 항상 **보안, 테스트, 최소 변경**을 우선한다.
