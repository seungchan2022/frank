# MVP1 2단계: 온보딩 (태그/키워드 등록) — 학습 노트

> 세션 시작: 2026-05-04
> 기준 흐름도: history/study/flow_onboarding.html
> 학습 목표: 전체 흐름 파악 + 흐름 속 개념 자연스럽게 익히기

---

## 온보딩이 뭐야?

로그인이 완전히 끝난 다음 단계야. 신규 유저가 처음 로그인했을 때 **딱 한 번만** 거치는 페이지야.

```
처음 가입한 사람 → 로그인 성공 → /onboarding (태그 고르는 화면) → /feed
이미 온보딩 마친 사람 → 로그인 성공 → 바로 /feed
```

온보딩에서 하는 일은 하나야. **"어떤 주제에 관심 있어요?"** 태그를 고르게 하는 것.
이 태그를 기반으로 피드에 뉴스를 추천해주니까, 로그인 직후 딱 한 번 취향을 수집하는 거야.

---

## 등장인물

| 이름 | 역할 |
|---|---|
| **사용자 (브라우저)** | 사용자가 보는 화면 |
| **+layout.server.ts** | 앱 전체에 공통으로 적용되는 서버 관문. 어느 페이지를 열든 먼저 실행됨 |
| **onboarding/+page.svelte** | 실제 온보딩 화면을 그리는 컴포넌트 |
| **auth.svelte.ts** | 로그인 상태를 앱 전체에서 공유하는 스토어 |
| **Rust API (8080)** | 태그 목록 조회, 태그 저장 같은 실제 기능을 처리하는 서버 |
| **PostgreSQL** | 태그 데이터, 유저-태그 관계를 저장하는 데이터베이스 |

> 💡 **SvelteKit이 뭐야?**
> 웹 프론트엔드를 만드는 프레임워크야. iOS로 치면 UIKit/SwiftUI 같은 거야.
> 화면을 어떻게 그릴지, 페이지 이동은 어떻게 할지, 서버 통신은 어떻게 할지를
> 정해진 방식으로 구현하게 해줘.

---

## 구간 [A] — 인증 가드 + 페이지 진입 (화살표 ①②③)

로그인을 마친 사용자가 `/onboarding` 페이지로 이동하는 순간이야. 사용자 눈에는 그냥 다음 화면이 뜨는 것처럼 보이지만, 화면이 그려지기 **전에** 서버에서 먼저 처리되는 단계가 있어.

---

**① GET /onboarding — 브라우저가 서버에 페이지를 요청한다**

사용자가 `/onboarding` 주소로 이동하면 브라우저가 서버에 "이 페이지 줘"라고 요청을 보내.

여기서 `/onboarding`은 **URL 경로(path)**야. 웹에서 "어느 페이지인지"를 문자열 주소로 표현하는 방식이야. 주소창에 `localhost:5173/onboarding`이라고 입력했을 때 뒤쪽 부분이 바로 경로야. iOS로 치면 `NavigationLink(destination: OnboardingView())`처럼 뷰를 지정하는 거랑 같은데, 웹은 그걸 문자열 주소로 하는 거야.

SvelteKit에서는 이 경로가 폴더 구조 그대로야.

```
src/routes/
├── onboarding/+page.svelte   →  /onboarding
├── feed/+page.svelte         →  /feed
├── login/+page.svelte        →  /login
```

`onboarding/+page.svelte`는 `/onboarding` 주소로 접속했을 때 브라우저에 그려지는 화면 코드 파일이야. iOS로 치면 `OnboardingView.swift`랑 정확히 대응돼. `.svelte` 파일 하나 안에 화면 로직 + UI 구조가 같이 들어있어.

요청할 때 로그인 때 저장해둔 **쿠키(토큰)**를 자동으로 함께 담아서 보내.

> 💡 **쿠키가 뭐야?**
> 브라우저가 특정 사이트 정보를 로컬에 저장해두는 작은 데이터야.
> 한 번 저장되면 그 사이트에 요청할 때마다 자동으로 같이 보내줘.
> iOS로 치면 Keychain에 저장해두고 매 요청마다 헤더에 붙여 보내는 것과 비슷해.
> 단, iOS는 코드가 명시적으로 꺼내서 붙이는 반면, 쿠키는 브라우저가 자동으로 해줘.

---

**② safeGetSession() — 서버가 "이 사람 로그인된 사람이야?" 확인한다**

