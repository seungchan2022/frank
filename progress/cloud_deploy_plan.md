# 클라우드 배포 계획 — Mac 없이 실사용 가능

> 작성일: 2026-05-11
> 최종 갱신: 2026-05-11
> 상태: planning
> 목표: Mac을 켜지 않아도 iPhone + 웹 브라우저에서 항상 앱을 사용할 수 있도록 클라우드 배포 완성.
> 배경: MVP13 M3(Oracle Cloud 배포)가 ARM 용량 부족으로 deferred됨. 이후 Koyeb도 2026년 2월 Mistral 인수로 신규 무료 가입 불가. Render로 최종 확정.
> 비용: $0

---

## 배포 후 구조

```
지금:
  iPhone   → API 요청 → Mac 로컬 서버 (Mac 꺼지면 끊김)
  브라우저 → Mac 로컬 웹  (Mac 꺼지면 끊김)

배포 후:
  iPhone   → API 요청 → Render 서버  (항상 켜짐)
  브라우저 → Vercel 웹  (항상 켜짐)
  Mac      → Xcode 빌드 → iPhone 설치  (앱 업데이트 시에만 필요)
```

---

## 플랫폼

| 레이어 | 플랫폼 | 코드 변경 | 비고 |
|--------|--------|-----------|------|
| Rust API 서버 | **Render** | 없음 | 기존 Dockerfile 그대로, 무료, 카드 불필요 |
| SvelteKit 웹 | **Vercel** | 1줄 | adapter-node → adapter-vercel 교체 |
| DB + Auth | Supabase | 없음 | 이미 클라우드 |
| iOS 앱 | — | 1줄 | Secrets.plist SERVER_URL 교체 후 Xcode 실기기 빌드 |

### Render 무료 티어
- 신용카드 불필요
- Docker 컨테이너 직접 지원, 코드 변경 없음
- 750시간/월 (한 달 = 744시간, 서비스 1개 기준 딱 맞음)
- 슬립: 15분 비활성 시 자동 슬립 → UptimeRobot으로 방지
- 한도 초과 시 과금이 아닌 서비스 중단 (카드 없으면 과금 불가)
- 리전: Singapore 권장 (한국에서 가장 가까움, ~80ms)

### Vercel 무료 Hobby 플랜
- 신용카드 불필요
- 월 100GB 대역폭, 서버리스 함수 100만 회
- 한도 초과 시 과금이 아닌 서비스 차단
- GitHub push → 자동 배포

### iOS 배포 방식
- Xcode에서 실기기에 직접 빌드 (유료 Apple Developer 계정 기준 1년 유효)
- TestFlight 불필요 — 혼자 쓰는 앱이므로 직접 빌드로 충분
- 1년에 한 번 Mac에서 재빌드 필요

---

## 작업 순서

### Step 1 — Render: Rust API 서버 배포

**사전 준비**: 없음. 기존 `server/Dockerfile` 그대로 사용.

1. render.com 가입 (신용카드 불필요)
2. "New Web Service" → GitHub 연결 → `server/` 경로
3. Runtime: Docker 선택, Dockerfile 자동 감지 확인
4. Region: **Singapore (Southeast Asia)** 선택 (한국에서 가장 가까움)
5. 아래 환경변수 입력

```
DATABASE_URL=
SUPABASE_URL=
SUPABASE_ANON_KEY=
SUPABASE_JWT_SECRET=
TAVILY_API_KEY=
EXA_API_KEY=
FIRECRAWL_API_KEY=
GROQ_API_KEY=
PORT=8080
ALLOWED_ORIGINS=https://{vercel-domain}  ← Step 2 완료 후 추가
```

6. 배포 → URL 발급 (`https://xxx.onrender.com`)
7. 동작 확인:
```bash
curl https://xxx.onrender.com/health
```

---

### Step 2 — Vercel: SvelteKit 웹 배포

**코드 변경**: `web/svelte.config.js` adapter 1줄 교체

```bash
cd web
npm install -D @sveltejs/adapter-vercel
```

