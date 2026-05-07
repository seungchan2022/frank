# F1 진단 보고서 — 실기기 오답 저장 회귀

> 작성일: 2026-05-07
> 상태: **픽스 대기** — 런타임 신호 수집 후 최종 확정 필요

---

## 1. 변경 요약 (현재 가설 기반)

**1차 원인 가설**: iOS 14+ Local Network Privacy 키 누락  
- `NSLocalNetworkUsageDescription` + `NSBonjourServices` 미선언 시 mDNS(`*.local`) 해석이 OS 레벨에서 silently fail.
- 시뮬레이터는 `http://localhost:8080` → 영향 없음. 실기기는 `http://MacBook-Pro.local:8080` → mDNS 해석 시도 → OS가 차단.
- 증상 패턴과 정확히 일치: "시뮬레이터는 정상, 실기기에서만 실패"

**기존 설정 확인**:
- `NSAllowsArbitraryLoads: true` ✅ (ATS 아님)
- `NSAllowsLocalNetworking: true` ✅ (localhost ATS 아님)
- `NSLocalNetworkUsageDescription` ❌ **없음**
- `NSBonjourServices` ❌ **없음**
- `SERVER_URL = http://MacBook-Pro.local:8080` (xcconfig) — mDNS 호스트명 사용 중

---

## 2. 수정 파일 목록 (픽스 적용 시)

| 파일 | 변경 내용 |
|------|---------|
| `ios/Frank/Project.swift` | `NSLocalNetworkUsageDescription` + `NSBonjourServices` 키 추가 |
| `ios/Frank/Derived/InfoPlists/Frank-Info.plist` | tuist generate 후 자동 갱신 |

---

## 3. 테스트 명령 + 결과

### 픽스 전 (현재)

```bash
# 실기기에서 Safari 접속 시도
# http://MacBook-Pro.local:8080/health → 응답 없음 또는 타임아웃
```

### 픽스 후 예상 검증

```bash
# Info.plist 키 존재 확인
grep -A2 "NSLocalNetworkUsageDescription" ios/Frank/Derived/InfoPlists/Frank-Info.plist
grep -A2 "NSBonjourServices" ios/Frank/Derived/InfoPlists/Frank-Info.plist

# Tuist 재생성
cd ios/Frank && ~/.tuist/Versions/4.31.0/tuist generate --no-open

# 빌드
xcodebuild build \
  -workspace ios/Frank/Frank.xcworkspace \
  -scheme Frank \
  -destination 'platform=iOS Simulator,name=iPhone 17 Pro'
```

---

## 4. 스모크 테스트 (실기기 필수)

1. iPhone 실기기 + Mac 동일 Wi-Fi 연결 확인
2. 앱 실행 → **Local Network 접근 권한 요청 팝업 표시 여부** 확인 (처음 mDNS 호출 시 팝업)
3. 퀴즈 진행 → 오답 선택 → 오답 저장 탭
4. 오답노트 탭 → 저장된 오답 표시 확인
5. 즐겨찾기 탭 → 표시 확인

**런타임 신호 수집 (픽스 적용 전 먼저 시도)**:
- Xcode → Devices → 실기기 콘솔에서 `[오답저장]` / `[APIWrongAnswer]` 로그 캡처
- 실기기 Safari에서 `http://MacBook-Pro.local:8080/health` 직접 접속 → 응답 여부 확인
- 응답 없으면 NSLocalNetworkUsageDescription 가설 확정

---

## 5. 리스크

| 리스크 | 가능성 | 대응 |
|--------|-------|------|
| NSLocalNetworkUsageDescription 추가 후에도 실패 → mDNS 해석은 같은 Wi-Fi 서브넷 필요 | M | 같은 Wi-Fi 연결 여부 먼저 확인. 다른 서브넷이면 서버 IP 직접 사용 검토 |
| Mac 방화벽이 8080 포트 인바운드 차단 | L | System Settings → Firewall → 포트 8080 허용 확인 |
| Supabase Auth 토큰 Keychain access group 이슈 | L | 실기기 콘솔 로그에서 Supabase 인증 에러 여부 확인 후 판단 |
| 픽스 후 권한 팝업 거부 → 여전히 실패 | L | 설정 앱에서 Frank → Local Network 권한 재허용 |
| NSBonjourServices 배열 형식 오류 (Tuist plist 직렬화) | L | `_http._tcp` 형식으로 추가, tuist generate 후 plist 확인 |

---

## 6. 회귀 안전망 (픽스 완료 후 추가)

- `FrankTests/Config/InfoPlistTests.swift`: `NSLocalNetworkUsageDescription` 키가 Info.plist에 존재하는지 assert하는 단위 테스트 추가 → CI에서 누락 즉시 감지.

---

## 7. 판단 보류 항목 (런타임 신호 필요)

- 실기기 콘솔 로그에서 `URLSession` 에러 타입 확인
- 실기기 Safari → `http://MacBook-Pro.local:8080/health` 응답 여부
- Supabase `getAccessToken()` 실기기 동작 여부

**위 신호 없이 픽스를 커밋하면 "추측 픽스"가 된다. 런타임 신호 1개 이상 확인 후 ST-5 커밋 진행.**
