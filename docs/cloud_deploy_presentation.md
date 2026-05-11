# Frank 클라우드 배포 발표 자료
> 작성일: 2026-05-11

---

## Slide 1 — 제목

# Frank 앱, 클라우드에 올리다
### Mac 없이도 언제 어디서나 쓸 수 있는 앱으로

---

## Slide 2 — 배포 전 문제

### 문제: Mac이 꺼지면 앱도 꺼진다

- 기존 구조: 서버가 **내 MacBook 위에서만** 실행
- iOS 앱이 집 Mac에 붙어 있어야 동작
- Mac 꺼짐 = 앱 먹통

> "실제로 쓸 수 있는 앱이 아니었다"

---

## Slide 3 — 목표

### 목표: $0로 항상 켜져 있는 앱

| 조건 | 내용 |
|------|------|
| 비용 | 월 $0 (무료 플랜만 사용) |
| 가용성 | Mac 꺼져도 앱 동작 |
| 플랫폼 | iOS 실기기 + 웹 브라우저 |

---

## Slide 4 — 전체 구조 (Before/After)

### Before
```
iPhone → (집 MacBook:8080) → DB
```

### After
```
iPhone ──┐
         ├→ Render (API 서버) → Supabase DB
웹 브라우저 ─┘
              ↑
         UptimeRobot (5분마다 깨우기)
```

---

## Slide 5 — 사용한 서비스 3가지

### 모두 무료, 카드 불필요

| 서비스 | 역할 | 비용 |
|--------|------|------|
| **Render** | API 서버 (Rust) 호스팅 | $0 |
| **Vercel** | 웹 (SvelteKit) 호스팅 | $0 |
| **UptimeRobot** | 서버 슬립 방지 모니터링 | $0 |

---

## Slide 6 — Render: API 서버 배포

### Rust 서버를 Docker로 배포

- GitHub 연동 → main 브랜치 push 시 **자동 재배포**
- 무료 티어: 월 750시간, 15분 비활성 시 슬립
- Singapore 리전 선택 (아시아 응답 최적)
- 환경변수로 DB 연결 정보·API 키 안전하게 관리

**기술 포인트:** 서버 코드에 `/health` 엔드포인트 추가  
→ DB 연결까지 확인하는 헬스체크

---

## Slide 7 — Vercel: 웹 배포

### SvelteKit 앱을 Vercel에 배포

- GitHub 연동 → 자동 배포
- `adapter-vercel` 한 줄 교체로 배포 완료
- HTTPS 자동 적용, CDN 포함
- 배포 URL: `frank-green.vercel.app`

**문제 해결:** 웹→API 요청 시 CORS 오류 발생  
→ 서버 환경변수 `ALLOWED_ORIGINS`에 Vercel 도메인 추가로 해결

---

## Slide 8 — UptimeRobot: 슬립 방지

### 5분마다 서버를 두드려 깨운다

- Render 무료 티어는 15분 미사용 시 슬립
- UptimeRobot이 5분마다 `/health` 호출 → 슬립 방지
- 서버 다운 시 이메일 알림도 자동 발송
- 부가 효과: Supabase DB 연결도 유지

---

## Slide 9 — iOS 실기기 연동

### iPhone에서 Render 서버로 직접 연결

**수정한 것:**
- `Config.xcconfig`의 `SERVER_URL`  
  `http://MacBook-Pro.local:8080` → `https://frank-onvv.onrender.com`

**문제 해결 과정:**
- 처음엔 `Secrets.plist`만 수정 → 실제론 `Config.xcconfig`가 우선 적용됨
- 콘솔 로그로 여전히 `macbook-pro.local`로 요청하는 것 확인
- `Config.xcconfig` 수정 후 정상 동작

---

## Slide 10 — 검증 결과

### 실기기 E2E 검증 완료

| 검증 항목 | 결과 |
|----------|------|
| iOS 실기기 피드 로드 | ✅ |
| 웹 브라우저 로그인 + 피드 | ✅ |
| Mac 덮은 상태에서 앱 동작 | ✅ |
| 서버 응답 시간 | ~0.9초 |
| 월 비용 | $0 |

---

## Slide 11 — 배포 파이프라인 정리

### 앞으로 코드 업데이트 방법

```
코드 수정 → main 브랜치 push
              ↓
    Render 자동 감지 → API 서버 재배포
    Vercel 자동 감지 → 웹 재배포
```

별도 배포 작업 없음. push 한 번으로 끝.

---

## Slide 12 — 마무리

### 결과

- **Mac 의존성 완전 제거**
- 언제 어디서나 iPhone + 브라우저로 접근 가능
- 월 $0 운영비
- GitHub push 한 번으로 자동 배포

> "개인 프로젝트를 실제 서비스처럼 운영할 수 있게 됐다"
