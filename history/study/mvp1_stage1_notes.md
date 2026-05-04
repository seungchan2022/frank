# MVP1 1단계: 로그인 — 학습 노트

> 세션 시작: 2026-04-22 / 재개: 2026-04-30
> 기준 흐름도: history/study/login_flow/index.html
> 학습 목표: 전체 흐름 파악 + 흐름 속 개념 자연스럽게 익히기

---

## 등장인물

흐름도에 나오는 이름들이 각각 뭔지 먼저 알아두면 읽기 편해.

| 이름 | 역할 |
|---|---|
| **User** | 사용자가 보는 브라우저 창 |
| **SvelteKit(웹)** | 브라우저에서 실행되는 JS 코드. 화면을 그리고 버튼 클릭 같은 이벤트를 처리함 |
| **+page.server.ts** | 서버에서 실행되는 SvelteKit 서버 액션. 폼 제출을 받아서 `signInWithPassword()` 같은 Supabase SDK 함수를 실행함 |
| **hooks.server.ts** | 서버에서 실행되는 SvelteKit 코드. 모든 페이지 요청을 가로채는 문지기. 쿠키 관리, 인증 처리 담당 |
| **Supabase Auth** | 로그인 검증 전담 서버. 비밀번호 확인, JWT 발급을 맡음 |
| **Rust API(Axum)** | 뉴스 가져오기, 프로필 저장 같은 실제 기능이 있는 서버 |

> 💡 SvelteKit은 화면(웹), 서버 액션(`+page.server.ts`), 문지기(`hooks.server.ts`) 세 부분으로 나뉘어 있어서 흐름도에서 따로 표현된 것. User는 내가 보는 브라우저 화면 자체를 가리키고, SvelteKit(웹)은 그 화면을 구성하는 JS 코드야.

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

> 💡 **JWT란?** 신분증 같은 것. 안에 `user_id=abc123, 1시간 뒤 만료` 같은 정보가 들어있어. **암호화가 아니라 서명**된 것이라 누구나 디코딩해서 읽을 수 있지만, Supabase가 서명해서 위변조를 막는 구조야. 그래서 JWT에 비밀번호 같은 민감한 정보는 안 담아. **Access Token**은 JWT의 다른 이름으로 "접근 허가증"이라는 뜻이야.

---

**④ httpOnly 쿠키에 JWT 저장**

`+page.server.ts`에서 `locals.supabase`(hooks.server.ts에서 쿠키 처리가 설정된 Supabase SSR 클라이언트)가 JWT를 **httpOnly 쿠키**에 자동으로 저장한다. 이후 브라우저는 모든 요청에 이 쿠키를 자동으로 첨부한다.

> 💡 **쿠키란?**
> 브라우저가 기억해두는 메모지. 웹사이트가 "이 사람 로그인 됐어"를 기억하려면 어딘가에 저장해야 하는데 그게 쿠키야. 브라우저가 서버에 요청할 때 자동으로 붙여서 보내줘서 매번 다시 로그인 안 해도 되는 것이다.

> 💡 **httpOnly 쿠키란?**
> 일반 쿠키에 특수 규칙이 붙은 것. **JS(자바스크립트)가 아예 읽을 수 없도록** 브라우저가 차단해둔 쿠키야. `document.cookie`로 읽으려 해도 보이지 않아. 서버랑 브라우저 사이에서만 오가는 비밀 메모지라고 생각하면 돼.
>
> 이름에 **HTTP**가 붙은 이유는, 이 쿠키가 HTTP 통신(브라우저 ↔ 서버)에서만 사용되고 JS에서는 접근할 수 없다는 뜻이야. 즉, "HTTP로만 쓸 수 있는 쿠키" = httpOnly 쿠키.

> 💡 **localStorage란?**
> 브라우저 안에 있는 메모장. JS가 자유롭게 읽고 쓸 수 있어서 편리하지만, 그게 단점이기도 해. 해커가 JS를 심어놓으면(XSS) 여기 있는 토큰을 그대로 훔쳐갈 수 있거든.

> 💡 **XSS란?**
> Cross-Site Scripting의 줄임말. 해커가 댓글 같은 입력창에 `<script>토큰 훔치는 코드</script>`를 심어두면, 다른 사람이 그 페이지를 열었을 때 악성 코드가 실행되면서 localStorage의 토큰이 해커한테 전송되는 공격이야. httpOnly 쿠키는 JS가 접근 자체를 못 하니까 XSS가 성공해도 토큰을 훔칠 수 없어.

---

**⑤ redirect(303, '/')**

JWT 저장이 끝나면 `+page.server.ts`가 `redirect(303, '/')`를 반환하고, 브라우저가 홈 페이지로 자동 이동한다.

---

