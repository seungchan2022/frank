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

## MVP2 — iOS 네이티브 앱 (260405~260406)

> 기간 2일. iOS 26 + SwiftUI + Tuist. 에코 서버 패턴 + @Observable Feature Reducer. 117 테스트. 2,433줄.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 기획 | [MVP2 iOS 로드맵](mvp2/260405_MVP2_iOS_로드맵.md) | 6개 마일스톤 정의, Feature-Driven Unidirectional Flow, 디렉토리 구조 |
| 기능 | [M2 인증플로우](mvp2/260406_M2_인증플로우.md) | Apple/Email 로그인, 세션 복원, signUp 계약 위험 발견 |
| 기능 | [M3 온보딩](mvp2/260406_M3_온보딩.md) | 태그 선택 FlowLayout, slug→category 스키마 수정 |
| 기능 | [M4 피드](mvp2/260406_M4_피드.md) | 탭 필터 + 카드 + 수집/요약 + 페이지네이션 + per-tag 캐시 |
| 기능 | [M5 기사상세](mvp2/260406_M5_기사상세.md) | DetailFeature + NavigationStack 연결 |
| 기능 | [M6 설정](mvp2/260406_M6_설정.md) | 태그 관리 + 로그아웃 + 피드 동기화, sheet 타이밍 버그 3건 |
| 다이어그램 | [M4 피드 DAG](mvp2/260406_M4_피드_dag.svg) | 수집/요약 파이프라인 의존관계 시각화 |
| 회고 | [MVP2 완료 회고](mvp2/260406_mvp2_completion_retro.md) | Keep/Problem/Surprise, 아키텍처 스코어 8.9/10, MVP2.5 부채 10건 |
| 회고 | [일일 회고 260405](260405_daily_retro.html) | M1 완료, hook 강제 체감, Tuist MCP 한계, 포트/어댑터 iOS 적용 |

---

---

## MVP3 — 웹+iOS API 통합 (260406~260408)

> 기간 3일. 웹+iOS가 Rust API 직통 호출. httpOnly 쿠키 세션 전환. Apple 로그인 3개 플랫폼. 테스트 89(웹)+155(iOS).

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 로드맵 | [MVP3 통합 로드맵](mvp3/260406_MVP3_통합_로드맵.md) | M1~M4 마일스톤, 병렬 개발 전략, Mock-First 흐름 |
| 기획 | [M1 API Contract](mvp3/260406_MVP3_M1_API_Contract.md) | Rust 서버 엔드포인트 보완 (fetchArticle, updateProfile) |
| 기획 | [M1.5 병렬 준비](mvp3/260407_MVP3_M1.5_병렬준비.md) | API SPEC 문서화, fixture JSON, 웹/iOS Mock 어댑터 |
| 기획 | [M2 웹 전환](mvp3/260406_MVP3_M2_웹전환.md) | @supabase/ssr, httpOnly 쿠키, Rust API 직통 |
| 기획 | [M3 iOS 전환](mvp3/260406_MVP3_M3_iOS전환.md) | APIArticleAdapter, APITagAdapter, MVP2 부채 흡수 |
| 기획 | [M4 Apple 로그인](mvp3/260406_MVP3_M4_Apple로그인.md) | OAuth PKCE(웹) + ASAuthorizationController(iOS) |
| 참조 | [API SPEC](mvp3/260407_API_SPEC.md) | 전체 엔드포인트 명세 (요청/응답 타입 포함) |
| 참조 | [배포 스크립트](mvp3/260407_deploy_script.md) | scripts/deploy.sh 설계 문서 |
| 분석 | [hotfix 요약 태그필터](mvp3/260408_hotfix_요약_태그필터.md) | OpenRouter reasoning mandatory 400 fix |
| 회고 | [M2 회고](mvp3/260408_M2_회고.md) | 웹 전환 Keep/Problem, 태그 stale 해결 |
| 회고 | [M3 회고](mvp3/260408_M3_회고.md) | iOS 전환 Keep/Problem, MVP2 부채 흡수 현황 |
| 회고 | [M4 회고](mvp3/260408_M4_회고.md) | Apple 로그인 트러블슈팅, 크로스 플랫폼 계정 연동 |
| 회고 | [MVP3 완료 회고](mvp3/260408_mvp3_completion_retro.md) | 병렬 worktree 첫 실전, Mock-First 증명, 크로스 플랫폼 계정 연동, 부채 목록 |
| 회고 | [일일 회고 260408](260408_daily_retro.html) | M3 완료 + 병렬 워크트리 회고 |
| fixtures | [fixtures/](mvp3/fixtures/) | articles.json, profile.json, tags.json (Mock 기준 데이터) |

