# MVP13 로드맵 — 실사용 전환

> 기획일: 2026-04-28  
> 상태: in-progress  
> 테마: 오답 태그 수정 + 피드 품질 개선 + 클라우드 배포

## 목표

MVP12까지 기능 구현 위주였다면, MVP13은 **실제로 쓸 수 있는 앱**으로 전환하는 것.
- 오답노트 태그 필터 근본 수정 (DEBT-01 해소)
- 피드 기사 DB 저장 + 중복 제거로 품질 개선
- 클라우드 배포로 Mac 없이도 실기기 사용 가능

## 마일스톤

| 마일스톤 | 주제 | 앱 | 상태 |
|----------|------|----|------|
| M1 | DEBT-01 — 오답 태그 필터 (서버+DB) | 서버 | ✅ done |
| M2 | 오답 태그 필터 클라이언트 전환 + 피드 품질 개선 | 웹+iOS | ✅ done |
| M3 | 클라우드 배포 | 서버+웹+iOS | ⏸ deferred |

---

## M1 — 서버+DB (DEBT-01 오답 태그 필터)

### 현상
웹·iOS 오답노트에서 태그 필터가 제대로 동작하지 않음.
favorites에 없는 기사의 오답은 태그 필터에서 제외됨.

### 근본 원인
`quiz_wrong_answers` 테이블에 `tag_id` 컬럼이 없음.
웹·iOS 모두 `favorites.tag_id`를 브릿지로 삼아 간접 필터링 중.

### 구현 범위 (서버+DB만)
1. **DB**: `quiz_wrong_answers`에 `tag_id UUID` 컬럼 추가 (마이그레이션)
2. **서버**: 오답 저장 시 `tag_id` 함께 저장
3. **서버**: `GET /api/me/quiz/wrong-answers?tag_id=` 필터 파라미터 추가

### 관련 파일
- `supabase/migrations/` — 새 마이그레이션 추가
- `server/src/infra/postgres_quiz_wrong_answers.rs`
- `server/src/domain/models.rs` (QuizWrongAnswer, SaveWrongAnswerParams)

---

## M2 — 웹+iOS

### 구현 범위
**1. 오답 태그 필터 클라이언트 전환 (M1 API 기반)**
- 웹: `favorites-filter.ts` 클라이언트 필터 제거 → `?tag_id=` 서버 필터 사용
- iOS: `WrongAnswerTagFilter.swift` 클라이언트 필터 제거 → 서버 필터 사용

**2. 피드 품질 개선**
- 서버: Exa 기사 DB 저장 + URL unique constraint 중복 제거
- 서버: 새로고침 시 DB 우선 반환 → 백그라운드 Exa 갱신
- iOS: 피드 fetch 방식 웹과 통일 (전체 1회 fetch → 태그 탭 클라이언트 필터)

### 관련 파일
- `web/src/lib/utils/favorites-filter.ts`
- `ios/Frank/Frank/Sources/Features/Favorites/WrongAnswerTagFilter.swift`
- `server/src/infra/` — 피드 DB 캐싱 어댑터

### 미결 사항 (구현 중 판단)
- Exa 무료 플랜 한도 확인 후 기사 수 조정 여부 결정

---

## M3 — 클라우드 배포

기획 완료 → `history/mvp13/M3_cloud_deploy.md` 참조.

**범위**: GitHub Actions / 도메인 / HTTPS 없는 최소 구성 (개인 사용 목적)  
**플랫폼**: Oracle Cloud Always Free A1 (4 OCPU / 24 GB, ARM64) + PAYG 업그레이드  
**접근 방식**: VM 공인 IP + HTTP 직접 사용. iOS ATS 예외로 HTTP 허용.  
**보류 사유**: Oracle PAYG 업그레이드 서버 오류 지속. 지원팀 이메일 문의 완료 (2026-04-29).  
**재개 조건**: Oracle 지원팀 답변 후 PAYG 해결 시 도쿄 리전에서 재시도.

---

## 이관 부채

| ID | 내용 | 처리 |
|----|------|------|
| DEBT-01 | 오답 태그 필터 | M1(서버+DB) + M2(웹+iOS)에서 흡수 |
| DEBT-02 | iOS 피드 탭 전략 | M2에서 흡수 (iOS 싱크) |
| DEBT-03 | iOS 테스트 커버리지 자동화 | 보류 유지 |