요청이 서버에 도착하면 `+layout.server.ts`가 제일 먼저 실행돼.

> 💡 **+layout.server.ts가 뭐야?**
> SvelteKit은 파일 이름으로 역할이 정해지는 구조야.
> `+layout.server.ts`는 **앱 전체 페이지에 공통으로 적용되는 서버 코드**야.
> iOS로 치면 `AppDelegate`랑 비슷해.
> `/onboarding`을 열든 `/feed`를 열든 이 파일이 항상 먼저 실행돼.
> 이름에 `server`가 붙어있으면 브라우저가 아닌 **서버에서 실행**된다는 뜻이야.

여기서 `safeGetSession()`이 실행돼. ①에서 같이 온 쿠키 안의 토큰을 꺼내서 "이 토큰 진짜야?" 검증하는 거야.

"잠깐, 로그인할 때 이미 `safeGetSession()`으로 검증했잖아. 왜 또 해?" 싶을 수 있어. 이건 웹과 iOS의 구조 차이 때문이야. iOS 앱은 실행 중에 메모리에 로그인 상태가 살아있어. 한 번 인증하면 앱이 꺼지기 전까지 "로그인됨" 상태가 유지돼. 그런데 웹은 **매 페이지 요청마다 서버가 새로 확인해야 해.** 서버 입장에서는 `/onboarding`으로 요청이 들어왔을 때, 이 요청이 로그인한 사람이 보낸 건지 알 수가 없어. 그래서 매번 확인하는 거야.

```
/onboarding 요청 → safeGetSession() → 통과 → 화면 그려줌
/feed 요청       → safeGetSession() → 통과 → 화면 그려줌
/favorites 요청  → safeGetSession() → 통과 → 화면 그려줌
```

느리거나 비효율적으로 느껴질 수 있는데, 이 구조에는 이유가 있어. `safeGetSession()`은 사실 **두 단계**로 동작해:

1. `getSession()` — 쿠키 안의 토큰을 **로컬에서 디코드**. 네트워크 없이 즉시 처리.
2. `getUser()` — Supabase Auth 서버에 **실제 검증 요청**. 이 부분은 네트워크 호출이야.

두 단계를 거치는 게 "safe"한 이유야. `getSession()`만 하면 로컬 JWT를 믿는 건데, 누군가 토큰을 조작했더라도 잡아낼 수 없어. `getUser()`까지 해서 서버에 물어봐야 비로소 "진짜 유효한 사람인가"를 확신할 수 있어. 그 대신 매 요청마다 Supabase Auth 서버로 네트워크가 나가.

> 💡 **JWT가 뭐야?**
> 로그인 성공하면 서버가 발급해주는 암호화된 신분증이야.
> 안에 "이 사람 user_id는 abc, 유효기간은 언제까지" 같은 정보가 암호화되어 들어있어.
> 서버는 서명이 유효한지 확인할 수 있어. iOS의 Keychain에 저장하는 JWT랑 같은 개념이야.
> 단, `safeGetSession`은 JWT 로컬 검증에 그치지 않고 Supabase 서버에도 확인을 요청해 — 변조된 토큰까지 잡기 위해서야.

`+layout.server.ts`에 한 번만 써두면 모든 보호된 페이지가 자동으로 이 검증을 거쳐. 페이지마다 따로 쓸 필요가 없어. 검증에 실패하면? `/login`으로 튕겨내. 이게 **인증 가드**야.

---

**③ page.data.session 전달 — session 정보를 페이지로 넘겨준다**

검증을 통과하면 `session` 정보를 `page.data`에 담아서 `onboarding/+page.svelte`로 넘겨줘. 이걸 받은 뒤에야 온보딩 화면이 브라우저에 그려지기 시작해.

---

### 구간 [A] 흐름 정리

```
① 사용자가 /onboarding 접속
   → 브라우저가 쿠키(토큰)를 자동으로 담아 서버에 요청
② +layout.server.ts 실행 → safeGetSession() 실행
   → getSession()(로컬 디코드) + getUser()(Supabase Auth 서버 검증) 이중 확인
   → 유효하면 통과 / 유효하지 않으면 /login으로 튕겨냄
   → 모든 보호된 페이지에서 매번 반복됨
③ session 정보를 페이지로 전달 → 온보딩 화면 그리기 시작
```

---

## 구간 [B] — 태그 목록 + 내 태그 병렬 로드 (④~⑩)

