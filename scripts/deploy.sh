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

# Cloudflare Tunnel 시작 (선택적)
start_tunnel() {
    if ! command -v cloudflared > /dev/null 2>&1; then
        log_error "cloudflared CLI가 설치되어 있지 않습니다."
        log_info "설치: brew install cloudflared"
        exit 1
    fi

    log_info "Cloudflare Quick Tunnel 시작 (server:8080)..."
    log_info "터미널에 표시되는 *.trycloudflare.com URL로 외부 접속 가능"
    cloudflared tunnel --url http://localhost:8080
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

    if [ "$use_tunnel" = true ]; then
        start_tunnel
    fi

    log_info "배포 완료!"
    log_info "서버: http://localhost:8080"
    log_info "웹:   http://localhost:3000"
}

main "$@"
