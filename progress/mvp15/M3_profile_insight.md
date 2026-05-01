# M3: 프로필 + 내 시각 인사이트

> 프로젝트: Frank MVP15
> 상태: 대기
> 예상 기간: 1주
> 의존성: M2 (양 확대 완료)

## 목표

사용자 프로필(직업 한 줄)을 받아 LLM이 iOS 개발자 시각의 인사이트 단락을 생성하고, 웹·iOS 양쪽에서 설정·표시되도록 구현한다.

## 배경 (Q1·Q2 결정)

- **Q1**: 객관 요약 + 내 시각 단락 (같은 카드 안). 액션 곁들임은 MVP15 제외
- **Q2**: 직업 한 줄만 (예: "iOS 개발자"). 설정 페이지에 항목 추가, 선택적 (강제 X)
- **LLM 컨텍스트**: 직업 한 줄만. 피드 태그는 검색용으로만
- **미설정 시**: 인사이트 단락 생략, 일반 요약만 표시
- **백엔드**: 인사이트는 분리된 응답 필드 (UI 통합/분리 자유)

## 기존 인프라 (critical-review C1)

**profiles 테이블·트리거·`update_profile` API 이미 동작 중** — 메모리 `project_m6_profiles_migration` 참조.

- `supabase/migrations/20260404_create_profiles_tags_user_tags.sql` — profiles 테이블 (display_name, onboarding_completed)
- `handle_new_user` 트리거 — 가입 시 자동 row 생성
- `server/src/api/profile.rs::update_profile` — `UpdateProfileRequest { onboarding_completed, display_name }` 동작 중

→ **M3 작업은 "새로 만들기" X. ALTER 1줄 + 핸들러 occupation 처리 추가 + Profile 모델 occupation 필드.**

## LLM 호출 전략 (critical-review C3)

피드 1회 새로고침 = 10태그 × 10기사 = 100기사. 인사이트를 prefetch하면 LLM 호출 200+ → Groq RPM 30 즉시 초과.

→ **인사이트는 카드 펼침 시 lazy load**. 동시 호출 제한(semaphore) + 결과 캐시(같은 기사 재호출 방지).

## 응답 스키마 (critical-review M4)

웹·iOS 4개 컴포넌트가 동시 작업 → 응답 스키마가 SSOT 부재 시 디코딩 충돌. **응답 예시 JSON을 서버 PR 본문 또는 OpenAPI 스펙에 박제** + 공유 타입 정의 사용.

## 성공 기준 (Definition of Done)

- [ ] DB: 기존 profiles 테이블에 `occupation TEXT NULL` 단일 ALTER 마이그레이션
- [ ] 서버: `UpdateProfileRequest`에 occupation 필드 추가, `update_profile` 핸들러에 처리 추가
- [ ] 서버: `domain::models::Profile`에 occupation 필드 추가
- [ ] 서버: 인사이트 LLM 호출 서비스 (`services/insight.rs` 권장) 구현 — **lazy load 전제**
- [ ] 서버: 인사이트 호출에 동시 제한(semaphore) + 결과 메모리 캐시 적용 (Groq RPM 30 보호)
- [ ] 서버: 피드 응답에 `insight` 필드 추가 (인사이트 호출은 별도 엔드포인트 또는 펼침 시 호출)
- [ ] 서버: 인사이트 프롬프트 별도 파일/상수로 분리 (변경 비용 최소화)
- [ ] 서버: 응답 스키마 예시 JSON을 PR 본문 또는 OpenAPI 스펙에 박제 (웹·iOS 동시 작업 보호)
- [ ] 서버: occupation 입력 시 신분 단어("취준생", "학생" 등) 도움말 또는 필터 (선택)
- [ ] 웹: 설정 페이지에 직업 입력란 추가, PATCH 호출
- [ ] 웹: 피드 카드 "펼침" 시 인사이트 lazy 호출 + 단락 표시
- [ ] iOS: 설정 화면에 직업 입력 필드 추가
- [ ] iOS: 피드 카드 "펼침" 시 인사이트 lazy 호출 + 영역 표시
- [ ] 모든 플랫폼 테스트 통과 (서버 cargo test / 웹 vitest / iOS xcodebuild test)
- [ ] 본인 직접 사용: 직업 설정 후 인사이트 1회 이상 표시 확인 (E2E 검증)

## 아이템

