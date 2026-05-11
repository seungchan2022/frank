# Frank 클라우드 배포 — 진행 흐름 + 개념 노트
> 작성일: 2026-05-11

---

## 1. 시작 — 문제 발견

### 무슨 문제였냐

Frank 앱을 실제로 쓰려면 서버가 항상 켜져 있어야 한다. 그런데 서버가 내 맥북 위에서만 돌아가고 있었다. 맥북이 꺼지면 서버도 꺼지고, 앱도 먹통이 된다.

실제로 발생한 에러:

```
NSURLErrorDomain Code=-1001 "The request timed out."
http://macbook-pro.local:8080/api/tags
```

- `macbook-pro.local` — 집 공유기 안에서만 통하는 내 맥북 주소. 밖에서는 존재 자체를 모름
- `8080` — Rust 서버가 열어놓은 포트
- 타임아웃 이유 — Mac이 꺼져 있거나 같은 Wi-Fi가 아니면 연결 자체 불가

### 결론

서버를 맥북이 아닌 **24시간 켜져 있는 다른 컴퓨터**에 올려야 한다.

---

## 2. 플랫폼 리서치 — 어디에 올릴까

조건: 월 $0, 신용카드 불필요.

| 플랫폼 | 결과 | 이유 |
|---|---|---|
| Oracle Cloud | ❌ | ARM 리전 용량 부족, 19회 실패 |
| Koyeb | ❌ | 2026년 2월 Mistral 인수 후 신규 무료 가입 불가 |
| Google Cloud Run | ❌ | 카드 필수, 과금 캡 없음 |
| Fly.io | ❌ | 2024년 신규 가입자 무료 티어 종료 (기존 사용자 레거시 유지) |
| **Render** | ✅ | 무료, 카드 불필요, Docker 지원 |
| **Vercel** | ✅ | 무료, 카드 불필요, SvelteKit 완벽 지원 |

### 개념: 배포란 무엇인가

> 코드를 실행 가능한 상태로 어딘가에 올려두는 것.

기존엔 그 "어딘가"가 내 맥북이었다. 지금은 Render, Vercel 컴퓨터에 올린 것. 원리는 **남의 컴퓨터를 빌려 쓰는 것**이다.

### 개념: Render vs Vercel

Frank는 3개 부분으로 나뉜다:

```
서버 (Rust)      → 데이터 처리 엔진. DB 연결, API 응답
웹 (SvelteKit)   → 브라우저에서 여는 웹사이트
iOS 앱 (SwiftUI) → 아이폰 화면
```

웹과 앱은 "화면"이고, 서버는 "데이터를 주는 엔진"이다. 웹과 앱이 서버에 "태그 줘", "피드 줘" 요청하면 서버가 DB에서 꺼내서 답한다.

**Render** — 어떤 프로그램이든 24시간 실행시켜주는 서버 호스팅. Rust 서버처럼 "항상 켜서 요청 기다리는" 프로그램에 적합.

**Vercel** — 웹사이트 배포에 특화된 호스팅. SvelteKit 같은 웹 프레임워크를 빌드해서 전 세계에 빠르게 서빙. Rust 서버처럼 "항상 켜있는 프로세스"는 지원 안 함.

---

## 3. 아키텍처 변화

### Before

```
iPhone   → macbook-pro.local:8080 → Supabase DB
브라우저  → macbook-pro.local:5173

문제: Mac 꺼짐 = 앱 먹통
```

### After

```
iPhone ──┐
         ├→ frank-onvv.onrender.com (Render, 24시간)
브라우저 ─┘         └→ Supabase DB

frank-green.vercel.app (Vercel, 24시간)
  └→ frank-onvv.onrender.com

UptimeRobot → /health (5분마다, 슬립 방지)
```

Mac은 이제 코드 업데이트할 때만 필요. 평소엔 꺼도 앱이 잘 동작한다.

---

## 4. 코드 작업 — 헬스체크 수정

