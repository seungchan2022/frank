# 통합 배포 스크립트 메인태스크

> 생성일: 2026-04-07
> 브랜치: `main`
> 작업 파일: `scripts/deploy.sh`, `docker-compose.yml`, `web/.env`, `CLAUDE.md`

## 메인태스크

**통합 배포 스크립트 작성 — iOS · 웹 프론트 · API 서버를 단일 셸 스크립트로 실행/재배포**

---

## 설계 결정사항

### 1. 왜 단일 스크립트(`deploy.sh`)인가?

- 이전에는 iOS 전용 `ios/Frank/scripts/run-simulator.sh`가 별도 존재
- 타겟이 3개(iOS · api · front)로 늘어나면서 "어떤 스크립트를 써야 하는가" 혼란 방지
- 진입점을 하나로 통일해 CI·터널·로컬 실행 모두 같은 경로로 수렴

### 2. 플래그 설계: `--target=ios,front,api`

- 플래그 없음 → 전체 3개 실행 (가장 흔한 케이스 zero-friction)
- `,` 구분 복수 지정 → `--target=api,front` 처럼 서브셋 실행
- 잘못된 타겟 즉시 exit 1 + 에러 메시지 (사일런트 스킵 금지)

### 3. Docker 포트 킬 순서

재배포 시 포트 충돌을 막기 위해 아래 순서 강제:

```
docker compose stop $service
docker compose rm -f $service
lsof -ti tcp:$port | xargs kill -9   ← 네이티브 프로세스도 잔재 제거
docker compose build $service
docker compose up -d $service
```

- Docker 컨테이너가 없는 상태(첫 실행)에도 `|| true`로 안전 통과
- `kill_port()`를 별도 함수로 분리해 iOS 등 다른 타겟에서도 재사용 가능

### 4. 환경변수 단일 소스: `server/.env`

**변경 전:**
- `web/.env`: `PUBLIC_SUPABASE_URL`, `PUBLIC_SUPABASE_ANON_KEY` 중복 보유
- `docker-compose.yml` web 서비스: `env_file: ./web/.env` → 테스트 계정 정보가 컨테이너에 노출

**변경 후:**
- `server/.env`만 진실의 원천 (`SUPABASE_URL`, `SUPABASE_ANON_KEY`)
- 스크립트에서 빌드 직전 매핑 주입:
  ```bash
  export PUBLIC_SUPABASE_URL="${SUPABASE_URL:-}"
  export PUBLIC_SUPABASE_ANON_KEY="${SUPABASE_ANON_KEY:-}"
  ```
- `docker-compose.yml`에서 `env_file: ./web/.env` 제거 → 테스트 계정 컨테이너 유입 차단
- `web/.env`는 테스트 전용 (`TEST_EMAIL`, `TEST_PASSWORD`)으로만 남음

**이렇게 한 이유:**
- SvelteKit의 `PUBLIC_*` 변수는 빌드 타임에 JS 번들에 삽입됨 → docker build arg으로 주입이 정석
- 런타임 env_file이 필요 없음 (이미 번들에 박혀 있음)
- `.env` 파일 2개를 동기화하는 실수 원천 차단

### 5. Cloudflare 터널 + iMessage 알림

- `--tunnel` 플래그 → cloudflared quick tunnel 실행
- `server/.env`의 `IMESSAGE_RECIPIENT` 값으로 터널 URL을 iMessage 발송
- front 타겟 미포함 시 경고 후 skip (터널 없이 API만 띄우는 시나리오 대응)

### 6. iOS 스크립트 통합

- `ios/Frank/scripts/run-simulator.sh` 삭제
- `deploy.sh --target=ios`로 흡수
- `--simulator=` 플래그로 시뮬레이터 이름 오버라이드 가능 (기본: `iPhone 17 Pro`)
- `CLAUDE.md` 빌드 명령 섹션도 통합 배포 명령으로 갱신

---

## 서브태스크

| ID | 내용 | 상태 |
|----|------|------|
| ST-1 | `deploy.sh` 전면 작성 (3타겟 · 포트킬 · 터널) | ✅ 완료 |
| ST-2 | 기존 `ios/Frank/scripts/run-simulator.sh` 삭제 | ✅ 완료 |
| ST-3 | `CLAUDE.md` 빌드 명령 갱신 | ✅ 완료 |
| ST-4 | 환경변수 단일 소스 리팩토링 (`server/.env` → 스크립트 주입) | ✅ 완료 |
| ST-5 | `docker-compose.yml` `env_file: ./web/.env` 제거 | ✅ 완료 |
| ST-6 | `web/.env` 중복 Supabase 변수 삭제 | ✅ 완료 |
| ST-7 | 전체 테스트 (문법·헬프·에러처리·실배포·헬스체크) | ✅ 완료 |
| ST-8 | iOS ATS `NSAllowsLocalNetworking` 추가 (`Project.swift`) | ✅ 완료 |
| ST-9 | `profiles` 없는 유저 대비 `update_profile_onboarding` UPSERT 전환 | ✅ 완료 |

---

## 검증 결과

| 테스트 항목 | 결과 |
|------------|------|
| `bash -n deploy.sh` 문법 검사 | ✅ |
| `--help` 출력 | ✅ |
| 잘못된 타겟/옵션 → exit 1 | ✅ |
| `--target=api` 실제 배포 (stop→kill→build→up) | ✅ |
| `GET /health` → `{"status":"ok","version":"0.1.0"}` | ✅ |
| docker-compose 경고 없음 (PUBLIC_SUPABASE_* 미설정 경고 제거) | ✅ (ST-4 적용 후) |
| iOS 시뮬레이터 → `http://localhost:8080` 연결 (ATS 허용) | ✅ (ST-8 적용 후) |
| `profiles` row 없는 신규 유저 태그 저장 | ✅ (ST-9, UPSERT 전환) |