### ⑥ ~ ⑭ 로그인 이후 매 요청마다

**⑥ 페이지 요청**

User가 페이지를 열 때마다 브라우저는 ④에서 저장해둔 httpOnly 쿠키를 자동으로 붙여서 보낸다. User가 따로 뭔가 하는 게 아니라 브라우저가 알아서 첨부하는 것이다.

---

**⑦ safeGetSession() 호출**

모든 요청은 무조건 hooks.server.ts를 거치는데, 여기서 `safeGetSession()`으로 쿠키 속 JWT를 꺼낸다.

`safeGetSession()`은 `hooks.server.ts`에 직접 정의해서 쓰는 커스텀 헬퍼다. Supabase 공식 SSR 가이드가 권장하는 패턴인데, 내부에서 SDK의 `getSession()`(쿠키에서 JWT 꺼내기) + `getUser()`(Supabase에 재검증) 두 개를 묶어서 한 번에 처리하도록 만든 함수야.

---

**⑧ getUser() — 서버에서 토큰 재검증**

꺼낸 JWT를 Supabase Auth에 보내서 "이 토큰 아직 유효해?" 직접 확인한다. `getUser()`도 Supabase SDK가 제공하는 함수다.

> 💡 **왜 매 요청마다 확인해?**
> hooks.server.ts가 모든 요청을 무조건 거치는 문지기이기 때문에,
> 지나가는 길목에서 자동으로 체크하는 구조다.
> JWT는 보통 1시간 뒤 만료되는데, 만료된 경우 자동으로 로그인 페이지로 튕겨낸다.

---

**⑨ 유효한 user 반환**

Supabase Auth가 토큰을 확인하고 "유효해"라고 응답하면, 해당 유저 정보(user_id, 이메일 등)를 hooks.server.ts로 돌려준다.

---

**⑩ session + user → event.locals 저장**

hooks.server.ts가 받은 유저 정보를 `event.locals`에 저장한다.

> 💡 **event.locals란?**
> 한 번의 요청 안에서 데이터를 공유하는 임시 공간이야. hooks.server.ts가 유저 정보를 여기다 넣어두면, 같은 요청을 처리하는 다른 서버 코드들이 꺼내서 쓸 수 있어. 요청이 끝나면 사라지는 임시 메모장이라고 생각하면 돼.

---

**⑪ GET /api/... (Bearer 토큰 첨부)**

실제 데이터가 필요하면 SvelteKit 서버가 Rust API로 요청을 보낸다. 이때 JWT를 **Bearer 토큰** 형태로 붙여서 보낸다.

> 💡 **Bearer 토큰이란?**
> HTTP 요청 헤더에 토큰을 붙여서 보내는 방식이야. `Authorization: Bearer <JWT>` 형태로 보내는데, "이 토큰을 가진 사람한테 접근을 허용해줘"라는 뜻이야. Bearer는 "소지자"라는 뜻.

---

**⑫ /auth/v1/user 검증 → ⑬ 유효 확인**

Rust API의 `require_auth()` 미들웨어가 받은 Bearer 토큰을 Supabase Auth에 보내서 검증한다. `require_auth()`는 직접 구현한 Rust 코드야.

> 💡 **로그인은 Supabase가 하는데, Rust API가 또 검증하는 이유가 뭐야?**
> 역할이 달라. Supabase는 "이 사람이 회원 맞아? 비밀번호 맞아?"를 확인하고 JWT를 발급하는 역할이고, Rust API는 "이 토큰이 지금도 유효해?"만 확인하는 역할이야.
>
> 비유하면 이래:
> ```
> Supabase  →  여권 발급소 (신원 확인 후 여권 만들어줌)
> Rust API  →  공항 입국심사 (여권이 유효한지만 확인)
> ```
>
> Rust API가 직접 검증하는 이유는, 누군가 SvelteKit을 거치지 않고 Rust API로 직접 요청을 보낼 수도 있기 때문이야. Rust API 입장에선 요청이 어디서 왔는지 알 수 없으니까, 무조건 스스로 Supabase에 확인하는 거야.
>
> 그리고 JWT를 **만든 곳이 Supabase**니까, 유효한지 확인하는 것도 **Supabase한테 직접 물어보는 게 가장 확실**해 — 발급한 곳이 가장 정확한 정보를 갖고 있으니까.

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

### 등장인물 추가

| 이름 | 역할 |
|---|---|
| **+page.server.ts** | 서버에서 실행되는 SvelteKit 서버 액션. 폼 제출을 받아서 `signInWithOAuth()` 같은 Supabase SDK 함수를 실행함 |
| **Apple OAuth** | Apple의 로그인 인증 서버. 사용자가 Apple 계정으로 본인임을 인증해줌 |
| **/auth/callback** | 우리 앱 안에 있는 SvelteKit 페이지 경로. Apple 인증 후 브라우저가 돌아오는 도착지 |

