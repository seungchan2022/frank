# 심층분석: Feed 버그 메인태스크 검증

- 날짜: 2026-04-04
- 대상: `progress/260404_feed_loading_bug.md`

## 검증 결과 요약

| 항목 | 판정 | 비고 |
|------|------|------|
| Bug1 IntersectionObserver | **정확** | Svelte 5 teardown에서 에러 catch 안 함 → flush 중단 확인 |
| Bug2 loadInitial 3회 호출 | **정확, 보완** | 트리거 경로 3단계 확인 (getSession + INITIAL_SESSION + TOKEN_REFRESHED) |
| Bug3 usedTags 소멸 | **정확** | 문서 그대로 |
| Bug4 홈페이지 URL | **정확, 보완** | Tavily basic 모드의 알려진 동작 + URL 필터링 부재 |
| Bug5 빈약한 내용 | **정확, 보완** | firecrawl.rs가 SearchPort만 구현 → CrawlPort 신규 필요 |
| 모델 변경 | **타당** | qwen3.5-plus structured output 정상, ChatResponse에 Usage 파싱 추가 필요 |
| DB 스키마 | **보완 필요** | title_ko, content 컬럼도 포함, 영향 파일 8~9개 |

## 신규 발견 사항

1. **firecrawl.rs는 CrawlPort가 아닌 SearchPort를 구현** → 본문 크롤링용 새 trait 필요
2. **ChatResponse에 Usage 구조체 미존재** → 선행 작업으로 추가해야 함
3. **Bug4 보완**: search_depth 변경만으론 불충분, 크롤링 결과 품질 기반 필터링 병행 필요
4. **Bug2 정밀화**: $effect가 boolean 값이 아닌 session 참조를 추적하는 것이 핵심 원인
