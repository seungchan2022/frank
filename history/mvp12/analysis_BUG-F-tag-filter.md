# BUG-F 오답노트 태그 필터링 — 서버 변경(A) vs 클라이언트 필터링(B) 아키텍처 분석

> 생성: 2026-04-27  
> 유형: architecture  
> 대상: BUG-F (웹 + iOS 오답노트 태그 필터 미작동)

---

## 1. 버그 현황 (BUG-F)

`progress/mvp12_discovery.md` 기준:

- **증상**: 오답노트 탭에 스크랩 탭과 동일한 태그 필터 칩이 표시되지만, 칩을 눌러도 필터링이 실제로 동작하지 않음
- **원인 추정**: 오답노트 태그 필터가 스크랩의 상태를 공유하거나, 필터 핸들러가 오답노트 데이터에 연결 안 됨
- **심각도**: High (웹 + iOS 공통)

### 실제 코드 분석

**웹** (`web/src/routes/favorites/+page.svelte`):

```typescript
// line 154-155
const wrongAnswerTagMap = $derived(buildWrongAnswerTagMap(favoritesStore.favorites));
const filteredWrongAnswers = $derived(filterWrongAnswers(wrongAnswers, wrongAnswerTagMap, selectedTagId));
```

```typescript
// web/src/lib/utils/favorites-filter.ts
export function filterWrongAnswers(
    wrongAnswers: WrongAnswer[],
    tagMap: Record<string, string>,  // { articleUrl → tagId }
    selectedTagId: string | null
): WrongAnswer[] {
    if (!selectedTagId) return wrongAnswers;
    return wrongAnswers.filter((wa) => {
        const tagId = tagMap[wa.articleUrl];
        return tagId === undefined || tagId === selectedTagId;
    });
}
```

**구조적 분석**: `buildWrongAnswerTagMap`은 `favoritesStore.favorites`에서 `{ url → tagId }` 맵을 생성한다. `filterWrongAnswers`는 이 맵으로 `wa.articleUrl`을 조회해 필터한다. 로직 자체는 **이미 구현되어 있고 이론상 올바르다**.

**BUG-F의 실제 원인**: 증상 "필터가 동작하지 않음"은 두 가지 케이스로 나뉜다:

1. **tagMap 미스**: 오답의 `articleUrl`이 favorites에 없거나, favorites에 `tagId`가 null인 경우 → `tagMap[wa.articleUrl] === undefined` → 필터 통과 → 전체 표시 (설계 의도와 맞지 않음)
2. **iOS**: `FavoritesView.swift`의 ST-3_4 subtask가 구현 중이거나 미완료 상태로, `filteredWrongAnswers` computed 프로퍼티 연결이 빠진 것으로 추정

현재 웹 코드에서 `filterWrongAnswers`의 `tagId === undefined` 분기는 "즐겨찾기에 없는 오답은 어떤 태그 선택에도 항상 표시"하는 **의도적 설계**(ST-3_4 문서 명시)이지만, 사용자 관점에서는 "필터가 안 됨"으로 보인다.

---

## 2. 두 방식 비교

### 방식 A: 서버 사이드 필터링

**개요**: `QuizWrongAnswer` 테이블에 `tag_id` 컬럼을 추가하고, `GET /me/quiz/wrong-answers?tag_id=XXX` 파라미터로 서버에서 필터링.

**필요한 변경 범위**:

