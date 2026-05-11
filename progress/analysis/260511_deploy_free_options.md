# Frank 무료 배포 방안 심층 분석

> 작성일: 2026-05-11  
> 상황: Oracle Cloud ARM (춘천 리전) 용량 부족 19회 실패  
> 목표: Mac 로컬 서버 미구동 상태에서 무료로 실 사용 가능한 배포 구성 탐색

---

## 1. 현재 아키텍처 제약 파악

### 서비스 구성
| 레이어 | 기술 | 포트 | 외부 의존 |
|--------|------|------|-----------|
| API 서버 | Rust/Axum | 8080 | Supabase Auth JWT, PostgreSQL (Supabase Pooler), Tavily/Exa/Firecrawl/Groq/OpenRouter |
| 웹 프론트 | SvelteKit (Node adapter) | 3000 | API 서버 (server-side proxy), Supabase JS SDK |
| iOS | SwiftUI | — | API 서버 |
| DB | Supabase PostgreSQL | — | 이미 클라우드 (무료) |

### 핵심 제약 사항
1. **Rust 바이너리**: 크로스 컴파일 필요. 일부 플랫폼은 Docker 이미지로만 수용
2. **sqlx PgPool**: 런타임에 DB 연결 유지 필요 → 서버리스 함수 환경 부적합
3. **메모리**: InMemoryFeedCache, InMemoryCounter 사용 중 → 재시작 시 상태 소실 (이미 ephemeral 설계)
4. **긴 빌드 시간**: Rust 릴리즈 빌드는 5~15분 소요. 캐싱 없는 플랫폼은 빌드 타임아웃 위험
5. **SvelteKit**: Node adapter 사용 중. Cloudflare 배포 시 adapter-cloudflare로 교체 필요

---

## 2. 옵션별 평가

### 옵션 A: Render 무료 티어 (API + Web)

**구성**: Render Free Web Service × 2 (API, Web)

| 항목 | 내용 |
|------|------|
| 비용 | $0 (월 750 인스턴스 시간, 공유) |
| RAM | 512MB / 서비스 |
| 빌드 | Docker 지원 — 기존 Dockerfile 그대로 사용 가능 |
| 콜드 스타트 | **15분 비활성 후 슬립** → 첫 요청 30~60초 지연 |
| 지속성 | 재시작 시 에페머럴 파일 소실 (InMemory 상태는 이미 설계 반영) |
| DB | 필요 없음 (Supabase pooler 직접 연결) |

**장점**
- 기존 Dockerfile 그대로 사용. 코드 변경 없음
- GitHub 연동 자동 배포
- 무료 TLS, 커스텀 도메인 지원

**단점**
- 콜드 스타트가 치명적: iOS 앱이 첫 요청 시 1분 대기
- 월 750시간 = 두 서비스가 24/7 실행 불가 (750/2 = 375시간/월 = 약 15.6일/서비스)
- Rust 빌드 시간 길어 첫 배포·재배포가 느림

**실현 가능성**: ⭐⭐⭐ (개발/데모용 O, 실 사용 한계)

---

### 옵션 B: Railway Hobby (API + Web) — 월 $5

**구성**: Railway Hobby Plan $5/월 (무제한 슬립 없음)

| 항목 | 내용 |
|------|------|
| 비용 | $5/월 (usage credit $5 포함, 실질 $0~$5 사용) |
| RAM | 서비스당 설정 가능 |
| 빌드 | Docker 또는 Nixpacks (Rust 자동 감지) |
| 콜드 스타트 | 없음 (24/7 실행) |
| 무료 Trial | 신규 계정 $5 크레딧 30일 — **영구 무료 아님** |

**장점**
- 24/7 실행, 콜드 스타트 없음
- Dockerfile 그대로 사용
- 매달 $5 usage credit이 소규모 트래픽에서 실질 무료

**단점**
- 엄밀히 유료 ($5/월). "무료"가 아님
- 30일 Trial만 완전 무료

**실현 가능성**: ⭐⭐⭐⭐ (소규모 프로젝트에서 가장 실용적인 유사-무료 옵션)

---

### 옵션 C: Koyeb 무료 티어 (API) + Cloudflare Pages (Web)

**구성**: Koyeb Free (API) + Cloudflare Pages (Web)

| 서비스 | 비용 | 제약 |
|--------|------|------|
| Koyeb API | $0 (영구, 신용카드 불필요) | 512MB RAM, 0.1 vCPU, 인스턴스 1개 |
| Cloudflare Pages | $0 (영구, 무제한 대역폭) | SvelteKit adapter 교체 필요 |