[A]에서 인증 가드를 통과하고 `onboarding/+page.svelte`가 화면에 **마운트**됐어.

> 💡 **마운트가 뭐야?**
> 컴포넌트가 실제 화면(DOM)에 붙는 순간이야. iOS의 `viewDidLoad()`가 호출되는 시점이랑 같아.
>
> **DOM**은 브라우저가 HTML을 읽어서 만든 화면 구조야. 버튼, 텍스트, 이미지 같은 요소들이 트리 구조로 메모리에 올라와 있는 상태야. iOS의 UIView 계층 구조랑 비슷해.

태그 선택 화면을 그리려면 두 가지 데이터가 필요해.

첫째, "어떤 태그들이 있어?" — 전체 태그 목록
둘째, "이 사람이 이미 선택한 태그는 뭐야?" — 내 기존 태그

둘째가 의아할 수 있어. 신규 유저는 선택한 태그가 없으니까 필요 없어 보이잖아. 근데 이 온보딩 페이지는 피드에서 태그를 다시 수정할 때도 재사용돼. 그때는 이미 선택한 태그가 체크된 상태로 보여야 하니까 항상 같이 가져오는 거야. 신규 유저면 그냥 빈 배열이 오고 아무것도 체크 안 된 상태로 시작하는 거고.

---

**④ auth.isAuthenticated 확인 ($effect) — 클라이언트 보조 가드**

마운트 직후 `$effect`가 먼저 실행돼. `auth.isAuthenticated`가 false면 `/login`으로 보내는 보조 가드 역할이야.

[A]에서 서버가 이미 인증 체크를 했는데 왜 또 하냐면, [A]의 서버 체크는 페이지가 처음 열릴 때 딱 한 번만 실행돼. 페이지가 열려있는 상태에서 토큰이 만료되거나 다른 탭에서 로그아웃했다면 이 `$effect`가 상태 변화를 감지해서 즉시 `/login`으로 튕겨내. 서버 가드 + 클라이언트 가드의 이중 방어야.

> 💡 **$effect가 뭐야?**
> Svelte 5의 반응형 부작용 선언이야. 의존하는 `$state` 값이 바뀔 때마다 자동으로 다시 실행돼. iOS의 `.onChange(of:)` 모디파이어랑 비슷해.

---

**⑤⑥ Promise.all — 두 요청 동시 발송**

`onMount` 안에서 두 요청을 동시에 보내.

> 💡 **onMount가 뭐야?**
> 컴포넌트가 DOM에 처음 붙었을 때 딱 한 번 실행되는 생명주기 함수야. iOS의 `viewDidLoad()`랑 거의 같은 역할이야.

> 💡 **Promise가 뭐야?**
> 비동기 작업의 결과를 나중에 받겠다는 약속이야. 서버에 데이터를 요청하면 바로 결과가 오지 않잖아. 그 사이에 브라우저가 멈춰있으면 안 되니까, "요청은 보냈고 나중에 결과 줄게"라는 약속 객체를 먼저 반환하는 거야. iOS의 `async/await`로 네트워크 요청할 때 결과가 올 때까지 기다리는 거랑 같은 개념이야.

> 💡 **Promise.all이 뭐야? 왜 await을 따로 두 번 안 써?**
> `await fetchTags()` → `await fetchMyTagIds()` 이렇게 쓰면 첫 번째가 끝나야 두 번째가 시작돼. 두 요청 시간이 더해지는 거야.
>
> `Promise.all([A, B])`는 두 요청을 동시에 보내고 둘 다 완료될 때까지 기다려. 각각 200ms씩 걸린다면, 순차는 400ms, `Promise.all`은 200ms야. 두 결과가 서로 의존하지 않을 때 쓰는 패턴이야.

흐름도의 `Tag[] / Uuid[]` 표현은 빈 배열이 아니라 **배열 타입**을 표현하는 거야. `Tag[]`는 "Tag 여러 개로 이루어진 배열", `Uuid[]`는 "Uuid 여러 개로 이루어진 배열"이라는 뜻이야. iOS의 `[Tag]`, `[UUID]`랑 완전히 같은 표현이야.

---

**⑦⑧ Rust API → DB 쿼리**

Rust API는 두 요청을 받아서 각각 DB에 쿼리를 날려.

- `GET /api/tags` → `tags` 테이블에서 전체 태그 목록 조회
- `GET /api/me/tags` → `user_tags` 테이블에서 이 유저가 선택한 태그 조회

