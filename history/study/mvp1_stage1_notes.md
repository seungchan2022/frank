# MVP1 1단계: 로그인 — 학습 노트

> 세션 시작: 2026-04-22 / 재개: 2026-04-30
> 기준 흐름도: history/study/login_flow/index.html
> 학습 목표: 전체 흐름 파악 + 흐름 속 개념 자연스럽게 익히기

---

## 기초 개념 — 이 앱의 서버 구조 (흐름도 읽기 전에 먼저)

흐름도에 `+page.server.ts`, `hooks.server.ts` 같은 이름이 나오는데, "서버는 Rust 아니야?" 라는 의문이 드는 게 당연해. 흐름도 보기 전에 이것부터 정리해두면 훨씬 읽기 쉬워.

---

### 이 앱의 요청 흐름 — 한 줄 구조

```
웹:  브라우저 → SvelteKit 서버(5173) → Rust API(8080) → PostgreSQL
iOS:           iOS 앱               → Rust API(8080) → PostgreSQL
```

브라우저는 Rust API 주소(8080)를 직접 몰라도 돼. SvelteKit이 중간에서 대신 Rust API에 다녀오고 결과만 화면에 그려줘. iOS는 SvelteKit 없이 Rust API에 직접 붙어.

> 💡 **CRUD(데이터 읽기·쓰기)도 이 구조야.**
> 피드 조회, 스크랩 저장 같은 데이터 작업도 전부 `브라우저 → SvelteKit → Rust API → DB` 경로로 처리해.
> SvelteKit이 직접 DB 처리를 하는 게 아니라 **중간 다리** 역할이고, 실제 로직은 Rust API에 있어.

---

### SvelteKit 내부는 두 부분으로 나뉜다

```
SvelteKit
├── +page.svelte       → 브라우저에서 실행 (화면 그리기, UI 이벤트 처리)
├── +page.server.ts    → Node.js 서버에서 실행 (폼 제출 처리 — 서버 액션)
├── +layout.server.ts  → 서버에서 실행 (앱 전체 공통 인증 가드)
└── hooks.server.ts    → 서버에서 실행 (모든 요청을 가로채는 미들웨어)
```

> 💡 **화면 코드 vs 서버 코드**
> `+page.svelte`는 브라우저에서 실행되는 UI 코드. 화면에 뭘 보여줄지 담당해.
> `+page.server.ts`는 서버에서 실행되는 코드. 로그인 버튼처럼 `<form method="POST">`를 제출하면 그 요청을 받아서 처리해. 이걸 **서버 액션**이라고 불러.
>
> 단, 모든 버튼이 서버를 거치는 건 아니야:
> - `form method="POST"` 버튼 → 서버 액션 실행 O
> - 일반 onclick 버튼 (UI 토글, 탭 전환 등) → 브라우저 JS만 실행, 서버 X

---

### 왜 로그인 처리를 서버에서 해야 해?

로그인하면 Supabase가 토큰(JWT)을 줘. 웹에서는 이걸 **httpOnly 쿠키**에 저장해야 안전한데, httpOnly 쿠키는 브라우저 JS에서 저장이 불가능하고 **서버만 저장할 수 있어.** 그래서 SvelteKit 서버가 담당하는 거야. iOS는 Keychain에 저장하면 앱이 직접 접근하니까 서버가 필요 없어.

---

### 한눈에 비교

| | 역할 | 누가 써? |
|---|---|---|
| SvelteKit 서버 (5173) | 인증 처리, 쿠키 관리, 화면 서빙 | 웹 브라우저만 |
| Rust API (8080) | 데이터 처리 (피드, 태그, 스크랩) | 웹(SvelteKit) + iOS 앱 둘 다 |

---

## 등장인물

흐름도에 나오는 이름들이 각각 뭔지 먼저 알아두면 읽기 편해.

| 이름 | 역할 | 어디서 실행? |
|---|---|---|
| **User** | 버튼을 누르는 사람 + 브라우저 창 | — |
| **SvelteKit(웹)** | 화면을 그리고 User의 클릭 이벤트를 감지하는 JS 코드 | 브라우저 |
| **+page.server.ts** | form POST를 받아서 처리하는 서버 액션. `signInWithPassword()`를 실행함 | Node.js 서버 |
| **hooks.server.ts** | 모든 요청을 가로채는 미들웨어. 쿠키에서 JWT 검증 + event.locals에 저장 | Node.js 서버 |
| **Supabase Auth** | 비밀번호 확인, JWT 발급 전담 서버 | 외부 서버 |
| **Rust API(Axum)** | 피드, 스크랩 같은 실제 데이터 처리 서버 | 외부 서버 |

> 💡 **User와 SvelteKit(웹)이 헷갈리면?**
> User = 버튼을 누르는 사람(+브라우저 창). SvelteKit(웹) = 그 클릭을 감지하는 JS 코드.
> 흐름도에서 `User → SvelteKit(웹)`은 "사람이 버튼을 눌렀고, JS가 그 이벤트를 받았다"는 뜻이야.

---

## 웹 이메일 로그인 전체 흐름

### ① ~ ⑤ 로그인하고 홈으로 이동하기까지

**① 이메일+비밀번호 제출**

User가 이메일과 비밀번호를 입력하고 로그인 버튼을 누르면 form POST로 `+page.server.ts`에 전달된다. SvelteKit(웹)은 버튼 클릭 이벤트를 처리하지만, 실제 데이터 전송은 브라우저가 서버로 보내는 HTTP POST 요청이야.

---

**② signInWithPassword(email, password)**

`+page.server.ts`의 서버 액션이 `signInWithPassword()`를 실행해서 Supabase Auth로 전송한다. 브라우저 JS(SvelteKit(웹))가 아니라 서버에서 실행되는 이유는, 응답받은 JWT를 httpOnly 쿠키에 담아야 하는데 httpOnly 쿠키는 JS에서 set이 물리적으로 불가능하기 때문이다.