| 레이어 | 변경 항목 |
|---|---|
| DB 스키마 | `quiz_wrong_answers` 테이블에 `tag_id UUID REFERENCES tags(id)` 추가 (마이그레이션) |
| Domain model | `QuizWrongAnswer` + `SaveWrongAnswerParams`에 `tag_id: Option<Uuid>` 추가 |
| Port | `QuizWrongAnswerPort::list()` 시그니처 변경 → `tag_id: Option<Uuid>` 파라미터 |
| Infra | `PostgresQuizWrongAnswerAdapter::list()` SQL WHERE 절 수정 |
| API handler | `list_wrong_answers` 쿼리 파라미터 파싱 추가 |
| Web SvelteKit proxy | `/api/quiz/wrong-answers` 프록시에 tag_id 포워딩 |
| Web client | `listWrongAnswers(tagId?: string)` 시그니처 변경 |
| Web page | `$effect`에서 `selectedTagId` 변경 시 API 재호출 |
| iOS save flow | `SaveWrongAnswerBody`에 `tagId` 추가 (저장 시점에 현재 기사의 tagId 주입) |
| iOS list flow | API 호출 시 `tagId` 파라미터 전달 |
| Tests | Fake adapter + 핸들러 테스트 전면 수정 |

**핵심 문제점**:

1. **저장 시점 tag_id 공급 문제**: 오답을 저장하는 시점(`QuizModal`)에서 현재 기사의 `tag_id`를 알아야 한다. 웹 `feed/article/+page.svelte`, iOS `ArticleDetailView`에서 기사 context를 퀴즈 모달까지 전달해야 함.

2. **denormalization 정합성 붕괴**: 사용자가 즐겨찾기의 태그를 변경하면 `quiz_wrong_answers.tag_id`는 stale해진다. 두 테이블 간 동기화 메커니즘이 필요.

3. **의존 방향 위반 가능성**: Port 시그니처 변경 시 Domain(`ports.rs`) 수정이 필요하고, 이는 모든 어댑터(fake 포함)의 변경을 유발 → 변경 반경이 매우 넓음.

4. **규칙 위반**: `rules/0_CODEX_RULES.md §2.1` — "명시적 요청 없이 대규모 리팩터링 금지". 스키마 마이그레이션 + 4개 레이어 변경은 대규모 변경에 해당.

---

### 방식 B: 클라이언트 사이드 필터링 (현재 구조 유지/수정)

**개요**: 기존 `favorites` 테이블의 `tag_id`를 활용해 `articleUrl`을 브릿지로 오답 필터링. 이미 웹에 구현되어 있음.

**현재 웹 상태**: `favorites-filter.ts`의 `buildWrongAnswerTagMap` + `filterWrongAnswers`가 구현됨. `filteredWrongAnswers`가 `$derived`로 연결되고, 오답 탭에 태그 칩 UI도 있음.

**BUG-F 수정에 필요한 실제 변경**:

웹의 실제 버그 원인을 정확히 진단해야 한다. 현재 코드 분석상 두 가지 후보:

**후보 1 — UI 연결 누락**: 오답 탭의 태그 칩 `onclick`이 `selectedTagId`를 올바르게 변경하는지 확인 (현재 코드 line 391-407을 보면 연결되어 있음)

**후보 2 — tagMap 부재**: `wrongAnswerTagMap`이 올바르게 구성되는지. 오답의 기사 URL이 favorites에 없거나 tagId가 null이면 필터가 통과 → 전체 표시. 이는 "필터가 안 됨"처럼 보임.

**후보 2가 핵심**이라면 수정 방향:
```typescript
// favorites-filter.ts 수정안
export function filterWrongAnswers(
    wrongAnswers: WrongAnswer[],
    tagMap: Record<string, string>,
    selectedTagId: string | null
): WrongAnswer[] {
    if (!selectedTagId) return wrongAnswers;
    return wrongAnswers.filter((wa) => {
        const tagId = tagMap[wa.articleUrl];
        // tagId가 없는 오답(즐겨찾기에 없는 기사)은 태그 선택 시 제외
        return tagId === selectedTagId;
    });
}
```

iOS 수정 (ST-3_4 subtask 완료):
- `filteredWrongAnswers` computed 프로퍼티 구현 확인
- `wrongAnswerTagMap`에서 tagId 없는 오답 처리 정책 일치

---

