# MVP12 M1 서브태스크 목록

> 생성일: 260427  
> 마일스톤: M1 — 서버 품질 보완 + 피드 페이지네이션  
> 브랜치: feature/260427_mvp12-m1-server-quality  
> 상태: done

## 의존성 DAG

```
ST-M1-1 (BUG-A)  ──┐
                    ├──▶ cargo test (전체 검증)
ST-M1-2 (BUG-B)  ──┤
                    │
ST-M1-3 (페이지네이션) ──┘
```

세 서브태스크는 서로 독립적. 실행 순서: ST-M1-1 → ST-M1-2 → ST-M1-3

---

## 리뷰 결과 (step-5, 260427)

### Claude 리뷰 (문서 일치성)
- ST-M1-1: `STRICT_LISTING_SEGMENTS` 비대칭 근거 부재 → 차단 기준을 "해시/UUID 세그먼트"로 좁혀 근거 명시
- ST-M1-2: 패턴이 TBD 상태(Step 4에서 확정 예정) → 리뷰 불가 상태. 즉시 패턴 확정 필요
- ST-M1-3: `DbPort::get_feed_items` 변경 지시가 현재 코드베이스와 불일치 (해당 메서드 미존재)

### Codex 리뷰 (기술적 타당성)
- ST-M1-1: STRICT 리스트 4개 선택 근거 없음, `/tags/rust/some-article` 차단은 기존 `/category/tech/article-slug` 통과 원칙과 모순
- ST-M1-2: 패턴 스펙 미정, 오탐 범위(줄/문장/단어) 미명시, 한국어 대응 없음
- ST-M1-3: `get_feed_items` 미존재, 피드는 DB 기반이 아닌 검색 API 기반, `infra/postgres.rs` 파일명도 실제와 다름(`postgres_db.rs`)

### 구멍 찾기 리뷰 (critical-review)
- **치명 2건**: C1 캐시 키 분절화 전략 부재 / C2 `get_feed_items` 아키텍처 오류
- **중대 3건**: M1 오탐 위험(슬러그 차단) / M2 한국어 패턴 부재 / M3 FakeDbAdapter 잘못된 수정 지시
- **경미 3건**: T1 테스트 seed 부재 / T2 기본값 모호 / T3 캐시 무효화 검증 (※T3는 코드 확인 결과 prefix 매칭으로 이미 안전)

### 최종 결정: **조건부 승인 → 수정 완료 후 승인**

수정 완료 항목:
1. ST-M1-1: 차단 기준을 UUID/해시 세그먼트로 한정, 슬러그는 통과 허용, 설계 근거 명시
2. ST-M1-2: 패턴 확정 (영어+한국어), 줄 단위 적용 오탐 방지 원칙 명시
3. ST-M1-3: `DbPort::get_feed_items` 항목 삭제, 검색 결과 Vec 슬라이싱으로 교체, 캐시 전략(전체 캐시 후 슬라이싱) 확정, 기본값(limit 미지정 → 전체 반환) 확정

---

## ST-M1-1: BUG-A — BBC 토픽 URL 필터링

**목적**: `/news/topics/{topic_id}` 형태의 BBC URL이 피드에 기사로 노출되는 버그 수정

**원인**:  
`is_listing_url()`에 `topics`가 LISTING_SEGMENTS에 있지만, Rule 1은 "마지막 세그먼트"만 검사.  
BBC URL `/news/topics/c9qd23k0`는 마지막 세그먼트가 토픽 ID(랜덤 해시)라 차단 안 됨.

**구현 파일**: `server/src/api/feed.rs`

**작업 내용**:
- Rule 3 추가: `topic`/`topics` 키워드가 경로 중간에 나타나고, 그 바로 뒤 세그먼트가 **UUID 또는 영숫자 해시(8자 이상 알파벳+숫자 혼합)**이면 차단
- 판별 기준: 마지막 세그먼트가 의미 있는 슬러그(소문자 알파벳+하이픈)인 경우는 기사로 허용
- `category/section`은 Rule 3 대상에서 제외 — 기존 오탐 방지 정책(`/category/tech/article-slug` → 통과) 유지
- `tag/tags`는 기존 `/tag/ai-news` 통과 트레이드오프와 동일하게 Rule 3 제외

**설계 근거 (기록)**:
- BBC `/news/topics/c9qd23k0` — `c9qd23k0`는 해시 ID, 토픽 인덱스 페이지
- `/example.com/tags/rust/some-article` — `some-article`은 슬러그, 실제 기사 가능 → 통과 허용
- `/example.com/topic/tech/great-article` — `great-article`은 슬러그 → 통과 허용
- 즉 차단 기준은 "경로에 topic/topics가 있고 뒤 세그먼트가 해시/UUID"로 좁힘