> 💡 **OAuth란?** 비밀번호를 우리 앱에 직접 안 주고, 제3자(여기선 Apple)가 대신 신원을 보증해주는 방식이야. Apple이 비밀번호 관리를 통째로 담당하고, 우리 앱은 Apple로부터 "이 사람 맞아" 확인서만 받는 거야.

### ① ~ ⑩ 로그인하고 홈으로 이동하기까지

**① Apple로 로그인 클릭**

User가 "Apple로 계속하기" 버튼을 누르면 **form POST 요청**이 `+page.server.ts`의 `appleOAuth` 서버 액션으로 전달된다.

> 💡 **form POST란?**
> HTML에서 데이터를 서버로 보내는 가장 기본적인 방법이야. JS 없이도 브라우저가 알아서 서버에 POST 요청을 보내줘.
>
> ```html
> <form method="POST" action="?/appleOAuth">
>   <button type="submit">Apple로 계속하기</button>
> </form>
> ```
>
> - `method="POST"` — 서버로 데이터를 보내는 방식. GET은 URL에 데이터가 노출되고, POST는 본문에 담아서 보내.
> - `action="?/appleOAuth"` — 어떤 서버 액션을 실행할지 지정.
>
> 버튼 클릭 → 브라우저가 POST 요청 → SvelteKit 서버 액션 실행. 이메일 로그인도 같은 구조야. 이메일은 form에 입력값(이메일, 비밀번호)이 담겨서 전송되고, Apple은 입력값 없이 "Apple 로그인 해줘"라는 신호만 보내는 차이야.

---

**② signInWithOAuth({provider: 'apple'}) → URL 반환**

`+page.server.ts`의 `appleOAuth` 서버 액션이 실행된다. 여기서 `supabase.auth.signInWithOAuth()`를 호출하면, Supabase가 "Apple 로그인 페이지로 가는 URL"을 만들어서 `data.url`로 돌려준다.

이 URL에는 앱 식별 정보와 인증 후 돌아올 주소(`/auth/callback`)가 담겨 있다. `scopes: 'email name'`은 Apple에 "이메일과 이름 정보를 허용해줘"라고 요청하는 권한 목록이야.

> 💡 **SvelteKit 서버에서 실행된다는 게 중요해.** 이메일 흐름도 `signInWithPassword()`가 서버에서 실행됐던 것처럼, Apple 흐름도 서버 액션에서 처리해. 브라우저(클라이언트 JS)가 직접 Supabase를 부르는 게 아니야.

---

**③ Apple 서버로 redirect(302)**

서버가 `data.url`을 받아서 브라우저에게 `redirect(302, data.url)` 응답을 보낸다. 브라우저가 이 응답을 받아서 Apple 로그인 페이지로 통째로 이동한다 — Safari에서 Apple 로그인 화면이 뜨는 거야.

> 이게 이메일 로그인과의 핵심 차이야. 이메일은 `+page.server.ts`가 Supabase에 직접 요청을 보내고 끝났는데, Apple OAuth는 서버가 브라우저에게 "Apple로 갔다 와"라고 위임해. 브라우저가 Apple 서버로 직접 이동하는 과정이 중간에 끼어 있어.
>
> 이메일 흐름에서는 `+page.server.ts → Supabase` 순서였는데, Apple OAuth는 중간에 `/auth/callback`이 끼어서 순서가 달라 보이는 게 그 이유야. 두 흐름 모두 **JWT가 도착하면 `locals.supabase`(SSR 클라이언트)가 쿠키에 저장하는 역할**은 똑같아 — 이메일에서는 `+page.server.ts`에서, Apple OAuth에서는 `/auth/callback`에서 일어나는 거지.

---

**④ Apple 계정으로 인증 완료**

User가 Apple 로그인 화면에서 Face ID나 Apple ID/비밀번호로 인증을 마치면, Apple이 "이 사람 맞아"라고 확인해준다.

---

**⑤ /auth/callback?code=... 로 redirect**

Apple이 인증을 마치고 나서, 브라우저를 우리 앱의 `/auth/callback` 주소로 다시 보내준다. 이때 URL 뒤에 `?code=...` 형태로 **authorization code**가 붙어 온다.

> 💡 **authorization code란?**
> Apple이 "인증은 됐어, 대신 이 코드로 토큰을 받아가"라고 주는 일회용 교환권이야. 토큰 자체를 URL에 직접 넣으면 주소창에 노출되니까, 코드만 먼저 주고 나중에 서버끼리 교환하는 방식을 써.

---

**⑥ code 전달**

브라우저가 `/auth/callback` 페이지에 도착하면, 이 페이지가 URL에서 code를 꺼내서 Supabase에 전달한다.

