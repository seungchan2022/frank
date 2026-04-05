# History Index

완료된 마일스톤 아카이브. 각 항목은 `[유형] 제목 — 핵심 키워드` 형식.

---

## MVP1 — AI 뉴스 수집+요약 웹앱 (260403~260405)

> 기간 2일. 서버(Rust/Axum) + 웹(SvelteKit) 풀스택. 6개 외부 API 통합. 테스트 135개.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 기획 | [MVP1 기획](mvp1/260404_MVP1_기획.md) | 요구사항, 마일스톤 M0~M6, 아키텍처, 데이터 모델 |
| 기획 | [M6 배포파이프라인](mvp1/260404_M6_배포파이프라인.md) | Docker Compose, Cloudflare Tunnel, iMessage 알림 |
| 분석 | [DB 접근 성능](mvp1/260404_perf_db_access.md) | Supabase REST vs sqlx 직접연결 비교, 전환 결정 |
| 분석 | [Step7 리팩토링 리뷰](mvp1/260404_step7_refactoring_review.md) | 코드리뷰 이슈 23건 (Critical 3, Major 12, Minor 8) |
| 분석 | [피드 버그 검증](mvp1/260404_verification_feed_bug.md) | IntersectionObserver 무한호출, loadInitial 누락 등 5건 |
| 버그 | [피드 로딩 버그](mvp1/260404_feed_loading_bug.md) | 빈 피드, 태그 필터 미표시, 홈페이지 URL 혼입 |
| 회고 | [MVP1 완료 회고](mvp1/260405_mvp1_completion_retro.md) | Keep/Problem/Surprise, 기술부채 23건, MVP2 방향 |
| 회고 | [일일 회고 Day1](mvp1/260404_daily_retro.md) | 기획일 회고 |
| 회고 | [일일 회고 Day2](mvp1/260404_daily_retro_2.md) | 구현일 회고 |

---

## MVP1.5 — 기술 부채 해소 (260405)

> 기간 1일. 새 기능 0개, 품질 개선만. 테스트 135→216. 웹 커버리지 99.28%. 의존 위반 0건.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 기획 | [MVP1.5 기획](mvp15/260405_MVP1.5_기획.md) | 부채 23건 중 9건 선별, A/B/C 마일스톤 정의, 완료 기준 |
| 서브태스크 | [A 안정성확보](mvp15/260405_A_안정성확보_서브태스크.md) | A1~A4 상세 명세 (타임아웃, auth Client, 에러 마스킹, 구독 해제) |
| 다이어그램 | [A 서브태스크 DAG](mvp15/260405_A_subtask_dag.svg) | 의존관계 시각화 (D2) |
| 회고 | [MVP1.5 완료 회고](mvp15/260405_mvp15_completion_retro.md) | 숫자 비교, http_utils 분석, Keep/Problem/Surprise, 잔여 부채 12건 |

---

## MVP2 — iOS 네이티브 앱 (260405~)

> iOS 26 + SwiftUI + Tuist. 에코 서버 패턴 + @Observable Feature Reducer.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 회고 | [일일 회고 260405](260405_daily_retro.html) | M1 완료, hook 강제 체감, Tuist MCP 한계, 포트/어댑터 iOS 적용 |

---

## 빠른 검색 가이드

| 찾고 싶은 것 | 참조 파일 |
|-------------|----------|
| 프로젝트 초기 기획/요구사항 | `mvp1/260404_MVP1_기획.md` |
| 아키텍처 결정 (포트/어댑터, sqlx 전환) | `mvp1/260404_perf_db_access.md` |
| 코드리뷰 이슈 전체 목록 | `mvp1/260404_step7_refactoring_review.md` |
| 기술 부채 현황 + 해결 상태 | `mvp15/260405_MVP1.5_기획.md` |
| 실전 버그 패턴 | `mvp1/260404_feed_loading_bug.md` |
| 성능 분석 (DB, 병렬처리) | `mvp1/260404_perf_db_access.md`, `mvp15/260405_mvp15_completion_retro.md` |
| MVP2 방향/잔여 부채 | `mvp1/260405_mvp1_completion_retro.md`, `mvp15/260405_mvp15_completion_retro.md` |
| MVP2 iOS 로드맵 | `progress/260405_MVP2_iOS_로드맵.md` |
| iOS 아키텍처/회고 | `260405_daily_retro.html` |
