# ST-4: A2-a 한국어 키워드 병행 추가

> ⚠️ **통합됨**: 이 서브태스크는 ST-2로 합산. `tag_search_keyword()` 동일 함수 수정 충돌 방지.
> 구현은 ST-2 문서를 참조.

> 유형: feature | 의존: 없음 | 병렬: ST-2, ST-3, ST-5와 독립 | 예상: 1h (ST-2에 포함)

## 목적

`tag_search_keyword()` 매핑에 한국어 키워드를 병행 추가해 Tavily가 한국어 기사도 반환하도록 유도한다.

## 파일

`server/src/api/feed.rs` → `tag_search_keyword()` + 쿼리 빌드 로직

## 현재 상태

```rust
// 현재: 영어 키워드만
"모바일 개발" => "mobile development iOS Android Swift Kotlin",

// 쿼리 빌드 (feed.rs:305)
let search_query = format!("{search_keyword} latest news{suffix}");
```

## 작업

### 방안 A (단순): 키워드 매핑에 한국어 단어 추가

```rust
"모바일 개발" => "mobile development iOS Android Swift Kotlin 모바일 앱 개발",
"AI/ML"       => "artificial intelligence machine learning LLM AI 인공지능",
```

한국어 단어 추가로 Tavily가 KR 문서를 끌어올 수 있는지 검증.

### 방안 B (분리): 별도 KR 키워드 함수

```rust
pub(super) fn tag_search_keyword_kr(tag_name: &str) -> &str { ... }
```

ST-5(KR 파라미터 멀티패스)와 연계 시 사용.

## 판단 기준

방안 A 먼저 시도. Tavily 단일 호출에서 KR 기사가 나오면 방안 A 채택.
나오지 않으면 방안 B + ST-5 멀티패스 필요.

## 완료 기준

- [ ] 한국어 키워드 추가 (전체 태그)
- [ ] `tag_search_keyword()` 단위 테스트 갱신
- [ ] 방안 선택 근거 주석