---

## MVP4 — 부채 해소 + 품질 개선 (260408~260409)

> 기간 2일. M1~M5 전체 완료. UITest 4개 + coverage.sh + Mock 주입 인프라. 태그 stale 버그 해소.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 비전 | [MVP4 비전](mvp4/_vision.md) | 부채 해소 동기, MVP4 진입 기준, 완료 기준 |
| 발견 | [MVP4 Discovery](mvp4/_discovery.md) | 내부 부채 분석, 우선순위 결정 근거 |
| 로드맵 | [MVP4 로드맵](mvp4/_roadmap.md) | M1~M5 마일스톤, DoD, 의존성 그래프 |
| 마일스톤 | [M1 서버인프라](mvp4/M1_서버인프라.md) | LLM 복귀 + Apple Secret 만료 관리 |
| 마일스톤 | [M2 iOS UX개선](mvp4/M2_iOS_UX개선.md) | 요약 timeout + 로그인 인라인 에러 |
| 마일스톤 | [M3 웹 UX개선](mvp4/M3_웹_UX개선.md) | 요약 timeout UX 웹 버전 |
| 마일스톤 | [M4 iOS 테스트인프라](mvp4/M4_iOS_테스트인프라.md) | UITest 4개 + coverage.sh + Mock 주입 인프라 |
| 마일스톤 | [M5 데이터품질](mvp4/M5_데이터품질.md) | 태그 stale 해소 + Supabase 조사 |
| 회고 | [일일 회고 260409](260409_daily_retro.html) | 마일스톤 범위 정의 원칙 + 멘토 발표 + 나선형 학습 전환 |

---

## MVP5 — 학습 기능: 수집·요약·즐겨찾기 (260409~260410)

> "모두 수집·저장·요약" → "보고 싶은 것만 요약, 저장하고 싶은 것만 즐겨찾기" 아키텍처 전환.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 로드맵 | [MVP5 로드맵](mvp5/260409_MVP5_로드맵.md) | M1~M3 마일스톤, DB 스키마 재설계, 온디맨드 요약 흐름 |
| 서브태스크 | [M1 피드 아키텍처 전환](mvp5/260409_MVP5_M1_피드아키텍처전환.md) | FeedPort 분리, SearchPort·SummarizePort 독립, DB 스키마 재설계 |
| 서브태스크 | [M2 디테일+온디맨드 요약](mvp5/260409_MVP5_M2_서브태스크.md) | ArticleDetail, SummarizePort 세션 캐시, 웹+iOS 연동 |
| 서브태스크 | [M3 즐겨찾기+스크랩 목록](mvp5/260409_MVP5_M3_서브태스크.md) | FavoritesPort, /me/favorites CRUD, 웹+iOS 즐겨찾기 UI |
| 회고 | [일일 회고 260409 후반](260409b_daily_retro.html) | 아키텍처 대전환 · SummarizePort 세션 캐시 · 나선형 개념 정리 |

---

## MVP6 — 피드 UX 개선: 썸네일·성능·태그탭·마크다운 (260410~260411)

