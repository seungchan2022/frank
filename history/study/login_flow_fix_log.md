# 로그인 흐름 문서 수정 로그

> 생성: 2026-05-04
> 배경: 어드바이저 팩트체크에서 `signInWithPassword()`를 "SvelteKit(웹)(브라우저 JS)이 실행한다"고 잘못 기술한 것을 발견.
> 실제로는 `+page.server.ts`의 서버 액션이 실행한다. 이 수정을 시작으로 파생된 전체 불일치 목록.

---

## 완료된 수정

- [x] **노트** `mvp1_stage1_notes.md` — 등장인물 표에 `+page.server.ts` 행 추가
- [x] **노트** `mvp1_stage1_notes.md` — ② 설명: "SvelteKit(웹)이" → "`+page.server.ts`의 서버 액션이" + httpOnly 이유 추가
- [x] **노트** `mvp1_stage1_notes.md` — ② 아래 `+page.server.ts` 개념 블록쿼트 추가
- [x] **HTML** `index.html` 이메일 패널 — 등장인물 표에 `+page.server.ts` 행 추가
- [x] **HTML** `index.html` 이메일 패널 — ② 설명 동일하게 수정 + 개념 블록쿼트 추가
- [x] **HTML** `index.html` 이메일 패널 — 전체 흐름 요약 ②줄 수정
- [x] **SVG** `flow_email.svg` — `+page.server.ts` 액터 추가해서 재생성

---

## 남은 수정 목록

### A. 이메일 패널 — 설명 텍스트가 새 SVG와 불일치

SVG는 `+page.server.ts`가 새 액터로 들어갔는데, 설명 텍스트 일부가 아직 구 흐름 기준.

#### ~~A-1. 이메일 ① 설명 수정 (HTML + 노트)~~ ✅
- ~~**현재**: "로그인 버튼을 누르면 SvelteKit(웹)으로 전달된다"~~
- **수정 완료**: "로그인 버튼을 누르면 form POST로 `+page.server.ts`에 전달된다"
- 파일: `index.html` (이메일 패널 ①), `mvp1_stage1_notes.md` (웹 이메일 ①)

#### ~~A-2. 이메일 ④ 설명 수정 (HTML + 노트)~~ ✅
- ~~**현재**: "hooks.server.ts가 반환된 JWT를 httpOnly 쿠키에 저장한다"~~
- **수정 완료**: "`+page.server.ts`에서 `locals.supabase`(Supabase SSR 클라이언트)가 JWT를 httpOnly 쿠키에 자동으로 저장한다"
- 파일: `index.html` (이메일 패널 ④), `mvp1_stage1_notes.md` (웹 이메일 ④)

#### ~~A-3. 이메일 ⑤ 설명 수정 (HTML + 노트)~~ ✅
- ~~**현재**: "hooks.server.ts가 SvelteKit(웹)으로 '홈으로 가' 신호(303)를 보낸다"~~
- **수정 완료**: "`+page.server.ts`가 `redirect(303, '/')`를 반환하고, 브라우저가 홈으로 자동 이동한다"
- 파일: `index.html` (이메일 패널 ⑤), `mvp1_stage1_notes.md` (웹 이메일 ⑤)

#### ~~A-4. 전체 흐름 요약 ④줄 수정 (HTML)~~ ✅
- ~~**현재**: "④ hooks.server.ts → httpOnly 쿠키에 JWT 저장"~~
- **수정 완료**: "④ +page.server.ts(locals.supabase SSR) → httpOnly 쿠키에 JWT 자동 저장"
- 파일: `index.html` (이메일 패널 전체 흐름 요약)

---

### B. Apple OAuth 웹 패널 — SVG 재생성 + 설명 수정

`appleOAuth` 액션도 `+page.server.ts`에 있는데(코드 확인 완료: `+page.server.ts` line 78~94), 현재 SVG에는 `+page.server.ts` 액터가 없음.

#### ~~B-1. Apple OAuth SVG 재생성~~ ✅
- ~~**현재**: `flow_apple.svg`에 `SvelteKit(웹)`, `hooks.server.ts`, `Apple OAuth`, `Supabase Auth`, `/auth/callback` 5개 액터~~
- **수정 완료**: `+page.server.ts` 액터 추가. `signInWithOAuth()` 호출이 `+page.server.ts`에서 나가도록 변경
- 파일: `flow_apple.svg` 재생성