> 💡 **+page.server.ts가 뭐야?**
> SvelteKit에서 `.svelte` 파일(화면)과 같은 폴더에 있는 서버 전용 파일이야. `+page.svelte`는 브라우저에서 화면을 그리고, `+page.server.ts`는 서버에서 폼 제출을 받아 처리하는 역할을 해.
>
> `<form method="POST">` 버튼을 클릭하면 브라우저가 서버로 POST 요청을 보내는데, 그 요청을 받아서 처리하는 게 `+page.server.ts`에 정의된 **서버 액션**이야. 코드에서는 `export const actions = { signin: async (...) => { ... } }` 형태로 작성되고, 실행은 항상 서버에서만 이뤄져서 비밀번호나 JWT 같은 민감한 처리를 안전하게 할 수 있어.
>
> `hooks.server.ts`랑 뭐가 달라? `hooks.server.ts`는 **모든 요청**을 통과시키는 문지기고, `+page.server.ts`는 특정 페이지의 폼 제출만 처리하는 담당자야. 로그인 페이지의 폼은 `+page.server.ts`가 받고, 이후 쿠키 검증은 `hooks.server.ts`가 맡는 식으로 역할이 나뉘어.

이 함수는 Supabase SDK가 제공하는 함수로, 직접 구현한 게 아니라 가져다 쓰는 것이다. Rust API가 아닌 Supabase로 보내는 이유는, 비밀번호 검증 같은 걸 직접 구현하면 보안 구멍이 나기 쉬워서 전담 서버인 Supabase에 통째로 위임한 것이다.

---

**③ JWT(access_token) 반환**

Supabase Auth가 이메일과 비밀번호를 확인하면 **JWT**를 직접 만들어서 반환한다.

> 💡 **JWT = 출입증**
> "로그인 확인됐어. 이 출입증으로 앱을 자유롭게 돌아다녀"라고 주는 거야.
> 출입증 안에는 `나는 누구(user_id)`, `언제까지 유효(만료 시각, 보통 1시간)`, `발급처(Supabase 서명)`가 적혀 있어.
>
> 한 번 받으면 그 기간 동안 매 페이지를 이동할 때마다 다시 비밀번호를 입력할 필요 없이 출입증(쿠키)만 자동으로 제시하면 돼.
>
> **암호화가 아니라 서명**: 디코딩하면 안에 내용이 그대로 보여. 하지만 Supabase가 서명해뒀기 때문에 내용을 바꾸면 서명이 맞지 않아서 위조가 바로 걸려. 그래서 비밀번호 같은 민감한 정보는 안 담아.

---

**④ httpOnly 쿠키에 JWT 저장**

`locals.supabase`가 JWT를 **httpOnly 쿠키**에 자동으로 저장한다. 이후 브라우저는 모든 요청에 이 쿠키를 자동으로 첨부한다.

> 💡 **locals.supabase가 뭐야?**
> 일반 Supabase 클라이언트와 다른, 쿠키를 자동으로 읽고 쓸 수 있는 **SSR 전용 클라이언트**야.
> `hooks.server.ts`가 앱 시작 시 이 클라이언트를 만들어서 `event.locals.supabase`에 넣어두면, `+page.server.ts`에서 꺼내 쓸 수 있어.
> 이 클라이언트로 `signInWithPassword()`를 호출하면 JWT를 받는 것과 동시에 httpOnly 쿠키 저장까지 자동으로 처리해줘.

> 💡 **httpOnly 쿠키란?**
> 브라우저가 서버에 요청할 때 자동으로 붙여주는 저장소인데, **JS에서 아예 읽을 수 없도록** 막혀 있는 쿠키야. 토큰을 localStorage에 넣으면 JS로 훔칠 수 있는 반면(XSS 공격), httpOnly 쿠키에 넣으면 JS 접근 자체가 차단돼서 안전해.

---

**⑤ redirect(303, '/')**

JWT 저장이 끝나면 `+page.server.ts`가 `redirect(303, '/')`를 반환하고, 브라우저가 홈 페이지로 자동 이동한다.

---

### ⑥ ~ ⑭ 로그인 이후 매 요청마다

**⑥ 페이지 요청**

User가 페이지를 열 때마다 브라우저는 ④에서 저장해둔 httpOnly 쿠키를 자동으로 붙여서 보낸다. User가 따로 뭔가 하는 게 아니라 브라우저가 알아서 첨부하는 것이다.

---

**⑦ safeGetSession() — getSession()으로 JWT 꺼냄**

모든 요청은 무조건 `hooks.server.ts`를 거치는데, 여기서 `safeGetSession()`을 호출해 쿠키 속 JWT를 꺼낸다. `safeGetSession()`은 `hooks.server.ts`에 직접 정의하는 커스텀 헬퍼 함수야.

이 단계에서 `getSession()`으로 쿠키에서 JWT만 꺼내는 거야. 로컬에서만 읽는 거라 아직 유효한지 모르는 상태야.

---

**⑧ getUser()로 Supabase에 토큰 유효성 재확인**

⑦에서 꺼낸 JWT를 Supabase Auth에 보내서 "아직 유효해?"라고 실제로 확인한다. 흐름도에서 ⑦⑧이 화살표 두 개로 나뉜 건 이 두 단계가 `safeGetSession()` 함수 안에서 순서대로 실행되기 때문이야 — 따로 진행되는 게 아니라 한 함수 안에서 연속으로 실행되는 거야.

> 💡 **왜 getSession()만으로 끝내지 않아?**
> `getSession()`만 쓰면 만료된 토큰도 "있다"고 통과시킬 수 있어서 위험해. `getUser()`까지 해야 Supabase가 직접 확인해주니까 안전해. JWT는 보통 1시간 뒤 만료돼. 만료된 경우 로그인 페이지로 자동으로 튕겨내.

---

**⑨ 유효한 user 반환**

Supabase Auth가 "유효해"라고 응답하면 해당 유저 정보(user_id, 이메일 등)를 `hooks.server.ts`로 돌려준다.

---

**⑩ session + user → event.locals 저장**

`hooks.server.ts`가 검증한 유저 정보를 `event.locals`에 저장한다.

> 💡 **event.locals란?**
> 한 번의 요청을 처리하는 동안에만 살아있는 임시 공유 공간이야. 요청이 끝나면 사라져.
>
> **왜 여기 저장해?** 피드 페이지를 열었다고 하면, 그 요청을 처리하면서 여러 서버 코드가 순서대로 실행돼:
> `hooks.server.ts` → `+layout.server.ts` → `+page.server.ts`
>
> `hooks.server.ts`에서 Supabase에 JWT 검증을 한 번 해서 유저 정보를 `event.locals.user`에 넣어두면, 뒤에 실행되는 `+layout.server.ts`나 `+page.server.ts`에서 그냥 `event.locals.user`를 꺼내 쓰면 돼. 같은 요청에서 Supabase에 여러 번 검증하러 갈 필요가 없어.

