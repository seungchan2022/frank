# MVP15 회고

> **헤드라인**: 직업 시각 인사이트 — "내 관점으로 읽는 뉴스"의 첫 삽
> **기간**: 2026-05-01 ~ 2026-05-04
> **브랜치**: feature/mvp15-m3-profile-insight
> **총 커밋**: 16개 (M1~M3 + 버그픽스 포함)

---

## 무엇을 했나

### M1 — 검색엔진 진단 (2026-05-02)

"1개만 나오는 태그" 원인을 파악하기 위해 12 태그 × 5 변형 × 2 엔진 = 120셀 실측 진단을 수행했다. 결론은 **Tavily가 주력이고, 한국어 태그명을 그대로 쿼리에 넣으면 영어 결과가 빈약**하다는 것. 영어 검색 키워드 매핑 테이블(`tag_search_keyword()`)을 도입하는 근거를 확보했다.

### M2 — 양 확대 + 무료 한도 보호 (2026-05-02)

- limit 5 → 20으로 확대
- 월 누적 검색 횟수 추적 + iMessage 알림 구현
- Mock 토글 인프라 추가 (테스트 환경 격리)

### M3 — 프로필 + 내 시각 인사이트 (2026-05-03 ~ 05-04)

36개 피처(F/E/R/T/U 분류) 전항목 구현 완료.

- **서버**: `occupation` 컬럼 마이그레이션 → Profile API PATCH → `summarize_with_occupation` LLM 분기 → `POST /me/rewrite` 엔드포인트 신규
- **웹**: 설정 페이지 occupation 입력 → 요약하기 insight 섹션 → 재작성 버튼(occupation 필요)
- **iOS**: Settings occupation 필드 → ArticleDetailFeature rewrite 포트/어댑터 → insight nil 처리

### 버그픽스 (M3 이후 — 2026-05-04)

| 버그 | 원인 | 수정 |
|------|------|------|
| Firecrawl 403 | 일부 URL 크롤 차단 | snippet → title 폴백 |
| 한국어 태그 쿼리 노이즈 | 태그명 그대로 Tavily 쿼리 | `tag_search_keyword()` 매핑 |
| 짧은 snippet 랜딩페이지 | 30자 미만 snippet 필터 누락 | 30자 미만 URL skip |
| 재작성 → 자동 스크랩 | UPSERT + refreshFavorites() 콜백 | UPDATE-only + 클라이언트 refresh 제거 |
| 스크랩 시 rewrite 미포함 | addFavorite 파라미터 누락 | 전 레이어 rewrite 전파 |

---

## 핵심 의사결정

### 결정 1: 재작성은 자동 스크랩하지 않는다

**상황**: 재작성 버튼을 누르면 favorites DB에 UPSERT가 발생해 제목/source 없는 빈 행이 스크랩 탭에 나타났다.

**선택지**:
- A. 자동 스크랩 유지하되 title/source를 채워서 저장
- B. 요약하기처럼 결과만 표시, 스크랩은 사용자 명시 선택

**결정**: B.

**왜**: 요약/재작성은 "보고 판단하는 기능"이고 스크랩은 "보관하겠다는 의사 표시"다. 둘을 묶으면 사용자 의도와 어긋난다. 재작성 결과가 마음에 들 때만 스크랩하면 되고, 그때 rewrite 결과를 payload에 포함해서 함께 저장하면 충분하다.

**인사이트**: 자동화는 사용자 의도와 일치할 때만 유효하다. 일치 여부를 의심하지 않으면 조용한 데이터 오염이 생긴다.

### 결정 2: M4 스킵

**상황**: M4는 통합 E2E 시나리오 추가 + 2주 dogfooding이었다.

**선택지**:
- A. M4 계획대로 진행
- B. M3에서 이미 브라우저/시뮬레이터 E2E 검증 완료했으므로 스킵, MVP16으로 이행

**결정**: B.

**왜**: M3 과정에서 본인 E2E(브라우저 + iOS 시뮬레이터 수동) 이미 통과했고, 독립적 자동화 E2E는 MVP14에서 Playwright/XCUITest 인프라를 구축했다. 추가 E2E 시나리오는 MVP16 피드 품질 개선과 함께 사용 맥락에서 자연스럽게 생긴다.

### 결정 3: 태그 검색 키워드 영어 매핑 → MVP16 지속 튜닝

한국어 태그명 → 영어 쿼리 매핑 테이블을 도입했지만 여전히 일부 노이즈가 남아있다. 완전한 해결은 MVP16에서 실사용 피드백을 반영해 반복 튜닝하는 것이 현실적이다.

---

## 수치 요약

| 항목 | 수치 |
|------|------|
| 서버 테스트 통과 | 전체 통과 (`cargo test`) |
| 웹 테스트 통과 | 267개 (`vitest`) |
| iOS 테스트 통과 | 273개 (`xcodebuild test`) |
| 피처 완료 | 36/36 (100%) |
| 마일스톤 완료 | M1✅ M2✅ M3✅ M4⏭ |

---

## 다음 MVP에 넘기는 것

| 항목 | 내용 |
|------|------|
| 피드 노이즈 튜닝 | 태그별 keyword 매핑 정밀화, 결과 관련도 향상 |
| Playwright E2E | occupation → insight → rewrite → scrap 시나리오 자동화 |
| iOS XCUITest | 재작성 시나리오 UITest 추가 |
| 기술부채 DEBT-01~03 | MVP12 이관 부채 (오답 태그 칩, Prefetch 전략, iOS 유닛 테스트) |

---

## 배운 것

**UPSERT는 신중하게**: favorites `update_favorite_rewrite` 함수가 UPSERT로 구현돼 있어서, 사용자가 스크랩하기 전인 기사에도 빈 행이 생겼다. DB 설계에서 "없으면 건너뜀"이 당연한 게 아니라는 것을 다시 확인했다.

**레이어 전파 체크리스트**: `rewrite` 파라미터를 한 레이어에만 추가하고 나머지를 빠뜨리는 실수가 발생했다. client.ts → realClient.ts → mockClient.ts → favoritesStore → +page.svelte → 서버 AddFavoriteRequest → INSERT까지 전 레이어를 동시에 확인하는 습관이 필요하다.

**요약 = 보기, 스크랩 = 보관**: 기능 설계 시 사용자 의도 흐름을 명확히 정의해야 자동화의 경계가 보인다.
