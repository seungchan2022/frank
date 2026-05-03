# M3 응답 스키마 SSOT

> 이 파일은 ST-4에서 **클라이언트 병렬 작업 전 최우선 생성** (C1 수정 반영).
> 웹(ST-5/ST-6)과 iOS(ST-7/ST-8) 구현 시 이 파일을 SSOT로 사용한다.
> 작성일: 2026-05-03

---

## GET /me/profile

```json
{
  "id": "uuid",
  "display_name": "홍길동",
  "onboarding_completed": true,
  "occupation": "iOS 개발자"
}
```

- `occupation`: `string | null` — 미설정 시 `null`

---

## PUT /me/profile (occupation 포함)

**요청**:
```json
{
  "occupation": "iOS 개발자"
}
```

**응답**: 위 GET /me/profile 스키마와 동일

**엣지케이스**:
- occupation이 빈 문자열(`""`) 또는 공백만 → `occupation: null` 처리 (삭제 의도)
- occupation 50자 초과 → `400 Bad Request`

---

## POST /me/summarize

**요청**:
```json
{
  "url": "https://example.com/article",
  "title": "Article Title"
}
```

**응답**:
```json
{
  "summary": "기사 핵심 내용을 한국어로...",
  "insight": "iOS 개발자 시각에서 이 기사가 중요한 이유..."
}
```

- `summary`: `string` — 항상 존재
- `insight`: `string | null` — occupation 설정 시에만 존재, 미설정 시 `null`

**클라이언트 타입 정의**:
- 웹: `insight: string | null` (non-null 단언 `!` 금지)
- iOS: `let insight: String?` (optional 처리 필수)

---

## POST /me/rewrite

**요청**:
```json
{
  "url": "https://example.com/article",
  "title": "Article Title"
}
```

**성공 응답 (200)**:
```json
{
  "rewrite": "iOS 개발자 시각에서 재작성된 기사 내용..."
}
```

**에러 응답**:
- `400 Bad Request`: occupation 미설정 시 `{ "error": "occupation_required" }`
- `503 Service Unavailable`: Groq timeout/retry 실패

**동작**:
1. JWT → DB에서 occupation 조회
2. occupation 없으면 400 반환
3. LLM `rewrite_with_occupation` 호출
4. 결과를 `{ "rewrite": "..." }` 반환
5. favorites.rewrite 컬럼에 upsert (best-effort, 실패해도 200)

---

## 변경 이력

| 날짜 | 변경 내용 |
|------|----------|
| 2026-05-03 | 최초 생성 — ST-4 SSOT 확정 |