---

**⑪ GET /api/... (Bearer 토큰 첨부)**

실제 데이터가 필요하면 SvelteKit 서버가 Rust API로 요청을 보낸다. 이때 JWT를 **Bearer 토큰** 형태로 붙여서 보낸다.

> 💡 **Bearer 토큰이란?**
> HTTP 요청 헤더에 토큰을 붙여서 보내는 방식이야. `Authorization: Bearer <JWT>` 형태로 보내는데, "이 토큰을 가진 사람한테 접근을 허용해줘"라는 뜻이야. Bearer는 "소지자"라는 뜻.

---

**⑫ /auth/v1/user 검증 → ⑬ 유효 확인**

Rust API의 `require_auth()` 미들웨어가 받은 Bearer 토큰을 Supabase Auth에 보내서 검증한다. `require_auth()`는 직접 구현한 Rust 코드야.

> 💡 **미들웨어(middleware)란?**
> 요청이 실제 처리 코드에 도달하기 전에 **무조건 거쳐야 하는 관문**이야.
> `require_auth()`는 Rust API의 모든 인증 필요 엔드포인트 앞에 붙어 있어서, 요청이 들어오면 자동으로 "Bearer 토큰 있어? 유효해?"를 먼저 확인해. 통과하면 실제 로직 실행, 실패하면 401 에러 반환.
>
> 이 앱에서 미들웨어는 두 군데야:
> - SvelteKit의 `hooks.server.ts` — 웹 요청마다 쿠키에서 JWT 검증
> - Rust API의 `require_auth()` — API 요청마다 Bearer 토큰 검증

> 💡 **Supabase가 이미 로그인시켰는데, Rust API가 또 검증하는 이유?**
> 역할이 달라. Supabase는 "이 사람이 회원 맞아? 비밀번호 맞아?"를 확인하고 JWT를 발급하는 역할이고, Rust API는 "이 토큰이 지금도 유효해?"만 확인하는 역할이야.
>
> ```
> Supabase  →  여권 발급소 (신원 확인 후 여권 만들어줌)
> Rust API  →  공항 입국심사 (여권이 유효한지만 확인)
> ```
>
> 누군가 SvelteKit을 거치지 않고 Rust API로 직접 요청을 보낼 수도 있어. Rust API 입장에선 요청이 어디서 왔는지 알 수 없으니까 무조건 스스로 Supabase에 확인하는 거야. JWT를 **만든 곳이 Supabase**니까, 유효한지 확인하는 것도 **Supabase한테 직접 물어보는 게 가장 확실**해.

---

**⑭ 데이터 반환**

검증이 통과되면 Rust API가 요청한 데이터(뉴스, 프로필 등)를 SvelteKit 서버로 반환하고, SvelteKit이 화면을 그려서 User에게 보여준다.

---

### 전체 흐름 한눈에 보기

```
① User가 이메일+비밀번호 입력
② form POST → +page.server.ts 서버 액션 → Supabase Auth로 전송
③ Supabase Auth → JWT 발급해서 반환
④ +page.server.ts(locals.supabase SSR) → httpOnly 쿠키에 JWT 자동 저장
⑤ 홈으로 redirect(303, '/')

-- 이후 페이지 요청마다 자동 실행 --

⑥ 브라우저 → 쿠키 자동 첨부해서 페이지 요청
⑦ hooks.server.ts → safeGetSession()으로 JWT 꺼냄
⑧ getUser()로 Supabase에 토큰 유효성 재확인
⑨ Supabase → 유효한 user 반환
⑩ session + user → event.locals에 저장

-- 실제 데이터가 필요할 때 --

⑪ SvelteKit → Bearer 토큰 붙여서 Rust API에 요청
⑫ Rust API → Supabase에 토큰 검증
⑬ Supabase → 유효 확인
⑭ Rust API → 데이터 반환 → SvelteKit → 화면에 표시
```

---

## 웹 Apple OAuth 로그인 전체 흐름

이메일이랑 가장 큰 차이는 **브라우저가 Apple 서버로 한 번 나갔다 돌아오는 과정**이 중간에 끼어 있다는 거야.

### 사용자 눈에 보이는 흐름 (주소창 기준)

실제로 브라우저에서 어떤 일이 일어나는지 주소창 기준으로 먼저 보면 전체 구조가 훨씬 잘 잡혀.

```
[1] yourapp.com/login          ← "Apple로 계속하기" 버튼이 있는 로그인 화면
      ↓ 버튼 클릭
[2] appleid.apple.com/...      ← 브라우저가 Apple 서버로 통째로 이동
      ↓ Face ID / Apple ID 인증
[3] yourapp.com/auth/callback?code=...   ← 인증 직후 로딩 중에 순간 거쳐가는 경로
      ↓ code → JWT 교환 처리 (사용자 눈엔 안 보임)
[4] yourapp.com/               ← 홈 화면 도착
```

> 💡 **/auth/callback은 로그인 화면이 아니야.**
> `/login`은 버튼이 있는 화면이고, `/auth/callback`은 버튼도 없는 **처리 전용 경로**야. Apple 인증 후 브라우저가 잠깐 거쳤다가 바로 홈으로 튀어가기 때문에 사용자는 화면을 볼 틈이 없어. 인증하고 나서 잠깐 로딩되는 그 순간에 `/auth/callback`을 스치고 홈으로 넘어가는 거야.
> `/auth/callback`은 우리가 만든 SvelteKit 라우트(`src/routes/auth/callback/+server.ts`)야 — Supabase도 Apple도 아닌 **우리 앱 서버의 한 경로**야.

---

### 등장인물 추가

| 이름 | 역할 |
|---|---|
| **+page.server.ts** | 서버에서 실행되는 SvelteKit 서버 액션. 폼 제출을 받아서 `signInWithOAuth()` 같은 Supabase SDK 함수를 실행함 |
| **Apple OAuth** | Apple의 로그인 인증 서버. 사용자가 Apple 계정으로 본인임을 인증해줌 |
| **/auth/callback** | 우리 앱의 SvelteKit 처리 전용 경로. Apple 인증 후 브라우저가 돌아오는 도착지. 사용자가 보는 화면이 아니라 code→JWT 교환을 처리하고 홈으로 redirect하는 역할 |