## 3. 포트/어댑터 패턴 + 의존 방향 기준 평가

프로젝트 의존 방향: `api → services → domain(ports) ← infra(adapters)`

| 기준 | 방식 A | 방식 B |
|---|---|---|
| 의존 방향 준수 | 유지됨 (단, Port 시그니처 변경으로 전 레이어 영향) | 유지됨 (서버 변경 없음) |
| 포트 추상화 활용 | Domain 모델/Port 변경 필요 | Port 변경 없음 |
| 변경 범위 최소화 원칙 | 위반 (DB + 5개 레이어) | 준수 (Web 1-2개 파일, iOS 1개 파일) |
| 정합성 보장 | 스키마 FK로 강제 가능하나, favorites.tag_id 변경 시 stale 문제 남음 | favorites → wrong-answers 브릿지, 정합성은 favorites에 의존 |
| 테스트 영향 | FakeQuizWrongAnswerAdapter 전면 수정 필요 | 기존 Fake 변경 없음, 순수 함수 단위 테스트 추가만 |

**아키텍처 원칙 적합성**:

방식 A는 "오답 엔티티가 자신의 태그 정보를 직접 보유"하는 정규화된 설계이나, 이 프로젝트에서 `WrongAnswer`는 기사에 종속된 부수 데이터다. 기사의 태그는 `favorites.tag_id`에 있고, 오답은 `articleUrl`로 기사를 참조한다. 오답 테이블에 tag_id를 중복 저장하면 favorites 엔티티와의 결합을 높이고 denormalization 부채를 만든다.

방식 B는 "표현 로직(뷰 레벨 computed)"으로 허용하는 계층 분리를 따른다. ST-3_4 아키텍처 주석이 이를 명시적으로 허용하고 있다. `rules/0_CODEX_RULES.md §2.1`의 변경 범위 최소화 원칙에도 부합한다.

---

## 4. 세 가지 개선안

### 개선안 A: 서버 사이드 tag_id 필터 (풀 마이그레이션)

**대상**: DB 스키마 + Domain + Port + Infra + API + Web + iOS 전 레이어 변경

**장점**:
- 서버가 단일 진실의 원천 (페이지네이션 도입 시 필수)
- 오답 데이터 자체에 태그 정보 내재화 → 독립 조회 가능
- 플랫폼 간 필터 로직 중복 제거

**단점**:
- 변경 범위 최대 (마이그레이션 + 6-8개 파일)
- `favorites.tag_id` 변경 시 `quiz_wrong_answers.tag_id` stale 문제 미해결
- 저장 시점에 tag_id context 주입 복잡도 증가
- iOS ST-3_4 subtask 재설계 필요
- `rules/0_CODEX_RULES.md §2.1` 대규모 리팩터링 금지 원칙과 충돌

**권장 시점**: 오답 데이터가 10,000건 이상 누적되어 클라이언트 전수 로드가 문제가 되는 시점, 또는 페이지네이션 도입 마일스톤에서 검토.

---

### 개선안 B-1: 클라이언트 필터 버그 수정 (최소 변경)

**대상**: `web/src/lib/utils/favorites-filter.ts` 1개 함수 + iOS `FavoritesView.swift` computed 로직

**장점**:
- 변경 범위 최소 (서버/DB 변경 없음)
- 기존 포트/어댑터 구조 유지
- ST-3_4 subtask 방향과 일치
- 즉시 수정 가능, 테스트도 순수 함수 단위 테스트로 검증 가능

**단점**:
- favorites에 없는 오답의 처리 정책을 명시적으로 결정해야 함 (현재 "항상 표시" → 변경 시 UX 영향)
- 오답이 즐겨찾기에서 삭제된 기사에 연결된 경우 필터 기준 없음

**수정 포인트**:

1. `filterWrongAnswers`에서 `tagId === undefined` 처리 정책 수정:
   - 현재: 즐겨찾기에 없는 오답은 항상 표시 (필터 미적용)
   - 변경: 선택된 태그에 해당하는 즐겨찾기가 있는 오답만 표시

