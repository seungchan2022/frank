---
name: debate
description: "3자 토론/심층토론. Claude + Codex + Serena가 경쟁 가설로 토론. 트리거 키워드: 토론해줘, 심층토론, 3자 토론, debate, 의견 비교."
context: fork
allowed-tools:
  - mcp__codex__codex
  - mcp__codex__codex-reply
  - mcp__serena__find_symbol
  - mcp__serena__get_symbols_overview
  - mcp__serena__find_referencing_symbols
  - mcp__serena__read_file
  - mcp__serena__list_dir
  - mcp__serena__search_for_pattern
  - mcp__context7__resolve-library-id
  - mcp__context7__query-docs
  - mcp__memory__search_nodes
  - mcp__memory__read_graph
  - mcp__memory__open_nodes
  - mcp__tavily__tavily_search
  - mcp__tavily__tavily_research
  - mcp__sequential-thinking__sequentialthinking
  - mcp__exa__web_search_exa
  - mcp__arxiv-mcp-server__search_papers
  - mcp__arxiv-mcp-server__download_paper
  - mcp__arxiv-mcp-server__read_paper
  - Task
  - Read
  - Write
  - Glob
  - Grep
---

# 3자 토론 (Scientific Debate)

> Claude + Codex + Serena 경쟁 가설 토론. `/debate {주제}`로 호출.

## 참여자 역할

| 참여자 | 관점 | 주 도구 |
|--------|------|---------|
| Claude | 사용자 의도, 문서 일치성 | Read, Write, Glob, Grep |
| Codex | 기술적 타당성, 구현 가능성 | mcp__codex__codex |
| Serena | 구조적 영향, 아키텍처 분석 | mcp__serena__* |

## 보조 도구

| 도구 | 용도 |
|------|------|
| context7 | 라이브러리 공식 문서 참조 |
| memory (읽기 전용) | 이전 토론/결정 사항 참조 |
| tavily | 외부 모범 사례/패턴 조사 |
| sequential-thinking | 복잡한 쟁점 분해 |
| exa | 시맨틱 검색으로 외부 근거 수집 |
| perplexity | 빠른 팩트체크 + 심층추론 |
| arxiv / paper-search | 학술 논문 근거 |

## 토론 절차

### 1. 주제 제시
### 2. 가설 수집 (보조 도구로 근거 수집 후 각자 가설 제시)
### 3. 상호 반박
### 4. 합의 도출
### 5. 결론 문서화 → `progress/debate/{YYMMDD}_{단계}_{주제}.md`

## 토론 후 액션

1. 로그 저장
2. 결론을 해당 단계에 반영
3. 필요시 memory에 결정사항 저장 (토론 중 memory 쓰기 금지)
