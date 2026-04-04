#!/usr/bin/env bash
set -euo pipefail

# Frank 배포 스크립트
# 사용법: ./scripts/deploy.sh [--tunnel]

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

# 색상 출력
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# .env 파일 존재 확인
check_env_files() {
    local missing=0
    if [ ! -f server/.env ]; then
        log_error "server/.env 파일이 없습니다."
        missing=1
    fi
    if [ ! -f web/.env ]; then
        log_error "web/.env 파일이 없습니다."
        missing=1
    fi
    if [ "$missing" -eq 1 ]; then
        log_error ".env 파일을 생성한 후 다시 실행하세요."
        exit 1
    fi
    log_info ".env 파일 확인 완료"
}

# Docker 실행 확인
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        log_error "Docker가 실행 중이 아닙니다. Docker Desktop을 시작하세요."
        exit 1
    fi
    log_info "Docker 실행 확인 완료"
}

# 빌드 + 실행
deploy() {
    log_info "Docker 이미지 빌드 시작..."
    # SvelteKit $env/static/public 빌드 시 주입
    set -a
    # shellcheck disable=SC1091
    . "$PROJECT_ROOT/web/.env"
    set +a
    docker compose build

    log_info "서비스 시작..."
    docker compose up -d

    log_info "헬스체크 대기 중..."
    local retries=0
    local max_retries=30
    while [ $retries -lt $max_retries ]; do
        if docker compose ps | grep -q "healthy"; then
            log_info "서비스 정상 기동 확인"
            docker compose ps
            return 0
        fi
        retries=$((retries + 1))
        sleep 2
    done

    log_warn "헬스체크 타임아웃 (${max_retries}회 시도). 상태 확인:"
    docker compose ps
}

# server/.env에서 IMESSAGE_RECIPIENT 읽기
load_imessage_recipient() {
    local env_file="$PROJECT_ROOT/server/.env"
    if [ -f "$env_file" ]; then
        IMESSAGE_RECIPIENT=$(grep -E '^IMESSAGE_RECIPIENT=' "$env_file" | cut -d'=' -f2- | tr -d '"' | tr -d "'")
    fi
}

# iMessage로 메시지 전송
send_imessage() {
    local message="$1"
    local recipient="${IMESSAGE_RECIPIENT:-}"

    if [ -z "$recipient" ]; then
        log_warn "IMESSAGE_RECIPIENT 미설정 — iMessage 전송 건너뜀"
        return 0
    fi

    # AppleScript injection 방지: 특수문자 이스케이프
    local escaped_message="${message//\\/\\\\}"
    escaped_message="${escaped_message//\"/\\\"}"
    local escaped_recipient="${recipient//\\/\\\\}"
    escaped_recipient="${escaped_recipient//\"/\\\"}"

    osascript -e "
tell application \"Messages\"
    set targetService to 1st account whose service type = iMessage
    set targetBuddy to participant \"${escaped_recipient}\" of targetService
    send \"${escaped_message}\" to targetBuddy
end tell" > /dev/null 2>&1 && log_info "iMessage 전송 완료 → $recipient" \
                           || log_warn "iMessage 전송 실패 (osascript 오류)"
}

# Cloudflare Tunnel 시작 (선택적)
start_tunnel() {
    if ! command -v cloudflared > /dev/null 2>&1; then
        log_error "cloudflared CLI가 설치되어 있지 않습니다."
        log_info "설치: brew install cloudflared"
        exit 1
    fi

    log_info "Cloudflare Quick Tunnel 시작 (web:3000)..."

    local tunnel_log
    tunnel_log=$(mktemp)

    # cloudflared를 백그라운드로 실행, stderr → 로그 파일
    cloudflared tunnel --url http://localhost:3000 2>"$tunnel_log" &
    local tunnel_pid=$!

    # 스크립트 종료 시 터널 프로세스 + 임시파일 정리
    trap "kill $tunnel_pid 2>/dev/null; rm -f '$tunnel_log'" EXIT INT TERM

    # URL이 출력될 때까지 대기 (최대 30초)
    local retries=0
    local max_retries=30
    local tunnel_url=""

    while [ $retries -lt $max_retries ]; do
        tunnel_url=$(grep -oE 'https://[a-zA-Z0-9-]+\.trycloudflare\.com' "$tunnel_log" 2>/dev/null | head -1 || true)
        if [ -n "$tunnel_url" ]; then
            break
        fi
        retries=$((retries + 1))
        sleep 1
    done

    if [ -z "$tunnel_url" ]; then
        log_error "터널 URL 감지 실패 (${max_retries}초 타임아웃)"
        log_warn "로그 확인: $tunnel_log"
        return 1
    fi

    log_info "터널 URL: $tunnel_url"

    # iMessage로 터널 URL 전송
    load_imessage_recipient
    send_imessage "[Frank] 배포 완료 — $tunnel_url"

    log_info "터널 실행 중 (Ctrl+C로 종료)"
    wait $tunnel_pid
}

# 메인
main() {
    local use_tunnel=false

    for arg in "$@"; do
        case "$arg" in
            --tunnel) use_tunnel=true ;;
            --help|-h)
                echo "사용법: $0 [--tunnel]"
                echo "  --tunnel  Cloudflare Quick Tunnel로 외부 공개"
                exit 0
                ;;
            *)
                log_error "알 수 없는 옵션: $arg"
                exit 1
                ;;
        esac
    done

    check_env_files
    check_docker
    deploy

    log_info "배포 완료!"
    log_info "서버: http://localhost:8080"
    log_info "웹:   http://localhost:3000"

    if [ "$use_tunnel" = true ]; then
        start_tunnel
    fi
}

main "$@"
