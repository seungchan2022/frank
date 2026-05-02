//! `.env` / `.env.diagnose` 로딩 + 필수 변수 검증.
//!
//! SSOT 진단 흐름 §부트스트랩:
//! - 필수: `DATABASE_URL`, `DIAGNOSE_USER_ID`, `TAVILY_API_KEY`, `EXA_API_KEY`
//! - 운영 변수 오염 방지: `.env.diagnose` 우선 (없으면 `.env`)

use std::env;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum EnvError {
    #[error("필수 환경변수 누락: {0}")]
    Missing(String),
    #[error("환경변수 형식 오류: {key} (사유: {reason})")]
    Invalid { key: String, reason: String },
}

#[derive(Debug, Clone)]
pub struct DiagnoseEnv {
    pub database_url: String,
    pub diagnose_user_id: Uuid,
    pub tavily_api_key: String,
    pub exa_api_key: String,
}

/// `.env.diagnose` 우선, 없으면 `.env` 로드. 그 후 필수 변수 검증.
/// 호출자는 main 시작 시 1회 호출.
pub fn load() -> Result<DiagnoseEnv, EnvError> {
    // 우선순위: .env.diagnose → .env (먼저 로드된 게 우선이라 .diagnose 먼저)
    let _ = dotenvy::from_filename(".env.diagnose");
    let _ = dotenvy::dotenv();
    load_from_env()
}

/// 단위 테스트용 — `dotenvy` 호출 없이 현재 process env에서만 읽음.
pub fn load_from_env() -> Result<DiagnoseEnv, EnvError> {
    let database_url = require("DATABASE_URL")?;
    let user_raw = require("DIAGNOSE_USER_ID")?;
    let diagnose_user_id = Uuid::parse_str(&user_raw).map_err(|e| EnvError::Invalid {
        key: "DIAGNOSE_USER_ID".to_string(),
        reason: e.to_string(),
    })?;
    let tavily_api_key = require("TAVILY_API_KEY")?;
    let exa_api_key = require("EXA_API_KEY")?;
    Ok(DiagnoseEnv {
        database_url,
        diagnose_user_id,
        tavily_api_key,
        exa_api_key,
    })
}

fn require(key: &str) -> Result<String, EnvError> {
    match env::var(key) {
        Ok(v) if !v.trim().is_empty() => Ok(v),
        _ => Err(EnvError::Missing(key.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 테스트는 실제 env를 변경하므로 직렬 실행이 안전.
    /// 단일 테스트 안에서 모든 키를 set/unset 한 뒤 호출.
    fn with_env<F: FnOnce()>(pairs: &[(&str, &str)], f: F) {
        let prev: Vec<(String, Option<String>)> = pairs
            .iter()
            .map(|(k, _)| (k.to_string(), env::var(k).ok()))
            .collect();
        for (k, v) in pairs {
            // SAFETY: 테스트 단일 스레드 가정. set_var/remove_var는 unsafe (Rust 1.78+).
            // diagnose 바이너리 단위 테스트라 main과 겹치지 않음.
            unsafe { env::set_var(k, v) };
        }
        f();
        for (k, v) in prev {
            // SAFETY: 동일 사유.
            match v {
                Some(val) => unsafe { env::set_var(&k, val) },
                None => unsafe { env::remove_var(&k) },
            }
        }
    }

    /// 환경변수 변경은 process-global이라 cargo test의 기본 멀티스레드와 race한다.
    /// 본 모듈은 단일 `#[test]`로 시나리오 3개를 직렬 실행해 안정화.
    #[test]
    fn env_loader_scenarios() {
        // 1) 모두 존재 → Ok
        with_env(
            &[
                ("DATABASE_URL", "postgres://localhost/db"),
                ("DIAGNOSE_USER_ID", "00000000-0000-0000-0000-000000000001"),
                ("TAVILY_API_KEY", "tk-x"),
                ("EXA_API_KEY", "ek-x"),
            ],
            || {
                let env = load_from_env().expect("should load");
                assert_eq!(env.database_url, "postgres://localhost/db");
                assert_eq!(env.tavily_api_key, "tk-x");
                assert_eq!(env.exa_api_key, "ek-x");
            },
        );

        // 2) USER_ID 누락 → Missing
        with_env(
            &[
                ("DATABASE_URL", "postgres://x"),
                ("TAVILY_API_KEY", "t"),
                ("EXA_API_KEY", "e"),
            ],
            || {
                let prev = env::var("DIAGNOSE_USER_ID").ok();
                unsafe { env::remove_var("DIAGNOSE_USER_ID") };
                let err = load_from_env().unwrap_err();
                assert!(matches!(err, EnvError::Missing(ref k) if k == "DIAGNOSE_USER_ID"));
                if let Some(v) = prev {
                    unsafe { env::set_var("DIAGNOSE_USER_ID", v) };
                }
            },
        );

        // 3) USER_ID 형식 오류 → Invalid
        with_env(
            &[
                ("DATABASE_URL", "postgres://x"),
                ("DIAGNOSE_USER_ID", "not-a-uuid"),
                ("TAVILY_API_KEY", "t"),
                ("EXA_API_KEY", "e"),
            ],
            || {
                let err = load_from_env().unwrap_err();
                match err {
                    EnvError::Invalid { key, .. } => assert_eq!(key, "DIAGNOSE_USER_ID"),
                    _ => panic!("expected Invalid"),
                }
            },
        );
    }
}
