# MVP13 M3 — 클라우드 배포 (개인 사용 최소 경로)

> 기획일: 2026-04-28  
> 상태: planning  
> 목표: Mac 없이도 iPhone에서 항상 앱 사용 가능. 비용 $0.  
> 범위: GitHub Actions / 도메인 / HTTPS 없는 최소 구성. 개인 사용 목적.

---

## 역할 분리

```
지금:
  iPhone → API 요청 → Mac의 서버 (Mac 꺼지면 끊김)

M3 이후:
  iPhone → API 요청 → Oracle VM의 서버 (항상 켜짐)
  Mac    → Xcode 빌드 → iPhone에 앱 설치 (업데이트 시만 필요)
```

Mac은 앱을 iPhone에 설치할 때만 필요. 이후에는 Mac이 꺼져 있어도 됨.

---

## 플랫폼: Oracle Cloud Always Free A1

| 항목 | 내용 |
|------|------|
| 인스턴스 | VM.Standard.A1.Flex (ARM64) |
| 스펙 | 4 OCPU / 24 GB RAM / 200 GB 스토리지 |
| 비용 | $0 (PAYG 업그레이드 필수 — 안정성, 실제 청구 없음) |
| OS | Ubuntu 22.04 (arm64) |
| 주소 | VM 공인 IP 직접 사용 (`http://{IP}:8080`) |

**PAYG 업그레이드 이유**: 순수 Always Free 계정은 무예고 정지 사례 다수. 업그레이드 후에도 Always Free 자원은 $0.

---

## 아키텍처 (최소 구성)

```
iPhone (iOS 앱)
  └── HTTP → Oracle VM 공인 IP:8080 (Rust API)
  └── HTTP → Oracle VM 공인 IP:3000 (SvelteKit 웹, 선택)

Oracle A1 VM (Ubuntu 22.04, ARM64)
  ├── server (Rust/Axum, :8080) — docker compose
  └── web    (SvelteKit,  :3000) — docker compose

DB: Supabase (기존 클라우드, 변경 없음)
```

HTTPS/도메인 없이 IP + HTTP 직접 사용. iOS ATS 예외를 Info.plist에 추가하여 허용.

---

## 구현 태스크

### 사용자 직접 수행 (코드 작업 아님, 워크플로우 전에 완료)

**P-01: Oracle 계정 생성 + PAYG 업그레이드**
1. oracle.com/kr/cloud/free 접속 → 계정 생성
2. 신용카드 등록 (체크카드 Visa/Mastercard 가능)
3. PAYG로 업그레이드 (실제 청구 없음, $1 지출 알림 설정 권장)

**P-02: A1 인스턴스 생성**
1. OCI 콘솔 → Compute → Instances → Create Instance
2. Shape: VM.Standard.A1.Flex (4 OCPU, 24 GB)
3. Image: Canonical Ubuntu 22.04 (arm64)
4. SSH 키 생성 + 저장 (이후 접속에 사용)
5. 공인 IP 메모
6. **서울 리전 용량 부족 시**: AD(Availability Domain) 변경 또는 일본(ap-tokyo-1) 리전으로 시도

**P-03: VM 방화벽 설정**
```bash
# OCI 콘솔: Security List에 Ingress 추가
# TCP 22 (SSH), 8080 (API), 3000 (웹)

# VM 내부 ufw 설정
sudo ufw allow 22/tcp
sudo ufw allow 8080/tcp
sudo ufw allow 3000/tcp
sudo ufw enable
```

**P-04: Docker 설치**
```bash
ssh -i {키파일} ubuntu@{VM_IP}
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker ubuntu
# 재접속 후 docker 명령 사용 가능
```

---

### 코드 작업 (워크플로우에서 진행)

**T-01: reqwest rustls 전환 (ARM64 크로스컴파일 호환)**

현재 `reqwest`가 `native-tls`(OpenSSL) 기본 사용 → ARM64 Docker 빌드 복잡.
`rustls-tls`로 교체하면 순수 Rust TLS, 크로스컴파일 단순화.

```toml
# server/Cargo.toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
```

**T-02: Dockerfile linux/arm64 플랫폼 명시**

```dockerfile
# server/Dockerfile
FROM --platform=linux/arm64 rust:1.94-slim AS builder
```

```dockerfile
# web/Dockerfile  
FROM --platform=linux/arm64 node:22-slim AS builder
```

