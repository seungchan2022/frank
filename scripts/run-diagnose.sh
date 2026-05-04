#!/usr/bin/env bash
# run-diagnose.sh — 진단 바이너리(diagnose_search) 실행 표준화 스크립트
#
# 기능:
#   F-01: 필수 환경변수 체크 → 누락 시 어떤 변수가 없는지 명시 후 exit 1
#   F-02: server/.env 자동 소싱 후 cargo run --bin diagnose_search 실행
#
# 사용법:
#   bash scripts/run-diagnose.sh [args...]   (기본 실행)
#   bash scripts/run-diagnose.sh -h          (도움말 출력)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${REPO_ROOT}/server/.env"

# -h 도움말 처리
if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    cat <<'EOF'
run-diagnose.sh: 검색엔진 진단 바이너리(diagnose_search) 실행 도구

사용법:
  bash scripts/run-diagnose.sh [args...]

필수 환경변수 (server/.env 또는 환경에서 설정):
  EXA_API_KEY        Exa API 키
  TAVILY_API_KEY     Tavily API 키
  DATABASE_URL       PostgreSQL 연결 URL

옵션:
  -h, --help   도움말 출력
EOF
    exit 0
fi

# F-02: server/.env 자동 소싱 (파일이 있으면)
if [[ -f "${ENV_FILE}" ]]; then
    # shellcheck disable=SC1090
    set -a
    source "${ENV_FILE}"
    set +a
fi

# F-01: 필수 환경변수 체크
MISSING_VARS=()
for var in EXA_API_KEY TAVILY_API_KEY DATABASE_URL; do
    if [[ -z "${!var:-}" ]]; then
        MISSING_VARS+=("${var}")
    fi
done

if [[ ${#MISSING_VARS[@]} -gt 0 ]]; then
    echo "ERROR: 다음 환경변수가 설정되지 않았습니다:" >&2
    for v in "${MISSING_VARS[@]}"; do
        echo "  - ${v}" >&2
    done
    echo "" >&2
    echo "server/.env 파일에 해당 변수를 추가하거나 환경에서 직접 설정하세요." >&2
    exit 1
fi

# R-01: cargo 빌드/실행 실패 시 exit code 그대로 전파
exec cargo run \
    --manifest-path "${REPO_ROOT}/server/Cargo.toml" \
    --bin diagnose_search \
    -- "$@"
