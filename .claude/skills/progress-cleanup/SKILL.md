---
name: progress-cleanup
description: "진행상황 정리. 완료 태스크 아카이브+stale TODO 감지+요약 인덱스 생성. 트리거 키워드: progress 정리, cleanup."
---

# /progress-cleanup 스킬

progress/ 디렉토리의 태스크 파일을 정리하고, 완료된 항목을 아카이브하며 요약 인덱스를 생성한다.

## 프로세스

1. 전체 스캔
2. 상태 판별 (COMPLETED / STALE / ACTIVE / INACTIVE)
3. 아카이브 (사용자 확인 후)
4. Stale TODO 리포트
5. 요약 인덱스 생성 (`progress/INDEX.md`)
