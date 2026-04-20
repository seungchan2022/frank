# step-9 스킬에 Feature List 미체크 시 커밋 차단 로직 추가

> 상위: 260420_g1_feature_list_skill → **G1 사이클에서 분리됨 (2026-04-20)**
> 번호: 03
> 상태: **이관 대기 — 후속 §8.2 태스크에서 재착수 예정**
> 유형: 추가기능
>
> ---
>
> ## 🚨 이관 안내 (2026-04-20 step-5 리뷰 결정)
>
> step-5 critical-review에서 **G1(Feature List 생성)과 §8.2(미체크 커밋 차단)는 분석 문서상 별개 항목**으로 재확인 (근거: `progress/analysis/260417_하네스_비교분석.md` §7 실행 순서 `G1 → G6 → 8.2`).
>
> 또한 critical-review 치명 이슈 3건이 본 서브태스크에 집중:
> - **외부 우회**: 스킬 내부 검증만으로는 직접 `git commit` 차단 불가 → pre-commit hook 연계 필요
> - **거짓 체크 유도 위험**: 상태 모델 없이 차단하면 타입②(거짓말) 역설 조장
> - **§8.2와 범위 중복**: G1에 흡수 시 04가 "G1 ✅" 찍어도 실제 §8.2 미완료 상태 모순
>
> **결정 (사용자 Q1-B)**: G1 사이클은 01+02+04로 축소. 03은 이번 사이클에서 제외하고, G1 실측 피드백을 반영한 후 별도 §8.2 후속 태스크로 재착수.
>
> 본 문서는 **참조용으로 보존**. §8.2 태스크 착수 시 아래 설계 초안 재활용 가능.
>
> §8.2 재착수 시 반영 필수 항목:
> 1. **pre-commit hook 연계** (기존 `scripts/kpi-check.sh` 체계와 공존)
> 2. **차단 기준 재설계**: "모든 `[x]`" → "필수 카테고리 최소 충족 + `[~]`/`[-]` 사유 허용"
> 3. **bypass.log 파일 분리**: `progress/kpi/bypass.log` → `progress/feature-list/bypass.log`
> 4. **우회 플래그 변경**: 환경변수 → 명시적 CLI 플래그(`--skip-manual` 등) + 사유 필수
> 5. **4상태 모델 반영**: `[ ]`만 미완료 카운트, `[~]`/`[-]`는 사유 있으면 통과
> 6. **G1 실측 피드백 반영**: 실제 Feature List 사용 경험에서 발견된 패턴(거짓 체크 유도, 상태 전환 빈도, 파싱 실패 케이스) 반영
>
> ---
>
> **이하 섹션은 G1 사이클 원안. 현재는 비활성.**

---

## 스코프

### In-Scope
- `.claude/skills/step-9/SKILL.md` 수정 — 커밋 직전 Feature List 체크 상태 검증
- 미체크(`- [ ]`) 항목 1개 이상이면 커밋 차단 + 미체크 목록 출력
- `feat:`/`fix:`/`test:` 커밋에만 차단 적용 (`docs:`/`chore:`는 스킵 — 기존 kpi-check 정책과 정합)
- Feature List 섹션이 없거나 소형 스킵된 서브태스크는 정상 통과
- 우회: `FEATURE_LIST_BYPASS=1 /step-9` + `progress/kpi/bypass.log`에 `type: feature_list` 항목으로 사유 기록

### Out-of-Scope
- pre-commit hook 수정 (스킬 내부 검증만 우선, hook 통합은 후속)
- 자동 체크 (사용자가 step-8에서 직접 체크)
- Feature List 생성 로직 (01번)

---

## Definition of Done (DoD)

### 코드 품질
- [ ] `.claude/skills/step-9/SKILL.md` 마크다운 포맷 유효
- [ ] 기존 커밋 흐름 유지 (docs/chore 자동 스킵)

### 문서/프로세스
- [ ] 차단 조건 명시 (미체크 ≥ 1 + feat/fix/test)
- [ ] 우회 절차 명시
- [ ] 사용자 승인

---

## 상세 구현 명세

### 변경 파일
| 파일 | 변경 유형 | 설명 |
|------|----------|------|
| `.claude/skills/step-9/SKILL.md` | 수정 | Feature List 검증 + 차단 로직 추가 |