> 기간 2일. M1 썸네일(og:image 크롤링) → M2 병렬 검색(join_all) → M3 태그탭(tagCache) → M4 마크다운(marked/AttributedString). 테스트 서버 185 / 웹 162 / iOS 158.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 로드맵 | [MVP6 로드맵](mvp6/260410_MVP6_로드맵.md) | M1~M4 마일스톤, 실행 순서 근거, UX 확정 사항 |
| 서브태스크 | [M1 썸네일](mvp6/260410_MVP6_M1_썸네일.md) | og:image 크롤링, favorites image_url, 중복 제거 |
| 서브태스크 | [M2 피드성능](mvp6/260410_MVP6_M2_서브태스크.md) | join_all SearchJob 패턴, stale-while-revalidate, progress bar |
| 서브태스크 | [M3 태그탭](mvp6/260410_MVP6_M3_서브태스크.md) | tag_id 쿼리 파라미터, tagCache Map/Dict, 프리패치, 클라이언트 필터 제거 |
| 서브태스크 | [M4 마크다운](mvp6/260410_MVP6_M4_서브태스크.md) | exa highlights, marked+prose, AttributedString, 토큰 자동갱신 BF |
| 회고 | [일일 회고 260410~11](260410_daily_retro.html) | og:image 크롤링 · join_all lifetime · tagCache 전략 · xcconfig 우회 |

---

## MVP7 — 좋아요·피드 개인화·연관 기사·퀴즈 (260411~260412)

> 기간 2일. M1~M4 전체 완료. "읽기" 중심에서 "학습" 중심으로 전환. 테스트 서버 252 / 웹 183 / iOS 189+3.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 로드맵 | [MVP7 로드맵](mvp7/260411_MVP7_로드맵.md) | M1~M4 마일스톤, 좋아요→개인화→퀴즈 학습 루프, DB 스키마 확정 |
| 서브태스크 | [M2 좋아요+키워드추출](mvp7/260411_MVP7_M2_서브태스크.md) | LlmPort.extract_keywords, user_keyword_weights, 하트 버튼 웹+iOS |
| 서브태스크 | [M3 피드개인화+연관기사](mvp7/260412_MVP7_M3_서브태스크.md) | like_count>=3 키워드 boost, GET /me/articles/related, cross-tag 오염 수정 |
| 서브태스크 | [M4 퀴즈](mvp7/260412_MVP7_M4_서브태스크.md) | 4지선다 3문제, favorites.concepts 저장, LLM 503 처리, QuizModal/QuizView |
| 분석 | [요약 속도 개선](mvp7/analysis/260411_perf_summarize.md) | 요약 파이프라인 성능 분석, timeout 조정 근거 |
| 참조 | [Supabase 수동 연결](mvp7/analysis/supabase_manual_linking.md) | RLS 정책 수동 적용 절차 |

---

## MVP8 — UX 개선 + 오답 아카이빙 + 퀴즈 완료 배지 (260412~260413)

> 기간 2일. M1~M3 전체 완료. "읽기·학습" → "학습 루프 완성". 테스트 서버 280 / 웹 203 / iOS 200.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 로드맵 | [MVP8 로드맵](mvp8/260412_MVP8_로드맵.md) | M1~M3 마일스톤, 오답 아카이빙·퀴즈 배지·UX 개선, DB 스키마 확정 |
| 회고 | [일일 회고 260413](260413_retro.html) | iOS Swift 버그 발견 경위, 마일스톤 분리 기준, Known Issues 정책, 확인 버튼 UX, 연관 기사 제거, M2/M3 QA 결과 |

---

## MVP9 — 실사용 장벽 제거 + 퀴즈 학습 루프 완성 (260413)