`web/svelte.config.js`:
```js
// 변경 전
import adapter from '@sveltejs/adapter-node';
// 변경 후
import adapter from '@sveltejs/adapter-vercel';
```

1. vercel.com 가입 + GitHub 연결
2. "Import Project" → Root Directory: `web/`
3. 아래 환경변수 입력

```
VITE_RUST_API_URL=https://xxx.onrender.com   ← Step 1에서 발급된 URL
PUBLIC_SUPABASE_URL=
PUBLIC_SUPABASE_ANON_KEY=
```

4. 배포 → URL 발급 (`https://frank-xxx.vercel.app`)
5. Step 1로 돌아가 Render `ALLOWED_ORIGINS`에 Vercel 도메인 추가

---

### Step 3 — UptimeRobot: 슬립 방지 + Supabase 유지

슬립 방지와 Supabase 7일 비활성 정지 방지를 동시에 처리.

**주의**: 현재 `/health` 엔드포인트는 DB를 조회하지 않음.
→ UptimeRobot 핑만으로는 Supabase keep-alive가 안 됨.
→ 배포 전 `/health` 엔드포인트에 DB ping (`SELECT 1`) 추가 필요.

#### health 엔드포인트 DB ping 추가
`server/src/api/health.rs`에 DB 조회 추가 (경량 `SELECT 1`).
→ UptimeRobot이 `/health`를 5분마다 호출하면 Supabase도 자동 유지됨.

#### UptimeRobot 설정
1. uptimerobot.com 가입 (무료)
2. "Add New Monitor"
   - Monitor Type: HTTP(s)
   - URL: `https://xxx.onrender.com/health`
   - Monitoring Interval: **5 minutes**
3. 설정 완료 → Render 슬립 없음 + Supabase 7일 정지 없음

---

### Step 4 — iOS: 서버 URL 전환 + 실기기 빌드

**코드 변경**: `Secrets.plist`의 `SERVER_URL` 값 교체

`ios/Frank/Frank/Secrets.plist`:
```xml
<key>SERVER_URL</key>
<string>https://xxx.onrender.com</string>   ← Render URL로 교체
```

> `ServerConfig.swift` 참고: 실기기는 `Info.plist` → `Secrets.plist` 순으로 `SERVER_URL`을 읽는다.
> 시뮬레이터는 localhost 고정이므로 변경 불필요.

1. Secrets.plist SERVER_URL 교체
2. Xcode → iPhone 빌드 + 설치
3. **Mac을 끈 상태에서** 실기기 E2E 검증:
   - [ ] 로그인
   - [ ] 피드 로딩
   - [ ] 기사 상세
   - [ ] 퀴즈

---

## 완료 기준

| 항목 | 확인 방법 |
|------|-----------|
| iPhone에서 로그인 성공 | 실기기 직접 확인 |
| 피드 로딩 성공 | 실기기 직접 확인 |
| Mac 꺼진 상태에서 앱 동작 | Mac 종료 후 iPhone으로 확인 |
| 웹 브라우저에서 접속 성공 | Vercel URL로 브라우저 확인 |
| 슬립 없이 즉시 응답 | UptimeRobot 모니터링 확인 |
| 배포 비용 $0 | Render/Vercel 대시보드 확인 |

---

## 검토한 플랫폼 기록

| 플랫폼 | 결과 | 사유 |
|--------|------|------|
| Oracle Cloud | ❌ | ARM 리전 용량 부족 (춘천 19회 실패) |
| Koyeb | ❌ | 2026년 2월 Mistral 인수, 신규 무료 가입 불가 |
| Google Cloud Run | ❌ | 카드 필수 + 하드 과금 캡 없음 + 리전 함정 |
| Fly.io | ❌ | 2024년 무료 티어 완전 폐지 |
| **Render** | ✅ | 무료, 카드 불필요, 즉시 배포, Oracle형 함정 없음 |
| **Vercel** | ✅ | 무료, 카드 불필요, SSR 지원 |
