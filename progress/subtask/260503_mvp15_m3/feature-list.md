# M3 Feature List

> MVP15 M3 — 프로필 + 내 시각 인사이트
> 작성일: 2026-05-03
> 인터뷰 확정값 반영

## Feature List
<!-- size: 대형 | count: 36 | skip: false -->

### 기능
- [x] F-01 profiles 테이블에 `occupation TEXT NULL` 컬럼 추가 (ALTER 마이그레이션) ← ST-1
- [x] F-02 favorites 테이블에 `rewrite TEXT NULL` 컬럼 추가 (ALTER 마이그레이션) ← ST-1 (step-5 C2 수정: 동일 마이그레이션 파일에 포함)
- [x] F-03 `Profile` 도메인 모델에 `occupation: Option<String>` 필드 추가
- [x] F-04 `UpdateProfileRequest`에 `occupation: Option<String>` 필드 추가
- [x] F-05 `update_profile` 핸들러에 occupation 저장 처리 추가
- [x] F-06 `LlmSummary.insight` 타입 `String → Option<String>` 변경
- [x] F-07 `LlmPort::summarize_with_occupation` — occupation 있으면 insight 생성, 없으면 None
- [x] F-08 `POST /me/summarize` — JWT → occupation 조회 후 `summarize_with_occupation` 호출
- [x] F-09 `POST /me/rewrite` 엔드포인트 신규 — occupation 없으면 400, 있으면 LLM 재작성 호출
- [x] F-10 재작성 결과 favorites `rewrite` 컬럼에 저장
- [x] F-11 `M3_response_schema.md` 응답 스키마 예시 JSON 박제
- [~] deferred (ST-5에서 구현 예정) F-12 웹 설정 페이지 occupation 입력란 추가 + PATCH 저장
- [~] deferred (ST-6에서 구현 예정) F-13 웹 요약하기 — insight 있으면 표시, null이면 섹션 숨김
- [~] deferred (ST-6에서 구현 예정) F-14 웹 재작성 버튼 — occupation 설정 시에만 렌더링
- [x] F-15 iOS 설정 화면 occupation 입력 필드 추가 + PUT 저장 ← ST-7 완료
- [x] F-16 iOS 요약하기 — insight null 시 섹션 숨김 ← ST-8 완료
- [x] F-17 iOS 재작성 버튼 — occupation 설정 시에만 렌더링 ← ST-8 완료

### 엣지
- [x] E-01 occupation 50자 초과 입력 시 400 반환
- [x] E-02 occupation 공백 문자열 입력 시 빈 값으로 처리 (trim 후 None)
- [x] E-03 occupation 미설정 유저가 `/me/rewrite` 호출 시 400
- [x] E-04 재작성 결과가 favorites 저장 실패해도 200 반환 (저장은 best-effort)
- [~] deferred (E2E에서 검증 예정) E-05 요약하기 + 재작성 동시 호출 시 각각 독립 처리

### 에러
- [~] deferred (기존 Groq 어댑터에서 처리됨, E2E 검증 예정) R-01 Groq 호출 timeout (10s) 시 503 반환
- [~] deferred (기존 Groq 어댑터에서 처리됨) R-02 Groq 1회 retry 후에도 실패 시 503 반환
- [x] R-03 에러 응답에 occupation 내용 노출 금지 (에러 메시지는 occupation_required 고정 문자열)
- [~] deferred (현재 400 반환으로 설계 변경 — degrade 없음) R-04 DB occupation 조회 실패 시 500 반환

### 테스트
- [x] T-01 `cargo test profile` — occupation PATCH 정상/50자 초과/공백 3개 시나리오
- [x] T-02 `cargo test summarize` — occupation 있/없을 때 insight 포함/None 검증
- [x] T-03 `cargo test rewrite` — 정상 200, occupation 없음 400 검증
- [~] deferred (ST-5~6에서 구현 예정) T-04 웹 vitest — occupation 입력/저장 동작, 재작성 버튼 숨김 조건
- [x] T-05 iOS xcodebuild test — SettingsFeature occupation, ArticleDetailFeature rewrite 시나리오 ← ST-7/ST-8 완료 (273 tests passed)
- [~] deferred (M3 완료 전 검증 예정) T-06 본인 E2E — 직업 설정 → 요약하기 인사이트 표시 1회 확인
- [~] deferred (M3 완료 전 검증 예정) T-07 본인 E2E — 직업 설정 → 재작성 1회 확인

### UI·UX
- [~] deferred (ST-6 스크린샷으로 검증 예정) U-01 웹 insight 섹션 — null 시 레이아웃 깨짐 없음 확인
- [x] U-02 iOS insight 섹션 — nil 시 섹션 숨김 처리 구현 완료 (E2E 시뮬레이터 검증 사용자에게 위임)
- [~] deferred (ST-6~8에서 구현 예정) U-03 재작성 로딩 중 버튼 비활성화 (요약하기와 동일 패턴)