> 기간 1일. M1(서버) + M2(클라이언트) 병렬 완료. 테스트 서버 269 / 웹 216 / iOS 211.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 인터뷰 | [MVP9 기획 인터뷰](mvp9/260413_MVP9_인터뷰.md) | 실사용 발견 이슈 — 속도·snippet 노이즈·피드 반복·퀴즈 UX |
| 비전 | [MVP9 비전](mvp9/_vision.md) | 실사용 장벽 3가지 제거 + 학습 루프 완성 |
| 발견 | [MVP9 Discovery](mvp9/_discovery.md) | 코드 확인 기반 원인 분석 및 해결 방향 |
| 로드맵 | [MVP9 로드맵](mvp9/_roadmap.md) | M1(서버)·M2(클라이언트) 병렬 전략, M3 후보 목록 |
| 마일스톤 | [M1 서버 개선](mvp9/M1_서버개선.md) | Groq 교체(30s→5s) + Tavily time_range + snippet 패턴 필터 |
| 마일스톤 | [M2 퀴즈 UX 개선](mvp9/M2_퀴즈UX개선.md) | 세션 오답 인라인 + 버튼 재설계(다시 풀기·오답 보기) 웹+iOS |

---

## MVP10 — 버그 수정 + 피드 TTL 캐시 (260414)

> 기간 1일. M1~M3 전체 완료. 실사용 버그 5건 수정 + 서버 TTL 인메모리 캐시(5분) 도입. 테스트 서버 285 / 웹 221 / iOS 전체.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 비전 | [MVP10 비전](mvp10/_vision.md) | 실사용 버그 5건 목록, 성능 개선 목표, 완료 기준 |
| 발견 | [MVP10 Discovery](mvp10/_discovery.md) | 코드 확인 기반 원인 분석, 우선순위 결정 근거 |
| 로드맵 | [MVP10 로드맵](mvp10/_roadmap.md) | M1~M3 마일스톤, 순차+내부병렬 전략, 통합 테스트 완료 |
| 마일스톤 | [M1 iOS 퀴즈 UX 완성](mvp10/M1_iOS퀴즈UX완성.md) | 다시 풀기 버튼, 오답 보기, 배지 수정 |
| 마일스톤 | [M2 버그 수정](mvp10/M2_버그수정.md) | keyword_weights tag_id 분리, 요약 에러 메시지 개선 웹+iOS |
| 마일스톤 | [M3 피드 성능 개선](mvp10/M3_피드성능개선.md) | FeedCachePort + InMemoryFeedCache(TTL 5분), no-cache 헤더 웹+iOS |
| 서브태스크 | [M1 서브태스크](mvp10/260414_MVP10_M1_퀴즈UX완성_배지수정.md) | QuizView 버튼 추가, FavoritesFeature 배지 수정 |
| 서브태스크 | [M2 서브태스크](mvp10/260414_M2_BUG_subtasks.md) | keyword_weights 분리 + 요약 에러 메시지 서브태스크 분해 |
| 회고 | [일일 회고 260414](260414_daily_retro.html) | cross-tag 오염 수정, 에러 코드 분리, NoopFeedCache 패턴, TTL 캐시 설계 |

---

## MVP11 — 버그 수정 + 뉴스 품질 개선 (260423~260424)

