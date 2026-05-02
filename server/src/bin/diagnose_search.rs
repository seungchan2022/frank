//! MVP15 M1 검색엔진 진단 바이너리.
//!
//! 책임 (얇은 wiring):
//! 1. `.env` / `.env.diagnose` 로드 + 필수 변수 검증
//! 2. DB 연결 후 활성 태그 + created_at 조회 (raw SQL — 운영 service 우회)
//! 3. pre-flight confirm (TTY · stdin · 한도 잔여 입력)
//! 4. Tavily/Exa 어댑터 직접 인스턴스화 (공용 factory 미경유)
//! 5. `runner::run` 위임
//!
//! SSOT: `progress/mvp15/M1_search_diagnosis.md`.
//! 운영 서버와 격리: `feed_cache` / `SearchFallbackChain` import 안 함.

use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use server::domain::error::AppError;
use server::infra::exa::ExaAdapter;
use server::infra::tavily::TavilyAdapter;
use sqlx::Row;
use sqlx::postgres::PgPoolOptions;

#[path = "diagnose/mod.rs"]
mod diagnose;

use diagnose::env_loader;
use diagnose::runner::{ActiveTag, EngineLimits, EngineQuotaInput, RunParams};

const PARTIAL_REL: &str = "progress/mvp15/M1_diagnosis_data.md.partial";
const FINAL_REL: &str = "progress/mvp15/M1_diagnosis_data.md";
// Tavily 공식 max_results 상한 — 사전 검증한 실효값. 운영 limit과 분리.
// 출처: Tavily API doc (2026-05 기준 max_results=20 상한). 변경 시 SSOT 업데이트.
const TAVILY_EFFECTIVE: usize = 20;
const EXA_EFFECTIVE: usize = 100;
const LIMIT_REQUESTED: usize = 100;
const RECALL_THRESHOLD: usize = 3;
const ENGINE_FILTER_NOTES: &str =
    "Tavily: time_range=week / Exa: startPublishedDate=now-7d (SSOT: 어댑터 운영 코드 직사용)";

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => {
            eprintln!("✅ 진단 완료 → {FINAL_REL}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("❌ 진단 중단: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 1. env
    let env = env_loader::load()?;

    // 2. DB 연결 + 활성 태그 + created_at
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&env.database_url)
        .await
        .map_err(|e| AppError::Internal(format!("R-05 DB 연결 실패: {e}")))?;

    let rows = sqlx::query(
        r#"
        SELECT t.name AS name, ut.created_at AS created_at
        FROM user_tags ut
        JOIN tags t ON t.id = ut.tag_id
        WHERE ut.user_id = $1
        ORDER BY ut.created_at NULLS LAST, t.name
        "#,
    )
    .bind(env.diagnose_user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Internal(format!("R-05 활성 태그 조회 실패: {e}")))?;

    let tags: Vec<ActiveTag> = rows
        .iter()
        .map(|r| {
            let name: String = r.try_get("name").unwrap_or_default();
            // created_at 컬럼 없을 수 있음 — Option 핸들링
            let created_at: Option<chrono::DateTime<chrono::Utc>> =
                r.try_get("created_at").ok().flatten();
            ActiveTag {
                korean_name: name,
                created_at: created_at
                    .map(|d| d.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
                    .unwrap_or_else(|| "unknown".to_string()),
            }
        })
        .collect();

    if tags.is_empty() {
        return Err("E-01 활성 태그 0개".into());
    }

    // 3. pre-flight confirm
    let stdin = io::stdin();
    if !stdin.is_terminal() {
        return Err("E-04 stdin이 TTY 아님 (CI/파이프) — 자동 중단".into());
    }
    eprintln!("=== MVP15 M1 검색엔진 진단 ===");
    eprintln!("user_id: {}", env.diagnose_user_id);
    eprintln!("활성 태그 ({}개):", tags.len());
    for t in &tags {
        eprintln!("  - {} (created_at: {})", t.korean_name, t.created_at);
    }
    let estimated_calls = tags.len() * 5 * 2; // 정상 — 재시도/recall 제외
    let estimated_max = estimated_calls + 30; // 최악치 추정 (재시도 + recall)
    eprintln!(
        "예상 호출 수: {} (정상) ~ {} (최악) — 엔진별 1/2",
        estimated_calls, estimated_max
    );

    let tavily_remaining = read_quota_input("Tavily 사전 한도 잔여 (대시보드 확인)")?;
    let exa_remaining = read_quota_input("Exa 사전 한도 잔여 (대시보드 확인)")?;

    eprint!("진행하시겠습니까? [y/yes/Y]: ");
    io::stderr().flush().ok();
    let mut buf = String::new();
    stdin.lock().read_line(&mut buf)?;
    let trimmed = buf.trim();
    if !matches!(trimmed, "y" | "yes" | "Y") {
        return Err(format!("E-03 사용자 중단 (입력: {trimmed:?})").into());
    }

    // 4. 어댑터 직접 인스턴스화 (공용 factory · feed_cache 미경유)
    let tavily = TavilyAdapter::new(&env.tavily_api_key);
    let exa = ExaAdapter::new(&env.exa_api_key);

    // 5. 경로 구성 (CWD = 저장소 루트 가정)
    let final_path = PathBuf::from(FINAL_REL);
    let partial_path = PathBuf::from(PARTIAL_REL);
    if let Some(parent) = final_path.parent()
        && !parent.exists()
    {
        return Err(format!(
            "출력 디렉토리 없음: {} (CWD={:?})",
            parent.display(),
            std::env::current_dir().ok()
        )
        .into());
    }

    // 6. runner 호출
    let params = RunParams {
        user_id: env.diagnose_user_id.to_string(),
        tags,
        limit_requested: LIMIT_REQUESTED,
        limits: EngineLimits {
            tavily_effective: TAVILY_EFFECTIVE,
            exa_effective: EXA_EFFECTIVE,
        },
        tavily: &tavily,
        exa: &exa,
        tavily_quota: EngineQuotaInput {
            remaining_before: tavily_remaining,
        },
        exa_quota: EngineQuotaInput {
            remaining_before: exa_remaining,
        },
        partial_path,
        final_path,
        engine_filter_notes: ENGINE_FILTER_NOTES.to_string(),
        recall_threshold: RECALL_THRESHOLD,
    };
    diagnose::runner::run(params).await?;

    Ok(())
}

/// stdin 정수 입력 (0~10,000 범위). E-05 sanity check.
fn read_quota_input(label: &str) -> Result<u32, Box<dyn std::error::Error>> {
    eprint!("{label} (정수 0~10000): ");
    io::stderr().flush().ok();
    let mut buf = String::new();
    io::stdin().lock().read_line(&mut buf)?;
    let n: u32 = buf
        .trim()
        .parse()
        .map_err(|e| format!("E-05 한도 입력 형식 오류: {e}"))?;
    if n > 10_000 {
        return Err("E-05 한도 입력 범위 초과 (>10000)".into());
    }
    Ok(n)
}
