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
| **hooks.server.ts** | 서버에서 실행되는 SvelteKit 코드. 모든 페이지 요청을 가로채는 문지기. 쿠키 관리, 인증 처리 담당 |
| **Supabase Auth** | 로그인 검증 전담 서버. 비밀번호 확인, JWT 발급을 맡음 |
| **Rust API(Axum)** | 뉴스 가져오기, 프로필 저장 같은 실제 기능이 있는 서버 |

> 💡 SvelteKit은 화면(웹)과 서버(hooks.server.ts) 두 부분으로 나뉘어 있어서 흐름도에서 따로 표현된 것. 내가 보는 웹 화면이 곧 SvelteKit이 만들어서 뿌려준 결과물이야.

---

## 웹 이메일 로그인 전체 흐름

### ① ~ ⑤ 로그인하고 홈으로 이동하기까지

**① 이메일+비밀번호 제출**

User가 이메일과 비밀번호를 입력하고 로그인 버튼을 누르면 SvelteKit(웹)으로 전달된다.

---

**② signInWithPassword(email, password)**

SvelteKit(웹)이 `signInWithPassword()`를 실행해서 Supabase Auth로 전송한다.

이 함수는 Supabase SDK가 제공하는 함수로, 직접 구현한 게 아니라 가져다 쓰는 것이다. Rust API가 아닌 Supabase로 보내는 이유는, 비밀번호 검증 같은 걸 직접 구현하면 보안 구멍이 나기 쉬워서 전담 서버인 Supabase에 통째로 위임한 것이다.

---

**③ JWT(access_token) 반환**

Supabase Auth가 이메일과 비밀번호를 확인하면 **JWT**를 직접 만들어서 반환한다.

> 💡 **JWT란?** 신분증 같은 것. 안에 `user_id=abc123, 1시간 뒤 만료` 같은 정보가 암호화되어 들어있어. **Access Token**은 JWT의 다른 이름으로 "접근 허가증"이라는 뜻이야.

---

**④ httpOnly 쿠키에 JWT 저장**

hooks.server.ts가 반환된 JWT를 **httpOnly 쿠키**에 저장한다. 이후 브라우저는 모든 요청에 이 쿠키를 자동으로 첨부한다.

> 💡 **왜 localStorage가 아니고 쿠키야?**
> localStorage는 JS가 자유롭게 읽을 수 있어서 XSS 공격에 취약하다.
> XSS는 해커가 댓글 같은 곳에 악성 JS 코드를 심어서 토큰을 훔치는 공격인데,
> httpOnly 쿠키는 JS가 아예 접근 자체를 못 하도록 브라우저가 막아두기 때문에 훨씬 안전하다.

---

**⑤ redirect(303, '/')**

JWT 저장이 끝나면 hooks.server.ts가 SvelteKit(웹)으로 "홈으로 가" 신호(303)를 보내고, 브라우저가 홈 페이지로 자동 이동한다.

---

### ⑥ ~ ⑭ 로그인 이후 매 요청마다

**⑥ 페이지 요청**

User가 페이지를 열 때마다 브라우저는 ④에서 저장해둔 httpOnly 쿠키를 자동으로 붙여서 보낸다. User가 따로 뭔가 하는 게 아니라 브라우저가 알아서 첨부하는 것이다.

---

**⑦ safeGetSession() 호출**

모든 요청은 무조건 hooks.server.ts를 거치는데, 여기서 `safeGetSession()`으로 쿠키 속 JWT를 꺼낸다.

`safeGetSession()`은 Supabase SDK가 제공하는 함수다. 일반 `getSession()`은 쿠키에 JWT가 있으면 그냥 믿어버리는데, `safeGetSession()`은 "꺼내긴 했는데 진짜 유효한지는 Supabase에 따로 확인해야 해"라고 표시해둔다. 그 확인을 ⑧이 담당한다.

---

**⑧ getUser() — 서버에서 토큰 재검증**

꺼낸 JWT를 Supabase Auth에 보내서 "이 토큰 아직 유효해?" 직접 확인한다. `getUser()`도 Supabase SDK가 제공하는 함수다.

> 💡 **왜 매 요청마다 확인해?**
> hooks.server.ts가 모든 요청을 무조건 거치는 문지기이기 때문에,
> 지나가는 길목에서 자동으로 체크하는 구조다.
> JWT는 보통 1시간 뒤 만료되는데, 만료된 경우 자동으로 로그인 페이지로 튕겨낸다.

---

(⑨ ~ ⑭ 학습 계속 진행하면서 채워집니다)
