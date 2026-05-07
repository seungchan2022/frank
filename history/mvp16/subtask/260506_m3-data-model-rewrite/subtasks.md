# M3 서브태스크 목록: 데이터 모델 + 재작성 정합

> MVP: 16 | 마일스톤: M3 | 생성: 2026-05-06
> 메인태스크: occupation 삭제 정합 + rewrite_occupation 컬럼 추가 + 클라이언트 버튼 UX

## step-1 미결정 사항 처리 결과

| 항목 | 결정 | 근거 |
|------|------|------|
| D1 페이로드 패턴 | `Option<Option<String>>` + 커스텀 deserializer | serde_with 추가 없이 직접 구현. None=변경없음, Some(None)=삭제, Some(Some(s))=설정 |
| C2-bug 데이터 모델 | 덮어쓰기 유지 + `rewrite_occupation TEXT` 컬럼 추가 | 다중 시점 누적 불필요. 직업 변경 시 버튼 재활성화로 UX 해소 |
| C2-bug 기존 데이터 마이그레이션 | 컬럼 추가만, 기존 데이터 보존 (rewrite_occupation NULL 초기화) | 구조 변경 없으므로 마이그레이션 단순 |
| C4 occupation NULL rewrite 정책 | 버튼 자체 안 보임. 서버 400 안전망 유지 | occupation 없으면 재작성 의미 없음 |
| 캐스케이드 실행 위치 | DbPort 복합 메서드 `update_profile_and_clear_rewrite` 신설 | 단일 트랜잭션 보장. profile_service.rs 신설 불필요 |
| 빈 문자열 처리 | 클라이언트에서 null 변환 전송, 서버 기존 로직 유지 | 서버 `""` → None(변경없음) 유지로 충돌 없음 |

---

## 서브태스크 목록

| # | ID | 내용 | 유형 | 의존 | 예상 |
|---|-----|------|------|------|------|
| 1 | ST-1 | D1 서버: `Option<Option<String>>` 전환 + DbPort 복합 메서드 신설 | feature | — | 2h |
| 2 | ST-2 | D1 단위 테스트 5종 | test | ST-1 | 1h |
| 3 | ST-3 | `rewrite_occupation` 마이그레이션 SQL + 도메인 모델 추가 | feature | — | 0.5h |
| 4 | ST-4 | 저장/조회 핸들러 `rewrite_occupation` 포함 + 클라이언트 타입 파싱 | feature | ST-3 | 1.5h |
| 5 | ST-5 | 클라이언트 재작성 버튼 UX (웹 + iOS) | feature | ST-4 | 2h |
| 6 | ST-6 | E2E 시나리오 3종 + 비용 영향 검토 | chore | ST-5 | 1h |

## 의존성 DAG

```
ST-1 (D1 서버) ──→ ST-2 (D1 테스트)
ST-3 (마이그레이션) ──→ ST-4 (핸들러) ──→ ST-5 (클라이언트) ──→ ST-6 (E2E)
```

병렬 진입 가능: **ST-1 ∥ ST-3**

## 변경 파일 표

| 파일 | 변경 내용 | ST |
|------|-----------|-----|
| `server/src/domain/ports.rs` | `update_profile` 시그니처 변경 + `update_profile_and_clear_rewrite` 신설 | ST-1 |
| `server/src/api/profile.rs` | 커스텀 deserializer + 복합 메서드 호출 | ST-1 |
| `server/src/infrastructure/postgres_db.rs` (또는 동등) | 복합 메서드 SQL 트랜잭션 구현 + update_profile 시그니처 동기화 | ST-1 |
| `server/src/infrastructure/fake_db.rs` | mock 시그니처 동기화 | ST-1, ST-2 |
| `supabase/migrations/20260506_mvp16_m3_rewrite_occupation.sql` | `ALTER TABLE favorites ADD COLUMN rewrite_occupation TEXT` | ST-3 |
| `server/src/domain/models.rs` | `Favorite.rewrite_occupation: Option<String>` 추가 | ST-3 |
| `server/src/domain/ports.rs` | `FavoritesPort.update_favorite_rewrite` 시그니처 확장 | ST-4 |
| `server/src/infrastructure/fake_favorites.rs` (또는 동등) | FavoritesPort mock 동기화 | ST-4 |
| `server/src/services/rewrite_service.rs` | rewrite_occupation 저장/조회 | ST-4 |
| `server/src/api/rewrite.rs` | 응답에 rewrite_occupation 포함 | ST-4 |
| `web/src/lib/...` | 타입 파싱 + 버튼 UX | ST-4, ST-5 |
| `ios/Frank/...` | 타입 파싱 + 버튼 UX (null 인코딩: `encode(nil, forKey:)`) | ST-4, ST-5 |
