# M3 워크플로우 메인태스크

> 생성일: 2026-04-08
> 워크트리: `frank-m3-ios`
> 브랜치: `feature/260408_m3_ios`
> 기획 원본: `progress/260406_MVP3_M3_iOS전환.md` (ST-1~ST-9 동결)
> API SPEC: `progress/260407_API_SPEC.md` (FROZEN)
> 인프라: `progress/260407_MVP3_M1.5_병렬준비.md`

## 메인태스크

**M3: iOS 데이터 호출을 Rust API로 통합 + iOS 모델을 server SPEC과 일치 + MVP2 부채 흡수**

## 핵심 원칙 (절대 위반 금지)

1. **격리**: `ios/` 디렉토리만 수정. `server/`, `web/`은 read-only
2. **Mock-First**: ST-3까지 외부 자원(Supabase, Rust 서버) 호출 0건
3. **M1.5 인프라 강제**: `Core/Adapters/Mock*.swift` 우회/삭제 금지
4. **Supabase 어댑터 보존**: `SupabaseTagAdapter`, `SupabaseArticleAdapter` 파일 삭제 금지 (DI에서 사용만 안 함)
5. **공통 문서 read-only**: 본 문서 + `260408_M3_*` 회고만 수정. 로드맵/M2/M1.5 문서 손대지 말 것
6. **Server 작업 발견 시 즉시 정지**: server/ 변경 필요 시 작업 멈추고 사용자에게 보고
7. **Tuist 직접 호출**: `~/.tuist/Versions/4.31.0/tuist generate --no-open` (tuistenv 회피)

## 병렬 컨텍스트

다른 탭에서 **M2(웹)**가 동시 진행 중. 외부 자원 공유 없음 (Mock-First로 격리).

## 인터뷰 결정사항

| Q | 결정 | 근거 |
|---|---|---|
| Q1(전): ST-7 부채 흡수 진행 방식 | **A. 메인 문서대로 한 PR에 포함** | 동결된 계획, 분량 합리적, 재정비 커밋(c1c2e7b)으로 최신화 완료 |
| Q1(후): Profile.email 처리 | **A. SPEC 그대로 제거** | UI 사용처 0건 (grep 확인) → 단순 제거 |
| Q2: ProfilePort 신설 | **A. 안 함** | M3 메인 문서가 binding source. 부채는 회고에 기록 (MVP3 후속/M4 sync 시점) |
| Q3: LoadingPhase enum 형태 | **C. ST-7 진입 시 결정** | FeedFeature 4 Bool 동시성 시나리오 사전 측정 후 결정 |
| Q4: ST-8 실 서버 검증 | **C. ST-8 도달 시 재확인** | M2 탭 진행 상태에 따라 결정 |

## 서브태스크 (원본 그대로, 순서 변경 금지)

| ID | 유형 | 내용 | 의존 |
|---|---|---|---|
| ST-1 | refactor | iOS 모델 미스매치 정정 (Profile/Article/Tag) + MockFixtures 갱신 | — |
| ST-2 | test | Mock 모드 화면/네비/상태 동작 검증 (외부 호출 0) | ST-1 |
| ST-3 | feature | APITagAdapter 구현 (TDD, URLProtocol mock) | ST-1 |
| ST-4 | feature | APIArticleAdapter 구현 (TDD, URLProtocol mock) | ST-1 |
| ST-5 | feature | SupabaseAuthAdapter.updateOnboardingCompleted → Rust PUT /api/me/profile | ST-1 |
| ST-6 | chore | AppDependencies.live() 어댑터 교체 (Supabase 보존) | ST-3, ST-4, ST-5 |
| ST-7 | refactor | MVP2 부채 흡수 (LoadingPhase enum, UITest E2E, 태그 로딩 중복 추출) | ST-6 |
| ST-8 | test | 전체 테스트 + 시뮬레이터 검증 (단위+UITest+swiftlint) | ST-3~ST-7 |
| ST-9 | chore | main rebase + 머지 준비 + 회고 작성 | ST-8 |

## 완료 기준

- [ ] `xcodebuild build` 통과
- [ ] `xcodebuild test` 단위 117+ / UITest 통과, 커버리지 90%+
- [ ] `swiftlint lint --strict` 통과
- [ ] iOS 모델이 server SPEC과 일치
- [ ] `AppDependencies.live()`가 APITagAdapter/APIArticleAdapter 사용
- [ ] Supabase/Mock 어댑터 파일 보존
- [ ] FeedFeature LoadingPhase enum 적용
- [ ] UITest 크로스-Feature E2E 추가
- [ ] `progress/260408_M3_회고.md` 작성

## 워크플로우 진행 상태

- [x] /step-1 — 메인태스크 설정 (현재)
- [ ] /step-2 — 룰즈 검증
- [ ] /step-3 — 서브태스크 분리
- [ ] /step-4 — 서브태스크 인터뷰
- [ ] /step-5 — 서브태스크 리뷰
- [ ] /step-6 — 구현
- [ ] /step-7 — 리팩토링 + 코드 리뷰
- [ ] /step-8 — 테스트
- [ ] /step-9 — 커밋
