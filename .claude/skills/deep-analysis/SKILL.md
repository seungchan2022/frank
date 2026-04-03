---
name: deep-analysis
description: "심층추론/심층분석. 다중 MCP 활용 심층 코드/아키텍처 분석. 트리거 키워드: 심층추론, 심층분석, deep analysis, 깊이 분석, 코드 분석해줘."
---

# /deep-analysis 스킬

다중 MCP 도구를 활용하여 심층적인 코드/아키텍처 분석을 수행한다.

## 호출 형식

```
/deep-analysis {type} {target}
```

## 분석 유형 (type)

| 유형 | 약어 | 설명 |
|------|------|------|
| architecture | arch | 아키텍처 위반, 순환 의존성, 결합도 분석 |
| code-quality | quality | 복잡도, 중복, 네이밍, 패턴 준수 분석 |
| performance | perf | 병목, 메모리, 응답 시간 분석 |
| security | sec | 입력 검증, 인증, 민감 데이터 분석 |
| test-coverage | test | 누락 경로, 경계값, 커버리지 분석 |
| full | full | 5개 유형 전체 통합 보고서 |

## 프로세스

1. 대상/유형 선택 (인자 없으면 인터뷰 모드)
2. deep-analysis 에이전트 호출
3. 보고서 생성
4. A/B/C 3가지 개선안 제시
5. progress/ 문서화

## 출력: `progress/analysis/{YYMMDD}_{type}_{target}.md`

## 제약: Context7 최대 3회/태스크, 코드 수정 금지, 보안 분석 시 공격 시도 금지
