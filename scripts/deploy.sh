#!/usr/bin/env bash
# Frank 통합 실행 스크립트
#
# 사용법:
#   ./scripts/deploy.sh                       # ios + api + front 모두 실행
#   ./scripts/deploy.sh --target=api,front    # api, front만 실행
#   ./scripts/deploy.sh --target=ios          # iOS 시뮬레이터만 실행
#   ./scripts/deploy.sh --target=api --tunnel # api 실행 + Cloudflare 터널
#   ./scripts/deploy.sh --help
#
# 타겟:
#   ios    - iOS 시뮬레이터 빌드 + 런치
#   front  - 웹 프론트엔드 (Docker, port 3000)
#   api    - Rust API 서버 (Docker, port 8080)

set -euo pipefail

# ─── 상수 ────────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Docker 연결 실패 시 네이티브 모드 fallback 플래그
USE_NATIVE=false

IOS_WORKSPACE="$PROJECT_ROOT/ios/Frank/Frank.xcworkspace"
IOS_SCHEME="Frank"
IOS_BUNDLE_ID="dev.frank.app"
IOS_DERIVED_DATA="/tmp/FrankBuild"
IOS_SIMULATOR_NAME="${IOS_SIMULATOR:-iPhone 17 Pro}"

API_PORT=8080
FRONT_PORT=3000

# ─── CORS / CSRF 허용 오리진 ──────────────────────────────────────────────────
# 새 오리진 추가 시 이 배열에만 추가하면 됩니다.
# ALLOWED_ORIGINS(API CORS) + ORIGIN(웹 CSRF) 모두 여기서 관리합니다.
LOCAL_ORIGINS=(
    "http://localhost:${FRONT_PORT}"   # 웹 프론트 (Docker)
    "http://localhost:5173"            # 웹 프론트 (dev)
    "http://localhost:4173"            # 웹 프론트 (preview)
    "http://127.0.0.1:5173"            # 웹 프론트 (dev, IP)
)

# ─── 색상 출력 ────────────────────────────────────────────────────────────────
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

log_info()    { echo -e "${GREEN}[INFO]${NC}  $1"; }
log_warn()    { echo -e "${YELLOW}[WARN]${NC}  $1"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $1"; }
log_section() { echo -e "\n${CYAN}${BOLD}==> $1${NC}"; }

# ─── 공통 유틸 ───────────────────────────────────────────────────────────────

# 로컬 포트를 사용 중인 프로세스를 킬한다 (Docker 외 네이티브 프로세스용)
kill_port() {
    local port=$1
    local pids
    pids=$(lsof -ti tcp:"$port" 2>/dev/null || true)
    if [ -n "$pids" ]; then
        log_warn "포트 $port 사용 중인 프로세스 종료: PID $pids"
        # shellcheck disable=SC2086
        kill -9 $pids 2>/dev/null || true
    fi
}

# ─── 사전 검증 ───────────────────────────────────────────────────────────────

check_env_files() {
    if [ ! -f "$PROJECT_ROOT/server/.env" ]; then
        log_error "server/.env 파일이 없습니다."
        exit 1
    fi
    log_info ".env 파일 확인 완료 (server/.env)"
}

# Supabase 대시보드로 계정을 만들면 token 컬럼이 NULL로 초기화돼
# 로그인 시 "Database error querying schema" 에러가 발생한다.
# 이 함수는 그 사실을 경고하는 안내만 출력한다 (자동 수정은 불가 — Supabase auth 내부 테이블).
warn_supabase_dashboard_accounts() {
    log_warn "테스트 계정 주의: Supabase 대시보드에서 직접 생성한 계정은"
    log_warn "  로그인 시 'Database error querying schema' 에러가 발생할 수 있습니다."
    log_warn "  → 앱의 이메일 회원가입 플로우로 계정을 만들거나,"
    log_warn "  → 대시보드 계정은 Supabase SQL Editor에서 다음 쿼리를 실행하세요:"
    log_warn "  UPDATE auth.users"
    log_warn "    SET confirmation_token = COALESCE(NULLIF(confirmation_token,''),'')"
    log_warn "      , recovery_token = COALESCE(NULLIF(recovery_token,''),'')"
    log_warn "      , email_change_token_new = COALESCE(NULLIF(email_change_token_new,''),'')"
    log_warn "      , email_change = COALESCE(NULLIF(email_change,''),'')"
    log_warn "      , reauthentication_token = COALESCE(NULLIF(reauthentication_token,''),'')"
    log_warn "    WHERE email = '<대상_이메일>';"
}

check_docker() {
    docker info > /dev/null 2>&1
}

# Docker 사용 불가 시 네이티브 모드로 실행할지 한 번만 묻는다.
ensure_docker_or_native() {
    if check_docker; then
        return 0
    fi

    # 이미 네이티브 모드로 결정된 경우 재질문 없이 통과
    if [ "$USE_NATIVE" = true ]; then
        return 0
    fi

    log_error "Docker daemon에 연결할 수 없습니다."
    echo ""
    echo -e "${YELLOW}  원인 및 해결 방법:${NC}"
    echo -e "  • Docker Desktop이 아직 시작 중이라면 잠시 기다린 후 다시 실행하세요."
    echo -e "  • Docker Desktop 업데이트 후 처음 실패한 경우:"
    echo -e "    Docker Desktop 메뉴바 고래 아이콘 → Troubleshoot → Reset to factory defaults"
    echo -e "    (VM 상태를 초기화합니다. 로컬 이미지는 삭제되지만 재빌드로 복구 가능)"
    echo ""
    echo -e "${CYAN}  ── 네이티브 모드 fallback ──────────────────────────────────────${NC}"
    echo -e "  Docker 없이도 로컬 테스트가 가능합니다."
    echo -e "  아래 두 프로세스를 백그라운드로 직접 실행합니다:"
    echo -e "    • API 서버: cargo run  → http://localhost:${API_PORT}"
    echo -e "    • 웹 프론트: npm run dev → http://localhost:5173"
    echo -e "  (Docker 모드와 동작은 동일하지만 포트가 다를 수 있습니다)"
    echo ""
    echo -e "  입력 방법: ${BOLD}y${NC} 또는 ${BOLD}yes${NC} → 네이티브 모드 실행"
    echo -e "             ${BOLD}n${NC} 또는 Enter  → 실행 취소 (Docker 문제 해결 후 재시도)"
    echo ""
    read -r -p "$(echo -e "${YELLOW}[?]${NC} 네이티브 모드로 테스트를 진행하겠습니까? [y/N]: ")" answer
    case "$answer" in
        [yY][eE][sS]|[yY])
            USE_NATIVE=true
            log_info "네이티브 모드로 전환합니다."
            ;;
        *)
            log_error "실행을 취소합니다. Docker 문제 해결 후 다시 시도하세요."
            exit 1
            ;;
    esac
}

