# 서브태스크: MVP16 M4 — 클라이언트 표시·회귀

> 작성일: 2026-05-07
> 메인태스크: iOS 피드 카드 태그 칩 표시 + 실기기 오답 저장 회귀 진단·복원 + 오답노트 태그 필터 회귀 진단·복원 + 오답 → 원문 이동 UI 추가
> 브랜치: feature/260507_m4-client-display-regression
> 상태: planning

---

## 분해 배경

M4 아이템 4건(E1, F1, F2, D2)을 구현 가능한 최소 단위로 분해.
의존성 분석 후 실행 순서를 결정한다.

### 코드 탐색 결과 (진단 사전 조사)

**E1 (iOS 피드 카드 태그 칩)**
- `ArticleCardView.swift`: 현재 태그 칩 렌더링 없음. title, source, publishedAt만 표시.
- `Article(= FeedItem)` 모델: `tagId: UUID?` 필드 존재. 태그 이름(name)은 없음.
- FeedView는 `feature.tags: [Tag]`를 보유 — `tagId → Tag.name` 매핑 가능.
- 태그 칩 표시를 위해 ArticleCardView에 `tagName: String?` 또는 `tag: Tag?` 파라미터 추가 필요.
- `TagChipView.swift`: 이미 존재하는 칩 컴포넌트 — 재사용 가능.

**F1 (실기기 오답 저장 회귀)**
- `ServerConfig.swift:live()`: 시뮬레이터는 `http://localhost:8080`, 실기기는 Info.plist → Secrets.plist 순으로 탐색.
- `Config.xcconfig`: `SERVER_URL = http://MacBook-Pro.local:8080` — mDNS 호스트명 사용.
- ~~ATS 차단 가능성~~ → **제외됨**: `Frank-Info.plist`에 `NSAllowsArbitraryLoads: true` + `NSAllowsLocalNetworking: true` 이미 설정. ATS는 원인 아님.
- 실질 후보: mDNS(`MacBook-Pro.local`) 해석 실패 (실기기가 다른 Wi-Fi/서브넷), Mac 방화벽 8080 포트 차단, Keychain access group 설정 차이.
- `APIWrongAnswerAdapter.swift`: 토큰 취득 후 서버 호출 — 토큰 자체 문제 가능성도 있음.
- 진단 없이 픽스하면 위험 → **ST-2에서 먼저 F1 진단 보고서 작성 필수**.

**F2 (오답노트 태그 필터 회귀)**
- `WrongAnswerTagFilter.swift`: `filter(items:selectedTagId:)` 정적 함수 — 로직은 단순.
- `FavoritesView.swift`: `wrongAnswersListForTag(_:)` → `WrongAnswerTagFilter.filter` 호출 — 이미 연결됨.
- **원인 특정됨**: `wrongAnswersContent`의 `TagChipBarView.onSelect` 콜백에서 `wrongAnswersPageIndex = idx`만 갱신하고 `wrongAnswersScrollID = idx`를 갱신하지 않음. `ScrollView.scrollPosition(id: $wrongAnswersScrollID)` 바인딩이 갱신되지 않아 실제 스크롤이 이동하지 않음. (반면 `articlesContent`는 두 값 모두 갱신함.)
- 픽스: 한 줄 추가 — `wrongAnswersScrollID = idx`.
- ST-3은 가설 검증 + 회귀 케이스 작성으로 범위 축소.

**D2 (오답 → 원문 이동 UI)**
- 서버: `quiz_wrong_answers.article_url TEXT NOT NULL` 컬럼 이미 존재 (MVP8 M1 스키마 확인).
- iOS `WrongAnswer.articleUrl: String` 필드 이미 있음.
- 웹 `WrongAnswer.articleUrl: string` 타입 이미 있음.
- `WrongAnswerRow.swift`: 현재 원문 이동 액션 없음 → iOS 구현 필요.
- `WrongAnswerCard.svelte`: 현재 원문 이동 없음 → 웹 구현 필요.
- 이동 방식 결정 필요: 새 탭 vs 인앱 웹뷰(SFSafariViewController) vs 기사 디테일 재진입.
- ~~빈 문자열 분기 필요~~ → **제외됨**: `article_url TEXT NOT NULL`이고 INSERT 시 항상 articleUrl을 포함. 기존 오답도 모두 값이 있음.

---

## 서브태스크 목록

### ST-1. D2 원문 이동 방식 사용자 확인 (confirmation)

