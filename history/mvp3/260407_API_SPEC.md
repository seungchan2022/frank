# Frank API SPEC

> 생성일: 260407 (M1.5)
> 진실의 원천: `server/src/api/`, `server/src/domain/models.rs`
> 대상: web (M2), iOS (M3)
> 상태: 🔒 **FROZEN** — 변경 시 양쪽 작업 일시 중지 + hotfix

---

## 인증

모든 `/api/me/*` 엔드포인트는 `Authorization: Bearer <jwt>` 헤더 필수. JWT는 Supabase Auth에서 발급. 누락/잘못된 토큰 → **401**.

---

## 데이터 모델

### Article (서버 → 클라이언트)

서버 응답 타입은 `ArticleResponse` DTO. 내부 필드(`content`, `llm_model`, `prompt_tokens`, `completion_tokens`)는 노출되지 않는다.

```typescript
// TypeScript (web)
interface Article {
  id: string;            // UUID
  user_id: string;       // UUID
  tag_id: string | null; // UUID | null
  title: string;
  title_ko: string | null;
  url: string;
  snippet: string | null;
  source: string;
  search_query: string | null;
  summary: string | null;
  insight: string | null;
  summarized_at: string | null; // ISO 8601
  published_at: string | null;  // ISO 8601
  created_at: string | null;    // ISO 8601
}
```

```swift
// Swift (iOS)
struct Article: Identifiable, Equatable, Sendable {
    let id: UUID
    let userId: UUID
    let tagId: UUID?
    let title: String
    let titleKo: String?
    let url: URL
    let snippet: String?
    let source: String
    let searchQuery: String?
    let summary: String?
    let insight: String?
    let summarizedAt: Date?
    let publishedAt: Date?
    let createdAt: Date?
}
```

⚠️ **iOS 현재 상태**: `userId`, `searchQuery`, `createdAt` 필드가 없음. M3 진행 시 모델 확장 필요.

### Profile

```typescript
interface Profile {
  id: string;              // UUID
  display_name: string | null;
  onboarding_completed: boolean;
}
```

```swift
struct Profile: Equatable, Sendable {
    let id: UUID
    let displayName: String?
    let onboardingCompleted: Bool
}
```

⚠️ **iOS 현재 상태**: `email` 필드가 있고 `displayName` 필드가 없음. M3에서 모델 정정 필요. (`email`은 Supabase Auth에서 별도 조회)

### Tag

```typescript
interface Tag {
  id: string;
  name: string;
  category: string | null;
}
```

```swift
struct Tag: Identifiable, Equatable, Sendable {
    let id: UUID
    let name: String
    let category: String?
}
```

⚠️ **iOS 현재 상태**: `category`가 non-optional. 서버 모델은 nullable.

---

## 엔드포인트

### `GET /health`
- **인증**: 없음
- **응답**: `200` `{"status":"ok"}`

### `GET /api/tags`
- **인증**: 필요
- **응답**: `200` `Tag[]` (전체 시스템 태그, 카테고리/이름 정렬)

### `GET /api/me/tags`
- **인증**: 필요
- **응답**: `200` `string[]` (현재 사용자가 선택한 tag_id 배열)

### `POST /api/me/tags`
- **인증**: 필요
- **요청**: `{ "tag_ids": string[] }`
- **응답**: `200` `{ "ok": true }`
- **부수효과**: 사용자 태그 전체 교체. `onboarding_completed` 자동 `true`

### `GET /api/me/profile`
- **인증**: 필요
- **응답**: `200` `Profile`
- **에러**: `404` 프로필 없음

### `PUT /api/me/profile` 🆕 (M1)
- **인증**: 필요
- **요청 바디** (모두 optional):
  ```json
  { "onboarding_completed": true, "display_name": "이름" }
  ```
- **빈 바디** `{}` 허용 → no-op, 현재 프로필 반환
- **검증**:
  - `display_name` trim, 빈 문자열 → 400
  - `display_name` 50자 초과 → 400
- **응답**: `200` `Profile` (수정 후)

### `GET /api/me/articles` 🔄 (M1 확장)
- **인증**: 필요
- **쿼리** (모두 optional):
  - `limit` (기본 50, clamp 1..=100)
  - `offset` (기본 0, ≥0)
  - `tag_id` (UUID 문자열)