> 💡 **OAuth란?** 비밀번호를 우리 앱에 직접 안 주고, 제3자(여기선 Apple)가 대신 신원을 보증해주는 방식이야. Apple이 비밀번호 관리를 통째로 담당하고, 우리 앱은 Apple로부터 "이 사람 맞아" 확인서만 받는 거야.

---

### ① ~ ⑩ 로그인하고 홈으로 이동하기까지

**① Apple로 로그인 클릭**

User가 "Apple로 계속하기" 버튼을 누르면 form POST 요청이 `+page.server.ts`의 `appleOAuth` 서버 액션으로 전달된다. 이메일 로그인과 구조가 같아 — 이메일은 form에 이메일+비밀번호를 담아 보내고, Apple은 입력값 없이 "Apple 로그인 해줘"라는 신호만 보내는 차이야.

---

**② signInWithOAuth({provider: 'apple'}) → Apple URL 받아서 돌아옴**

`+page.server.ts`가 Supabase에 `signInWithOAuth()`를 호출한다. Supabase가 "Apple 로그인 페이지로 가는 URL"을 만들어서 `+page.server.ts`로 돌려준다.

```
+page.server.ts → Supabase: "Apple 로그인 URL 만들어줘"
+page.server.ts ← Supabase: "여기 URL이야 (data.url)"   ← 돌아옴
```

이 URL에는 앱 식별 정보와 인증 후 돌아올 주소(`/auth/callback`)가 담겨 있다.

> 💡 **② → ③ 연결이 헷갈리면?**
> ②에서 화살표가 Supabase로 갔으니 ③이 Supabase에서 시작해야 할 것 같지만, ②는 **왕복**이야. Supabase가 Apple URL을 `+page.server.ts`에게 돌려줬고, ③은 그 URL을 들고 `+page.server.ts`가 브라우저에게 "여기로 가"라고 하는 거야. 그래서 ③의 시작이 `+page.server.ts`인 게 맞아.

---

**③ Apple 서버로 redirect(302)**

`+page.server.ts`가 `redirect(302, data.url)` 응답을 브라우저에게 보낸다. 브라우저가 이 응답을 받아 Apple 로그인 페이지로 통째로 이동한다 — 주소창이 `appleid.apple.com/...`으로 바뀌면서 Apple 로그인 화면이 뜨는 거야.

이메일 로그인과의 핵심 차이가 여기야. 이메일은 `+page.server.ts`가 Supabase에 직접 요청하고 끝났는데, Apple OAuth는 서버가 브라우저에게 "Apple로 갔다 와"라고 위임하면서 브라우저가 실제로 외부 서버로 이동하는 단계가 끼어 있어.

---

**④ Apple 계정으로 인증 완료**

User가 Apple 로그인 화면에서 Face ID나 Apple ID/비밀번호로 인증을 마치면, Apple이 "이 사람 맞아"라고 확인해준다.

---

**⑤ /auth/callback?code=... 로 redirect**

Apple이 인증을 마치고 브라우저를 우리 앱의 `/auth/callback`으로 돌려보낸다. URL에 `?code=...` 형태로 **authorization code**가 붙어온다.

이 시점이 사용자 입장에서 "인증하고 나서 로딩 중"인 구간이야. 브라우저 주소창이 `yourapp.com/auth/callback?code=...`으로 순간 바뀌는데, `/auth/callback`이 처리를 끝내고 바로 홈으로 redirect해버리니까 화면이 보일 틈이 없어.

> 💡 **authorization code란?**
> Apple이 "인증은 됐어, 대신 이 코드로 토큰을 받아가"라고 주는 일회용 교환권이야. 토큰 자체를 URL에 직접 넣으면 주소창에 노출되니까, 코드만 먼저 주고 서버끼리 교환하는 방식을 써.

---

**⑥ code 전달**

브라우저가 `/auth/callback`에 도착하면, SvelteKit 서버가 URL에서 code를 꺼내서 Supabase에 전달한다.

---

**⑦ exchangeCodeForSession(code)**

`/auth/callback`이 Supabase SDK의 `exchangeCodeForSession()`을 호출한다. Supabase가 이 code를 Apple 서버에 보내서 진짜인지 확인하고 토큰을 교환한다.

---

**⑧ JWT(access_token) 반환**

Supabase가 Apple 검증을 마치고 JWT를 만들어서 돌려준다. 여기서부터는 이메일 로그인이랑 똑같다.

---

**⑨ httpOnly 쿠키에 JWT 저장 → ⑩ 홈으로 redirect(303, '/')**

`/auth/callback`에서 `locals.supabase`(SSR 클라이언트)가 JWT를 httpOnly 쿠키에 자동으로 저장하고, `redirect(303, '/')`로 홈으로 보낸다. 이메일 흐름의 ④⑤와 동일해 — JWT 도착 → SSR 클라이언트가 쿠키 저장 → 홈으로 이동.

---

### ⑪ ~ ⑮ 로그인 이후 매 요청마다

이메일 흐름 ⑥~⑩과 완전히 동일하다. `safeGetSession()` → `getUser()` → `event.locals` 저장 순서로 동작한다.

Rust API 호출도 이메일과 동일하게 Bearer 토큰을 붙여서 요청하며, `require_auth()` 미들웨어가 Supabase에 검증한다.

---

### 전체 흐름 한눈에 보기

```
① 클릭 → form POST → +page.server.ts 서버 액션 진입
② +page.server.ts → Supabase: "Apple URL 만들어줘"
   Supabase → +page.server.ts: Apple URL 반환 (왕복)
③ +page.server.ts → redirect(302, Apple URL) → 브라우저가 Apple 서버로 이동
④ User가 Apple에서 인증 (Face ID / Apple ID)
⑤ Apple → 브라우저를 yourapp.com/auth/callback?code=... 으로 redirect
   (인증 직후 로딩 구간 — 주소창에 /auth/callback이 순간 보임)
⑥ /auth/callback 서버가 URL에서 code 꺼냄
⑦ exchangeCodeForSession(code) → Supabase가 Apple에 코드 교환
⑧ Supabase → JWT 반환
⑨ /auth/callback → httpOnly 쿠키에 JWT 저장
⑩ /auth/callback → redirect(303, '/') → 홈 도착
⑪~⑮ 매 요청마다 동일 (이메일과 동일)
```

---

