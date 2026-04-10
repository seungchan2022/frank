# Git 규칙 (v1.1)

이 문서는 Git 커밋 형식, 푸시 금지 규칙을 정의한다.

---

## 1. 핵심 규칙

- **MUST** 커밋만 수행한다. 푸시는 사용자가 직접 수행한다.
- **MUST NOT** `git push` 명령을 실행한다.
- **MUST NOT** `git push --force` 명령을 실행한다.
- **MUST NOT** 커밋 메시지에 `Co-Authored-By:` 줄을 추가한다. 어떤 상황에서도 절대 금지.

## 2. 커밋 메시지 형식

```
tag_en: 제목(한글)

본문 3~4줄 요약(한글)
```

### 2.1 태그 (tag_en)

| 태그 | 설명 |
|------|------|
| feat | 새로운 기능 |
| fix | 버그 수정 |
| docs | 문서 변경 |
| refactor | 리팩토링 |
| test | 테스트 |
| chore | 빌드/설정 |
| style | 포매팅 |
| perf | 성능 개선 |
| security | 보안 관련 |
| hotfix | 긴급 수정 |

### 2.2 예시

```
feat: 사용자 인증 미들웨어 추가

- JWT 기반 인증 미들웨어 구현
- 토큰 만료 검증 로직 추가
- 인증 실패 시 401 응답 반환
```

```
fix: 데이터베이스 커넥션 풀 타임아웃 처리 수정

- 타임아웃 에러 코드 매핑 추가
- 재시도 로직에 exponential backoff 적용
- 타임아웃 시 적절한 에러 응답 반환
```

## 3. 커밋 전 체크리스트

- [ ] 테스트 통과 확인
- [ ] 린트 통과 확인
- [ ] 타입 체크 통과 확인
- [ ] 커밋 메시지 형식 준수
- [ ] 하나의 논리적 변경 단위
- [ ] 민감 정보 포함 여부 확인 (.env, 토큰, API 키 등)
- [ ] 불필요한 파일 제외 (.gitignore 확인)
- [ ] **iOS 신규 .swift 파일 추가 시**: `tuist generate --no-open` 실행 후 `project.pbxproj` 변경 포함

## 3-1. iOS 파일 등록 규칙

신규 Swift 파일 추가 후 반드시 아래 중 하나 실행:

```bash
# 방법 1: 직접 실행
cd ios/Frank && ~/.tuist/Versions/4.31.0/tuist generate --no-open

# 방법 2: 배포 스크립트 (자동 포함)
scripts/deploy.sh --target=ios
```

**이유**: Tuist는 `tuist generate` 실행 시점의 스냅샷만 `project.pbxproj`에 기록.
신규 파일 추가 후 재생성 없으면 Xcode IDE/LSP 인식 불가 (빌드는 통과하나 SourceKit 오탐 발생).

> SourceKit `No such module 'Testing'` 오류: LSP 인덱싱 오탐 — 실제 빌드/테스트 통과 시 무시.

## 4. 커밋 실행 명령

```bash
# 상태 확인
git status

# 변경 내용 확인
git diff

# 스테이징
git add {파일들}

# 커밋
git commit -m "$(cat <<'EOF'
tag_en: 제목(한글)

본문 3~4줄 요약(한글)
EOF
)"

# 커밋 로그 확인
git log --oneline -5
```

## 5. 금지 사항

| 금지 | 이유 |
|------|------|
| `git push` | 사용자 직접 리뷰 후 푸시 |
| `git push --force` | 히스토리 훼손 |
| `Co-Authored-By:` 커밋 태그 | 사용자 명시적 금지 |
| `.env` 커밋 | 민감 정보 노출 |
| 대용량 파일 커밋 | 저장소 비대화 |
| `git reset --hard` | 작업물 손실 |

## 6. 브랜치 전략

- **main에 직접 커밋 금지** -- 반드시 로컬 feature 브랜치에서 작업
- **main에 머지 전 반드시 최신 상태로 갱신 + rebase 머지**:

```bash
git checkout main
git pull origin main
git checkout feature/작업명
git rebase main
git checkout main
git merge feature/작업명    # fast-forward
```

- **main 브랜치에서 직접 rebase 금지** -- feature 브랜치에서 rebase 후 fast-forward merge
- 충돌 발생 시 `--ours`/`--theirs` 일괄 해결 금지 -- 개별 파일 확인 필수