**T-03: iOS ATS 예외 추가**

iOS는 기본적으로 HTTP 원격 접속 차단(ATS). VM IP로 HTTP 통신을 허용하려면 Info.plist에 예외 추가.

```xml
<!-- ios/Frank/Frank/Sources/App/Info.plist -->
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSExceptionDomains</key>
    <dict>
        <key>{VM_IP}</key>
        <dict>
            <key>NSExceptionAllowsInsecureHTTPLoads</key>
            <true/>
        </dict>
    </dict>
</dict>
```

**T-04: iOS API base URL 변경**

```swift
// VM IP로 변경 (P-02에서 메모한 IP 사용)
static let apiBaseURL = "http://{VM_IP}:8080/api"
```

**T-05: VM에 앱 배포**

SSH 접속 후:
```bash
# 코드 복사 (scp 또는 git clone)
git clone https://github.com/seungchan2022/frank.git
cd frank

# .env 파일 작성 (서버 환경변수)
cp server/.env.example server/.env
# server/.env 편집: 실제 키값 입력

# docker compose 빌드 + 실행 (ARM64 빌드 — 최초 시 시간 소요)
docker compose up -d --build
```

**T-06: Idle 회수 방지 keepalive**

7일 연속 CPU/메모리 < 20th percentile 시 Oracle이 인스턴스 회수 경고 발송.
초기 트래픽이 적으면 해당될 수 있으므로 최소 부하 유지.

```bash
# /usr/local/bin/frank-keepalive.sh
#!/bin/bash
sudo apt-get install -y stress-ng
while true; do
    stress-ng --vm 1 --vm-bytes 128M --vm-keep --timeout 30s &>/dev/null
    sleep 270
done
```

```bash
# systemd 등록
sudo systemctl enable frank-keepalive --now
```

**T-07: 배포 검증 (E2E)**
- [ ] `http://{VM_IP}:8080/health` → `{"status":"ok"}`
- [ ] iOS 실기기 — Mac 꺼진 상태에서 앱 실행 → 피드 로드 정상
- [ ] iOS 실기기 — 로그인 → 퀴즈 → 오답노트 정상
- [ ] **[M1 R-03 이월]** DB 연결 실패 시 500 반환 — VM에서 Supabase 연결 끊음(방화벽 차단 또는 DB URL 임시 오염) 후 API 호출 → 500 응답 확인

---

## KPI

| ID | 지표 | 목표 | 타입 |
|----|------|------|------|
| K1 | `GET /api/health` 응답 시간 (VM) | < 500ms | Hard |
| K2 | iOS 실기기 — Mac 꺼진 상태 피드 로드 | 정상 동작 | Hard |
| K3 | docker compose 재시작 후 서비스 자동 기동 | `restart: unless-stopped` 확인 | Hard |

---

## 구현 순서

```
[사용자 수동] P-01 Oracle 계정 + PAYG
    └→ P-02 A1 인스턴스 생성 + IP 확보
         └→ P-03 방화벽 설정
              └→ P-04 Docker 설치
                   │
                   ▼ (VM IP 확보 후 워크플로우 시작)
              T-01 reqwest rustls 전환
              T-02 Dockerfile ARM64 명시
              T-03 iOS ATS 예외 + T-04 API URL 변경
                   └→ T-05 VM 배포
                        └→ T-06 keepalive
                             └→ T-07 E2E 검증
```

---

## 리스크 및 대응

| 리스크 | 가능성 | 대응 |
|--------|--------|------|
| A1 서울 리전 용량 부족 | 중 | 다른 AD 시도; 일본(ap-tokyo-1) 리전 대안 |
| Oracle 계정 정지 | 중 | PAYG 업그레이드로 위험 감소; 백업: Hetzner CAX11 €5/월 |
| ARM64 빌드 실패 | 중 | T-01 rustls 전환으로 OpenSSL 제거; 최초 빌드 30분 소요 예상 |
| 앱 업데이트 시 Xcode 재빌드 필요 | 항상 | 개인 사용 범위에서 수용 가능 |

---

## 향후 개선 (M3 이후 선택)

- DuckDNS + Caddy로 HTTPS + 도메인 → IP 외우지 않아도 됨
- GitHub Actions CI/CD → 서버 코드 push 시 자동 배포
- Apple Developer 계정 + TestFlight → Mac 없이 앱 업데이트 가능