**Koyeb 평가**
- Docker 또는 Git 배포 지원 → 기존 server/Dockerfile 사용 가능
- 512MB RAM은 Rust 서버 + DB 커넥션 풀(5개)에 **빡빡하지만 가능**
- 0.1 vCPU = 동시 요청 처리 느림, 단일 사용자 개인 앱 수준
- 슬립 없음 — 24/7 실행

**Cloudflare Pages 평가**
- adapter-cloudflare로 변경 필요 (현재 Node adapter)
- Cloudflare Workers 런타임 = Node.js API 일부 미지원
- SvelteKit API routes (server-side) 동작하지만 제약 있음
- API_SERVER_URL 환경변수로 Koyeb API 서버 연결

**필요한 코드 변경**
```bash
# web/package.json
- "@sveltejs/adapter-node": "..."
+ "@sveltejs/adapter-cloudflare": "..."
```
```js
// svelte.config.js
- import adapter from '@sveltejs/adapter-node';
+ import adapter from '@sveltejs/adapter-cloudflare';
```

**장점**
- 완전 무료, 영구, 신용카드 불필요
- Cloudflare Pages 무제한 대역폭
- 콜드 스타트 없음 (Koyeb)

**단점**
- Koyeb 0.1 vCPU — Groq/Firecrawl 병렬 요청 시 응답 느림
- Web adapter 변경 필요 — Node.js 전용 코드 있으면 호환성 검증 필요
- 인스턴스 1개 제한 — API 서버와 웹 프론트를 각각 1개씩밖에 못 올림

**실현 가능성**: ⭐⭐⭐⭐ (완전 무료 중 가장 현실적)

---

### 옵션 D: Oracle Cloud 다른 리전 재시도

**구성**: OCI Always Free ARM A1 (춘천 대신 다른 리전)

| 리전 | 특성 |
|------|------|
| 서울 (ap-seoul-1) | 국내 레이턴시 우수, ARM 용량 상대적으로 안정 |
| 싱가포르 (ap-singapore-1) | 용량 여유 있음 (보고됨) |
| 프랑크푸르트 (eu-frankfurt-1) | 유럽 리전 중 안정적으로 알려짐 |

**장점**
- 4 OCPU + 24GB RAM — 압도적 스펙 (무료 최강)
- 24/7 실행, Docker 완전 지원
- 코드 변경 전혀 없음

**단점**
- 리전별 가용성이 불규칙 — 서울도 실패 가능성
- PAYG(종량제) 계정으로 전환 시 Always Free 혜택 유지하면서 용량 확보 가능
- 새 계정 생성 시 리전 이전 불가 (계정당 홈 리전 고정)

**실현 가능성**: ⭐⭐⭐ (운에 의존, 빠른 해결책이 아님)

---

### 옵션 E: Cloudflare Tunnel (로컬 맥 미사용) — 별도 저전력 기기

**구성**: 라즈베리파이/저전력 ARM 기기 + Cloudflare Tunnel

> 맥 로컬 서버 미사용 조건이 "맥북을 끄고 싶다"는 의미라면 이 옵션은 해당 없음.
> 단, 소형 ARM 기기(라즈베리파이, 미니PC)가 있다면 완전 무료 배포 가능.

- Cloudflare Tunnel: 무료, 퍼블릭 IP 없어도 터널로 외부 노출
- Rust 크로스 컴파일 필요 (aarch64-unknown-linux-musl)
- 이미 deploy.sh에 `--tunnel` 플래그 구현되어 있음

**실현 가능성**: 기기 있으면 ⭐⭐⭐⭐⭐, 없으면 해당 없음

---

### 옵션 F: Cloudflare Workers (Rust→WASM) — 불가 판정

Tokio/async_std 미지원. sqlx PgPool 불가. Frank API 서버 구조상 **사용 불가**.

---

## 3. 종합 추천

### 즉시 실행 가능한 최우선 선택: **옵션 C (Koyeb + Cloudflare Pages)**

```
[iOS 앱] ──────────────────────────────────────────────────────────┐
[웹 브라우저] → Cloudflare Pages (SvelteKit, adapter 교체) ─────────┤
                                                                    ▼
                                         Koyeb Free (Rust/Axum API, 24/7)
                                                                    │
                              ┌────────────────────┬───────────────┘
                              ▼                    ▼
               Supabase PostgreSQL           외부 API (Tavily/Exa/Groq...)
               (이미 클라우드)
```