### 구현 상세
1. 커밋 직전 서브태스크 문서의 `## Feature List` 섹션 파싱
2. `- [ ]` 미체크 개수 집계
3. 커밋 태그 판정: `feat|fix|test` → 차단 대상, `docs|chore|style|refactor` → 스킵
4. **skip 메타 확인**: `<!-- skip: true -->` 시 차단 생략 (소형 스킵 모드)
5. 차단 시 출력 포맷 (인터뷰 Q2 A):
   ```
   ✗ Feature List 미체크 항목 N개 — 커밋 차단

   ### 기능 (3/10 미체크)
   - [ ] 항목1
   - [ ] 항목2
   - [ ] 항목3

   ### 엣지 (2/5 미체크)
   - [ ] 항목4
   - [ ] 항목5

   ### 에러 (1/4 미체크)
   - [ ] 항목6

   → /step-8로 돌아가 시나리오 재수행하거나
      FEATURE_LIST_BYPASS=1 + bypass.log 기록 후 진행
   ```
6. **우회 로그 통합** (인터뷰 Q1 B): `progress/kpi/bypass.log`에 append
   ```
   {YYYY-MM-DD HH:mm} | type: feature_list | subtask: {번호-제목} | unchecked: N | reason: {사유}
   ```

---

## 테스트 계획
| 테스트 | 설명 | 유형 |
|--------|------|------|
| 스킬 drive-through | 미체크 있는 서브태스크로 `/step-9` 실행 → 차단 확인 | 수동 |
| 태그 판정 | `feat/fix/test/docs/chore` 각각 커밋 시 차단/통과 확인 | 수동 |
| 우회 경로 | `FEATURE_LIST_BYPASS=1` 통과 + `progress/kpi/bypass.log`에 `type: feature_list` 항목 기록 확인 | 수동 |
| 스킵 예외 | Feature List 없는 서브태스크 정상 커밋 | 수동 |

---

## 리뷰 결과

### Claude 리뷰 (문서 일치성)
- **하네스 분석 범위 경계 위반**: 메인태스크는 "근거 문서 §5 G1·§7 우선순위표"를 선언하나 03의 "미체크 시 커밋 차단"은 §8.2 영역. 03을 G1에 포함할지 8.2 별도 태스크로 분리할지 의사결정 필요.
- **태그 판정 주체 미정**: `feat|fix|test` 판정은 커밋 메시지에서 파생되는데 step-9 스킬은 커밋 메시지 작성 직전 호출. 판정 시점·입력 경로 명시 없음.
- **pre-commit hook 미연계**: 스킬 내부 검증만으로는 워크플로우 외부 커밋(직접 `git commit`/다른 터미널/에이전트) 시 차단 무력화 → 실효성 있는 차단이라 부르기 어려움.

### Codex 리뷰 (기술적 타당성)
- ⚠️ **조건부 승인**
- 근거 문서상 이건 G1이 아니라 §8.2 항목. G1 내부에 8.2 흡수해 놓고 04는 8.2 손대지 않는다고 써서 문서 체계 충돌.
- `progress/kpi/bypass.log`는 KPI 전용 로그로 README에 명시됨 → `progress/feature-list/bypass.log`로 분리하거나 KPI README·스크립트 함께 SSOT 재작성 필요.
- `skip` 메타 해석이 01 규약과 다른 패턴.

### 구멍 찾기 리뷰 (critical-review)
- **치명 2건**:
  - **외부 우회 그대로 열림**: 스킬 미경유 시 무력화. 최소 `pre-commit`/`commit-msg` hook 연계 필수. `FEATURE_LIST_BYPASS` 환경변수 대신 명시적 CLI 플래그 + 로그 강제.
  - **03이 8.2 범위 선점 → 문서 모순**: G1 완료로 찍혀도 실제 8.2는 미완료 상태. 03을 G1에서 빼거나 분석 문서 8.2 정의를 "G1 흡수"로 재작성하거나 택일 필요.
- **중대 1건**: 커밋 차단이 거짓 체크 학습 유도 → 차단 기준을 "모든 `[x]`"가 아닌 "필수 카테고리 최소 충족 + 잔여는 deferred/NA 사유" 로 전환. bypass·manual defer를 KPI로 별도 추적.

### 최종 결정: ⚠️ **조건부 승인** — 재범위 결정 후 진행
선결 의사결정:
- **A안**: 03을 G1 내부 유지 → 분석 문서 §8.2 정의를 즉시 "G1에 흡수" 로 재작성 + 04 범위도 8.2 상태 문구 동기화로 확장
- **B안**: 03을 G1에서 분리 → 별도 8.2 태스크로 이관 (G1은 01+02+04로 축소, 커밋 차단은 후속)

반영 필수:
1. A/B 중 택1 후 메인태스크 서브태스크 표·근거 문서 섹션 업데이트
2. pre-commit hook 연계 In-Scope 포함 여부 결정
3. bypass.log 파일 분리 (`progress/feature-list/bypass.log`)
4. 차단 기준 재설계 ("필수 카테고리 최소 충족")
