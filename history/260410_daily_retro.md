# 회고 — 2026-04-10 ~ 04-11

> 4개 마일스톤을 이틀에 끝낸 날

**작업 범위**: MVP5 M3 완료 + MVP6 M1 썸네일 + M2 병렬 검색 + M3 태그탭 필터링

---

## TL;DR

이틀 동안 4개 마일스톤을 완료했다. MVP5 M3 즐겨찾기로 MVP5를 마무리하고, 이어서 MVP6 M1~M3까지 연속으로 끝냈다. 한 마일스톤의 모델이 다음 마일스톤 기반이 되는 순서(M1→M2→M3)를 미리 확정해서 재작업 없이 흘렀다. 크게 세 가지 기술 결정이 이번 진행을 만들었다 — 이미지 소스 전략 변경, Rust lifetime 실전 해결, 탭 캐시 전략.

---

## 결정 01 — 이미지 소스: Tavily images[] → og:image 직접 크롤링

**결정**: 썸네일 이미지를 Tavily가 반환하는 `images[]` 배열이 아니라, 기사 URL에서 `og:image`를 직접 크롤링해서 가져오도록 변경. 병렬 크롤링 + 5초 타임아웃.

**왜 중요한가**: Tavily의 images[]는 기사 페이지에 있는 모든 이미지를 반환하는데, 광고나 아이콘이 섞여서 대표 이미지로 쓸 수 없었다. og:image는 기사 발행자가 직접 지정한 대표 이미지라 품질이 일정하다. 5초 타임아웃으로 느린 사이트 대응, URL 정규화(http/https·www·trailing slash 통일) 후 HashSet dedup으로 중복 기사 제거까지 묶어서 해결했다.

---

## 결정 02 — Rust join_all lifetime: SearchJob owned 구조체

**결정**: 병렬 검색(`tokio::join_all`)을 구현할 때 `&str` 대신 `String` 필드를 가지는 `SearchJob` owned 구조체로 job을 묶었다.

**왜**: `join_all`은 여러 Future를 동시에 실행하는데, 각 Future가 외부 참조(`&str`)를 들고 있으면 컴파일러가 lifetime을 보장하지 못한다. owned 타입(String)으로 교체하면 Future가 자기 데이터를 직접 소유해 lifetime 문제가 사라진다. Rust에서 비동기 병렬 처리에 참조를 넘기면 이 문제가 반드시 나온다 — 패턴으로 기억해둘 만하다.

---

## 결정 03 — iOS scroll to top 구현 → UX 판단으로 롤백

**결정**: M2 stale-while-revalidate 구현 시 새 데이터 도착 후 scroll to top을 추가했다가, iOS에서는 제거했다.

**왜**: pull-to-refresh는 완료되면 사용자의 손이 화면 상단에서 떼어지고 자동으로 맨 위로 돌아오는 특성이 있다. 여기에 추가로 scroll to top을 하면 어색하다. 구현하고 직접 써보니 바로 느껴졌다. 웹은 progress bar + 백그라운드 갱신이라 scroll to top이 유효하지만 iOS pull-to-refresh는 다르다.

---

## 결정 04 — tagCache 캐시 전략: 탭 전환은 캐시 히트, pull-to-refresh만 무효화

**결정**: 탭 전환 시 캐시가 있으면 재요청 없이 즉시 표시. pull-to-refresh만 현재 탭 캐시를 삭제하고 재요청. Settings에서 태그를 변경하면 `feedStore.reset()`으로 전체 캐시 무효화.

**왜 중요한가**: 탭 전환마다 서버에 요청하면 네트워크 딜레이로 탭이 끊기는 느낌이 난다. 반면 캐시를 너무 오래 들고 있으면 실제 구독 태그가 바뀌어도 모른다. "수동 새로고침(pull-to-refresh)만 갱신"이 중간 지점이다.

Settings feedStore.reset 버그: 태그를 변경하고 저장하면 새 태그 기준으로 피드를 봐야 하는데, 이전 캐시가 남아서 이전 태그 결과를 보여주는 버그가 있었다. `handleSave` 후 `feedStore.reset()` 추가로 해결.

---

## 결정 05 — iOS xcconfig `://` 주석 처리 문제

**결정**: Supabase URL 값을 xcconfig에서 읽어오려 했는데, xcconfig 파일에서 `://`가 줄 주석(`//`)으로 해석되어 URL이 잘린다. Project.swift에 직접 값을 하드코딩하고, Config.xcconfig는 .gitignore에 추가.

**왜**: xcconfig 문법은 `// 주석` 형태를 쓰는데, URL의 `://` 부분이 주석 시작으로 파싱된다. 이걸 우회하는 공식 방법이 없어서 Project.swift에서 직접 settings 블록으로 값을 주입하는 방식으로 전환했다. 민감 정보가 xcconfig에 있으므로 .gitignore 추가는 필수였다.

---

## 숫자

| 항목 | 값 |
|------|-----|
| 커밋 수 | 27개 (04-10~04-11) |
| 완료 마일스톤 | MVP5 M3 + MVP6 M1 + M2 + M3 |
| 테스트: 서버 | 173 → 183개 |
| 테스트: 웹 | 146 → 155개 |
| 테스트: iOS | 155 → 158개 |
| MVP6 실행 순서 | M1→M2→M3 사전 확정 (재작업 0건) |

---

## 다음에 할 것

- MVP6 M4 — 마크다운 렌더링 (LLM 프롬프트 + 웹 svelte-markdown + iOS AttributedString)
