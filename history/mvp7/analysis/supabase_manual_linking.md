# Supabase Manual Linking 현황 조사

> 작성일: 2026-04-09
> 관련 마일스톤: MVP4 M5 ST-2

---

## 무엇인가

Supabase는 기본적으로 **동일 이메일 = 동일 계정** 규칙으로 자동 연동한다.  
Manual Linking은 이메일이 달라도 관리자나 사용자가 직접 두 아이덴티티를 하나의 계정으로 묶는 기능이다.

---

## 현재 상태 (2026-04-09 기준)

| 항목 | 상태 |
|------|------|
| Manual Linking 전체 | **Beta 유지** (2023년 12월 공개 이후 변동 없음) |
| `supabase.auth.linkIdentity()` | Beta (클라이언트 SDK) |
| `supabase.auth.linkIdentityWithIdToken()` | Beta (클라이언트 SDK, iOS 네이티브 플로우 호환) |
| `auth.admin.linkUserIdentity` | **공식 SDK 문서에 없음** — Admin API로 미제공 |

GA 졸업을 알리는 changelog, 블로그 포스트, GitHub Discussion 없음.

---

## Frank와의 연관성

### 문제 시나리오

```
1. 사용자가 이메일/패스워드로 user@gmail.com 가입
2. 나중에 Apple 로그인 시도
   → Apple이 privaterelay 이메일(abc123@privaterelay.appleid.com) 제공
3. 두 이메일이 달라 Supabase 자동 연동 실패
4. 같은 사람인데 계정 2개로 분리 → 태그/기사 데이터 분리
```

### 현재 실제 위험도

**낮음.** 발생 조건이 세 가지 동시 충족 필요:
- 이메일로 먼저 가입
- 나중에 Apple 로그인도 추가 시도
- Apple에서 privaterelay 이메일 선택

MVP 단계에서는 무시 가능한 엣지 케이스.

### 장기 위험

사용자가 늘면 "Apple로 로그인하면 기사가 없어요" 류의 CS 발생 가능.  
두 계정에 데이터가 쌓인 후 병합은 어느 데이터를 살릴지 결정해야 해서 복잡해짐.

---

## 결정 사항

### 지금 당장: 보류

Beta API 리스크(API 변경 가능) + 실제 발생 가능성 낮음.

### 단기 예방책 (UX만으로 충돌 예방)

Apple 첫 로그인 시 "이미 이메일 계정이 있으신가요?" 안내 UI 추가.  
있으면 이메일로 먼저 로그인하도록 유도 → 기술 구현 없이 충돌 방지.

### Manual Linking GA 졸업 시: 즉시 구현

`auth.admin.linkUserIdentity`가 공식 SDK에 추가되는 시점이 신호.  
구현 경로: 서버 Admin API 기반 계정 병합 엔드포인트.

---

## 다음 조사 시점

**MVP5 시작 전** 재조사.  
- [ ] Supabase changelog에서 Manual Linking GA 졸업 여부 확인
- [ ] `auth.admin.linkUserIdentity` SDK 문서 등재 여부 확인
