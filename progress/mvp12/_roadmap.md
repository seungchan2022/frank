# Frank MVP12 로드맵

> 생성일: 260427
> 상태: in-progress
> 원칙: 버그 수정 우선 → 서버 먼저 → M1 완료 후 웹·iOS 병렬 진행

## 목표

실사용 중 발견된 버그 6개를 수정하고, 좋아요/즐겨찾기 UX를 재설계한다.
서버 피드 페이지네이션을 도입해 데이터 누적에 따른 첫 로드 성능 저하를 해결한다.

## 버그 현황

| ID | 증상 | 플랫폼 | 심각도 | 마일스톤 |
|----|------|--------|--------|---------|
| BUG-A | BBC 토픽 URL이 피드에 기사로 노출 | 서버 | Medium | M1 |
| BUG-B | snippet에 마크다운 헤더·메타 텍스트 오염 | 서버 | Medium | M1 |
| BUG-C | 마지막 기사 삭제 후 스크랩 탭 전체 먹통 | 웹 | High | M2 |
| BUG-D | 즐겨찾기 추가 후 새 태그 미생성 | iOS | High | M3 |
| BUG-E | 마지막 기사 삭제 후 스크랩 탭 빈 화면 고정 | iOS | High | M3 |
| BUG-F | 오답노트 태그 필터 미작동 | 웹+iOS | High | M2·M3 |

## 마일스톤 구성

```
M1 (서버) ──────────────────────────────────────────▶
          │
          └─ M2 (웹) ──────────────────────────────▶  ← 병렬
          └─ M3 (iOS) ─────────────────────────────▶  ← 병렬
```

> M1 완료 후 M2·M3는 서로 독립적으로 병렬 진행 가능.

## 마일스톤 목록

| 마일스톤 | 주제 | 내용 | 앱 | 상태 |
|---|---|---|---|---|
| M1 | 서버 품질 보완 + 피드 페이지네이션 | BUG-A, BUG-B, 피드 limit/offset API | 서버 | ✅ done |
| M2 | 웹 버그 수정 + UX 개선 | BUG-C, BUG-F(웹), 좋아요/즐겨찾기 UX 재설계, 무한 스크롤 | 웹 | 🔲 planning |
| M3 | iOS 버그 수정 + UX 개선 | BUG-D, BUG-E, BUG-F(iOS), 좋아요/즐겨찾기 UX 재설계, 무한 스크롤 | iOS | 🔲 planning |

## M1 상세

- **BUG-A**: `feed.rs` `is_listing_url()`에 `/news/topics/` 패턴 추가
- **BUG-B**: `clean_snippet()` 보강 — `#` 헤더 제거, 메타 텍스트(저자·댓글수·목차) 필터
- **피드 페이지네이션**: `FeedQuery`에 `limit: Option<u32>` + `offset: Option<u32>` 추가, 검색 결과 Vec을 핸들러에서 슬라이싱 (캐시 키 유지, limit 미지정 시 전체 반환)

## M2 상세

- **BUG-C**: 태그 삭제 시 `selectedTagId`를 `null`(전체)로 자동 초기화
- **BUG-F 웹**: `filterWrongAnswers` `tagId === undefined` 처리 정책 수정 + `wrongAnswerFilterTags` derived 추가
- **좋아요/즐겨찾기 UX**: 두 기능 구분이 직관적으로 보이도록 아이콘·레이블·배치 재설계 (서버 API 변경 없음)
- **무한 스크롤**: 스크롤 끝 감지 → `offset` 증가 → 추가 피드 요청 (`IntersectionObserver`)

## M3 상세

- **BUG-D**: 즐겨찾기 추가 이벤트 시 스크랩 탭 태그 목록 재조회
- **BUG-E**: 마지막 기사 삭제 시 `selectedTagId` nil 초기화 + "기사 없음" 메시지 표시
- **BUG-F iOS**: `filteredWrongAnswers` computed 연결 수정 + 오답이 있는 태그만 칩 표시
- **좋아요/즐겨찾기 UX**: 웹과 동일 방향, iOS 네이티브 컴포넌트로 재설계
- **무한 스크롤**: 피드 리스트 하단 도달 시 다음 페이지 요청 (`onAppear` 또는 `LazyVStack`)

## KPI (MVP12 최종)

| 지표 | 측정 방법 | 목표 | 게이트 | 기준선 |
|---|---|---|---|---|
| 서버 테스트 커버리지 | cargo-tarpaulin | ≥90% | Hard | MVP10 285개 |
| 웹 테스트 커버리지 | vitest --coverage | ≥90% | Hard | MVP11 231개 |
| iOS 테스트 커버리지 | xcodebuild + xccov | ≥85% | Soft | MVP10 221개 |
| 버그 6건 재현 0 | 수동 QA (각 버그 재현 시나리오) | 0건 | Hard | — |
| 피드 무한 스크롤 동작 | 수동 확인 (웹+iOS) | 정상 동작 | Hard | — |
| MVP 회고 작성 | history/mvp12/retro.md 존재 | exists | Hard | — |

## 기술 결정 기록

| 항목 | 결정 | 이유 |
|---|---|---|
| BUG-F 필터링 방식 | 클라이언트 사이드 (B안) | 서버 스키마 변경 없이 웹에 이미 구현된 구조 활용. A안은 4개 레이어 변경 + denormalization 부채 발생 |
| 좋아요→즐겨찾기 통합 | 통합 없이 UX 재설계만 | 두 기능의 역할(개인화 신호 vs 스크랩 저장)은 유지, 사용자 혼란만 제거 |
| 페이지네이션 범위 | 피드만 | 즐겨찾기·오답노트는 데이터량 적어 불필요. 필요 시 다음 MVP에서 추가 |