## iOS 흐름 읽기 전에 — 웹이랑 뭐가 다른가

웹은 SvelteKit 안에서 역할이 두 개로 나뉘어 있어.

```
SvelteKit 안
├── +page.svelte      → 화면 그리기 (브라우저에서 실행)
└── +page.server.ts   → 처리 담당 (서버에서 실행 — 인증, Supabase 호출 등)
   hooks.server.ts   → 처리 담당 (모든 요청을 가로채는 미들웨어)
```

iOS도 같은 구분이 존재해. 다만 "처리 담당"이 서버가 아니라 앱 안의 구현체야.

```
iOS 앱 안
├── SwiftUI               → 화면 그리기 (↔ +page.svelte)
└── SupabaseAuthAdapter   → 처리 담당 (↔ +page.server.ts 역할)
```

> 💡 **같은 역할, 다른 위치**
> 웹의 `+page.server.ts`는 **SvelteKit 서버(Node.js)** 에서 실행돼 — 브라우저와 물리적으로 분리된 서버 프로세스야.
> iOS의 `SupabaseAuthAdapter`는 **앱 안(클라이언트)** 에서 실행돼 — 별도 서버 없이 앱 자체가 Supabase SDK를 직접 호출해.
>
> 그래서 iOS는 SvelteKit이 아예 없는 거야. 웹에서 서버가 하던 "인증 처리" 역할을 앱 내부 구현체가 직접 담당하는 구조야.

`hooks.server.ts` 역할(모든 요청 가로채기)은 iOS에서 **Rust API의 `require_auth()` 미들웨어**가 대신해. 앱이 API를 호출할 때마다 Rust 서버가 토큰을 검증하는 방식이야.

---

## iOS 이메일 로그인 전체 흐름

### 등장인물

| 이름 | 역할 |
|---|---|
| **User** | iOS 앱 화면 |
| **Frank 앱(SwiftUI)** | 화면을 그리고 버튼 이벤트를 처리하는 UI 레이어 |
| **SupabaseAuthAdapter** | Supabase SDK를 실제로 호출하는 인프라 레이어. AuthPort 인터페이스의 구현체 |
| **Supabase Auth** | 로그인 검증 전담 서버 (웹이랑 동일) |
| **iOS Keychain** | 토큰을 안전하게 저장하는 iOS 암호화 보관함 |
| **Rust API(Axum)** | 실제 기능 서버 (웹이랑 동일) |

> 💡 **Frank 앱(SwiftUI)이랑 SupabaseAuthAdapter가 같은 프로젝트 안에 있는데 뭐가 달라?**
> 같은 프로젝트 안에 있지만 **역할(레이어)**이 달라. Frank 앱(SwiftUI)은 UI 레이어로, `AuthPort`(인터페이스)에게 "로그인해줘"라고만 해. `SupabaseAuthAdapter`가 뭔지 몰라. `SupabaseAuthAdapter`는 인프라 레이어로, `AuthPort`를 실제로 Supabase SDK로 구현한 구현체야. 앱 시작 시 `AppDependencies`에서 "이 앱이 로그인할 때 `SupabaseAuthAdapter`를 써"라고 연결해줘.
>
> 웹에서 UI(`+page.svelte`)랑 서버 로직이 같은 프로젝트에 있어도 역할이 다른 것처럼.

---

### ① ~ ⑤ 로그인하고 토큰 저장까지

**① 이메일+비밀번호 입력 후 로그인 탭**

User가 이메일과 비밀번호를 입력하고 로그인 버튼을 누르면 Frank 앱(SwiftUI)이 이벤트를 받아서 다음 단계로 넘어간다.

---

**② `signIn(email:password:)` 호출**

Frank 앱(SwiftUI)이 `SupabaseAuthAdapter.signIn(email:password:)`를 호출한다.

`SupabaseAuthAdapter`는 우리가 직접 만든 Swift 파일(`SupabaseAuthAdapter.swift`)인데, 포트/어댑터 패턴에서 **어댑터** 역할을 한다.
- `AuthPort` — 프로토콜(인터페이스). "로그인해줘", "로그아웃해줘" 같은 기능 목록만 선언
- `SupabaseAuthAdapter` — `AuthPort`를 Supabase Swift SDK로 실제 구현한 구현체

Frank 앱(SwiftUI)은 `AuthPort`만 알고, `SupabaseAuthAdapter`가 뭔지 몰라. 같은 프로젝트 안에 있지만 레이어가 다른 거야. 나중에 Supabase를 다른 인증 서비스로 바꿔도 앱 코드는 건드릴 필요가 없는 구조야.

---

**③ 이메일+비밀번호로 인증 요청**

`SupabaseAuthAdapter`가 내부에서 `client.auth.signIn(email:password:)`를 호출한다. 이 함수는 **Supabase Swift SDK**가 제공하는 함수야. 이 호출로 이메일과 비밀번호가 Supabase Auth 서버로 전송된다.

웹에서 SvelteKit 서버가 `signInWithPassword()`를 호출한 것과 같은 역할이야. 이름은 다르지만 하는 일은 동일해.

---

**④ JWT(access_token) 반환**

Supabase Auth가 이메일/비밀번호를 확인하고 JWT를 만들어서 SDK로 돌려준다. 웹이랑 완전히 동일한 단계야.

---

**⑤ 토큰 자동 저장 — SDK가 JWT를 iOS Keychain에 자동 저장 (SDK 내부 처리)**

Supabase Swift SDK가 ④에서 받은 JWT를 **iOS Keychain에 자동으로 저장**한다. 우리가 만든 `SupabaseAuthAdapter` 코드 안에는 "Keychain에 저장해라"는 코드가 없어. SDK가 내부에서 알아서 처리하는 거야.

실제 처리 순서는 이래:
```
client.auth.signIn() 호출 → Supabase Auth로 요청 전송
→ Supabase Auth가 JWT 반환
→ SDK 내부에서 Keychain에 자동 저장
→ supabaseSession을 우리 코드로 반환
```

SDK가 세 번째 단계를 우리 눈에 안 보이게 처리하는 거야.

웹에서는 `locals.supabase`(SSR 클라이언트)가 이 역할을 했어 — `hooks.server.ts`가 쿠키 처리가 설정된 SSR 클라이언트를 만들어두고, 인증 함수 호출 시 그 클라이언트가 JWT를 httpOnly 쿠키에 자동으로 저장했지. iOS에서는 그 저장 코드가 SDK 안에 들어가 있어서 우리가 짤 필요가 없어. **SDK가 웹 서버 레이어가 하던 역할(토큰 저장, 갱신, API 요청 시 헤더 첨부)을 자동으로 처리해주는 거야.**