> 기간 2일. BUG-001~003 수정 + 검색엔진 뉴스 수집 품질 개선. M1~M4 전체 완료.
> M1(서버 품질) → M2·M3·M4 병렬(iOS 버그·웹 태그필터·iOS 태그필터). 239 단위 테스트 통과.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 로드맵 | [MVP11 로드맵](mvp11/_roadmap.md) | M1~M4 마일스톤, BUG-001~004, 병렬 전략 — ✅ done |
| 마일스톤 | [M1 검색엔진 품질](mvp11/M1_검색엔진뉴스품질개선.md) | Exa category:news + Tavily topic:news + URL 패턴 필터 |
| 마일스톤 | [M2 iOS 버그수정](mvp11/M2_iOS초기화환경설정버그.md) | BUG-001 콜드스타트·BUG-002 ServerConfig 분기 수정 |
| 마일스톤 | [M3 웹 태그필터](mvp11/M3_웹즐겨찾기오답태그필터.md) | 즐겨찾기·오답 태그 칩 필터, favorites-filter.ts |
| 마일스톤 | [M4 iOS 태그필터](mvp11/M4_iOS즐겨찾기오답태그필터.md) | WrongAnswerTagFilter 순수 함수, @State 단일 소스 |
| 메인태스크 | [M1 메인태스크](mvp11/260423_M1_검색엔진뉴스품질개선.md) | 구현 상세 기록 |
| 메인태스크 | [M2 메인태스크](mvp11/260423_MVP11_M2_iOS초기화환경설정버그수정.md) | 구현 상세 기록 |
| 메인태스크 | [M3 메인태스크](mvp11/260423_MVP11_M3_웹즐겨찾기오답태그필터.md) | 구현 상세 기록 |
| 메인태스크 | [M4 메인태스크](mvp11/260424_MVP11_M4_iOS즐겨찾기오답태그필터.md) | 구현 상세 기록 |
| 분석 | [검색 파이프라인 아키텍처](mvp11/260423_architecture_search_pipeline.md) | M1 Exa+Tavily 파이프라인 설계 결정 기록 |
| 회고 | [일일 회고 260418](mvp11/260418_daily_retro.md) | MVP11 기획 전 환경 점검 회고 |
| 회고 | [일일 회고 260424](../history/260424_daily_retro.html) | MVP11 완주 · BUG-001/002 수정 · Codex 무한 루프 사고 · 튜닝은 실행 속 검증 |
| 버그 | BUG-004 고아 태그 버그 (progress/bugs/ 미결) | 즐겨찾기 삭제 후 고아 selectedTagId — 다음 MVP 수정 예정 |

---

## Harness Engineering — AI 협업 환경 튜닝 (260417~260420)

> 기간 3일. MVP1~10 실측 데이터 기반 하네스 분석 → G1·G6·§8.2 순차 적용. 다음 튜닝 기준점: MVP11부터의 실제 흐름.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 분석 | [하네스 비교분석](harness/analysis/260417_하네스_비교분석.md) | Anthropic·OpenAI 하네스 원리 × frank 실측 대조. 갭 식별 + G1~G6 실행 순서 확정 |
| 태스크 | [G1 Feature List 스킬](harness/260420_g1_feature_list_skill.md) | step-4 초안 생성 + step-8 순회 검증. 안정 ID·4상태 모델·소형 스킵 |
| 태스크 | [G6 allowed-tools 정비](harness/260420_g6_allowed_tools_pruning.md) | 스킬 6개 frontmatter 보수 원칙 + agents.md §2.2 명문화 |
| 태스크 | [§8.2 커밋 차단](harness/260420_g8_2_commit_block.md) | feature-list-check.sh + pre-commit hook + step-9 UX. Anthropic 실패 ①②③ 완전 대응 |
| 회고 | [일일 회고 260420](260420_daily_retro.html) | 하네스 튜닝 완료 회고. 왜 지금 튜닝 / 취사선택 원칙 / MVP11 이후 기대 / 이터레이티브 교훈 |

---

## MVP 버전 체계 (260406 확정)

> .5 버전 폐지. 부채 해소는 다음 MVP에 흡수.

| MVP | 한 줄 정의 | 상태 |
|-----|-----------|------|
| MVP1 | 웹으로 뉴스 읽기 | ✅ 완료 |
| MVP2 | 앱으로 뉴스 읽기 | ✅ 완료 |
| MVP3 | 웹+앱 API 통합 + Apple 로그인 | ✅ 완료 |
| MVP4 | 부채 해소 + 품질 개선 | ✅ 완료 |
| MVP5 | 학습 기능 (수집·요약·즐겨찾기 아키텍처 전환) | ✅ 완료 |
| MVP6 | 피드 UX 개선 (썸네일·성능·태그탭·마크다운) | ✅ 완료 |
| MVP7 | 좋아요·피드 개인화·연관 기사·퀴즈 | ✅ 완료 |
| MVP8 | UX 개선 + 오답 아카이빙 + 퀴즈 완료 배지 | ✅ 완료 |
| MVP9 | 실사용 장벽 제거 + 퀴즈 학습 루프 완성 | ✅ 완료 |
| MVP10 | 버그 수정 + 피드 TTL 캐시 | ✅ 완료 |
| MVP11 | 버그 수정 + 뉴스 품질 개선 | ✅ 완료 |