**목적**: ST-7이 제안하는 이동 방식(iOS: SFSafariViewController, 웹: 새 탭)을 사용자에게 확인한다.
ST-7 사양이 이미 방향을 포함하므로 ST-1은 결정이 아닌 **확인** 단계다.

**⚠️ 진입 전 DB 실측 필수 (M1 구멍 해소)**

`article_url TEXT NOT NULL`은 빈 문자열(`""`)을 막지 않는다. ST-7에서 분기 처리 불필요를 선언하기 전에 반드시 실측해야 한다:

```sql
SELECT count(*) FROM quiz_wrong_answers
WHERE article_url = '' OR article_url IS NULL;
```

- 결과 = 0 → ST-7에서 분기 제거 유지 (현재 계획대로)
- 결과 > 0 → ST-7에 빈 문자열 분기 추가 + M4 DoD 항목 복원

**확인 항목**:
- iOS: SFSafariViewController(인앱 웹뷰) 방식 동의 여부
- 웹: `<a target="_blank">` 새 탭 방식 동의 여부
- DB 실측 결과에 따른 분기 처리 여부 결정

**산출물**: DB 실측 결과 기록 + 사용자 승인 → ST-7 구현 즉시 진입

**의존성**: 없음
**실행 순서**: 1
**예상 규모**: 확인 (코드 없음, DB 쿼리 1회)

---

### ST-2. F1 실기기 오답 저장 회귀 진단 (research)

**목적**: 실기기에서만 오답 저장이 안 되는 원인을 코드·설정 분석으로 확정 후 보고서 작성.

**⚠️ 진단 전 필수 확인 (코드 탐색만으로 원인 확정 불가)**

정적 분석으로 원인 후보를 좁힌 뒤, **다음 런타임 신호를 반드시 수집**해야 ST-5 "추측 픽스"를 피할 수 있다:
- 실기기 Xcode Console 로그 — 오답 저장 탭 후 `[오답저장]` / `[APIWrongAnswer]` 로그 캡처
- 실기기 → Mac 연결 가능 여부 — 실기기 Safari에서 `http://MacBook-Pro.local:8080/health` 직접 접속해 응답 확인
- 네트워크 요청 발사 여부 — Proxyman/Charles MITM 또는 콘솔 `URLSession` 에러 로그로 확인

**진단 항목**:
1. ~~ATS 정책 확인~~ → **제외**: `Frank-Info.plist`에 `NSAllowsArbitraryLoads: true` 이미 확인됨.
2. **[최우선] iOS 14+ Local Network Privacy 키 누락 여부** — `NSLocalNetworkUsageDescription` + `NSBonjourServices` 미선언 시 mDNS(`*.local`) 해석이 OS 레벨에서 silently fail. 시뮬레이터(`localhost`)에서는 걸리지 않아 "시뮬에선 되는데 실기기에서만 안 됨" 패턴과 정확히 일치. `Frank-Info.plist` 에 두 키 없음 확인됨 → **가장 유력한 단일 원인**.
3. 실기기 SERVER_URL 탐색 경로 — Info.plist `SERVER_URL` 키 확인 (현재 `$(SERVER_URL)` 치환, xcconfig에서 `MacBook-Pro.local:8080`).
4. mDNS 해석 — `MacBook-Pro.local` 이 실기기 네트워크에서 해석 가능한지 (같은 Wi-Fi + Bonjour 활성화 필요). 항목 2가 원인이면 이 항목은 파생 증상.
5. Mac 방화벽 — 8080 포트 인바운드 허용 여부.
6. Supabase Auth 토큰 취득 — 실기기에서 `getAccessToken()` 정상 동작 여부 (Keychain access group).

**산출물**: `progress/mvp16/F1_diagnosis.md` (변경 요약·수정 파일 목록·테스트 명령+결과·스모크 테스트·리스크 5개 섹션)

**의존성**: 없음
**실행 순서**: 2
**예상 규모**: 소 (분석 + 문서)

---

### ST-3. F2 오답노트 태그 필터 회귀 진단 (research)

**목적**: 오답노트 태그 칩 클릭 시 필터링이 동작하지 않는 원인을 진단.

**진단 항목**:
1. git log/blame — `WrongAnswerTagFilter`, `FavoritesView.wrongAnswersContent` 마지막 변경 커밋.
2. `wrongAnswersPageIndex` + `wrongAnswersScrollID` 동기화 — `.onChange(of: wrongAnswersScrollID)` 로직 추적.
3. `wrongAnswerTagIds` 배열 구성 — `wrongAnswerTags`(computed) 결과가 비어있으면 칩 자체가 없어 필터 동작 안 함.
4. `TagChipBarView.onSelect` → `wrongAnswersPageIndex` 갱신 → `wrongAnswersScrollID` 갱신 → 콘텐츠 재렌더 흐름 확인.