### 개념: `/health` 엔드포인트

서버에는 여러 주소(엔드포인트)가 있다:

```
/api/tags   → 태그 목록 반환
/api/feed   → 피드 반환
/health     → 서버 살아있는지 확인 ("ok" 반환)
```

`/health`는 서버가 정상 작동 중인지 확인하는 전용 주소. 가장 가벼운 응답.

### 개념: UptimeRobot

Render 무료 플랜은 **15분 동안 요청이 없으면 서버를 슬립**시킨다. 슬립 상태에서 깨어나는 데 30~50초 걸린다.

UptimeRobot은 5분마다 `/health`를 자동 호출해서 슬립을 방지하는 무료 모니터링 서비스. 사이트에서 URL 등록만 하면 알아서 호출해준다. 서버 다운 시 이메일 알림도 옴.

### 개념: SELECT 1 — 왜 추가했나

Supabase(DB)도 무료 플랜이라 **7일 동안 DB 쿼리가 없으면 DB를 정지**시킨다.

기존 `/health`는 DB를 전혀 안 건드렸다:

```
UptimeRobot → /health → "ok" 반환
                 (DB는 아무것도 안 함)
Supabase: "7일째 아무도 DB 안 쓰네 → 정지"
```

`SELECT 1`은 DB에 보내는 가장 가벼운 쿼리. 아무 데이터도 변경하지 않고 "DB야, 살아있어?" 확인만 한다. DB가 정상이면 `1`을 반환.

수정 후:

```
UptimeRobot → /health → SELECT 1 → DB
Supabase: "누군가 쓰고 있네 → 유지"
```

UptimeRobot 핑 하나로 두 가지를 동시에 해결:

| 문제 | 해결 |
|---|---|
| Render 15분 슬립 | 5분마다 `/health` 호출 |
| Supabase 7일 정지 | `/health`에서 `SELECT 1` 실행 |

---

## 5. Render 배포

GitHub 연동 → New Web Service → Runtime: Docker → 기존 `Dockerfile` 그대로 사용. 서버 코드 변경 없음.

환경변수는 대시보드에서 입력. DB URL, API 키 등 코드에 하드코딩 없이 안전하게 관리.

배포 완료 후 `curl https://frank-onvv.onrender.com/health` → 0.87초에 200 OK 확인.

---

## 6. Vercel 배포 + CORS 해결

### 배포

`svelte.config.js`에서 `adapter-node` → `adapter-vercel` 한 줄 교체. npm 패키지 하나 설치하면 끝.

### 개념: CORS

브라우저의 보안 정책. 다른 도메인으로 요청 보낼 때 서버한테 먼저 허락을 받아야 한다.

문제 상황:

```
웹: frank-green.vercel.app
서버: frank-onvv.onrender.com
→ 도메인이 다름 → 브라우저가 서버에 허락 요청
→ 서버: 설정 없음 → 거부 → CORS 에러
```

iPhone 앱은 브라우저가 아니라 CORS 자체가 없음. 영향 없음.

해결: Render 환경변수에 추가:

```
ALLOWED_ORIGINS=https://frank-green.vercel.app
```

"이 도메인에서 오는 요청은 내가 만든 웹이니까 허락"이라고 서버에 등록한 것.

---

## 7. iOS 설정 수정

### 개념: Secrets.plist가 뭐냐

직접 만들어서 프로젝트에 추가한 파일. `ios/Frank/Frank/Resources/Secrets.plist` 에 위치.

`.plist`는 Apple이 iOS/macOS에서 설정값을 저장할 때 쓰는 표준 파일 형식(XML 구조). 앱 코드에서 직접 열어서 읽어온다.

```xml
<key>SERVER_URL</key>
<string>https://frank-onvv.onrender.com</string>
```

현재 들어있는 값:
- `SUPABASE_URL` — Supabase 프로젝트 주소
- `SUPABASE_ANON_KEY` — Supabase 접근 키
- `SERVER_URL` — API 서버 주소