> 💡 **왜 tags 테이블이랑 user_tags 테이블이 따로 있어?**
> `tags`는 앱에 존재하는 태그 목록 자체야. `user_tags`는 유저가 어떤 태그를 선택했는지 연결 정보야. 이렇게 분리해두면 태그 이름이 바뀌어도 `tags` 테이블 한 곳만 수정하면 돼. 이 분리를 **정규화**라고 해. SQL 쿼리문 자체는 지금 알 필요 없어.

---

**⑨⑩ 결과 반환 → $state 갱신 → 화면 자동 재렌더링**

두 요청이 완료되면 결과를 화면 상태에 넣어.

```typescript
const [allTags, myTagIds] = await Promise.all([...])
tags = allTags
selectedIds = new Set(myTagIds)
```

두 요청을 동시에 보내고, 받은 결과인 `allTags`와 `myTagIds`를 `tags`와 `selectedIds`에 넣고, 그걸 기반으로 화면이 그려지는 거야.

> 💡 **$state가 뭐야?**
> Svelte 5의 반응형 변수 선언이야. `$state`로 선언된 변수가 바뀌면 이 값을 참조하는 UI가 자동으로 다시 그려져. iOS의 `@State`나 `@Published`랑 같은 역할이야.
>
> 여기서 `selectedIds = new Set(myTagIds)` 이렇게 **새 Set을 만들어서 재할당**하는 게 보여? `selectedIds.add()`처럼 기존 Set에 추가하면 Svelte가 변화를 감지 못해. Svelte 5의 `$state`는 **참조(reference) 비교**로 변경을 감지하거든. 새 객체를 만들어서 넣어야 "바뀌었다"고 인식해.

---

### 구간 [B] 흐름 정리

```
④ 페이지 마운트 → $effect로 auth.isAuthenticated 확인 (클라이언트 보조 가드)
⑤⑥ onMount → Promise.all([fetchTags(), fetchMyTagIds()]) 동시 발송
⑦⑧ Rust API → DB 쿼리 (tags 전체 / user_tags 필터)
⑨⑩ 결과 반환 → tags, selectedIds $state 갱신 → 화면 자동 재렌더링
```

---

## 구간 [C] — 태그 선택 UI (⑪⑫⑬)

[B]에서 `Promise.all`로 받아온 `Tag[]`와 `Uuid[]`를 `tags`와 `selectedIds`에 넣었어. 이제 그 데이터를 화면에 그리고, 사용자가 태그를 클릭할 때 어떻게 처리하는지야.

---

**⑪ $derived — tags를 category로 그룹핑**

태그 목록을 그냥 나열하지 않고 카테고리별로 묶어서 보여줘. "Technology", "Science" 이런 식으로 섹션이 나뉘는 거야. 이 그룹핑을 `$derived`로 처리해.

```typescript
const grouped = $derived(
    tags.reduce((acc, tag) => {
        const cat = tag.category ?? 'Other';
        if (!acc[cat]) acc[cat] = [];
        acc[cat].push(tag);
        return acc;
    }, {})
)
```

여기서 `grouped`가 변수 이름이고, `$derived(...)`가 "이 값은 계산해서 만들어줘"라고 Svelte에게 알려주는 선언 문법이야.

> 💡 **$derived가 뭐야?**
> `$state` 값을 기반으로 자동으로 계산되는 값이야. `tags`가 바뀌면 `grouped`도 자동으로 다시 계산돼. iOS의 계산 프로퍼티(`var grouped: [String: [Tag]] { ... }`)랑 같은 개념이야. 직접 값을 넣는 게 아니라 "이 값에서 이렇게 계산해줘"라고 선언해두는 거야.
>
> ```swift
> // Swift 계산 프로퍼티
> var grouped: [String: [Tag]] { ... }
> // Svelte $derived
> const grouped = $derived(...)
> ```

---

**⑫⑬ 태그 버튼 클릭 → toggleTag() → selectedIds 재할당**

⑫는 유저가 버튼을 눌러서 함수가 실행되는 거고, ⑬은 그 함수 안에서 `selectedIds`가 새 Set으로 교체되는 거야. 같은 `toggleTag()` 함수 안에서 일어나는 일을 흐름도가 두 단계로 나눠 표현한 것뿐이야.

```typescript
function toggleTag(tagId: string) {
    const next = new Set(selectedIds);
    if (next.has(tagId)) {
        next.delete(tagId);
    } else {
        next.add(tagId);
    }
    selectedIds = next;
}
```