> 💡 **Keychain이 뭐야?**
> iOS가 제공하는 암호화된 비밀 보관함이야. 앱 샌드박스 안에 있어서 다른 앱이 접근할 수 없어. 웹의 httpOnly 쿠키랑 역할이 같아 — 토큰을 안전하게 저장하는 곳이고, 앱 코드도 직접 접근하기 어렵게 막혀 있어.

---

### ⑥ ~ ⑨ 프로필 조회

> 💡 **이 프로젝트에서 Supabase와 Rust의 역할 분리**
>
> | 역할 | 담당 |
> |---|---|
> | 인증 (로그인/로그아웃/세션) | Supabase SDK |
> | DB 테이블 저장소 (PostgreSQL 호스팅) | Supabase |
> | DB CRUD 로직 + 비즈니스 로직 | Rust API |
>
> Supabase가 DB 테이블을 갖고 있지만, 그 테이블을 읽고 쓰는 코드는 전부 Rust 서버 안에 있어. Supabase는 "인증 서버 + DB 공간"을 빌려주는 역할이고, 그 공간을 어떻게 쓸지는 Rust가 결정해.
>
> Supabase SDK만 쓰는 방식에서는 `supabase.from("profiles").select()` 처럼 SDK로 DB에 직접 붙는데, 이 프로젝트는 DB CRUD를 전부 Rust API 엔드포인트 호출로 처리해.

**⑥ GET /api/me/profile (Bearer 토큰 첨부)**

⑤에서 SDK는 Keychain에 JWT를 먼저 저장하고, 완료되면 `Session` 객체를 Adapter 코드로 반환한다.

> 💡 **`Session`은 Supabase Swift SDK가 정의한 타입이야**
>
> ```swift
> // Supabase Swift SDK 내부에 정의된 타입
> struct Session {
>     let accessToken: String   // JWT
>     let refreshToken: String
>     let user: User
>     // ...
> }
> ```
>
> `client.auth.signIn()` 호출 하나로 SDK가 두 가지를 동시에 해:
> 1. Keychain에 JWT 자동 저장 (내부 처리 — 앱 재시작 후에도 로그인 유지용)
> 2. `Session` 객체를 우리 코드로 반환
>
> 우리는 반환된 `Session.accessToken`을 바로 꺼내 쓰는 거고, Keychain을 다시 열어서 꺼내는 게 아니야.

실제 코드로 보면 딱 이렇게 돼:

```swift
let session = try await client.auth.signIn(...)   // ③④⑤ 여기서 다 처리
//  ↑ SDK가 반환한 Session 타입 (Keychain 저장도 SDK가 내부에서 알아서 함)

fetchServerProfile(token: session.accessToken, ...)  // ⑥ 반환값에서 바로 꺼냄
```

이때 `ProfileAPI.fetchProfile`이 `URLRequest`에 직접 `Authorization: Bearer {token}` 헤더를 설정해서 보내:

```swift
request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
```

**SDK가 자동으로 해주는 게 아니야.** Supabase SDK는 자기 Auth 엔드포인트(③④에서 쓴 `client.auth.signIn` 같은 것들)에만 자동으로 토큰을 붙여줘. Rust API 호출은 우리 코드(`ProfileAPI`)가 수동으로 헤더를 설정하는 거야.

> 💡 **엔드포인트가 뭐야?**
> URL 주소 + 요청 방식을 합쳐서 부르는 말이야. `GET /api/me/profile`이면 — `/api/me/profile` 주소에 `GET` 방식으로 요청을 보내는 것. "호출한다"는 건 그 주소로 HTTP 요청을 보낸다는 뜻이야.
>
> 💡 **DB는 Supabase에 있는데 왜 Rust API를 거쳐?**
> DB(PostgreSQL)는 Supabase 인프라 위에 있어. 근데 우리 앱은 DB에 직접 접근하지 않고 항상 Rust API를 거쳐. 이유는 두 가지야:
> - Rust API가 "이 요청이 진짜 로그인된 사용자인지" 먼저 검증(⑦⑧)한 다음에야 DB를 조회해줘 — 직접 접근하면 인증 우회가 가능해져서 보안상 위험해
> - `displayName`, `onboardingCompleted`, `occupation` 같은 프로필 데이터는 Rust 서버의 `profiles` 테이블에서 관리해 — Supabase Auth가 주는 JWT엔 인증 정보만 있고 이런 앱 데이터는 없어

---

**⑦ 토큰 유효성 검증 / ⑧ 유효 확인**

Rust API가 Bearer 토큰을 받으면 직접 검증하지 않고 Supabase Auth에게 물어본다. "이 토큰 유효한 사용자 것 맞아?" Supabase Auth가 "맞아"라고 응답(⑧)하면 다음으로 넘어간다.

웹에서도 Rust API가 Supabase Auth에 토큰 검증을 위임하는 구조야. 동일해.

---

**⑨ Profile 반환**

검증이 통과되면 Rust API가 `user_id`로 `profiles` 테이블을 조회해서 해당 유저의 `displayName`, `onboardingCompleted`, `occupation` 같은 앱 프로필 데이터를 **SupabaseAuthAdapter**로 반환한다. SwiftUI가 아닌 Adapter로 돌아오는 건 ⑥에서 요청을 보낸 게 Adapter이기 때문이야. 요청-응답에서 응답은 항상 요청을 보낸 쪽으로 돌아간다.

> 💡 **Supabase SDK로 기사 같은 앱 데이터도 접근하는 거야?**
> 아니야. **Supabase SDK는 인증(Auth) 전용**이야 — 로그인, 로그아웃, 세션 관리만 담당해. 기사, 피드, 즐겨찾기 같은 앱 데이터는 전부 **Rust API 엔드포인트를 직접 호출**해서 가져와. Supabase는 "누구인지" 확인해주고, 실제 앱 데이터는 Rust가 담당하는 구조야.

---

### ⑩ ~ ⑪ 앱 상태 전환하고 홈 화면까지

**⑩ .authenticated(profile) 상태 전환**