**산출물**: 진단 결과 (원인·수정 범위) ST-6 구현 사양에 기록

**의존성**: 없음
**실행 순서**: 3
**예상 규모**: 소 (분석)

---

### ST-4. E1 iOS 피드 카드 태그 칩 렌더링 (feature — TDD)

**목적**: iOS 피드 카드(`ArticleCardView`)에 기사 태그 이름을 칩 형태로 표시.

**구현 사양**:
- `ArticleCardView`에 `tagName: String?` 파라미터 추가.
- `tagName` 있을 때 source 레이블 옆에 나란히 태그 칩 렌더링 (웹과 동일 레이아웃 — gray source 뱃지 옆 blue 칩).
- `FeedView.articleList(items:isCurrent:)` — `feature.tags`에서 `item.tagId`로 태그 이름 룩업 후 `ArticleCardView`에 전달.
- 태그 없는 기사(`tagId == nil`) → 칩 미표시.

**⚠️ 테스트 방법 선택 필수 (M2 구멍 해소)**

기존 `ArticleCardViewTests.swift`는 뷰 렌더링을 전혀 검증하지 않는다 — `view.article.title` 모델 프로퍼티 접근만 있다. M4 DoD가 요구하는 "ViewInspector 또는 스냅샷 테스트"를 위해 ST-4 진입 전 방법을 결정해야 한다:

- **(A) ViewInspector** — `ViewInspector` SPM 의존성 추가. 렌더 트리에서 특정 뷰 존재 여부 직접 assert 가능. 의존성 추가 비용 있음.
- **(B) 스냅샷 테스트** — `swift-snapshot-testing` 추가. 레이아웃 회귀까지 커버. 초기 레퍼런스 이미지 생성 필요.
- **(C) 모델 헬퍼 단위 테스트** — 렌더링 없이 `tagName` 파라미터 존재 여부 + nil 처리 로직만 테스트. 가장 빠르지만 실제 렌더링은 미검증.

**권장**: (C)로 단위 테스트 먼저 작성 후, 태그 칩 렌더링 확인이 시뮬레이터 E2E로 대체 가능하면 (C)로 완결. 뷰 계층 검증이 필요하면 (A) 선택.

**TDD 순서**:
1. 테스트 방법 (A/B/C) 결정.
2. `ArticleCardViewTests.swift`에 실패 테스트 추가 (tagName 있을 때 칩 표시, 없을 때 미표시).
3. `ArticleCardView` 구현 → 테스트 통과.
4. `FeedView` 연동 → FeedFeatureTests 스모크.

**산출물**: `ArticleCardView.swift` 수정, 테스트 통과

**의존성**: 없음 (서버 응답 `article.tags` 스키마는 M1에서 이미 확정)
**실행 순서**: 4
**예상 규모**: 소~중 (~40줄)

---

### ST-5. F1 실기기 오답 저장 픽스 (feature — TDD)

**목적**: ST-2 진단 결과를 반영해 실기기에서 오답 저장이 정상 동작하도록 수정.

**예상 픽스 방향** (ST-2 진단 결과에 따라 확정):
- ATS 이슈라면: Info.plist에 `NSAllowsArbitraryLoads` 또는 도메인별 예외 추가.
- mDNS 이슈라면: `Config.xcconfig`의 SERVER_URL을 IP 주소로 변경하거나 운영 서버 URL 사용.
- Keychain access group 이슈라면: Keychain 설정 수정.
- 토큰 만료 이슈라면: refresh 로직 보강.

**TDD 순서**:
1. 진단 결과가 mDNS/방화벽 설정이라면 — 단위 테스트 불가. `ServerConfigTests`에 xcconfig URL 파싱 테스트만 추가하고 실기기 스모크로 대체.
2. 진단 결과가 Keychain access group이라면 — 단위 테스트 추가 가능 (MockKeychain).
3. 픽스 적용 → 테스트 통과.
4. 실기기 스모크 테스트 기록 (F1_diagnosis.md에 추가).

**⚠️ 회귀 안전망 추가 필수 (M3 구멍 해소)**

ST-2 진단 결과가 `NSLocalNetworkUsageDescription` 또는 `NSBonjourServices` 누락이었다면, 픽스 후 재발을 방지하는 안전망 중 하나를 반드시 추가한다:

- **plist 키 존재 테스트**: `InfoPlistTests.swift`에 `NSLocalNetworkUsageDescription` 키가 Info.plist에 존재하는지 assert하는 단위 테스트 추가. CI에서 plist 누락 즉시 감지.
- **ServerConfig 파싱 단위 테스트**: 실기기 경로에서 URL이 실제로 반환되는지 `MockBundle`으로 검증. URL 파싱 회귀 방지.

ST-5 완료 후 위 중 한 가지를 적용하지 않으면 동일 회귀가 재발한다. F1_diagnosis.md 리스크 섹션에도 기록한다.

**산출물**: 픽스 코드 + 단위 테스트 통과 + F1_diagnosis.md 갱신 + 회귀 안전망 테스트 추가

**의존성**: ST-2 (진단 결과)
**실행 순서**: 5
**예상 규모**: 소~중 (원인에 따라 다름)

---

### ST-6. F2 오답노트 태그 필터 픽스 (feature — TDD)

**목적**: ST-3 가설 검증 후 오답노트 태그 칩 클릭 → 필터링 동작 복원.

**확정된 픽스 방향**:
- `FavoritesView.wrongAnswersContent`의 `TagChipBarView.onSelect` 콜백에 `wrongAnswersScrollID = idx` 한 줄 추가.
- 현재: `wrongAnswersPageIndex = idx` 만 있음 → 추가: `wrongAnswersScrollID = idx`.
- `wrongAnswerTags` computed 결과가 비어있는 경우 — WrongAnswer 로드 타이밍 이슈라면 로드 완료 후 태그 배열 갱신 확인.

**TDD 순서**:
1. `WrongAnswerTagFilterTests.swift` 기존 테스트 확인 + 회귀 케이스 추가.
2. 뷰 로직 픽스 → 시뮬레이터 E2E 확인.

**산출물**: `FavoritesView.swift` 수정, 테스트 통과

**의존성**: ST-3 (진단 결과)
**실행 순서**: 6
**예상 규모**: 소 (~20줄)

---

### ST-7. D2 오답 → 원문 이동 UI 구현 (feature — TDD, iOS + 웹)

**목적**: 오답노트에서 원문 기사로 이동하는 UI를 iOS + 웹 양쪽에 추가.

**구현 사양** (ST-1 사용자 확인 후 진입):
- iOS `WrongAnswerRow.swift`: "원문 보기" 버튼 추가. `article_url TEXT NOT NULL`로 항상 존재 — 분기 처리 불필요. `SafariView` (SFSafariViewController, 인앱 웹뷰) 적용.
- 웹 `WrongAnswerCard.svelte`: "원문 보기" 링크 추가. `articleUrl` 항상 존재 — 조건 분기 불필요. 새 탭: `<a href={item.articleUrl} target="_blank" rel="noopener noreferrer">` 방식.
- `articleUrl`이 빈 문자열인 방어 분기 불필요 — DB 스키마로 보장됨.

**TDD 순서**:
1. iOS: `WrongAnswerRow` 테스트에 "articleUrl 있을 때 버튼 노출" 실패 테스트 추가.
2. iOS: 구현 → 테스트 통과.
3. 웹: `WrongAnswerCard` 컴포넌트 테스트 추가 → 구현.

**산출물**: `WrongAnswerRow.swift` + `WrongAnswerCard.svelte` 수정, 테스트 통과

**의존성**: ST-1 (이동 방식 결정)
**실행 순서**: 7
**예상 규모**: 소~중 (~60줄 합산)

---

### ST-8. E2E 4종 시나리오 통과 확인 (chore)

**목적**: M4 DoD 기준 4종 E2E를 모두 통과시키고 자동화 테스트를 실행해 완료를 확인.

**체크리스트**:
- [ ] iOS 시뮬레이터: 피드 카드에 태그 칩 표시 (E1)
- [ ] 실기기: 오답 저장 → 즐겨찾기/오답노트 표시 (F1) — **실기기 필수**
- [ ] iOS 시뮬레이터: 오답노트 태그 칩 클릭 → 필터링 동작 (F2)
- [ ] iOS 시뮬레이터 + 웹: 오답 클릭 → 원문 이동 (D2)

**자동화 검증**:
```bash
# iOS 단위+UI 테스트
xcodebuild test -workspace Frank.xcworkspace -scheme Frank \
  -destination 'platform=iOS Simulator,name=iPhone 17 Pro'

# 웹 테스트
cd web && npm run test

# KPI 게이트
bash scripts/kpi-check.sh
```

