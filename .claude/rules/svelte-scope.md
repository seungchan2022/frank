# Svelte 규칙 (자동 적용)

이 파일은 Svelte/SvelteKit 프론트엔드 경로 작업 시 자동 로드된다.
프로젝트의 `.claude/rules/` 디렉토리에 복사하여 사용한다.

## 기술 스택

- SvelteKit 2.x + Svelte 5 (runes: $state, $derived, $effect)
- TypeScript strict mode
- Tailwind CSS
- Vitest (단위/통합 테스트)
- Playwright (E2E 테스트)

## Svelte 5 반응성 규칙 (필수)

`$state()`에 Set/Map/Array 사용 시 뮤테이션은 반응성을 트리거하지 않는다:

```
❌ state.add(item) / state.push(item) / state.delete(item)
✅ state = new Set([...state, item])
✅ state = [...state, item]
✅ state = new Map([...state, [key, value]])
```

Svelte 5의 `$state`는 **참조 비교**로 변경 감지. 반드시 새 객체를 할당해야 반응성이 동작한다.

## 코드 스타일

- Svelte 5 runes (`$state`, `$derived`, `$effect`) 사용
- Tailwind CSS 유틸리티 클래스 기반 스타일링
- i18n 키 기반 텍스트 (하드코딩 금지)
- 다크 모드 지원
- API 프록시 라우트: throw 대신 JSON 에러 응답 반환

## 상태 초기화 원자성 (필수)

새 대화/세션 전환 구현 시 관련 상태 전체를 원자적으로 리셋한다.
하나라도 빠지면 이전 세션 데이터 혼입 버그 발생.

## UI 상태 전이 완결성 (필수)

버튼/탭 클릭 핸들러 구현 시 **상태 전이 매핑** 작성:

```
[사용자 액션] → [변경할 상태 변수 전체] → [UI 반영 확인]
```

## 필수 검증 (커밋 전)

```bash
cd {프론트엔드_경로}

# 린트
npm run lint

# 타입 체크
npm run check

# 테스트
npm run test

# 통합 (권장)
npm run validate    # lint + check + test
```

## 프론트엔드 수정 후 시각적 검증 (필수)

프론트엔드 파일(.svelte, .ts, .css) 수정 후 완료 보고 전:

1. 개발 서버 실행 중이면 **Playwright/Chrome DevTools로 스크린샷 촬영**
2. 스크린샷으로 레이아웃, 간격, 색상, 반응형 확인

## SvelteKit 모듈 Mock

테스트에서 SvelteKit 모듈(`$app/*`, `$env/*`)은 `vi.mock()`으로 처리.

## 디렉토리 구조 (권장)

```
src/
├── app.css                # Tailwind CSS
├── app.html               # HTML 셸
├── lib/
│   ├── components/        # Svelte 5 컴포넌트
│   │   └── __tests__/     # 컴포넌트 테스트
│   ├── stores/            # 반응형 스토어 (.svelte.ts)
│   ├── server/            # 서버 사이드 헬퍼
│   ├── types/             # 타입 정의
│   ├── i18n/              # 다국어
│   └── utils/             # 유틸리티
├── routes/                # SvelteKit 파일 기반 라우팅
│   ├── +layout.svelte
│   ├── +page.svelte
│   └── api/               # API 프록시 라우트
└── test-utils/
    └── setup.ts           # Vitest 글로벌 설정
```