check_xcode() {
    if ! command -v xcodebuild > /dev/null 2>&1; then
        log_error "xcodebuild를 찾을 수 없습니다. Xcode를 설치하세요."
        exit 1
    fi
    if [ ! -d "$IOS_WORKSPACE" ]; then
        log_error "Xcode workspace 없음: $IOS_WORKSPACE"
        log_warn "먼저 'cd ios/Frank && tuist generate --no-open' 을 실행하세요."
        exit 1
    fi
    log_info "Xcode 환경 확인 완료"
}

# ios/Frank/Config.xcconfig 자동 생성.
# server/.env에서 SUPABASE_URL / SUPABASE_ANON_KEY 를 읽어 xcconfig 형식으로 변환.
# xcconfig 파서는 "//" 를 주석으로 처리하므로 $(SLASH)$(SLASH) 트릭을 사용한다.
# Config.xcconfig가 이미 존재하면 덮어쓰지 않는다 (커스텀 설정 보존).
generate_ios_xcconfig() {
    local xcconfig_path="$PROJECT_ROOT/ios/Frank/Config.xcconfig"

    if [ -f "$xcconfig_path" ]; then
        log_info "Config.xcconfig 이미 존재 — 건너뜀"
        return 0
    fi

    local env_file="$PROJECT_ROOT/server/.env"
    if [ ! -f "$env_file" ]; then
        log_error "server/.env 없음 — Config.xcconfig 자동 생성 불가"
        log_warn "ios/Frank/Config.xcconfig 를 수동으로 생성하세요."
        exit 1
    fi

    local supabase_url supabase_anon_key
    supabase_url=$(grep '^SUPABASE_URL=' "$env_file" | cut -d'=' -f2-)
    supabase_anon_key=$(grep '^SUPABASE_ANON_KEY=' "$env_file" | cut -d'=' -f2-)

    if [ -z "$supabase_url" ] || [ -z "$supabase_anon_key" ]; then
        log_error "server/.env에 SUPABASE_URL 또는 SUPABASE_ANON_KEY 없음"
        exit 1
    fi

    # scheme과 host 분리 (https://host → scheme=https, host=host)
    local supabase_scheme supabase_host server_scheme server_host
    supabase_scheme="${supabase_url%%://*}"
    supabase_host="${supabase_url#*://}"
    server_scheme="http"
    server_host="localhost:8080"

    log_info "Config.xcconfig 자동 생성 중: $xcconfig_path"

    # 헤더 (단일 따옴표 heredoc → 변수 전개 없음)
    cat > "$xcconfig_path" << 'HEADER'
// Auto-generated by scripts/deploy.sh — DO NOT commit (git-ignored)
//
// xcconfig 파서는 변수 치환 전에 "//" 를 주석으로 처리한다.
// SLASH = / 를 정의하고 $(SLASH)$(SLASH) 로 "//" 를 구성해서 우회한다.
SLASH = /
HEADER

    # URL 라인은 printf로 작성 — $(SLASH) 를 bash가 평가하지 않도록 리터럴 전달
    printf 'SUPABASE_URL = %s:$(SLASH)$(SLASH)%s\n' \
        "$supabase_scheme" "$supabase_host" >> "$xcconfig_path"
    printf 'SUPABASE_ANON_KEY = %s\n' \
        "$supabase_anon_key" >> "$xcconfig_path"
    printf 'SERVER_URL = %s:$(SLASH)$(SLASH)%s\n' \
        "$server_scheme" "$server_host" >> "$xcconfig_path"

    log_info "Config.xcconfig 생성 완료"
}