`SupabaseAuthAdapter`가 ⑨에서 받은 Profile을 Frank 앱(SwiftUI)으로 반환한다. 앱이 "로그인 완료, 이 사람의 프로필이야"라는 신호를 받는 거야.

---

**⑪ 홈 화면으로 이동**

SwiftUI가 상태 변화를 감지하고 자동으로 홈 화면으로 전환한다.

웹에서는 서버가 `redirect(303, '/')`를 보내서 브라우저가 이동했는데, iOS는 서버 없이 상태가 바뀌면 SwiftUI가 알아서 화면을 전환하는 구조야.

---

## iOS Apple 로그인 전체 흐름

### 등장인물

이메일 흐름과 거의 같은데, 새로 추가된 액터가 딱 하나 있다.

| 이름 | 역할 |
|---|---|
| **User** | iOS 앱 화면 |
| **Frank 앱(SwiftUI)** | 화면을 그리고 버튼 이벤트를 처리하는 UI 레이어 |
| **iOS 시스템(ASAuthorization)** | Apple이 iOS에 내장한 "Sign in with Apple" 전담 처리기. 우리가 만든 코드가 아닌 iOS 자체 기능 |
| **SupabaseAuthAdapter** | Supabase SDK를 실제로 호출하는 인프라 레이어. AuthPort 인터페이스의 구현체 |
| **Supabase Auth** | Apple idToken을 검증하고 우리 앱의 JWT를 발급하는 인증 서버 |
| **iOS Keychain** | 토큰을 안전하게 저장하는 iOS 암호화 보관함 |
| **Rust API(Axum)** | 실제 기능 서버 (이메일 흐름과 동일) |

> 💡 Apple 로그인은 시스템 레벨에서 처리해야 하는 기능이라 iOS 시스템이 별도 액터로 분리되어 있어. 이메일 흐름에서는 App → Adapter → Supabase로 바로 연결됐는데, Apple 흐름은 중간에 iOS 시스템이 끼어 있는 게 차이야.

---

### ① ~ ② 버튼 탭 → iOS 시스템에 위임

**① Apple로 로그인 버튼 탭**

User가 버튼을 누르면 Frank 앱(SwiftUI)이 탭 이벤트를 받는다. 이메일 흐름 ①과 동일.

---

**② ASAuthorizationController로 인증 요청**

Frank 앱이 iOS 시스템에 "Apple 로그인 처리해줘"라고 위임한다. 이때 `requestedScopes`와 `nonce`를 함께 보낸다.

> 💡 **ASAuthorizationController란?**
> Apple이 제공하는 클래스야. "Sign in with Apple 요청을 받아서 처리해주는 컨트롤러"라고 보면 돼. 우리가 직접 Apple 서버에 연결하거나 팝업을 만드는 게 아니라, 이 컨트롤러한테 넘기면 iOS 시스템이 알아서 다 처리해줘.

> 💡 **requestedScopes란?**
> Apple한테 "이 정보 줘"라고 요청하는 권한 목록이야. `[.fullName, .email]`을 넘기면 이름과 이메일을 달라는 거야. 주의할 점은 Apple이 이 정보를 **최초 로그인 때만** 준다는 것. 두 번째 로그인부터는 안 줘서, 첫 로그인 때 받은 정보를 DB에 저장해두는 게 중요해.

> 💡 **nonce란?**
> 일회용 랜덤 문자열이야. App이 미리 만들어서 SHA256으로 해시한 값을 요청에 담아 보내면, Apple이 idToken 안에 그 해시값(SHA256)을 박아서 서명해줘. 나중에 Supabase가 idToken을 받았을 때 "이 토큰이 지금 이 요청에 맞는 거 맞아?"를 확인하는 데 써. 누군가 idToken을 중간에 가로채서 재사용하려 해도 nonce가 맞지 않아서 Supabase가 거부해.

버튼 탭 한 번에 ①②가 연달아 일어난다. 사용자 눈에는 버튼 눌렀더니 팝업이 뜨는 것처럼 보이지만, 내부적으로는 App이 이벤트를 받고 → iOS 시스템에 넘기는 두 단계가 있다.

---

### ③ ~ ⑤ iOS 시스템이 팝업 띄우고 인증하고 결과 돌려주기

**③ 네이티브 Apple 로그인 팝업 표시**

②에서 요청을 위임받은 iOS 시스템이 직접 팝업을 띄운다. 이 팝업은 우리가 SwiftUI로 만든 UI가 아니라 iOS 운영체제가 직접 그리는 화면이야. 어떤 앱이 Sign in with Apple을 쓰든 동일한 모양이 나오는 이유가 그거야.

> 💡 **iOS 시스템 (ASAuthorization)이 Apple 서버야?**
> 아니야. `ASAuthorization`은 아이폰 안에 내장된 프레임워크야. 외부 네트워크로 연결하는 서버가 아니라, iOS 운영체제 안에 이미 설치되어 있는 코드야. 카메라 앱이 iOS 카메라 프레임워크를 쓰는 것처럼, 우리 앱이 이 프레임워크를 불러서 쓰는 거야. iOS가 내부적으로 Apple 서버에 연결하는 건 맞지만, 그건 iOS 시스템이 알아서 처리하는 거고 우리 코드에는 안 보여.

---

**④ Face ID / Touch ID / 패스코드 인증**

③에서 팝업이 뜨면, ④에서 사용자가 실제로 인증을 완료한다. Face ID / Touch ID / 패스코드 중 어떤 걸 쓸지는 iOS 시스템이 기기 설정 보고 자동으로 판단해. 개발자가 코드로 지정할 수 없어.

③이 "팝업 표시", ④가 "인증 완료"로 흐름도에서 둘을 나눈 건 iOS 시스템이 팝업을 그리는 것과 인증을 처리하는 게 내부적으로 다른 단계이기 때문이야.

---

**⑤ idToken + rawNonce 반환**

인증이 끝나면 iOS 시스템이 Frank 앱에게 `idToken`을 돌려준다. `rawNonce`는 iOS 콜백이 반환하는 게 아니라, ①에서 앱이 직접 만들어 `@State private var currentNonce`에 보관해뒀던 값을 이 시점에 꺼내 쓰는 거야.