- MVP1.5, MVP2.5는 역사적 기록으로 유지. 이후 .5 버전 신규 생성 안 함.
- MVP2.5 부채 10건은 MVP3에서 흡수 해소.
- MVP3 이후 모든 마일스톤은 서버+웹+iOS 동시 구현.

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
| MVP2 iOS 아키텍처/패턴 | `mvp2/260405_MVP2_iOS_로드맵.md`, `mvp2/260406_mvp2_completion_retro.md` |
| MVP2.5 기술 부채 목록 | `mvp2/260406_mvp2_completion_retro.md` |
| 3개 플랫폼 규모 비교 | `mvp2/260406_mvp2_completion_retro.md` (9,999줄, 333테스트) |
| Claude Code 설정 자동화 | `mvp15/260405_claude_code_refactoring.md` |
| MVP3 전체 아키텍처 흐름 | `mvp3/260406_MVP3_통합_로드맵.md` |
| Rust API 엔드포인트 명세 | `mvp3/260407_API_SPEC.md` |
| Apple 로그인 트러블슈팅 | `mvp3/260408_M4_회고.md` |
| Mock fixture 기준 데이터 | `mvp3/fixtures/` |
| 전체 흐름도 + 개념 정리 (최신) | `260408_개념정리.md` |
| 인증·토큰·저장소 흐름 도식화 (시각 자료) | `260409_개념정리_도식화.html` |
| MVP3 마일스톤별 흐름 스냅샷 | `mvp3/260408_흐름도.md` |
| MVP4 부채 해소 전체 현황 | `mvp4/_roadmap.md` |
| MVP4 UITest + coverage 인프라 | `mvp4/M4_iOS_테스트인프라.md` |
| MVP5 DB 스키마 + 온디맨드 요약 흐름 | `mvp5/260409_MVP5_로드맵.md` |
| MVP5 FeedPort·SearchPort·SummarizePort 분리 | `mvp5/260409_MVP5_M1_피드아키텍처전환.md` |
| MVP6 M4 마크다운 렌더러 선택 근거 | `mvp6/260410_MVP6_M4_서브태스크.md` |
| 웹 토큰 만료 자동갱신 구조 (httpOnly 쿠키 우회) | `mvp6/260410_MVP6_M4_서브태스크.md` (BF-1) |
| iOS xcconfig 자동 생성 + DB 에러 경고 | `mvp6/260410_MVP6_M4_서브태스크.md` (BF-3) |
| MVP7 좋아요·개인화·퀴즈 전체 설계 | `mvp7/260411_MVP7_로드맵.md` |
| 퀴즈 생성 엔드포인트 설계 (body url, 503 처리) | `mvp7/260412_MVP7_M4_서브태스크.md` |
| 피드 개인화 threshold + cross-tag 오염 수정 | `mvp7/260412_MVP7_M3_서브태스크.md` |
| OpenRouter 키워드 추출 + 가중치 저장 흐름 | `mvp7/260411_MVP7_M2_서브태스크.md` |
| MVP8 오답 아카이빙·퀴즈 배지·UX 개선 전체 설계 | `mvp8/260412_MVP8_로드맵.md` |
| iOS AppDependencies quiz 포트 누락 버그 | `260413_retro.md` |
| snippet 정제 코드 필터링 선택 근거 + Known Issues | `260413_retro.md` |
| MVP8 M2/M3 QA 결과 + 타이포그래피·로딩 색상 수정 경위 | `260413_retro.md` |
| 하네스 분석 + 갭 식별 + 튜닝 실행 순서 | `harness/analysis/260417_하네스_비교분석.md` |
| Feature List 체계 (안정 ID·4상태·소형 스킵) | `harness/260420_g1_feature_list_skill.md` |
| Feature List 미체크 커밋 차단 구조 | `harness/260420_g8_2_commit_block.md` |
| /critical-review 스킬 설계 + step-5 연계 | `harness/analysis/260417_하네스_비교분석.md` |
| Codex 무한 루프 사고 경위 + 에이전트 역할 경계 | `260424_daily_retro.md` |
| 세팅 튜닝 후 첫 MVP 검증 사이클 교훈 | `260424_daily_retro.md` |

