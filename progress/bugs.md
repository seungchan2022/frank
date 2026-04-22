# 알려진 버그 목록

발견된 버그를 기록. 다음 MVP 기능 구현 전에 수정 후 진행.

---

## [BUG-001] 앱 첫 실행 시 태그/데이터 로딩 실패

**발견**: 2026-04-22 실기기 테스트 중
**재현**: 시뮬레이터·실기기 공통, 항상 재현됨
**증상**: 앱 시작 직후 "태그를 불러오지 못했습니다" 에러 표시 → 다시 시도 누르면 정상 동작
**원인**: Supabase 세션 복원이 완료되기 전에 API 요청이 먼저 나감 → Bearer 토큰 없음 → 401/연결 실패

콘솔 경고:
```
Initial session emitted after attempting to refresh the local stored session.
To opt-in to the new behavior now, set `emitLocalSessionAsInitialSession: true`
```

**수정 방법**: Supabase AuthClient 설정에 `emitLocalSessionAsInitialSession: true` 추가,
또는 세션 준비 완료 이벤트를 기다린 후 데이터 요청 시작하도록 초기화 순서 수정

**우선순위**: 다음 MVP 시작 전 수정