**테스트 케이스 추가**:
- `https://www.bbc.com/news/topics/c9qd23k0` → 차단 (해시 ID)
- `https://example.com/news/topics/abc12345` → 차단 (8자 영숫자 혼합 해시)
- `https://example.com/topic/tech/great-article` → 통과 (슬러그, 실제 기사)
- `https://example.com/tags/rust/some-article` → 통과 (슬러그, 실제 기사)
- `https://example.com/category/tech/article-slug` → 통과 (기존 유지)

**산출물**: `cargo test` 통과 (is_listing_url 관련 테스트 포함)

---

## ST-M1-2: BUG-B — snippet 메타 텍스트 필터 보강

**목적**: snippet에 저자명·댓글수·목차 등 메타 텍스트가 포함되는 오염 수정

**원인**:  
`clean_snippet()`이 HTML·마크다운 헤더·플레이스홀더는 제거하지만,  
저자 패턴("By John Smith"), 댓글수("5 comments"), 목차 등은 제거하지 못함.

**구현 파일**: `server/src/infra/exa.rs`

**현재 상태**: 이미 마크다운 헤더·[...] 제거가 이전 세션에서 구현됨 (코드 확인).

**필터링 패턴 (확정)**:
- 영어 저자: 줄 시작이 `By ` (대소문자 무관) / `Written by ` / `Author:` → 해당 줄 전체 제거
- 영어 댓글수: `^\d+ comments?` / `^\d+ replies?` 패턴 → 해당 줄 전체 제거
- 목차: `Table of Contents` (대소문자 무관) → 해당 줄 전체 제거
- 한국어 저자: 줄에 `기자`, `작성자:`, `글쓴이:` 포함 시 → 해당 줄 전체 제거
- 한국어 댓글수: `댓글 \d+개` / `댓글\d+` 패턴 → 해당 줄 전체 제거
- 한국어 목차: 줄 시작이 `목차` → 해당 줄 전체 제거

**오탐 방지 원칙**:
- 패턴은 **줄 단위** 적용 (문장 중간 매칭 금지 — 오탐 최소화)
- 패턴이 줄 전체를 차지하는 경우만 제거 (본문 중간 "by the way" 등 보호)
- 정규식 적용 순서: 다른 필터보다 먼저 줄 단위 메타 텍스트 제거 → 이후 기존 HTML/헤더 제거 흐름 유지

**테스트 케이스 추가**:
- "By John Smith\nArticle content here." → "Article content here."
- "5 comments\nThis is the news." → "This is the news."
- "작성자: 이민우 기자\n오늘의 뉴스입니다." → "오늘의 뉴스입니다."
- "댓글 5개\n기사 본문입니다." → "기사 본문입니다."
- "The result was driven by the new policy." → 변경 없음 (줄 중간 by → 오탐 방지)
- 기존 테스트 모두 통과 유지

**산출물**: `cargo test` 통과 (clean_snippet 관련 테스트 포함)

---

## ST-M1-3: 피드 페이지네이션 — limit/offset API

**목적**: 피드 첫 로드 성능 저하 해결을 위한 limit/offset 페이지네이션 도입

**아키텍처 전제 (필독)**:
피드는 DB 저장 없음. `GET /me/feed`는 검색 API(Exa/Tavily)를 직접 호출해 결과를 조합한다.
`DbPort::get_feed_items`는 존재하지 않으며, SQL LIMIT/OFFSET은 이 서브태스크에 해당 없음.
페이지네이션은 **검색 결과 Vec을 핸들러 레이어에서 슬라이싱**하는 방식으로 구현한다.

**구현 파일**:
- `server/src/api/feed.rs` (FeedQuery 확장, 핸들러 슬라이싱 로직 추가, 캐시 키 수정)

**작업 내용**:
1. `FeedQuery`에 `limit: Option<u32>`, `offset: Option<u32>` 추가
2. 핸들러에서 전체 검색 결과 조합 후 `.skip(offset).take(limit)` 슬라이싱
3. 캐시 전략: **전체 결과를 캐시 후 슬라이싱** 방식 채택
   - 캐시 키는 기존 `{user_id}:{tag_ids}` 유지 (limit/offset 미포함)
   - 캐시에서 전체 결과를 가져온 뒤 핸들러에서 페이지 슬라이싱
   - 이유: limit/offset별 캐시 분절화 방지, 캐시 용량 폭발 예방
4. 하위 호환: `limit` 미지정 시 전체 반환 (기존 동작 유지). `offset` 기본값 0.
5. `FakeDbAdapter`·`PostgresAdapter` 변경 없음 (피드는 DB 기반 아님)

**테스트 케이스 추가**:
- FakeSearchAdapter가 5개 결과를 반환하는 상태에서:
  - `GET /me/feed?limit=2&offset=0` → 2개 반환
  - `GET /me/feed?limit=2&offset=2` → 그 다음 2개 반환 (3번째, 4번째)
  - `GET /me/feed?limit=2&offset=4` → 1개 반환 (마지막 1개, 범위 초과 안전 처리)
  - `GET /me/feed` (파라미터 없음) → 5개 전체 반환 (기존 동작 유지)

**산출물**: `cargo test` 통과 (피드 핸들러 통합 테스트 포함)
