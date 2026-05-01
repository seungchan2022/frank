# 비전: Frank MVP15

> 생성일: 260501
> 진입점: MVP14 → MVP15
> 시드: progress/mvp15/_seed.md

## 씨앗

**기사 양·다양성 확보 + 내 커스텀 인사이트** 두 축.

- (E) 양·다양성: 검색엔진 활용 전략 변경 + 양 limit 확대로 피드 풍부하게
- (A-1) 내 커스텀 인사이트: 사용자 프로필 기반 맞춤 시각으로 요약/인사이트 생성

## 핵심 문제

> **선언:** "내 관점으로 트렌드를 보는 개인 레이더"
>
> **현재 토대:** 태그당 5개 · 5분 후 사라짐 · 누적 0 · 시각 단일

→ 정체성과 토대 사이의 갭이 MVP15 채택 이유. 일부 태그는 검색 결과가 1개뿐이라 "레이더" 역할 미흡.

## 타겟 사용자

본인 사용 (AI 접목 iOS 개발자 취준생). 클라우드 배포는 우선순위 낮음 — 맥 켜놓고 본인이 사용.

## 성공 기준

- 피드 양: 태그당 평균 5개 → 10개로 확대
- "1개만 나오는 태그" 원인 파악 + 처방안 제시 (M1 진단)
- iOS 개발자 시각의 인사이트가 기사 카드에 표시됨 (선택적 설정)
- 비용: 현재 $0 유지 (무료 리소스만 활용)
- "훑고 넘김" 정체성 보존 (피드 ephemeral 유지, DB 영속 저장 안 함)

## 보존 필수 디자인 의도

1. **기사 ephemeral 유지** — DB 영속 저장 안 함. 즐겨찾기만 저장. ([feedback_feed_ephemeral](../../../.claude/projects/-Users-seungchan-Workspace-frank/memory/feedback_feed_ephemeral.md))
2. **본인 사용 전제** — 가입 강제·온보딩 없음. 안내·푸시 X
3. **무료 정책 강화** — $0 유지, 무료 리소스 한도 안에서 최대 활용 ([project_api_cost_policy](../../../.claude/projects/-Users-seungchan-Workspace-frank/memory/project_api_cost_policy.md))