### 개념: Config.xcconfig가 뭐냐

마찬가지로 직접 만들어서 추가한 파일. `ios/Frank/Config.xcconfig` 에 위치.

Secrets.plist와 다른 점은 **누가 읽느냐**:

```
Config.xcconfig  → Xcode 빌드 시스템이 읽음 → Info.plist에 값을 주입
Secrets.plist    → 앱 코드가 실행 중에 직접 읽음
```

Xcode가 빌드할 때 Config.xcconfig를 읽어서 Info.plist에 값을 넣어준다. 앱 코드는 Config.xcconfig를 직접 열지 않고 Info.plist를 통해 값을 읽게 된다.

### 개념: 왜 둘 다 git에 올라가지 않냐

프로젝트가 **공개 저장소(Public)**이기 때문에 민감한 정보가 올라가면 누구나 볼 수 있다. 루트 `.gitignore`에 이미 제외 규칙이 등록돼 있다:

```
**/Secrets.plist
ios/Frank/Config.xcconfig
.env
.env.*
```

실제로 확인한 결과 두 파일 모두 git 추적 이력 없음. 과거 커밋에도 없음. ✅

> `SUPABASE_ANON_KEY`는 원래 클라이언트에서 써도 되는 공개 키라 당장 문제는 없지만, 습관적으로 민감 파일은 git에서 제외하는 게 맞다.

### 문제

`Secrets.plist`에서 SERVER_URL을 Render URL로 바꿨는데 여전히 `macbook-pro.local`로 요청이 가고 있었다.

### 원인: 설정 파일 우선순위

iOS에는 서버 주소를 읽는 설정 파일이 두 곳에 있다:

```
Config.xcconfig   ← 빌드 시 Info.plist에 주입 (우선순위 높음)
Secrets.plist     ← 앱이 직접 읽음 (우선순위 낮음)
```

앱이 값을 찾는 순서:
```
1. Info.plist (xcconfig가 주입한 값) ← 먼저 찾음
2. Secrets.plist                     ← 없으면 여기
```

`Secrets.plist`만 바꾸면 `Config.xcconfig`가 덮어써서 아무 효과가 없었던 것.

### 해결

`Config.xcconfig`에서 SERVER_URL 수정:

```
# 수정 전
SERVER_URL = http:$(SLASH)$(SLASH)MacBook-Pro.local:8080

# 수정 후
SERVER_URL = https:$(SLASH)$(SLASH)frank-onvv.onrender.com
```

> 주의: 이 파일은 git에 올라가지 않는 로컬 전용 파일. iOS 서버 주소 변경 시 반드시 두 파일 모두 확인 필요.

---

## 8. 자동 재배포 파이프라인

### 개념: Webhook

GitHub에 특정 이벤트(push 등)가 발생했을 때 지정한 주소로 자동 알림을 보내는 기능.

Render, Vercel에 GitHub 계정 연동하면 두 서비스가 자동으로 Webhook을 등록해준다. 별도 설정 불필요.

```
코드 수정 → git push (main)
→ GitHub → Render에 알림 → 자동 빌드 + 배포
→ GitHub → Vercel에 알림 → 자동 빌드 + 배포
```

iOS만 예외. Xcode에서 직접 빌드 후 설치 필요.

---

## 9. 검증

| 항목 | 결과 |
|---|---|
| iOS 실기기 피드 로드 | ✅ |
| 웹 브라우저 로그인 + 피드 | ✅ |
| Mac 덮은 상태(Sleep)에서 앱 동작 | ✅ |
| UptimeRobot 모니터 활성화 | ✅ |
| 자동 재배포 파이프라인 | ✅ |
| 서버 응답 시간 | ~0.9초 |
| 월 비용 | $0 |

---

## 10. 최종 결과

- Mac 의존성 완전 제거
- 언제 어디서나 iPhone + 브라우저로 접근 가능
- 월 $0 운영비
- GitHub push 한 번으로 자동 배포