**왜 Koyeb + Cloudflare?**
- 완전 무료, 신용카드 불필요, 영구
- Koyeb은 슬립 없음 → iOS 앱 콜드 스타트 문제 없음
- Cloudflare Pages는 무제한 대역폭
- 코드 변경: adapter 교체 1줄 + Node.js 전용 코드 검증

### 유사-무료로 안정성 원한다면: **옵션 B (Railway $5/월)**

월 $5이지만 usage credit이 소규모 트래픽에서 실질 $0. 코드 변경 전혀 없음.

### OCI 재시도: **서울 리전 (ap-seoul-1)** 또는 PAYG 전환

현재 춘천(ap-chuncheon-1)만 시도. 서울 리전 변경 또는 종량제 계정 전환(Always Free 혜택 유지)으로 재시도.

---

## 4. Koyeb + Cloudflare Pages 전환 작업 목록

### 4-1. Web adapter 교체 (필수)
```bash
cd /Users/seungchan/Workspace/frank/web
npm install @sveltejs/adapter-cloudflare
npm uninstall @sveltejs/adapter-node
```
```js
// svelte.config.js
import adapter from '@sveltejs/adapter-cloudflare';
```

### 4-2. Node.js 전용 코드 검증 (필수)
- `src/routes/api/` 하위 server-side route에서 Node.js 전용 API 사용 여부 확인
- Cloudflare Workers는 `crypto`, `fetch` 등 Web API 지원하지만 `fs`, `child_process` 등 미지원

### 4-3. 환경변수 설정
- Cloudflare Pages: `PUBLIC_SUPABASE_URL`, `PUBLIC_SUPABASE_ANON_KEY`, `API_SERVER_URL` (Koyeb URL)
- Koyeb: 기존 server/.env 내용 그대로 환경변수로 등록

### 4-4. CORS 설정 업데이트
- server/src의 CORS allowed_origins에 Cloudflare Pages 도메인 추가 필요
- `deploy.sh`의 `LOCAL_ORIGINS` 배열에 프로덕션 URL 추가

### 4-5. Koyeb Dockerfile 배포
- `server/Dockerfile` 그대로 사용 가능
- GitHub 연동 또는 Docker Hub 이미지로 배포

---

## 5. 리스크 & 대응

| 리스크 | 영향 | 대응 |
|--------|------|------|
| Koyeb 0.1 vCPU로 Groq/Firecrawl 병렬 요청 느림 | 중 | SearchFallbackChain 타임아웃 튜닝; Groq free tier는 빠름 |
| Cloudflare adapter 호환성 이슈 | 중 | `npm run build` 빌드 후 `wrangler dev`로 로컬 검증 |
| Koyeb 512MB OOM | 중-고 | max_connections를 5→2로 줄여 메모리 절감 |
| Supabase pooler 연결 수 제한 | 낮 | 이미 max_connections=5, Koyeb에서 더 줄이면 OK |
| Cloudflare Pages 빌드 시간 | 낮 | Node 빌드라 빠름 (Rust 아님) |

---

## 6. 플랫폼 비교 요약표

| 플랫폼 | 비용 | 슬립 | Rust 지원 | 코드 변경 | 권장 용도 |
|--------|------|------|-----------|-----------|-----------|
| **Koyeb** | 무료 영구 | 없음 | Docker | 없음 | API 서버 |
| **Cloudflare Pages** | 무료 영구 | 없음 | N/A | adapter 교체 | 웹 프론트 |
| Render Free | 무료 | 15분 슬립 | Docker | 없음 | 데모용 |
| Railway Hobby | $5/월 | 없음 | Docker | 없음 | 소규모 프로덕션 |
| Fly.io | 유료 (PAYG) | 설정 가능 | Docker | 없음 | 중규모 |
| OCI ARM (다른 리전) | 무료 영구 | 없음 | 네이티브 | 없음 | 최강 스펙 |
| Cloudflare Workers | 무료 | 없음 | WASM | 전면 재작성 | 불가 |

---

## 참고 리소스

- [Koyeb Rust/Docker 배포 문서](https://www.koyeb.com/docs)
- [Cloudflare Pages SvelteKit 가이드](https://developers.cloudflare.com/pages/framework-guides/deploy-a-svelte-kit-site/)
- [Railway Axum 배포 가이드](https://docs.railway.com/guides/axum)
- [OCI Always Free 리소스](https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm)
- [Render 무료 티어 문서](https://render.com/docs/free)
