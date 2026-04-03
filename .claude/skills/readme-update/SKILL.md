---
name: readme-update
description: 각 모듈/패키지 README 생성/업데이트.
allowed-tools:
  - Read
  - Write
  - Glob
  - Grep
  - Bash
  - Agent
---

# README 업데이트 (/readme-update)

> 각 모듈의 README.md를 표준 템플릿 기반으로 생성 또는 보강한다.

## 사용법

```
/readme-update              # 전체 모듈 일괄 업데이트
/readme-update "모듈명"     # 특정 모듈만 업데이트
```

## 수행 작업

1. 대상 파악
2. 각 모듈 분석 (엔드포인트, 환경변수, 아키텍처, 스키마)
3. README 생성/보강 (표준 템플릿)
4. 리뷰

## 규칙

- 한국어 작성
- 기존 내용 최대한 보존, 누락 섹션만 추가
- 코드 변경 없음, 문서 작업만