**산출물**: 테스트 통과 로그, E2E 통과 확인

**의존성**: ST-4, ST-5, ST-6, ST-7 모두 완료
**실행 순서**: 8
**예상 규모**: 검증 (코드 없음)

---

## 의존성 DAG

```
ST-1 (D2 결정) ──────────────────────────────┐
ST-2 (F1 진단) ──→ ST-5 (F1 픽스) ──────────┤
ST-3 (F2 진단) ──→ ST-6 (F2 픽스) ──────────┼──→ ST-8 (E2E 4종)
ST-4 (E1 iOS 칩) ────────────────────────────┤
                   ST-7 (D2 UI) ─────────────┘
```

- ST-1, ST-2, ST-3, ST-4 는 병렬 실행 가능 (상호 의존 없음)
- ST-5는 ST-2 완료 후 진입
- ST-6는 ST-3 완료 후 진입
- ST-7은 ST-1 완료 후 진입
- ST-8은 ST-4, ST-5, ST-6, ST-7 모두 완료 후 진입

## 전체 규모 요약

| 서브태스크 | 유형 | 예상 규모 | 대상 |
|-----------|------|----------|------|
| ST-1 | decision | — | 인터뷰 |
| ST-2 | research | 소 | iOS Config / ATS / Keychain |
| ST-3 | research | 소 | iOS FavoritesView |
| ST-4 | feature | 소~중 (~40줄) | iOS ArticleCardView |
| ST-5 | feature | 소~중 | iOS Config/ATS/Keychain |
| ST-6 | feature | 소 (~20줄) | iOS FavoritesView |
| ST-7 | feature | 소~중 (~60줄) | iOS WrongAnswerRow + 웹 WrongAnswerCard |
| ST-8 | chore | — | 검증 |

총 코드 변경 예상: ~120~140줄 (진단 결과에 따라 F1 픽스 규모 변동)

---

## Feature List
<!-- size: 중형 | count: 22 | skip: false -->

### 기능
- [x] F-01 ArticleCardView에 tagName 파라미터 추가 후 source 뱃지 옆에 태그 칩 렌더링
- [x] F-02 FeedView에서 tagId → tagName 룩업 후 ArticleCardView에 전달
- [x] F-03 태그 없는 기사(tagId == nil)는 칩 미표시
- [x] F-04 WrongAnswerRow에 "원문 보기" 버튼 추가 (SFSafariViewController)
- [x] F-05 웹 WrongAnswerCard에 "원문 보기" 링크 추가 (새 탭)
- [x] F-06 F1 진단 결과 반영 픽스 적용 (실기기 오답 저장 정상화)
- [x] F-07 F2 오답노트 태그 필터 회귀 복원

### 엣지
- [x] E-01 피드에 태그 없는 기사 혼합 시 카드 레이아웃 깨지지 않음
- [x] E-02 F1 진단 — mDNS/ATS/Keychain 각 케이스 시나리오 커버
- [x] E-03 F2 필터 — 전체 선택(nil) → 특정 태그 → 다시 전체 전환 시 정상 동작
- [x] E-04 오답 카드에서 "원문 보기" 탭 후 앱 복귀 시 상태 유지

### 에러
- [x] R-01 FeedView에서 tagId 있지만 tagMap에 없을 때 칩 미표시(크래시 없음)
- [x] R-02 SafariView URL 유효하지 않은 경우 크래시 없음
- [x] R-03 F1 픽스 후 실기기 네트워크 단절 시 에러 메시지 노출

### 테스트
- [x] T-01 ArticleCardViewTests — tagName 있을 때 칩 표시, nil일 때 미표시
- [x] T-02 WrongAnswerRowTests — articleUrl 있을 때 "원문 보기" 버튼 표시
- [x] T-03 WrongAnswerTagFilterTests — 기존 회귀 케이스 + 신규 필터 전환 케이스
- [x] T-04 웹 WrongAnswerCard 컴포넌트 테스트 — articleUrl 링크 렌더링
- [x] T-05 xcodebuild test 전체 통과 (iPhone 17 Pro 시뮬레이터)
- [x] T-06 npm run test 전체 통과 (웹)

### UI·UX
- [x] U-01 태그 칩 색상·크기가 웹 피드 카드와 일관 (blue-100 배경, 소형 텍스트)
- [x] U-02 "원문 보기" 버튼 위치·레이블이 iOS/웹 오답 카드에서 자연스럽게 배치

### 플랫폼
- [x] P-01 F1 픽스가 시뮬레이터/실기기 양쪽에서 동작 확인