이미 선택된 태그면 제거하고, 아니면 추가하는 토글이야. 여기서 핵심은 `selectedIds = next`야. 기존 Set에 바로 추가/제거하지 않고 **새 Set을 만들어서 재할당**해. [B]에서 설명한 것처럼 Svelte 5의 `$state`는 참조 비교로 변경을 감지하거든. 새 객체를 넣어야 "바뀌었다"고 인식하고 버튼 색상이 자동으로 반영돼.

---

### 구간 [C] 흐름 정리

```
⑪ $derived: tags를 category로 그룹핑 → 섹션별 UI 자동 계산
⑫ 태그 버튼 클릭 → toggleTag() 실행
⑬ selectedIds = new Set([...prev]) 재할당 → 버튼 색상 자동 반영
```

---

## 구간 [D] — 태그 저장 + 온보딩 완료 (⑭~㉓)

[C]에서 태그를 원하는 대로 골랐어. [D]는 그 선택을 실제로 저장하고 피드로 넘어가는 마무리 단계야.

---

**⑭ "Continue" 버튼 클릭**

사용자가 버튼을 누르면 화면이 들고 있던 `selectedIds`(선택된 태그 ID 목록)를 가지고 Rust API에 저장 요청을 보내.

---

**⑮ POST /api/me/tags { tag_ids: [...] } (Bearer 토큰)**

Rust API(8080)로 POST 요청을 보내. `{ tag_ids: ["uuid1", "uuid2", ...] }` 형태로 선택된 태그 ID들을 담아서.

여기서 **"원하는 데이터 + 내가 누구인지 증명"** 이 두 가지를 항상 같이 보내야 해. 쿠키는 SvelteKit ↔ Supabase 사이에서만 자동으로 처리되고, Rust API는 별개의 서버라 쿠키를 받지 않거든. 그래서 인증이 필요한 Rust API 요청은 **전부** `Authorization: Bearer {토큰}` 헤더를 직접 붙여야 해. [B]의 ⑤⑥도 같은 방식이었어.

번거롭지 않도록 프론트에 헬퍼 함수가 있어서 자동으로 붙여줘.

```typescript
function authHeaders() {
    return { Authorization: `Bearer ${currentAccessToken}` };
}
```

> 💡 **Bearer 토큰이 뭐야?**
> HTTP 헤더에 직접 붙여 보내는 인증 방식이야.
> `Authorization: Bearer eyJhbGci...` 이런 형태로, 로그인할 때 받은 JWT 토큰을 여기에 담는 거야.
> iOS에서 `URLRequest`에 `.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")` 수동으로 붙이는 거랑 완전히 같아.

---

**⑯⑰⑱ 트랜잭션: BEGIN → DELETE → INSERT × N → COMMIT**

Rust API가 태그를 저장할 때 **트랜잭션**으로 처리해.

```
BEGIN          ← "지금부터 하는 것들은 아직 확정 전이야"
  DELETE FROM user_tags WHERE user_id=$1
  ← ⑰-OK (삭제 완료)
  INSERT INTO user_tags (user_id, tag_id) × N
  ← ⑱-OK (삽입 완료)
COMMIT         ← "다 됐어, 이제 진짜 저장해"
```

BEGIN~COMMIT 사이의 변경사항들은 DB에 임시로만 기록된 상태야. COMMIT이 성공해야 비로소 확정돼. 중간에 오류가 나면 ROLLBACK이 일어나서 BEGIN 이전 상태로 돌아가. BEGIN이 "시작 알림", COMMIT이 "확정 저장"이라는 느낌이 맞아.

왜 기존 태그를 다 지우고 새로 넣냐면, 어떤 태그가 추가됐고 어떤 게 빠졌는지 비교하는 코드가 필요 없어서야. 그냥 전부 교체하면 돼.

> 💡 **트랜잭션이 뭐야?**
> "이 여러 쿼리를 한 묶음으로 처리해줘. 중간에 하나라도 실패하면 전부 없던 일로 해줘"라는 보장이야.
> DELETE는 됐는데 INSERT 도중 오류가 나면? 트랜잭션 없이는 태그가 모두 사라진 상태로 남아버려. 트랜잭션이 있으면 DELETE도 같이 롤백되니까 원래 상태가 유지돼.
> iOS의 CoreData에서 `context.save()` 실패 시 `context.rollback()` 하는 패턴이랑 같은 개념이야.

