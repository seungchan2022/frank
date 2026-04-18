# KPI 선언 템플릿 (2층 구조)

본 저장소는 KPI를 **마일스톤 단위**(각 M{X}) + **MVP 최종**(전체 통합) 두 층위로 선언한다.

## 구조 개요

```
progress/mvp{N}/
├─ _vision.md          # 비전
├─ _discovery.md       # 발견·브레인스토밍
├─ _roadmap.md         # 로드맵 + ★ MVP 최종 KPI (## KPI 섹션)
├─ M1_{이름}.md        # 마일스톤 M1 + ★ 마일스톤 KPI (## KPI 섹션)
├─ M2_{이름}.md        # 마일스톤 M2 + ★ 마일스톤 KPI
└─ ...
```

- **마일스톤 KPI**: 각 `M{X}_*.md`에 `## KPI` 섹션 — 해당 마일스톤 DoD 중심
- **MVP 최종 KPI**: `_roadmap.md`에 `## KPI` 섹션 — MVP 전체 통합 품질 (전 플랫폼 커버리지, 회고 존재 등)

## 상태 파일 2개

```
progress/active_mvp.txt          — 예: 11:in-progress
progress/active_milestone.txt    — 예: M2:in-progress
```

각 포맷: `{ID}:{state}` 또는 두 줄. 상태: `planning | in-progress | completing | done`

## 검증 대상 결정 규칙 (scripts/kpi-check.sh)

1. MVP 상태 = `completing` → **MVP 최종 KPI** 검증 (`_roadmap.md`)
2. 그 외 → **현재 활성 마일스톤 KPI** 검증 (`M{X}_*.md`)
3. 마일스톤 문서 없으면 → MVP 기획 문서 폴백

## 상태별 Hard 게이트 적용

| 상태 | 점진 지표(커버리지·회고·성능·manual) | 회귀형 지표(테스트 통과 등) |
|---|---|---|
| planning | Soft 강등 | Soft 강등 |
| in-progress | Soft 강등 | Hard 유지 |
| completing | 선언 그대로 | 선언 그대로 |
| done | 게이트 없음 | 게이트 없음 |

측정 불가(`-`/`missing`)는 `completing` 제외 자동 Soft 강등.

---

## 템플릿 1 — 마일스톤 KPI (M{X}_*.md에 삽입)

```markdown
## KPI (M{X})

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| M{X} DoD 테스트 통과 | cargo test + vitest + xcodebuild test | 전체 통과 | Hard | — |
| 신규 엔드포인트 응답 | manual 또는 curl | ≤500ms | Soft | — |
| 관련 버그 0건 | 수동 QA | 0건 | Hard | — |
```

- 마일스톤마다 **DoD에 직결되는 지표 2~5개**만 선언
- 범위가 좁아야 completing 전이 시 빠르게 통과 가능
- 해당 마일스톤과 무관한 지표(전 플랫폼 통합 커버리지 등)는 **MVP 최종 KPI**로 옮김

---

## 템플릿 2 — MVP 최종 KPI (_roadmap.md에 삽입)

```markdown
## KPI (MVP{N} 최종)

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| 서버 테스트 커버리지 | cargo-tarpaulin | ≥90% | Hard | MVP{N-1} 91% |
| 웹 테스트 커버리지 | vitest --coverage | ≥90% | Hard | MVP{N-1} 99% |
| iOS 테스트 커버리지 | xcodebuild + xccov | ≥85% | Soft | MVP{N-1} 82% |
| E2E 수집 성공률 | 실검증 로그 | ≥85% | Hard | MVP{N-1} 83% |
| MVP 회고 작성 | history/mvp{N}/retro.md 존재 | exists | Hard | — |
| 기술부채 증감 | progress/debt.md 카운트 | net 감소 | Soft | MVP{N-1} N건 |
```

- **MVP 전체 통합 품질** 지표만 모음
- 마지막 마일스톤이 done이 된 후 `/milestone-review`가 MVP:completing 전이 제안
- 이 시점에 이 표가 엄격 검증됨

---

## 필드 규칙

- **지표**: 명확한 이름
- **측정 방법**: 실제 측정 커맨드 또는 경로
- **목표**: `≥N%`, `≤N.Ns`, `N+`, `exists` 등 파싱 가능 포맷
- **게이트**: `Hard` (미달 시 커밋 차단) / `Soft` (경고만)
- **기준선**: 이전 MVP 또는 이전 마일스톤 값

## 파서가 인식하는 지표 키워드

| 키워드 포함 | 매핑 |
|---|---|
| 서버 테스트 커버리지 / rust coverage | `server/target/tarpaulin/cobertura.xml` |
| 웹 테스트 커버리지 / web coverage | `web/coverage/coverage-summary.json` |
| iOS 테스트 커버리지 / ios coverage | `ios/Frank/coverage.txt` |
| 회고 작성 | `history/mvp{N}/*retro*` 파일 존재 |
| 기술부채 | `progress/debt.md` 카운트 |
| E2E, 수집 성공률, P50, 로딩, 성능, 응답 | **수동** — `progress/kpi/{YYMMDD}_manual.md` |
