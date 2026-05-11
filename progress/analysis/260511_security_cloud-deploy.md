# 보안 분석: 클라우드 배포 환경 — API 키 및 민감 변수

> 분석일: 2026-05-11  
> 유형: security  
> 대상: Render 환경변수, .env 파일, git 히스토리, web 환경변수, iOS Secrets.plist, CORS 설정  
> 상태: 완료

---

## 총평

**즉각 차단 위험: 없음.** git 히스토리에 실제 API 키 노출 없음. service_role 키 누출 없음. iOS Secrets.plist는 gitignore 처리됨.

**배포 전 반드시 수정해야 할 사항: 2건.** 배포 계획 환경변수 누락(서버 패닉 위험)과 E2E 주석 크리덴셜 노출이 실제 배포를 막을 수 있는 항목.

---

## 발견 사항

### F-01 [HIGH] 배포 계획 환경변수 누락 — 서버 첫 부팅 패닉 위험

**파일**: `progress/cloud_deploy_plan.md` (Step 1, 69~80행)  
**증거**: `server/src/config/mod.rs`의 `AppConfig::from_env()`는 아래 8개 변수에 `.expect()`를 사용. 미설정 시 프로세스 즉시 종료.

| 필수 변수 | 배포 계획 포함 여부 |
|-----------|:------------------:|
| `SUPABASE_URL` | ✅ |
| `SUPABASE_ANON_KEY` | ✅ |
| `SUPABASE_JWT_SECRET` | ✅ |
| `DATABASE_URL` | ✅ |
| `TAVILY_API_KEY` | ✅ |
| `EXA_API_KEY` | ✅ |
| `FIRECRAWL_API_KEY` | ✅ |
| `GROQ_API_KEY` | ✅ |

위 8개는 배포 계획에 포함됨. **아래 2개는 누락**:

- `OPENROUTER_API_KEY` — `server/.env`에 존재하지만 현재 `AppConfig`나 소스에서 사용되지 않음. 죽은 변수. 배포 시 Render에 등록하지 않아도 패닉 없음. **단, 이전 MVP에서 실제 사용했다가 코드 제거 후 .env에 잔류한 경우 확인 필요.**
- `LLM_MODEL` — 마찬가지로 소스에서 직접 참조 없음. 죽은 변수로 추정.

**선택적 변수 (미설정해도 패닉 없음)**:
- `IMESSAGE_RECIPIENT` — `main.rs`에서 `env::var().ok()` 패턴, 미설정 시 "비활성화"로 처리
- `APPLE_CLIENT_SECRET_EXPIRES_AT` — 경고 로그만, 미설정 시 만료 체크 스킵
- `DIAGNOSE_USER_ID` — `bin/diagnose` 서브 바이너리 전용. 메인 서버와 무관

**권장 수정**: `progress/cloud_deploy_plan.md` Step 1 환경변수 목록에 옵션 변수와 필수 변수를 분리 표기. `OPENROUTER_API_KEY`, `LLM_MODEL`을 삭제 또는 "(사용 안 함)" 주석으로 정리.

---

### F-02 [MEDIUM] E2E 테스트 주석에 평문 크리덴셜

**파일**:
- `web/e2e/feed-summary.spec.ts:13`
- `web/e2e/feed-like.spec.ts:11`
- `web/e2e/occupation-insight-rewrite.spec.ts:16`
- `web/e2e/tag-navigation.spec.ts:11`

**증거**: 각 파일 상단 주석에 `test@test.com / Test1234!` 하드코딩.

```ts
// 계정: test@test.com / Test1234!
```

이 크리덴셜은 git에 커밋되어 히스토리에 영구 존재. `web/.env`의 `TEST_EMAIL=test@frank.dev / frank-test-2026!`와도 다른 계정이 사용됨 — **두 벌의 테스트 계정이 혼용**.

**위험도 평가**: 개인 앱이고 테스트 계정은 실제 데이터 없이 검증용으로만 사용. 그러나 해당 계정으로 공격자가 API를 대리 호출할 수 있음. Supabase RLS는 user_id 기반이므로 타 사용자 데이터 접근은 차단되나, 테스트 계정 자체 데이터 변조 가능.

**권장 수정**:
```ts
// AS-IS (주석에 하드코딩)
// 계정: test@test.com / Test1234!

// TO-BE (주석 삭제 또는 env 참조로 교체)
const email = process.env.TEST_EMAIL ?? 'test@test.com';
const password = process.env.TEST_PASSWORD ?? '';
```
`web/.env`의 `TEST_EMAIL/TEST_PASSWORD`가 현재 코드에서 전혀 사용되지 않음(죽은 변수). E2E 스펙이 `process.env.TEST_EMAIL`을 읽도록 통일 후, 주석 크리덴셜 제거.

---

### F-03 [LOW] iOS Secrets.plist — App Bundle에 anon key 포함

**파일**: `ios/Frank/Frank/Resources/Secrets.plist`

**증거**: 파일에 Supabase URL + anon key가 평문으로 저장. 빌드 시 `.app` 번들에 포함되므로 앱 추출 후 `strings` 명령으로 노출 가능.