---

**⑲ UPSERT profiles SET onboarding_completed=true (트랜잭션 외부)**

태그 저장이 끝난 뒤, 별도 쿼리로 `onboarding_completed`를 `true`로 바꿔. DB에는 이런 SQL 명령어들이 있어.

| 명령어 | 역할 |
|---|---|
| SELECT | 읽기 (변경 없음) |
| INSERT | 새 행 추가 |
| UPDATE | 기존 행 수정 |
| DELETE | 행 삭제 |
| UPSERT | 있으면 UPDATE, 없으면 INSERT |

여기서 UPSERT를 쓰는 이유는, 신규 유저는 `profiles` 행 자체가 없을 수 있기 때문이야. UPDATE만 쓰면 행이 없을 때 아무 일도 안 일어나. UPSERT는 없으면 새로 만들어줘서 어느 경우든 안전해.

이 쿼리가 트랜잭션 **바깥**에 있는 건 의도적이야. 태그 저장이 성공한 뒤 이 업데이트가 실패해도 큰 문제가 아니거든. 이렇게 이해할 수 있어 — 태그 저장은 "반드시 성공해야 하는 핵심 데이터", `onboarding_completed` 갱신은 "실패해도 재시도 가능한 메타데이터"라는 성격 차이 때문에 분리한 거 아닐까.

> 💡 **UPSERT가 뭐야?**
> `UPDATE`와 `INSERT`를 합친 말이야. "있으면 업데이트하고, 없으면 새로 넣어줘"라는 뜻이야.
> 신규 유저는 `profiles` 행이 아직 없을 수 있어. 그때 `UPDATE`만 쓰면 아무것도 안 바뀌거든. `UPSERT`를 쓰면 없으면 새로 만들고 있으면 값만 바꿔주니까 안전해.

← ⑲-OK (갱신 완료) 응답이 돌아오면 다음 단계로 넘어가.

---

**⑳ feed_cache.invalidate_user(user_id)**

태그가 바뀌었으니 이 유저의 피드 캐시를 지워. 신규 유저는 캐시가 없으니까 실제로는 아무 일도 안 해. 그런데 굳이 체크 없이 그냥 호출하는 이유는, 이 온보딩 저장 로직이 설정 화면에서 태그를 수정할 때도 그대로 재사용되거든. 그때는 캐시가 실제로 있으니까 무효화가 의미 있어. [B]에서 신규 유저도 `fetchMyTagIds()`를 호출하는 것처럼, 재사용을 위해 신규일 때 해가 없는 빈 호출을 하는 구조야.

---

**㉑ { ok: true } + ㉒ 200 OK**

모든 처리가 성공하면 API가 `{ ok: true }`를 응답으로 돌려보내. DB에서 뭔가 조회해서 반환하는 게 아니라 "다 됐어"라는 신호를 API가 직접 만들어서 보내는 거야.

**200 OK**는 HTTP 상태 코드야. 숫자로 요청 결과를 표현하는 방식이고, 200은 성공이라는 뜻이야. 실패했을 때는 401(인증 실패), 404(없음), 500(서버 오류) 같은 다른 숫자가 와.

---

**㉓ goto('/feed')**

200 OK를 받으면 `goto('/feed')`로 피드 화면으로 이동해. 온보딩 끝.

---

### 구간 [D] 흐름 정리

```
⑭ "Continue" 버튼 클릭
⑮ POST /api/me/tags { tag_ids: [...] } (Bearer 토큰) → Rust API
⑯ BEGIN 트랜잭션
⑰ DELETE FROM user_tags WHERE user_id=$1
   ← ⑰-OK (삭제 완료)
⑱ INSERT INTO user_tags × N → COMMIT
   ← ⑱-OK (삽입+커밋 완료)
⑲ UPSERT profiles SET onboarding_completed=true (트랜잭션 외부)
   ← ⑲-OK (갱신 완료)
⑳ feed_cache.invalidate_user(user_id)
㉑ { ok: true } + ㉒ 200 OK → 프론트로 반환
㉓ goto('/feed') → 피드 화면으로 이동
```

> DB 쓰기 쿼리(⑰⑱⑲)는 모두 성공 응답(OK)이 Rust API로 돌아와야 다음 단계로 진행돼.
> 하나라도 실패하면 에러로 처리되고 트랜잭션은 롤백돼.

---

