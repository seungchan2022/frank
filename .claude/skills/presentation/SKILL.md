---
name: presentation
description: "Gamma+Mermaid MCP로 디자인 품질 프레젠테이션/인포그래픽 생성. 트리거 키워드: PPT 만들어줘, 프레젠테이션, 슬라이드, 인포그래픽, 보고서, 발표자료."
context: fork
allowed-tools:
  - Write
  - mcp__mermaid__get-mermaid-draft
  - mcp__mermaid__save-mermaid-draft
  - mcp__mermaid__mermaid-mcp-app
  - mcp__gamma__*
  - mcp__d2__*
---

# /presentation 스킬

Gamma MCP + Mermaid MCP를 활용하여 디자인 품질 프레젠테이션을 자동 생성한다.

## 프로세스

1. 요구사항 확인 (주제, 슬라이드 수, 형식, 언어, 톤)
2. 다이어그램 사전 생성 (Mermaid/D2)
3. Gamma로 프레젠테이션 생성
4. 결과 확인 (Gamma URL + PDF/PPTX 다운로드)
5. 피드백 반영

## 허용 MCP

- Gamma: generate_gamma, get_generation, list_themes
- Mermaid: generate_mermaid
- D2: render-d2 (아키텍처 다이어그램 보조)
