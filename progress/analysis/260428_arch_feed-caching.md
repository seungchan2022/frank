# 피드 아키텍처 심층분석: DB 저장 vs 캐싱 vs 현행 유지

> 분석일: 2026-04-28  
> 스코프: 서버 피드 파이프라인 + iOS FeedFeature  
> 트리거: MVP13 M2 로드맵 "피드 DB 저장 + 중복 제거" 기획 검토

---

## 0. 분석 전 기준선 정정

### 실제 현재 아키텍처 (탐색 확인)

사용자 프레이밍: "현행 = DB 저장 없음"  
**실제**: `InMemoryFeedCache` (TTL = 300초, 5분)가 이미 프로덕션에 구현되어 있음.

```
GET /api/feed?tag_id=...&limit=20&offset=0
  → 캐시 키: "{user_id}:{sorted_tag_ids}"
  → HIT  → 캐시 반환 (즉시, ~1ms)
  → MISS → Exa API 호출 (tag별 병렬 join_all, ~800ms–2s)
           → og:image 병렬 크롤링
           → URL 정규화 + HashSet 중복 제거 (within-session)
           → 캐시 저장 → 반환
```

`NoopFeedCache`는 **테스트 전용**. 프로덕션 주입 코드에서 `InMemoryFeedCache` 사용 확인.  
결론: "현행 유지"는 이미 "인메모리 캐싱 있음" 상태.

---

## 1. "피드 품질 개선"의 4가지 하위 문제 분리

로드맵의 "피드 품질 개선" 레이블 아래에 실제로 4가지 서로 다른 문제가 혼재한다.  
각각 독립적으로 평가해야 한다.

| # | 문제 | 현재 상태 | 해결 방법 |
|---|------|----------|----------|
| A | Exa API 비용/quota 초과 | unknown | Exa 계정 quota 확인이 먼저 |
| B | 응답 속도 (cold MISS latency) | ~1–2s | 이미 5분 캐시로 완화 |
| C | Within-session 중복 기사 | **이미 해결** | `normalize_url` + HashSet |
| D | Cross-session 중복 기사 | 재방문 시 같은 기사 반복 | 미해결 |

### 문제 A: API 비용/quota

Exa 무료 플랜 기준:
- numResults=5 per tag, 최대 구독 태그 수에 따라 호출 횟수 결정
- MISS 시 태그 개수만큼 병렬 호출
- 5분 TTL → 같은 유저 5분 내 재요청은 API 호출 없음

**단일 사용자 앱에서의 실제 영향**: 하루 사용 패턴이 산발적이면 일별 API 호출은 수십 회 수준.  
Exa 무료 플랜(1,000 requests/month)이면 넉넉하다. **현재 quota 초과 여부를 먼저 확인**하는 것이 DB 저장 여부 결정의 전제 조건.

### 문제 B: 응답 속도

현재 5분 TTL 캐시가 있으므로, 연속 사용 중에는 즉시 반환된다.  
Cold MISS (5분 경과 후 첫 요청)만 Exa 호출 필요. 클라이언트 입장에서는 pull-to-refresh가 의도적 MISS이므로 사용자 기대와 일치한다.

### 문제 C: Within-session 중복 — 이미 해결

`normalize_url` 함수 + `HashSet<String>`으로 태그별 병렬 검색 결과 중복 URL을 이미 제거한다.  
MVP13 M2 계획에 "중복 제거"가 포함되어 있으나 **이미 구현됨**. 재작업 필요 없음.

### 문제 D: Cross-session 중복

5분 캐시 만료 후 다시 조회하면 같은 기사가 반복 노출될 수 있다.  
Exa는 `startPublishedDate`(7일 전)를 기준으로 반환하므로, 7일 이내 기사는 계속 재등장 가능.  
DB 저장 없이는 "이미 본 기사"를 서버가 추적할 수 없다.

**단일 사용자 앱에서의 실제 불편**: 사용자가 실제로 이를 문제로 인식하는지가 핵심.  
뉴스 피드 특성상 "같은 기사가 다시 보임"이 치명적 UX 문제인지 vs 허용 가능한 수준인지 판단 필요.

---

## 2. DEBT-02 상태 충돌 플래그

**충돌 발견**:

| 문서 | 상태 |
|------|------|
| `progress/debts.md` DEBT-02 | ✅ **RESOLVED** — C안 적용 완료 |
| `history/mvp13/_roadmap.md` M2 | `iOS: 피드 fetch 방식 웹과 통일` ✅ **done** |

실제 코드 확인:
- iOS `FeedFeature.swift`: "all" 탭 첫 페이지만 즉시 로드, 나머지 lazy — C안 적용 완료
- 서버: 태그별 독립 API + 페이지네이션 지원

**결론**: DEBT-02는 실제로 해소됨. M2 "iOS 피드 fetch 방식 웹과 통일" 항목은 **삭제 또는 정확한 잔여 이슈로 재기술**해야 한다.

웹과 iOS의 방식 차이 (현재, 기능 문제 없음):
- 웹: "all" 단일 캐시 + 클라이언트 태그 필터
- iOS: 탭별 독립 캐시 + 서버 요청 (C안)

이 차이는 설계 선택이지 버그가 아니다. 웹 무한 스크롤 시 태그 탭 기사 희박 가능성은 별도 이슈로 추적하거나 M2에 명확히 기술해야 한다.

---

## 3. DB 저장 vs 캐싱 vs 현행 유지 트레이드오프

### 옵션 A: 현행 유지 (InMemoryFeedCache 5분 TTL)

**장점**:
- 구현 비용 0. 서버 재시작 시 자동 초기화 (stale 기사 없음)
- 스키마 마이그레이션, retention 정책, 인덱스 관리 불필요
- 단일 사용자 앱에서 API quota가 문제 없으면 완전히 충분