#### ~~B-2. Apple OAuth 패널 등장인물 표에 `+page.server.ts` 추가 (HTML + 노트)~~ ✅
- ~~**현재**: 등장인물 표에 `+page.server.ts` 없음~~
- **수정 완료**: 이메일 패널과 동일하게 행 추가
- 파일: `index.html` (Apple OAuth 패널 등장인물 표), `mvp1_stage1_notes.md` (Apple OAuth 등장인물)

#### ~~B-3. Apple OAuth ① 설명 수정 (HTML + 노트)~~ ✅
- ~~**현재**: "SvelteKit 서버 액션으로 POST 요청을 보낸다"~~
- **수정 완료**: "`+page.server.ts`의 `appleOAuth` 서버 액션으로 POST 요청을 보낸다"
- 파일: `index.html` (Apple OAuth 패널 ①), `mvp1_stage1_notes.md`

#### ~~B-4. Apple OAuth ② 설명 수정 (HTML + 노트)~~ ✅
- ~~**현재**: "SvelteKit **서버**의 `appleOAuth` 액션이 실행된다"~~
- **수정 완료**: "`+page.server.ts`의 `appleOAuth` 서버 액션이 실행된다"
- 파일: `index.html` (Apple OAuth 패널 ②), `mvp1_stage1_notes.md`

---

### C. 어드바이저 2차 팩트체크 발견 항목 (2026-05-04)

#### ~~C-1. 등장인물 블록쿼트에 `+page.server.ts` 추가 (노트)~~ ✅
- ~~**현재**: "SvelteKit은 화면(웹)과 서버(hooks.server.ts) 두 부분"~~
- **수정 완료**: "화면(웹), 서버 액션(`+page.server.ts`), 문지기(`hooks.server.ts`) 세 부분"
- 파일: `mvp1_stage1_notes.md` (line 22)

#### ~~C-2. 이메일 전체 흐름 요약 갱신 (노트)~~ ✅
- ~~**현재**: "② SvelteKit(웹) → Supabase", "④ hooks.server.ts → httpOnly 쿠키"~~
- **수정 완료**: "② form POST → +page.server.ts → Supabase", "④ +page.server.ts(locals.supabase SSR) → httpOnly 쿠키 자동 저장"
- 파일: `mvp1_stage1_notes.md` (전체 흐름 요약 코드블록)

#### ~~C-3. Apple OAuth ③ 비교 블록쿼트 수정 (HTML + 노트)~~ ✅
- ~~**현재**: "이메일 흐름에서는 hooks.server.ts → Supabase" + "hooks.server.ts가 쿠키에 저장하는 역할"~~
- **수정 완료**: "+page.server.ts → Supabase" + "/auth/callback의 locals.supabase(SSR 클라이언트)가 쿠키에 저장"
- 파일: `mvp1_stage1_notes.md` (line 240), `index.html` (line 216)

#### ~~C-4. Apple OAuth 전체 흐름 요약 ①줄 수정 (HTML)~~ ✅
- ~~**현재**: "SvelteKit 서버 액션 진입"~~
- **수정 완료**: "+page.server.ts 서버 액션 진입"
- 파일: `index.html` (Apple OAuth 전체 흐름 요약)

#### ~~C-5. Apple OAuth ⑨ 쿠키 저장 주체 수정 (HTML + 노트)~~ ✅
- ~~**현재**: "hooks.server.ts가 JWT를 httpOnly 쿠키에 저장"~~
- **수정 완료**: "/auth/callback에서 locals.supabase(SSR 클라이언트)가 JWT를 httpOnly 쿠키에 자동 저장"
- 파일: `mvp1_stage1_notes.md` (line 279), `index.html` (line 229)

#### ~~C-6. iOS 비교 섹션 쿠키 저장 주체 수정 (노트)~~ ✅
- ~~**현재**: "웹에서는 hooks.server.ts가 JWT를 httpOnly 쿠키에 저장하는 코드를 직접 실행"~~
- **수정 완료**: "locals.supabase(SSR 클라이언트)가 저장, hooks.server.ts는 해당 클라이언트를 설정"
- 파일: `mvp1_stage1_notes.md` (line 377)

---

## 수정 순서 (권장)

1. A-1 → A-2 → A-3 → A-4 (이메일 패널 텍스트 완성)
2. B-1 (Apple OAuth SVG 재생성)
3. B-2 → B-3 → B-4 (Apple OAuth 패널 텍스트)
4. C-1 → C-6 (어드바이저 2차 팩트체크 항목)
5. 브라우저에서 두 탭 최종 확인