2. iOS: `filteredWrongAnswers` computed 구현 확인 + `wrongAnswerTagMap` 연결 검증

**권장 시점**: MVP12 M2(웹) + M3(iOS) 즉시 적용. BUG-F의 가장 빠른 수정 경로.

---

### 개선안 B-2: 클라이언트 필터 + 오답 전용 filterTags 분리

**대상**: `favorites/+page.svelte` + `favorites-filter.ts`

**개요**: 현재 `filterTags`는 즐겨찾기 기사의 태그만 기반으로 구성된다. 오답 탭에서는 "오답이 실제로 존재하는 태그"만 칩으로 표시하도록 별도 computed 추가.

```typescript
// 추가할 computed
const wrongAnswerFilterTags = $derived(
    buildFilterTags(allTags, buildWrongAnswerTagIds(wrongAnswers, wrongAnswerTagMap))
);
```

**장점**:
- 오답이 없는 태그의 칩은 표시하지 않음 → 빈 필터 결과 방지
- BUG-F의 "칩 눌러도 아무것도 안 보임" 케이스 원천 차단
- 서버 변경 없음

**단점**:
- `filterTags`와 `wrongAnswerFilterTags` 2개 derived 관리
- `favorites-filter.ts`에 함수 추가 필요

**권장 시점**: B-1과 동시에 적용. 추가 복잡도가 낮고 UX 품질을 높임.

---

## 5. 최종 권장

**BUG-F 수정에는 방식 B-1 + B-2를 권장한다.**

근거:

1. **현재 코드 상태**: 웹은 `filterWrongAnswers` 로직이 이미 구현되어 있고 `$derived`로 연결됨. 실제 버그는 `tagId === undefined` 정책 문제일 가능성이 높다.

2. **방식 A의 비용 대비 효과**: DB 마이그레이션 + 4개 레이어 변경이 필요한 반면, 오답 데이터는 즐겨찾기의 기사 URL을 통해 이미 tag_id와 연결될 수 있다. denormalization이 favorites 변경 시 stale 문제를 만든다.

3. **규칙 준수**: `rules/0_CODEX_RULES.md §2.1` 변경 범위 최소화 원칙. 방식 A는 명시적 요청 없는 대규모 리팩터링에 해당.

4. **방식 A가 유리한 유일한 시나리오**: 오답이 즐겨찾기에서 삭제된 기사에 연결되어 태그 정보를 잃는 케이스, 또는 페이지네이션 도입 시. 현 MVP 단계에서는 해당 없음.

5. **iOS ST-3_4 alignment**: 기존 subtask가 방식 B 기반으로 설계됨. 방식 A로 전환 시 해당 subtask 전면 재설계 필요.

**방식 A를 고려할 시점**:
- 오답 목록에 페이지네이션 도입 필요 시 (서버 쿼리 파라미터 필수)
- 즐겨찾기 없이도 오답 태그 필터가 동작해야 하는 요구사항 발생 시

---

## 참조 파일

- `server/src/domain/ports.rs` — `QuizWrongAnswerPort` 정의
- `server/src/domain/models.rs` — `QuizWrongAnswer`, `Favorite` 모델
- `server/src/infra/postgres_quiz_wrong_answers.rs` — DB 어댑터
- `web/src/lib/utils/favorites-filter.ts` — 클라이언트 필터 유틸
- `web/src/routes/favorites/+page.svelte` — 태그 필터 UI + derived 연결
- `web/src/lib/types/quiz.ts` — `WrongAnswer` 타입 (tag_id 필드 없음)
- `history/mvp11/subtask/mvp11_m4/ST-3_4_wrong-answers-tag-filter.md` — iOS 설계 문서
- `progress/mvp12_discovery.md` — BUG-F 증상 기록