**위험도 평가**: JWT payload 디코딩 결과 `"role":"anon"` 확인 — service_role 키 아님. anon key는 Supabase 공개 설계상 클라이언트에 노출이 의도됨. RLS 정책으로 데이터 보호. 실제 위험은 낮음.

**단, gitignore 적용 확인 완료**: `.gitignore`에 `**/Secrets.plist` 등록됨. git 히스토리에 커밋된 적 없음.

**권장 사항**: 현 수준 유지. 향후 민감도가 높아지는 키(결제, admin 등)는 절대 Secrets.plist에 넣지 않도록 팀 규칙 유지.

---

### F-04 [LOW] VITE_RUST_API_URL — 클라이언트 번들 노출

**파일**: `web/src/lib/api/realClient.ts:21`

**증거**:
```ts
const API_BASE = (import.meta.env.VITE_RUST_API_URL ?? 'http://localhost:8080').replace(/\/$/, '');
```

`VITE_` prefix 변수는 Vite 빌드 시 클라이언트 번들에 인라인됨. Render의 `https://xxx.onrender.com` URL이 번들에 포함되어 브라우저 개발자 도구로 확인 가능.

**위험도 평가**: API 서버 URL이 노출되어도 실제 위험 낮음. 모든 /api/* 엔드포인트에 JWT 인증이 적용됨(미인증 시 401). /health는 공개이나 민감 데이터 없음.

**권장 사항**: SvelteKit SSR 라우트(+server.ts)에서는 이미 `process.env`로 올바르게 처리하고 있음. realClient.ts는 클라이언트 사이드 라우팅용이므로 URL 노출이 설계 의도. 현 수준 유지.

---

### F-05 [INFO] CORS 설정 — 환경변수 기반, 기본값은 localhost만

**파일**: `server/src/lib.rs:25-50`

**증거**: `ALLOWED_ORIGINS` 미설정 시 `localhost:5173,localhost:4173,127.0.0.1:5173`만 허용. Render 배포 후 Vercel 도메인을 `ALLOWED_ORIGINS`에 추가하지 않으면 웹에서 API 호출 실패.

**배포 계획**: `progress/cloud_deploy_plan.md` Step 2에 이 절차가 이미 명시됨. `allow_credentials(true)` 설정되어 있으므로 wildcard(`*`) 사용은 불가 — 정확한 도메인 입력 필수.

**위험도 평가**: 보안 측면에서 올바른 설정. 배포 순서 실수 시 기능 장애이지 보안 취약점은 아님.

---

### F-06 [INFO] Internal 에러 메시지 클라이언트 노출 없음 (양호)

**파일**: `server/src/domain/error.rs:32-39`

`AppError::Internal`은 `tracing::error!`로 서버 로그에만 상세 내용 기록하고, 클라이언트에는 `"Internal server error"` 문자열만 반환. 내부 정보 누출 없음.

---

### F-07 [INFO] git 히스토리 API 키 누출 없음 (양호)

`.env`, `Secrets.plist` 어느 파일도 git에 커밋된 적 없음. git 히스토리 전체에서 실제 JWT/API 키 패턴 미발견. service_role 키 누출 없음.

---

## 개선안 A/B/C

### A안 (권장): 최소 수정 — 배포 차단 항목만 해결

1. `progress/cloud_deploy_plan.md` 환경변수 목록 정리 (F-01)
2. E2E 스펙 4개 파일의 하드코딩 주석 크리덴셜 제거 (F-02)

**작업량**: 문서 편집 + 코드 4줄 수정. 30분 이내.

### B안 (권장+보강): A안 + E2E 크리덴셜 env 통일

1. A안 내용 전부
2. `web/.env`의 `TEST_EMAIL/TEST_PASSWORD`를 실제 E2E 스펙에서 읽도록 통일
3. `web/.env.example`에 `TEST_EMAIL=`, `TEST_PASSWORD=` 빈 값으로 추가

**작업량**: 추가 1~2시간. E2E 재검증 필요.

### C안 (장기): B안 + iOS Secrets.plist 자동화

1. B안 내용 전부
2. Xcode 빌드 스크립트로 Secrets.plist를 빌드 타임에 생성 (xcconfig 또는 빌드 스크립트)
3. 저장소에는 Secrets.plist.template만 보관

**작업량**: 반나절 이상. 배포 전 필수 사항은 아님.

---

## 우선순위 요약

| # | 발견 | 심각도 | 배포 차단 여부 | 권장 조치 |
|---|------|--------|:--------------:|-----------|
| F-01 | Render 환경변수 목록 문서 오류 | HIGH | 조건부 (죽은 변수 확인 필요) | cloud_deploy_plan.md 정리 |
| F-02 | E2E 주석 크리덴셜 | MEDIUM | 아니오 | 주석 삭제 + env 변수 통일 |
| F-03 | iOS Secrets.plist anon key | LOW | 아니오 | 현 수준 유지 |
| F-04 | VITE_RUST_API_URL 번들 노출 | LOW | 아니오 | 현 수준 유지 |
| F-05 | CORS 도메인 순서 의존 | INFO | 운영 절차 위험 | 배포 문서 주의사항 강조 |
| F-06 | Internal 에러 처리 | INFO | 양호 | 조치 불필요 |
| F-07 | git 히스토리 | INFO | 양호 | 조치 불필요 |
