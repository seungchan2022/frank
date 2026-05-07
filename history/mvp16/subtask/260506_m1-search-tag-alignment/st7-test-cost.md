# ST-7: 단위·통합 테스트 + 비용 영향 검토

> 유형: chore | 의존: ST-2, ST-3, ST-6 | 예상: 1h
> (ST-4는 ST-2에, ST-5는 ST-3에 통합됨)

## 목적

ST-2~ST-6 구현 전체에 대한 테스트 통과를 확인하고, 호출 수 증가가 무료 한도 내인지 검토한다.

## 테스트

```bash
cd /Users/seungchan/Workspace/frank/server
cargo test
# 주요 확인 대상:
# - cargo test feed::tag_search_keyword
# - cargo test tavily (새 파라미터 포함 확인)
```

## 비용 영향 검토

파일: `progress/mvp16/cost_log.md` (없으면 신규 생성)

| 항목 | 검토 내용 |
|------|-----------|
| Tavily 호출 수 | ST-3 KR 멀티패스(옵션 2) 선택 시 2배 — 무료 한도(1,000 req/month) 대비. 공식: `2 × 태그 수 × 일일 요청 수 × 30` |
| Firecrawl og:image 크롤 | `태그 수 × avg 기사 수` 만큼 병렬 발생. include_domains 적용 후 한국어 도메인에서 크롤 실패 시 지연 여부 확인 |
| Exa 호출 수 | 폴백 발생 시만. 변경 없으면 기록 생략 |

## 완료 기준

- [x] `cargo test` 전체 통과 (410 passed)
- [x] `cargo clippy -- -D warnings` 통과
- [x] `cargo fmt --check` 통과
- [x] `progress/mvp16/cost_log.md` 갱신 — Tavily 호출 수 변화 없음, $0 유지 확인
