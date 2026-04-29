# 기술 부채 목록

의도적으로 보류한 설계·구현 결정. 다음 MVP 기획 시 흡수 여부 판단.

> 최종 갱신: 2026-04-29

---

## [DEBT-01] 오답 태그 필터링 — DB 컬럼 방식(A안) 보류

**발생**: MVP12 BUG-F 처리 시점
**상태**: ✅ **RESOLVED** (260428 MVP13 M1+M2에서 해소)
**관련 버그**: BUG-003 (RESOLVED), BUG-F (RESOLVED)

### 현상

웹·iOS 오답노트에서 태그 필터가 제대로 동작하지 않음.
- favorites에 없는 기사의 오답은 태그 필터에서 아예 제외됨
- 태그 칩 자체가 안 나오는 경우 발생

### 근본 원인 (260428 코드 탐색 확인)

`quiz_wrong_answers` 테이블에 **tag_id 컬럼이 없음**.

오답 저장 시점에 어느 태그 기사인지 기록하지 않아서,
웹·iOS 모두 `favorites.tag_id`를 브릿지로 삼아 `article_url` 기준 간접 매핑으로 클라이언트 필터링 중.

```
현재(B안):
  서버 → 전체 오답 반환
  클라이언트 → favorites[url → tag_id] 맵 생성 → 필터링
  문제: favorites에 없는 기사 오답은 태그 정보 없음 → 필터 제외
```

**관련 파일**:
- DB: `supabase/migrations/20260412_mvp8_m1_schema.sql` (tag_id 컬럼 없음)
- 서버: `server/src/infra/postgres_quiz_wrong_answers.rs` (WHERE user_id 만 필터)
- 서버: `server/src/domain/models.rs` (QuizWrongAnswer, SaveWrongAnswerParams — tag_id 필드 없음)
- 웹: `web/src/lib/utils/favorites-filter.ts` (favorites 기반 클라이언트 필터)
- iOS: `ios/Frank/Frank/Sources/Features/Favorites/WrongAnswerTagFilter.swift` (favorites 기반 클라이언트 필터)

### 해결 방향 (A안)

1. **DB 마이그레이션**: `quiz_wrong_answers`에 `tag_id UUID` 컬럼 추가
2. **서버**: 오답 저장 시 tag_id 함께 저장, API에 `?tag_id=` 쿼리 파라미터 추가
3. **웹**: 클라이언트 필터 로직 제거 → 서버 필터 사용
4. **iOS**: 동일하게 클라이언트 필터 제거 → 서버 필터 사용

**흡수 조건**: MVP13 M1에서 구현

---

## [DEBT-02] iOS 피드 탭별 로딩 전략

**발생**: MVP12 M3 완료 시점
**상태**: ✅ **RESOLVED** (260428 코드 탐색 확인) — C안 적용 완료
**수정 내용**: "all" 탭 첫 페이지만 즉시 로드, 나머지 태그 탭은 첫 접근 시 lazy 로드 (`selectTag()` 캐시 미스 시 API 호출)

**잔여 관찰 사항 (신규 추적 필요 시)**: 웹은 "all" 단일 캐시 + 클라이언트 필터, iOS는 탭별 독립 캐시 + 서버 요청으로 구현 방식이 다름. 현재 기능상 문제는 없으나 웹 무한 스크롤 시 태그 탭 기사 희박 이슈 가능성 있음 → MVP13 피드 UX 마일스톤 시 재검토

---

## [DEBT-03] iOS 유닛 테스트 커버리지 수치 측정 자동화

**발생**: MVP12 종료 시점
**상태**: 🟡 **DEFERRED** (실질적 영향 낮음)

**실제 현황** (260428 코드 탐색 확인): iOS 테스트 파일 17개 확인됨. Adapter 4개, Feature 8개, Component 3개, 순수함수 2개. 커버리지 수치 자동 측정 스크립트 미구성.

**흡수 조건**: 커버리지 수치가 KPI 게이트로 필요해지는 시점에 `xccov` 연동