| # | 아이템 | 유형 | 실행 스킬 | 순서 | 상태 |
|---|--------|------|----------|------|------|
| 1 | DB 마이그레이션 (profiles에 occupation ALTER) | feature | /workflow | 1 | 대기 |
| 2 | 서버 프로필 API에 occupation 추가 (Profile 모델 + UpdateProfileRequest) | feature | /workflow | 2 | 대기 |
| 3 | 서버 인사이트 서비스 (LLM 호출 함수, lazy 전제) + 프롬프트 분리 + semaphore + 캐시 | feature | /workflow | 3 | 대기 |
| 4 | 서버 인사이트 엔드포인트 또는 응답 필드 + 응답 스키마 예시 박제 + 통합 테스트 | feature | /workflow | 4 | 대기 |
| 5 | 웹 설정 페이지 직업 입력 | feature | /workflow | 5 (병렬) | 대기 |
| 6 | 웹 피드 카드 펼침 시 인사이트 lazy 호출 + 단락 표시 | feature | /workflow | 5 (병렬) | 대기 |
| 7 | iOS 설정 화면 직업 입력 | feature | /workflow | 5 (병렬) | 대기 |
| 8 | iOS 피드 카드 펼침 시 인사이트 lazy 호출 + 영역 표시 | feature | /workflow | 5 (병렬) | 대기 |

순서 1~4는 서버, 5~8은 클라이언트 4개 병렬 (메모리 `feedback_parallel_agents`).

## 워크플로우 진입점

```
/workflow "M3-profile-insight"
```

**메인태스크**: 사용자 프로필(직업 한 줄)을 받아 iOS 개발자 시각의 인사이트를 생성·표시한다 (서버 + 웹 + iOS).

## KPI (M3)

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| profiles.occupation 컬럼 존재 | DB 스키마 검사 또는 ALTER 마이그레이션 파일 존재 | exists | Hard | — |
| 서버 프로필 API 테스트 | `cargo test profile` | 통과 | Hard | — |
| 서버 인사이트 서비스 테스트 | `cargo test insight` | 통과 | Hard | — |
| 인사이트 프롬프트 별도 파일 | grep "system prompt" 위치 | 별도 파일/상수 | Hard | — |
| 인사이트 lazy 호출 검증 (prefetch 안 함) | 통합 테스트 또는 코드 리뷰 | 펼침 전 호출 0 | Hard | — |
| 인사이트 동시 호출 제한 동작 | 통합 테스트 (mock으로 RPM 시뮬레이션) | 통과 | Hard | — |
| 응답 스키마 예시 박제 | PR 본문, OpenAPI 스펙, 또는 `M3_response_schema.md` | exists | Hard | — |
| 웹 설정 페이지 동작 | vitest + 수동 1회 | 통과 | Hard | — |
| 웹 카드 인사이트 표시 | 컴포넌트 테스트 + 수동 1회 | 통과 | Hard | — |
| iOS 설정 화면 동작 | xcodebuild test + 시뮬레이터 1회 | 통과 | Hard | — |
| iOS 카드 인사이트 표시 | 시뮬레이터 1회 | 통과 | Hard | — |
| 미설정 시 인사이트 생략 | 직업 빈 상태로 피드 호출, response.insight == null | 확인 | Hard | — |
| 본인 E2E (직업 설정 → 인사이트 표시) | 본인 직접 1회 (메모리 `feedback_e2e_before_commit`) | 통과 | Hard | — |
| LLM 호출 비용 0 | Groq 무료 티어 안 | $0 | Hard | — |

## 리스크

| 리스크 | 영향(H/M/L) | 대응 |
|--------|------------|------|
| LLM 인사이트 품질 일반적·뻔함 | M | "잘 모르겠다" 상태 결정 — 써보고 부족하면 자유 텍스트 추가 (변경 비용 한나절). MVP15 범위 외 |
| 인사이트 호출이 LLM 한도 초과 (Groq RPM 30) | H | **lazy load + semaphore + 결과 캐시 필수**. 펼침 시에만 호출, 동시 호출 제한 |
| 직업 한 줄에 "취준생" 같은 단어 입력 시 LLM이 그대로 사용 | M | 도움말 안내 ("직업명 위주, 신분 단어 X"). 또는 서버에서 키워드 필터링 |
| 프로필 미설정 시 카드 레이아웃 깨짐 | L | 컴포넌트 테스트로 빈 상태 검증 |
| 웹+iOS 응답 스키마 불일치 | H | **응답 예시 JSON을 서버 PR/OpenAPI에 박제** + 공유 타입 정의. 4개 컴포넌트 병렬 작업 보호 필수 |
