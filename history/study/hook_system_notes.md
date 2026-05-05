# Hook 시스템 정리 노트

> 작성일: 2026-05-05  
> 출처: 직접 코드 분석 + 질문 기반 이해

---

## 전체 구조 한눈에

파일 2개 + 훅 2개로 구성된다.

```
스킬이 파일을 씀
       ↓
active_step.txt + active_interview.json
       ↓                    ↓
   장치2가 읽음          장치3이 읽음
   (항상 주입)           (인터뷰 중에만 차단)
```

---

## 상태 파일 (장치1 — 기억 저장소)

### `progress/active_step.txt`

현재 몇 단계인지 기록.

- **쓰는 곳:** step-1 스킬, step-4 스킬, step-9 스킬
- **값 예시:** `step-1` → `step-4` → `none`
- **주의:** step-2, 3, 5, 6, 7, 8은 갱신하지 않음 → DEBT-HOOK-01

### `progress/active_interview.json`

인터뷰 진행 상태 기록.

- **쓰는 곳:** workflow 스킬, step-1 스킬, step-4 스킬, step-9 스킬(cleanup)

**3가지 상태:**

| 상태 | 내용 | 의미 |
|------|------|------|
| 파일 없음 | — | 인터뷰 한 번도 시작 안 됨 |
| `{}` | 빈 객체 | 인터뷰 완료됨 |
| `{ current:2, total:5, remaining:[...] }` | 데이터 있음 | 인터뷰 진행 중 |

파일 없음과 `{}`는 둘 다 "통과"지만 코드 경로가 다르다.
- 파일 없음 → `[ -f "$INT_FILE" ] || exit 0` 에서 바로 통과
- `{}` → 파일 열고 `total: 0` 확인 후 통과

---

## 훅 파일

### `inject-step-status.sh` (장치2)

- **트리거:** UserPromptSubmit — 채팅 입력할 때마다
- **하는 일:** 두 파일 읽어서 Claude 컨텍스트 맨 앞에 주입
- **차단 없음. 안내만.**

출력 예시:
```
인터뷰 진행 중 → 📝 활성: step-1 | 인터뷰 Q2/5 | 남은 질문: Q3, Q4, Q5
인터뷰 없는 스텝 → 📝 활성 step: step-3
active_step.txt 없거나 none → (침묵)
```

토큰 비용은 메시지당 10~30토큰 수준. 실수 수습보다 훨씬 저렴.

### `block-on-interview.sh` (장치3)

- **트리거:** PreToolUse (Edit|Write) — Claude가 파일 수정하려 할 때마다
- **하는 일:** `active_interview.json` 읽어서 `current < total`이면 차단

```
파일 없음 → 통과
{} (total:0) → 통과
진행 중 (current < total) → ❌ 차단
```

차단 메시지:
```
❌ 인터뷰 Q2/5 미완료 — Edit/Write 차단
   남은 질문: Q3, Q4, Q5
   해결책 1: 인터뷰 마저 진행
   해결책 2 (긴급): WORKFLOW_BYPASS=1 환경변수로 우회
```

안전장치 6개 내장: bypass 환경변수, 활성 step 체크, 메타태스크 면제, 24h 자동 expire 등.

---

## 9단계 전체 흐름

`/workflow "M2-피드 캐시 개선"` 기준.

| 스텝 | active_step | active_interview | 장치2 | 장치3 |
|------|-------------|-----------------|-------|-------|
| step-1 인터뷰 중 | step-1 | `{데이터}` | step + Q상태 주입 | ✅ 차단 |
| step-1 완료 후 | step-1 | `{}` | step만 주입 | ❌ 통과 |
| step-2, 3 | step-1 ⚠️ | `{}` | step만 주입 | ❌ 통과 |
| step-4 인터뷰 중 | step-4 | `{데이터}` | step + Q상태 주입 | ✅ 차단 |
| step-4 완료 후 | step-4 | `{}` | step만 주입 | ❌ 통과 |
| step-5, 6, 7, 8 | step-5 ⚠️ | `{}` | step만 주입 | ❌ 통과 |
| step-9 후 | none | `{}` | 침묵 | ❌ 통과 |

⚠️ = DEBT-HOOK-01 미구현 (기능 문제 없음)

---

## 인터뷰가 step-1, step-4에만 있는 이유

- **step-1:** 메인태스크 전체 범위·방향 확정
- **step-4:** 서브태스크 세부사항 확정 (선택 — step-1이 충분히 깊었으면 생략 가능)
- 나머지 스텝은 실행·검증·커밋 단계라 질문할 필요 없음

---

## 관련 부채

**DEBT-HOOK-01** — step-2,3,5,6,7,8 `active_step.txt` 미갱신  
각 스킬 진입부에 `echo "step-N" > progress/active_step.txt` 한 줄 추가 필요.  
장치3엔 영향 없고 장치2 주입값만 부정확해짐. 실제 오작동은 없음.