> 💡 **idToken과 rawNonce가 각각 뭐야?**
> - `idToken` — Apple이 "이 사람 맞아"라고 서명해서 만든 JWT야. 안에 `user_id`, 이메일, 그리고 ②에서 보낸 nonce 해시값이 들어있어.
> - `rawNonce` — Apple이 만든 게 아니야. ①에서 **우리 앱(Frank)이 직접 만든 랜덤 값**이야. Apple 콜백으로 idToken은 돌아오지만 rawNonce는 안 돌아와 — 앱이 `currentNonce`에 따로 보관해뒀다가 콜백 받은 후에 꺼내 쓰는 거야.

> 💡 **nonce — 왜 필요해?**
> Apple 로그인 후 돌아오는 idToken을 중간에 누가 가로채서 그대로 Supabase에 보내면 — Supabase는 "진짜 Apple이 발급한 토큰이네, 통과"해버려. nonce는 "이 토큰은 지금 이 요청을 위해 딱 한 번만 유효해"를 증명하는 장치야.
>
> **nonce 흐름 정리**
> 1. Frank 앱이 `rawNonce` 생성 → `currentNonce`에 보관, 아무데도 안 보냄
> 2. `SHA256(rawNonce)` 해시값만 Apple에 전송 — 원본은 끝까지 앱 안에만 있어
> 3. Apple이 idToken 발급 시 받은 해시값을 내부에 박아서 자기 서명 봉인 — `{ nonce: "a3f9c2...", user: "..." }`
> 4. ⑤에서 iOS 콜백으로 idToken만 돌아옴 → 앱이 `currentNonce`에서 rawNonce 원본을 꺼냄
> 5. ⑦에서 Supabase에 idToken + rawNonce 둘 다 전달 → Supabase가 직접 `SHA256(rawNonce)` 계산해서 idToken 안의 해시값과 비교 → 일치하면 통과
>
> 중간에 idToken을 누가 가로채도 rawNonce 원본을 모르면 Supabase 검증을 통과할 수 없어.

---

### ⑥ ~ ⑧ SupabaseAuthAdapter가 idToken을 Supabase JWT로 교환하기

**⑥ signInWithApple(idToken:, rawNonce:)**

⑤에서 idToken + rawNonce를 받은 Frank 앱이 `SupabaseAuthAdapter.signInWithApple(idToken:rawNonce:)`를 호출한다. 이메일 흐름에서 `signIn(email:password:)`를 호출한 것과 같은 위치야 — Frank 앱(SwiftUI)이 AuthPort를 통해 Adapter한테 "로그인 처리해줘"라고 넘기는 거야.

---

**⑦ signInWithIdToken(OpenIDConnectCredentials)**

`SupabaseAuthAdapter`가 내부에서 Supabase Swift SDK의 `signInWithIdToken()`을 호출한다. idToken + rawNonce를 `OpenIDConnectCredentials`라는 타입으로 감싸서 Supabase Auth에 보내는 거야.

> 💡 **OpenIDConnectCredentials가 뭐야?**
> Supabase SDK가 제공하는 타입이야. "Apple idToken이랑 rawNonce를 이 포맷으로 담아서 줘"라는 의미야. 이메일 흐름에서는 `signIn(email:password:)` 하나로 끝났는데, Apple 흐름은 Apple이 이미 인증을 끝내고 idToken을 줬으니까 Supabase는 그 idToken이 진짜 Apple 것인지만 확인하고 우리 앱 JWT를 발급해줘. 검증 주체가 달라.

---

**⑧ Supabase JWT(access_token) 반환**

Supabase가 idToken을 검증하고 나면 우리 앱 전용 JWT를 만들어서 돌려준다. 결국 iOS 시스템한테 받은 idToken + rawNonce를 Adapter(구현체)에 넣어서 Supabase SDK의 `signInWithIdToken()`을 호출하고, 성공하면 Supabase JWT를 받는 구조야. 우리가 직접 구현한 건 Adapter 안에서 SDK 함수를 호출하는 부분뿐이고, 실제 Apple 검증이랑 JWT 발급은 전부 Supabase가 처리해.

> 💡 **⑥~⑧을 한 줄로 정리하면?**
> iOS 시스템한테 받은 idToken + rawNonce → SupabaseAuthAdapter(구현체)에 넣음 → Supabase SDK가 이미 만들어둔 `signInWithIdToken()` 호출 → 성공하면 Supabase JWT 반환.

여기서부터는 이메일 흐름이랑 완전히 동일해. Supabase가 발급한 JWT가 있으면 이후 흐름은 로그인 방식에 상관없이 똑같아.

---

### ⑨ ~ ⑫ JWT 저장하고 프로필 가져오기

**⑨ 토큰 자동 저장 (SDK 내부 처리)**

⑧에서 받은 Supabase JWT를 SDK가 자동으로 iOS Keychain에 저장한다. 이메일 흐름 ⑤와 완전히 동일한 단계야 — 우리 코드에 저장하는 코드가 없어도 SDK가 내부에서 알아서 처리해.

---

**⑩ ~ ⑫ GET /api/me/profile → 검증 → 반환**

⑨ 저장이 끝나면 Supabase JWT를 Bearer 토큰으로 붙여서 Rust API에 프로필을 요청한다. Rust API가 Supabase에 토큰 유효성을 확인(⑪)하고 유효하면(⑫) profiles 테이블에서 프로필 데이터를 꺼내서 돌려줘. 이메일 흐름 ⑥~⑨와 완전히 동일해.

---

### 전체 흐름 한눈에 보기

이메일 흐름이랑 비교하면 딱 하나 차이야. 이메일은 `이메일+비밀번호 → Supabase`로 바로 갔는데, Apple은 중간에 iOS 시스템이 껴서 `버튼 탭 → iOS 시스템(인증) → idToken+rawNonce → Supabase` 순서가 추가된 거야. 그 이후 JWT 받고, Keychain 저장하고, Rust API 호출하는 건 전부 동일해.

```
① 버튼 탭
② iOS 시스템에 인증 요청 (권한: 이름·이메일, nonce)
③ 네이티브 팝업 표시
④ Face ID / Touch ID / 패스코드 인증
⑤ idToken + rawNonce 반환  ← 이메일에 없는 단계
⑥ SupabaseAuthAdapter.signInWithApple() 호출
⑦ signInWithIdToken(OpenIDConnectCredentials) → Supabase에 제출
⑧ Supabase JWT 반환
⑨ Keychain 자동 저장 (이메일과 동일)
⑩~⑫ Rust API 프로필 조회 (이메일과 동일)
```