**단점**:
- Cross-session 중복 해결 불가
- 서버 재시작 시 캐시 소멸 → cold MISS latency 재발생
- Exa quota 초과 시 대안 없음

**적합 조건**: Exa quota 여유 있음 + 중복 노출이 실제 불편으로 보고되지 않음

---

### 옵션 B: DB 저장 (articles 테이블 추가)

**장점**:
- Cross-session 중복 제거 가능 (URL unique constraint)
- 서버 재시작 후에도 캐시 유지
- 향후 favorites 연동 (article_id FK) 정규화 가능
- "새로고침 시 DB 우선 반환 → 백그라운드 Exa 갱신" 패턴 가능

**단점**:
- 스키마 추가 (articles 테이블, URL unique index)
- Retention 정책 필요 (무한 성장 방지)
- 서버 코드 변경 범위: `SearchPort` + `FeedCachePort` + `ArticlePort` 조합 필요
- og:image, snippet 등 외부 데이터를 DB에 캐싱하면 stale 위험

**Retention 합리적 기준**:
1. **7일 TTL** — Exa `startPublishedDate` 기준(7일)과 일치. CRON으로 주기 삭제
2. **Favorites/오답 참조 제외** — `favorites`, `quiz_wrong_answers`에서 참조하는 article은 유지
3. **최대 N건** — 태그당 100건 상한 후 오래된 것 삭제

적용하기 쉬운 최소 정책: `created_at < NOW() - INTERVAL '7 days' AND url NOT IN (SELECT article_url FROM favorites WHERE ...)` CRON

**구현 범위 (최소)**:
```sql
CREATE TABLE articles (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  url        TEXT UNIQUE NOT NULL,
  title      TEXT,
  snippet    TEXT,
  image_url  TEXT,
  tag_id     UUID REFERENCES tags(id),
  fetched_at TIMESTAMPTZ DEFAULT NOW()
);
```

서버 변경:
- `ArticleDbPort` trait 추가 (insert_or_ignore_by_url, list_by_tag)
- 피드 핸들러: DB hit → 반환 + 백그라운드 Exa fetch (stale-while-revalidate)
- `FeedCachePort`를 DB 기반으로 교체 또는 L1(메모리)/L2(DB) 2계층

---

### 옵션 C: TTL 연장 또는 persist-on-disk 캐시

Redis 추가 없이 현재 InMemoryFeedCache의 TTL만 늘리는 방안.

**장점**:
- 코드 변경 최소 (TTL 상수 변경)
- 서버 재시작 전까지는 cross-session 중복 완화 효과

**단점**:
- 서버 재시작 시 소멸 (클라우드 배포 M3 이후 재시작 빈도 증가)
- 기사 staleness 문제 (TTL 1시간이면 1시간 전 기사 반환)
- 근본 해결이 아님

**적합 조건**: M3 클라우드 배포 전 임시 개선책

---

## 4. 개인용 단일 사용자 앱에서의 실제 판단 기준

단일 사용자 앱이라는 맥락에서 일반적인 "서비스" 관점 판단과 달라지는 지점:

| 일반 서비스 기준 | 단일 사용자 앱 현실 |
|----------------|------------------|
| DB 저장 = 안정성 필수 | 메모리 캐시로도 충분할 수 있음 |
| 중복 제거 = UX 필수 | 본인이 불편 안 느끼면 불필요 |
| API quota = 비용 관리 | 무료 플랜이면 실질 비용 없음 |
| 확장성 고려 | 본인 사용 + 소수 지인 정도라면 무관 |

**핵심 판단 질문**:
1. "피드에서 같은 기사가 반복 나오는 게 실제로 불편했는가?" → No라면 옵션 A
2. "Exa quota 경고나 초과가 발생했는가?" → No라면 옵션 A
3. "M3 클라우드 배포 후 서버 재시작이 잦을 것인가?" → Yes라면 옵션 B 고려

---

## 5. 결론 및 권고

### 즉시 확인 필요 (전제 조건)

- [ ] Exa 계정 대시보드에서 월간 API 사용량 확인
- [ ] "피드에서 같은 기사 반복"이 실제 불편으로 경험되었는지 자가 평가

### 권고 방향

**조건 1: Exa quota 여유 있음 + 중복 불편 없음**  
→ **옵션 A (현행 유지)**. MVP13 M2에서 "피드 DB 저장" 항목 삭제.  
리소스를 DEBT-01 M1/M2 완료와 M3 클라우드 배포에 집중.

**조건 2: Exa quota 빠듯하거나 중복이 불편함**  
→ **옵션 B (최소 DB 저장)**. 단, 범위를 최소화:  
  - articles 테이블 추가 (URL unique)  
  - 피드 핸들러 stale-while-revalidate  
  - 7일 TTL retention CRON  
  - og:image는 DB 저장 안 하고 프록시 캐시만 (staleness 회피)

**M3 이전 임시 개선이 필요하다면**  
→ **옵션 C**: FEED_CACHE_TTL을 1800(30분)으로 올리는 1줄 변경. 서버 재시작 전까지 유효.

### M2 로드맵 수정 필요 사항

1. "iOS 피드 fetch 방식 웹과 통일" → **삭제** (DEBT-02 이미 RESOLVED, C안 적용 완료)
2. "Exa 기사 DB 저장 + URL unique constraint 중복 제거" → **조건부 구현** (Exa quota 확인 후 결정)
3. "새로고침 시 DB 우선 반환 → 백그라운드 Exa 갱신" → **옵션 B 선택 시에만 포함**
4. Within-session 중복 제거 → **이미 구현됨, 재작업 불필요**