---

**⑦ exchangeCodeForSession(code)**

`/auth/callback`이 Supabase SDK의 `exchangeCodeForSession()` 함수를 호출한다. Supabase가 이 code를 Apple 서버에 보내서 "이 code 진짜야? 그러면 토큰 줘"라고 교환한다.

---

**⑧ JWT(access_token) 반환**

Supabase가 Apple에서 검증을 마치고 JWT를 만들어서 돌려준다. 여기서부터는 이메일 로그인이랑 똑같다.

---

**⑨ httpOnly 쿠키에 JWT 저장 → ⑩ 홈으로 redirect(303, '/')**

`/auth/callback`에서 `locals.supabase`(SSR 클라이언트)가 JWT를 httpOnly 쿠키에 자동으로 저장하고, `redirect(303, '/')`로 홈으로 보내준다. JWT가 도착하면 SSR 클라이언트가 쿠키에 저장하는 흐름은 이메일 흐름의 ④⑤와 동일하다.

---

### ⑪ ~ ⑮ 로그인 이후 매 요청마다

이메일 흐름 ⑥~⑩과 완전히 동일하다. `safeGetSession()` → `getUser()` → `event.locals` 저장 순서로 동작한다.

Rust API 호출도 이메일과 동일하게 Bearer 토큰을 붙여서 요청하며, `require_auth()` 미들웨어가 Supabase에 검증한다.

---

### 전체 흐름 한눈에 보기

```
① 클릭 → form POST 요청 → SvelteKit 서버 액션 진입
② 서버에서 signInWithOAuth() → Supabase로부터 Apple URL 받음
③ 서버가 redirect(302, Apple URL) → 브라우저가 Apple 서버로 이동 (이메일과의 핵심 차이)
④ User가 Apple에서 인증 (Face ID / Apple ID)
⑤ Apple → /auth/callback?code=... 으로 redirect
⑥ /auth/callback 서버가 URL에서 code 꺼냄
⑦ exchangeCodeForSession(code) → Supabase가 Apple에 코드 교환
⑧ JWT 반환
⑨ httpOnly 쿠키 저장
⑩ 홈으로 redirect
⑪~⑮ 매 요청마다 동일 (이메일과 동일)
```

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

⑤에서 SDK는 Keychain에 JWT를 먼저 저장하고, 완료되면 `supabaseSession(accessToken 포함)`을 Adapter 코드로 반환한다.

```
SDK 내부 순서:
  → Keychain에 JWT 저장 완료 (앱 재시작 후에도 로그인 유지하려고 보관)
  → supabaseSession(accessToken 포함)을 Adapter 코드로 반환
```

Adapter는 SDK가 돌려준 `supabaseSession.accessToken`을 꺼내서 Rust API의 `/api/me/profile`에 요청을 보낸다. Keychain에서 다시 꺼내는 게 아니야. 실제 코드로 보면 딱 이렇게 돼:

```swift
let supabaseSession = try await client.auth.signIn(...)   // ③④⑤ 여기서 다 처리
fetchServerProfile(token: supabaseSession.accessToken, ...)  // ⑥ 반환값에서 바로 꺼냄
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
> - `idToken` — Apple이 "이 사람 맞아"라고 서명해서 만든 JWT야. 안에 `user_id`, `이메일`, 그리고 ②에서 보낸 nonce 해시값이 들어있어.
> - `rawNonce` — Apple이 만든 게 아니야. ①에서 **우리 앱(Frank)이 직접 만든 랜덤 값**이야. iOS 콜백(`ASAuthorizationAppleIDCredential`)에는 rawNonce 필드가 없어. 앱이 `currentNonce`에 따로 저장해뒀다가 콜백 받은 후에 꺼내 쓰는 거야.

> 💡 **nonce 흐름이 헷갈려. 정리해줘.**
> 1. Frank 앱이 `rawNonce` 생성 (예: "abc123") → `currentNonce`에 보관
> 2. `SHA256(rawNonce)` 해시값만 Apple에 보냄 — 원본은 안 보내
> 3. Apple이 idToken 안에 그 해시값을 박아서 서명해줌
> 4. ⑤에서 iOS 콜백으로 idToken만 돌아옴 → 앱이 `currentNonce`에서 rawNonce를 꺼냄
> 5. ⑦에서 Supabase에 idToken + rawNonce 전달 → Supabase가 "idToken 안의 해시값 == SHA256(rawNonce) 맞아?" 검증
>
> 이 구조의 핵심은 idToken을 중간에 누가 가로채도 rawNonce를 모르면 Supabase 검증에서 막힌다는 거야. idToken + rawNonce 두 개를 동시에 갖고 있어야만 로그인이 완료돼.

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