---

## MVP12 — 버그 수정 + 좋아요/즐겨찾기 UX 재설계 + 피드 무한 스크롤 (260427)

> 기간 1일. 서버+웹+iOS 3개 플랫폼. 버그 5건 수정(BUG-A~E), BUG-F 정책 싱크. 무한 스크롤 도입.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 로드맵 | [MVP12 로드맵](mvp12/_roadmap.md) | M1~M3 마일스톤, KPI, 기술 결정 기록 6건 |
| 서브태스크 | [M1 서버](mvp12/M1_subtasks.md) | BUG-A(BBC 토픽 URL), BUG-B(snippet 오염), 피드 limit/offset API |
| 서브태스크 | [M2 웹](mvp12/M2_subtasks.md) | BUG-C(스크랩 탭 먹통), BUG-F(웹 오답 필터), UX 재설계, 무한 스크롤 |
| 서브태스크 | [M3 iOS](mvp12/M3_subtasks.md) | BUG-D/E, BUG-F 정책 싱크, UX 재설계, TagState 무한 스크롤 |
| 회고 | [MVP12 회고](mvp12/retro.md) | 의사결정 4건, 부채 3건 (BUG-F 근본 수정, Lazy vs Prefetch, 유닛 테스트) |
| 분석 | [BUG-F 태그 필터 아키텍처](mvp12/analysis_BUG-F-tag-filter.md) | 오답 태그 칩 소스 문제 분석, 서버 스키마 변경 필요 근거 |
| 분석 | [M3 착수 전 품질 스캔](mvp12/analysis_mvp12-m3-prescan.md) | iOS 코드베이스 사전 분석 |
| 토론 | [M2 피드 프리패치 전략](mvp12/debate_M2_feed_prefetch_strategy.md) | Prefetch vs Lazy Loading 비교, Lazy 유지 결정 |
| 발견 | [MVP12 Discovery](mvp12/discovery.md) | 앱 사용 중 발견한 버그 6건(BUG-A~F) + CHECK 항목 원본 메모 |

---

## MVP13 — 실사용 전환: 오답 태그 + 피드 품질 + 클라우드 배포 (260428~260429)

> M1(서버+DB) ✅ + M2(웹+iOS) ✅ 완료. M3(클라우드 배포) ⏸ deferred — Oracle PAYG 업그레이드 오류, 지원팀 문의 중. 재개 조건: Oracle 답변 후 도쿄(ap-tokyo-1) 리전으로 재시도.

| 유형 | 파일 | 핵심 내용 |
|------|------|----------|
| 로드맵 | [MVP13 로드맵](mvp13/_roadmap.md) | M1~M3 마일스톤, 실사용 전환 목표 |
| 마일스톤 | [M1 오답 태그 필터](mvp13/M1_debt01_tag_filter.md) | quiz_wrong_answers tag_id 컬럼 + 서버 필터 API (DEBT-01 해소) |
| 마일스톤 | [M2 피드 싱크](mvp13/M2_client_feed_sync.md) | 웹+iOS 오답 필터 전환 + 피드 하이브리드 캐싱 |
| 마일스톤 | [M3 클라우드 배포](mvp13/M3_cloud_deploy.md) | Oracle Always Free A1, HTTP 직접 사용, $0 배포 — ⏸ deferred |
| 회고 | [260428 일일 회고](260428_daily_retro.md) | "써봐야 없는 게 보인다" · 흐름 중단 기준 |