# ─── 타겟별 실행 ─────────────────────────────────────────────────────────────

run_ios() {
    log_section "iOS 시뮬레이터 실행"
    check_env_files
    check_xcode

    # Config.xcconfig 없으면 server/.env에서 자동 생성 (SUPABASE_URL 주입)
    generate_ios_xcconfig

    # 신규 Swift 파일 추가 시 pbxproj 자동 갱신 (항상 실행)
    log_info "Tuist 프로젝트 재생성 중..."
    (cd "$PROJECT_ROOT/ios/Frank" && ~/.tuist/Versions/4.31.0/tuist generate --no-open)

    # 이미 부팅된 동일 이름 시뮬레이터 UDID 우선 사용
    # (같은 이름이 여러 개 있을 때 잘못된 기기를 선택하는 문제 방지)
    local sim_udid=""
    sim_udid=$(xcrun simctl list devices --json 2>/dev/null \
        | python3 -c "
import sys, json
data = json.load(sys.stdin)
name = '${IOS_SIMULATOR_NAME}'
for runtime, devices in data.get('devices', {}).items():
    for d in devices:
        if d.get('name') == name and d.get('state') == 'Booted':
            print(d['udid'])
            raise SystemExit(0)
" 2>/dev/null || true)

    if [ -z "$sim_udid" ]; then
        # 부팅된 시뮬레이터 없으면 첫 번째 일치하는 기기 부팅
        sim_udid=$(xcrun simctl list devices --json 2>/dev/null \
            | python3 -c "
import sys, json
data = json.load(sys.stdin)
name = '${IOS_SIMULATOR_NAME}'
for runtime, devices in data.get('devices', {}).items():
    for d in devices:
        if d.get('name') == name:
            print(d['udid'])
            raise SystemExit(0)
" 2>/dev/null || true)
        if [ -z "$sim_udid" ]; then
            log_error "시뮬레이터를 찾을 수 없습니다: $IOS_SIMULATOR_NAME"
            exit 1
        fi
        log_info "시뮬레이터 부팅: $IOS_SIMULATOR_NAME ($sim_udid)"
        xcrun simctl boot "$sim_udid" 2>/dev/null || true
        open -a Simulator
    else
        log_info "부팅된 시뮬레이터 사용: $IOS_SIMULATOR_NAME ($sim_udid)"
        open -a Simulator
    fi

    log_info "빌드 중..."
    xcodebuild build \
        -workspace "$IOS_WORKSPACE" \
        -scheme "$IOS_SCHEME" \
        -destination "platform=iOS Simulator,id=$sim_udid" \
        -derivedDataPath "$IOS_DERIVED_DATA" \
        -quiet

    log_info "앱 설치 및 런치..."
    xcrun simctl install "$sim_udid" \
        "$IOS_DERIVED_DATA/Build/Products/Debug-iphonesimulator/Frank.app"
    xcrun simctl launch "$sim_udid" "$IOS_BUNDLE_ID"

    log_info "Frank 앱 실행 완료 ($IOS_SIMULATOR_NAME / $sim_udid)"
}

# Docker 서비스(api/front)를 중지·제거하고 포트를 해제한 뒤 재빌드·실행한다.
# $1: compose service 이름 (server | web)
# $2: 노출 포트 번호
run_docker_service() {
    local service="$1"
    local port="$2"

    log_section "Docker 서비스 재배포: $service (port $port)"
    cd "$PROJECT_ROOT"

    # 1) 기존 컨테이너 정지 + 제거 (포트 확보)
    if docker compose ps --services --filter "status=running" 2>/dev/null | grep -q "^${service}$"; then
        log_info "기존 컨테이너 중지: $service"
        docker compose stop "$service"
    fi
    docker compose rm -f "$service" 2>/dev/null || true
    # stale 컨테이너 ID 잔재 제거 (Docker Desktop 재시동 후 발생 가능)
    docker container prune -f > /dev/null 2>&1 || true

    # 2) 로컬 포트에 남은 프로세스도 정리
    kill_port "$port"

    # 3) 빌드 — server/.env에서 Supabase 공개 키 주입 (web 빌드 arg 포함)
    log_info "이미지 빌드: $service"
    set -a
    # shellcheck disable=SC1091
    . "$PROJECT_ROOT/server/.env"
    export PUBLIC_SUPABASE_URL="${SUPABASE_URL:-}"
    export PUBLIC_SUPABASE_ANON_KEY="${SUPABASE_ANON_KEY:-}"
    # CORS / CSRF 오리진 조립
    export ALLOWED_ORIGINS
    ALLOWED_ORIGINS="$(IFS=','; echo "${LOCAL_ORIGINS[*]}")"
    export ORIGIN="http://localhost:${FRONT_PORT}"
    set +a
    docker compose build "$service"

    # 4) 실행
    log_info "컨테이너 시작: $service"
    docker compose up -d "$service"

    log_info "$service 배포 완료 → http://localhost:$port"
}

run_api_native() {
    log_section "API 서버 네이티브 실행 (cargo run)"
    kill_port "$API_PORT"

    set -a
    # shellcheck disable=SC1091
    . "$PROJECT_ROOT/server/.env"
    export ALLOWED_ORIGINS
    ALLOWED_ORIGINS="$(IFS=','; echo "${LOCAL_ORIGINS[*]}")"
    set +a

    (cd "$PROJECT_ROOT/server" && cargo run) &
    log_info "API 서버 시작 대기 중 (최대 30초)..."
    local i
    for i in $(seq 1 30); do
        if curl -sf "http://localhost:${API_PORT}/health" > /dev/null 2>&1; then
            log_info "API 서버 준비 완료 → http://localhost:${API_PORT}"
            return 0
        fi
        sleep 1
    done
    log_warn "API 서버 헬스체크 타임아웃. 아직 컴파일 중일 수 있습니다."
}

run_front_native() {
    log_section "웹 프론트 네이티브 실행 (npm run dev)"
    kill_port 5173
    FRONT_PORT=5173

    (cd "$PROJECT_ROOT/web" && npm run dev) &
    sleep 2
    log_info "웹 프론트 시작 중 → http://localhost:5173"
    open "http://localhost:5173"
}

run_api() {
    check_env_files
    ensure_docker_or_native
    if [ "$USE_NATIVE" = true ]; then
        run_api_native
    else
        run_docker_service "server" "$API_PORT"
    fi
}

run_front() {
    check_env_files
    ensure_docker_or_native
    if [ "$USE_NATIVE" = true ]; then
        run_front_native
    else
        run_docker_service "web" "$FRONT_PORT"
        open "http://localhost:$FRONT_PORT"
    fi
}

# ─── Cloudflare 터널 ──────────────────────────────────────────────────────────

load_imessage_recipient() {
    local env_file="$PROJECT_ROOT/server/.env"
    if [ -f "$env_file" ]; then
        IMESSAGE_RECIPIENT=$(grep -E '^IMESSAGE_RECIPIENT=' "$env_file" \
            | cut -d'=' -f2- | tr -d '"' | tr -d "'" || true)
    fi
}

send_imessage() {
    local message="$1"
    local recipient="${IMESSAGE_RECIPIENT:-}"
    [ -z "$recipient" ] && { log_warn "IMESSAGE_RECIPIENT 미설정 — iMessage 전송 건너뜀"; return 0; }

    local esc_msg="${message//\\/\\\\}"; esc_msg="${esc_msg//\"/\\\"}"
    local esc_rec="${recipient//\\/\\\\}"; esc_rec="${esc_rec//\"/\\\"}"

    osascript -e "
tell application \"Messages\"
    set s to 1st account whose service type = iMessage
    set b to participant \"${esc_rec}\" of s
    send \"${esc_msg}\" to b
end tell" > /dev/null 2>&1 \
        && log_info "iMessage 전송 완료 → $recipient" \
        || log_warn "iMessage 전송 실패"
}

start_tunnel() {
    if ! command -v cloudflared > /dev/null 2>&1; then
        log_error "cloudflared CLI 없음. 설치: brew install cloudflared"
        exit 1
    fi

    log_info "Cloudflare Quick Tunnel 시작 (web:$FRONT_PORT)..."
    local tunnel_log; tunnel_log=$(mktemp)

    cloudflared tunnel --url "http://localhost:$FRONT_PORT" 2>"$tunnel_log" &
    local tunnel_pid=$!
    trap "kill $tunnel_pid 2>/dev/null; rm -f '$tunnel_log'" EXIT INT TERM

    local retries=0 tunnel_url=""
    while [ $retries -lt 30 ]; do
        tunnel_url=$(grep -oE 'https://[a-zA-Z0-9-]+\.trycloudflare\.com' \
            "$tunnel_log" 2>/dev/null | head -1 || true)
        [ -n "$tunnel_url" ] && break
        retries=$((retries + 1))
        sleep 1
    done

    if [ -z "$tunnel_url" ]; then
        log_error "터널 URL 감지 실패 (30초 타임아웃). 로그: $tunnel_log"
        return 1
    fi

    log_info "터널 URL: $tunnel_url"
    load_imessage_recipient
    send_imessage "[Frank] 배포 완료 — $tunnel_url"
    log_info "터널 실행 중 (Ctrl+C로 종료)"
    wait $tunnel_pid
}

# ─── 메인 ─────────────────────────────────────────────────────────────────────

usage() {
    cat <<EOF
사용법: $0 [--target=<타겟[,타겟...]>] [--tunnel] [--simulator=<이름>]

타겟 (기본값: ios,api,front 모두):
  ios    iOS 시뮬레이터 빌드 + 런치
  api    Rust API 서버  (Docker, port $API_PORT)
  front  웹 프론트엔드  (Docker, port $FRONT_PORT)

옵션:
  --target=ios,front    지정한 타겟만 실행 (,로 복수 지정)
  --tunnel              Cloudflare Quick Tunnel 시작 (front 포함 시 유효)
  --simulator=<이름>    iOS 시뮬레이터 이름 (기본: $IOS_SIMULATOR_NAME)
  --help, -h            이 도움말 출력

Docker 연결 실패 시:
  자동으로 네이티브 모드 실행 여부를 묻습니다.
  네이티브 모드: API=cargo run (:$API_PORT), 웹=npm run dev (:5173)

예시:
  $0                          # 전체 실행
  $0 --target=api,front       # 백엔드+프론트만
  $0 --target=ios             # iOS만
  $0 --target=api --tunnel    # API + 터널
EOF
}

main() {
    local targets_raw=""
    local use_tunnel=false

    for arg in "$@"; do
        case "$arg" in
            --target=*)  targets_raw="${arg#--target=}" ;;
            --simulator=*) IOS_SIMULATOR_NAME="${arg#--simulator=}" ;;
            --tunnel)    use_tunnel=true ;;
            --help|-h)   usage; exit 0 ;;
            *)
                log_error "알 수 없는 옵션: $arg"
                usage
                exit 1
                ;;
        esac
    done

    # 타겟 파싱 (기본값: 전체)
    local run_ios=false run_api=false run_front=false

    if [ -z "$targets_raw" ]; then
        run_ios=true; run_api=true; run_front=true
    else
        IFS=',' read -ra targets <<< "$targets_raw"
        for t in "${targets[@]}"; do
            case "$t" in
                ios)   run_ios=true ;;
                api)   run_api=true ;;
                front) run_front=true ;;
                *)
                    log_error "알 수 없는 타겟: '$t' (ios | api | front 중 하나)"
                    exit 1
                    ;;
            esac
        done
    fi

    # Supabase 대시보드 계정 관련 경고 (항상 출력)
    warn_supabase_dashboard_accounts

    # 실행
    $run_ios   && run_ios
    $run_api   && run_api
    $run_front && run_front

    # 요약
    log_section "실행 완료"
    $run_api   && log_info "API 서버: http://localhost:$API_PORT"
    $run_front && log_info "웹 프론트: http://localhost:$FRONT_PORT"
    $run_ios   && log_info "iOS: $IOS_SIMULATOR_NAME 시뮬레이터"

    # 터널은 front가 포함된 경우만 유효
    if [ "$use_tunnel" = true ]; then
        if [ "$run_front" = false ]; then
            log_warn "--tunnel은 front 타겟이 포함된 경우에만 유효합니다. 건너뜁니다."
        else
            start_tunnel
        fi
    fi
}

main "$@"
