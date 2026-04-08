# 로드맵: Frank MVP4

> 생성일: 260408
> 최종 갱신: 260408
> 상태: 계획

---

## MVP4 한 줄 목표

MVP3까지 쌓인 기술 부채와 UX 결함을 해소하여 MVP5(학습 기능) 진입 전 안정적 기반을 확보한다.

---

## 타임라인

| 마일스톤 | 내용 | 예상 기간 | 의존성 | 상태 |
|---------|------|----------|--------|------|
| M1 | 서버 인프라 (LLM 복귀 + Apple Secret 만료 관리) | 반나절~1일 | 없음 | 대기 |
| M2 | iOS UX 개선 (요약 timeout + 로그인 인라인 에러) | 1~2일 | 없음 | 대기 |
| M3 | 웹 UX 개선 (요약 timeout + ErrorBanner/Skeleton) | 1~2일 | 없음 | 대기 |
| M4 | iOS 테스트 인프라 (UITest 4개 + 커버리지 파이프라인) | 2~3일 | M2 완료 후 | 대기 |
| M5 | 데이터 품질 (태그 stale 해소 + Supabase 조사) | 2~3일 | 없음 | 대기 |

> M2/M3는 독립적이므로 병렬 worktree 진행 가능.
> M4는 M2 완료(LoginView 변경 안정화) 후 TC-01 시나리오 작성 가능.

**총 예상**: 7~10일

---

## 의존성 그래프

```mermaid
graph TD
    M1["M1: 서버 인프라\n- LLM 모델 복귀\n- Apple Secret 만료 관리"]
    M2["M2: iOS UX 개선\n- 요약 timeout\n- 로그인 인라인 에러"]
    M3["M3: 웹 UX 개선\n- 요약 timeout\n- ErrorBanner/Skeleton"]
    M4["M4: iOS 테스트 인프라\n- UITest 4 시나리오\n- 커버리지 파이프라인"]
    M5["M5: 데이터 품질\n- 태그 stale 해소\n- Supabase Manual Linking 조사"]
    MVP5["💭 MVP5: 학습 기능"]

    M1 --> M2
    M1 --> M3
    M1 --> M5
    M2 --> M4
    M4 --> MVP5
    M3 --> MVP5
    M5 --> MVP5
```

---

## 마일스톤별 실행 명령

| 마일스톤 | 명령 |
|---------|------|
| M1 | `/workflow "MVP4 M1: 서버 인프라"` |
| M2 | `/workflow "MVP4 M2: iOS UX 개선"` |
| M3 | `/workflow "MVP4 M3: 웹 UX 개선"` |
| M4 | `/workflow "MVP4 M4: iOS 테스트 인프라"` |
| M5 | `/workflow "MVP4 M5: 데이터 품질"` |

---

## 변경 이력

| 날짜 | 변경 내용 | 사유 |
|------|----------|------|
| 260408 | MVP4 로드맵 초안 생성 | MVP3 완료 회고 기반 |
| 260408 | 5개 workflow 마일스톤으로 재편 | 9단계 workflow 적합성 기준 재조정 |