- **응답**: `200` `Article[]` (created_at desc 정렬)
- **에러**:
  - `limit < 1` → 400 `"limit must be >= 1"`
  - `offset < 0` → 400 `"offset must be >= 0"`
  - `tag_id` 잘못된 UUID → 400 `"invalid tag_id"`

### `GET /api/me/articles/:id` 🆕 (M1)
- **인증**: 필요
- **응답**: `200` `Article` (본인 기사만)
- **에러**: `404` (타인 기사 또는 없는 기사 — 통일)

### `POST /api/me/collect`
- **인증**: 필요
- **응답**: `200` `{ "collected": number }`
- **부수효과**: LLM 검색 → 기사 수집

### `POST /api/me/summarize`
- **인증**: 필요
- **응답**: `200` `{ "summarized": number }`
- **부수효과**: 미요약 기사를 LLM으로 요약 처리

---

## 에러 응답 표준

```json
{ "error": "사람이 읽을 수 있는 메시지" }
```

| Status | 의미 |
|--------|------|
| 400 | 잘못된 요청 (검증 실패) |
| 401 | 인증 실패 (토큰 누락/만료/잘못됨) |
| 404 | 리소스 없음 (또는 권한 없음, 정보 누설 방지로 통일) |
| 500 | 서버 내부 오류 — 본문에 내부 메시지 노출 안 함 |

---

## 클라이언트 측 API 함수 명세

### web (`web/src/lib/api/client.ts`)

```typescript
interface ApiClient {
  fetchTags(): Promise<Tag[]>;
  fetchMyTagIds(): Promise<string[]>;
  saveMyTags(tagIds: string[]): Promise<void>;        // POST + 온보딩 처리
  updateMyTags(tagIds: string[]): Promise<void>;      // POST 단독 (설정 페이지)
  fetchProfile(): Promise<Profile>;
  updateProfile(patch: Partial<Pick<Profile, 'display_name' | 'onboarding_completed'>>): Promise<Profile>;
  fetchArticles(opts?: { offset?: number; limit?: number; tagId?: string }): Promise<Article[]>;
  fetchArticleById(id: string): Promise<Article | null>;
  collectArticles(): Promise<number>;
  summarizeArticles(): Promise<number>;
}
```

### iOS (`Core/Ports/`)

```swift
protocol ArticlePort: Sendable {
    func fetchArticles(filter: ArticleFilter) async throws -> [Article]
    func fetchArticle(id: UUID) async throws -> Article
}

protocol TagPort: Sendable {
    func fetchAllTags() async throws -> [Tag]
    func fetchMyTagIds() async throws -> [UUID]
    func saveMyTags(tagIds: [UUID]) async throws
}

protocol ProfilePort: Sendable {  // 🆕 신설 권장 (M3)
    func fetchProfile() async throws -> Profile
    func updateProfile(displayName: String?, onboardingCompleted: Bool?) async throws -> Profile
}

protocol AuthPort: Sendable { /* 기존 그대로 */ }
protocol CollectPort: Sendable { /* 기존 그대로 */ }
```

⚠️ **iOS 현재 상태**: `ProfilePort`가 없음. `AuthPort.updateOnboardingCompleted()`에 일부 기능이 섞여 있음. M3에서 정리 권장.

---

## 인증 토큰 흐름 (참고)

```
[웹/iOS] Supabase Auth SDK로 로그인
       ↓
[웹/iOS] access_token (JWT) 획득 + 저장 (쿠키/Keychain)
       ↓
[웹/iOS] 모든 API 호출에 Authorization: Bearer <jwt>
       ↓
[Rust 서버] middleware/auth.rs가 Supabase Auth /user 엔드포인트로 검증
       ↓
[Rust 서버] AuthUser { id: Uuid } 추출 → 핸들러에 주입
```

---

## 변경 시 절차 (FROZEN 깨기)

1. M2 또는 M3 진행 중 contract 부족분 발견
2. **즉시 양쪽 작업 일시 중지**
3. main 워크트리에서 hotfix 브랜치로 server 수정
4. 본 문서 갱신 + 머지
5. 양쪽 worktree에서 `git pull --rebase origin main`
6. 작업 재개
