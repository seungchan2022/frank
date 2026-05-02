//! 측정 루프. (태그 × 변형 × 엔진) 격자를 순차 실행하고
//! 셀 단위 append-flush로 `data.md.partial`에 누적 저장 후 main 종료 시 원자적 rename.
//!
//! 의존:
//! - `SearchPort` 어댑터(이름·실행) 한 쌍 — main에서 직접 인스턴스화 후 주입.
//! - DB는 raw row 조회 결과를 호출자가 미리 만들어 `ActiveTag` 리스트로 전달.
//!
//! 통합 테스트(T-05)에서는 mock `SearchPort` 두 개를 주입하면 end-to-end 검증 가능.

use std::path::PathBuf;

use chrono::Utc;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use server::domain::error::AppError;
#[cfg(test)]
use server::domain::models::SearchResult;
use server::domain::ports::SearchPort;

use super::mapping_resolver;
use super::markdown_formatter::{CellResult, FailureInfo, Meta, QuotaImpact, render};
use super::retry_classifier::{FailureKind, classify, is_retryable, judge_with_retry};
use super::variant_generator::{Variant, VariantKind, generate};

/// 활성 태그 한 개 — DB raw row를 호출자가 변환해서 전달.
#[derive(Debug, Clone)]
pub struct ActiveTag {
    pub korean_name: String,
    pub created_at: String,
}

/// 엔진 한 개의 사전 한도 잔여 (pre-flight stdin 입력값).
#[derive(Debug, Clone, Copy)]
pub struct EngineQuotaInput {
    pub remaining_before: u32,
}

/// 엔진별 실효 limit 상한 (사전 검증값). SSOT 측정 파라미터 §"limit (엔진별 실효 상한)".
#[derive(Debug, Clone, Copy)]
pub struct EngineLimits {
    pub tavily_effective: usize,
    pub exa_effective: usize,
}

/// 진단 1회 실행 파라미터.
pub struct RunParams<'a> {
    pub user_id: String,
    pub tags: Vec<ActiveTag>,
    pub limit_requested: usize,
    pub limits: EngineLimits,
    pub tavily: &'a dyn SearchPort,
    pub exa: &'a dyn SearchPort,
    pub tavily_quota: EngineQuotaInput,
    pub exa_quota: EngineQuotaInput,
    pub partial_path: PathBuf,
    pub final_path: PathBuf,
    pub engine_filter_notes: String,
    pub recall_threshold: usize, // SSOT: ≤3 → 추가 1회 호출
}

/// 측정 1회. 정상 종료 시 final_path 생성. panic/오류 시 partial_path 보존.
pub async fn run(params: RunParams<'_>) -> Result<(), AppError> {
    // E-01: 활성 태그 0개
    if params.tags.is_empty() {
        return Err(AppError::Internal("활성 태그 0개 — 진단 중단".to_string()));
    }
    // E-02: 영어 매핑 누락 감지 (사전 검증)
    for tag in &params.tags {
        mapping_resolver::resolve(&tag.korean_name)
            .map_err(|e| AppError::Internal(format!("E-02 매핑 누락: {e}")))?;
    }

    // SSOT "재실행 시 진행분 인지 후 재개 또는 사용자 결정" — 자동 삭제하지 않고
    // 기존 partial 발견 시 즉시 중단 ("사용자 결정" 분기). 사용자가 수동 확인 후
    // 삭제하거나 다른 경로로 옮긴 뒤 재실행.
    if params.partial_path.exists() {
        return Err(AppError::Internal(format!(
            "기존 partial 발견 — 이전 실행 진행분 가능성. 수동 확인 후 삭제/이동하고 재실행하세요: {}",
            params.partial_path.display()
        )));
    }
    let mut partial = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&params.partial_path)
        .await
        .map_err(|e| AppError::Internal(format!("R-06 partial open 실패: {e}")))?;
    partial
        .write_all(b"<!-- partial: in-progress -->\n")
        .await
        .map_err(|e| AppError::Internal(format!("R-06 partial 쓰기 실패: {e}")))?;

    let cells_capacity = params.tags.len() * VariantKind::all().len() * 2;
    let mut cells: Vec<CellResult> = Vec::with_capacity(cells_capacity);
    let mut tavily_calls: u32 = 0;
    let mut exa_calls: u32 = 0;

    for tag in &params.tags {
        let english = mapping_resolver::resolve(&tag.korean_name).expect("pre-validated above");
        let variants: Vec<Variant> = generate(&tag.korean_name, english);

        for variant in &variants {
            // Tavily
            let cell_t = measure_cell(
                &tag.korean_name,
                variant,
                "tavily",
                params.tavily,
                params.limit_requested,
                params.limits.tavily_effective,
                params.recall_threshold,
            )
            .await;
            tavily_calls += cell_t.api_calls;
            append_cell_line(&mut partial, &cell_t.cell).await?;
            cells.push(cell_t.cell);

            // Exa
            let cell_e = measure_cell(
                &tag.korean_name,
                variant,
                "exa",
                params.exa,
                params.limit_requested,
                params.limits.exa_effective,
                params.recall_threshold,
            )
            .await;
            exa_calls += cell_e.api_calls;
            append_cell_line(&mut partial, &cell_e.cell).await?;
            cells.push(cell_e.cell);
        }
    }

    partial
        .flush()
        .await
        .map_err(|e| AppError::Internal(format!("partial flush 실패: {e}")))?;
    drop(partial);

    let meta = Meta {
        measured_at: Utc::now(),
        user_id: params.user_id,
        tag_count: params.tags.len(),
        variant_count: VariantKind::all().len(),
        limit_requested: params.limit_requested,
        retry_policy: "5xx/timeout/network 1회 (양 엔진 대칭) / 200+빈결과·4xx 재시도 제외"
            .to_string(),
        overwrite_policy: "재실행 시 전체 덮어씀".to_string(),
        active_tags: params
            .tags
            .iter()
            .map(|t| (t.korean_name.clone(), t.created_at.clone()))
            .collect(),
        engine_filter_notes: params.engine_filter_notes,
    };
    let quota = vec![
        QuotaImpact {
            engine: "tavily".to_string(),
            remaining_before: params.tavily_quota.remaining_before,
            calls_used: tavily_calls,
            remaining_after_estimated: params
                .tavily_quota
                .remaining_before
                .saturating_sub(tavily_calls),
        },
        QuotaImpact {
            engine: "exa".to_string(),
            remaining_before: params.exa_quota.remaining_before,
            calls_used: exa_calls,
            remaining_after_estimated: params.exa_quota.remaining_before.saturating_sub(exa_calls),
        },
    ];
    let mapping = mapping_resolver::all_pairs();
    let body = render(&meta, &cells, &quota, &mapping);

    // 원자적 교체: tmp에 쓰고 final로 rename
    let tmp_path = params.final_path.with_extension("md.tmp");
    fs::write(&tmp_path, body.as_bytes())
        .await
        .map_err(|e| AppError::Internal(format!("final tmp 쓰기 실패: {e}")))?;
    fs::rename(&tmp_path, &params.final_path)
        .await
        .map_err(|e| AppError::Internal(format!("final rename 실패: {e}")))?;

    // partial 정리 (정상 완료)
    let _ = fs::remove_file(&params.partial_path).await;
    Ok(())
}

struct MeasuredCell {
    cell: CellResult,
    /// 이 셀에서 발생한 실제 API 호출 횟수 (재시도·재호출 합산).
    api_calls: u32,
}

async fn measure_cell(
    tag_korean: &str,
    variant: &Variant,
    engine_name: &str,
    port: &dyn SearchPort,
    requested: usize,
    effective: usize,
    recall_threshold: usize,
) -> MeasuredCell {
    let (outcome, calls_first) = call_with_retry(port, &variant.query, effective).await;
    let mut total_calls = calls_first;

    let cell = match outcome {
        CallOutcome::FirstOk(returned_n) => {
            let recall_n = run_recall(
                port,
                &variant.query,
                effective,
                returned_n,
                recall_threshold,
            )
            .await;
            if recall_n.is_some() || returned_n <= recall_threshold {
                // SSOT High-1: API 호출이 발생했으면 (성공이든 실패든) 한도 카운트에 반영.
                // recall_returned가 None이라도 호출 자체는 시도됐을 수 있음.
                // 정확성을 위해 run_recall이 호출했는지 별도 신호 받음.
                // 단순화: 임계 이하면 1회 호출 시도했다고 가정.
                if returned_n <= recall_threshold {
                    total_calls += 1;
                }
            }
            CellResult {
                tag_korean: tag_korean.to_string(),
                variant: variant.kind,
                engine: engine_name.to_string(),
                query: variant.query.clone(),
                requested,
                effective,
                returned: Some(returned_n),
                recall_returned: recall_n,
                failure: None,
                recovered_after_retry: None,
            }
        }
        CallOutcome::RecoveredAfterRetry {
            returned_n,
            first_failure,
        } => CellResult {
            tag_korean: tag_korean.to_string(),
            variant: variant.kind,
            engine: engine_name.to_string(),
            query: variant.query.clone(),
            requested,
            effective,
            returned: Some(returned_n),
            recall_returned: None, // 회복 셀은 recall 호출 안 함 (이미 2회 호출 사용)
            failure: None,
            recovered_after_retry: Some(first_failure),
        },
        CallOutcome::Failed(info) => CellResult {
            tag_korean: tag_korean.to_string(),
            variant: variant.kind,
            engine: engine_name.to_string(),
            query: variant.query.clone(),
            requested,
            effective,
            returned: None,
            recall_returned: None,
            failure: Some(info),
            recovered_after_retry: None,
        },
    };

    MeasuredCell {
        cell,
        api_calls: total_calls,
    }
}

/// recall 추가 호출. 임계 이하면 1회 시도 (재시도 없음 — SSOT 변동성 검증 단발).
/// 실패하면 None 반환. 호출 자체는 항상 발생.
async fn run_recall(
    port: &dyn SearchPort,
    query: &str,
    effective: usize,
    returned_n: usize,
    threshold: usize,
) -> Option<usize> {
    if returned_n > threshold {
        return None;
    }
    port.search(query, effective).await.ok().map(|v| v.len())
}

/// 1차 호출 결과. 성공/회복/실패 3가지로 명시 분기.
enum CallOutcome {
    FirstOk(usize),
    /// 1차 실패 → 2차 재시도 성공 (간헐). 1차 정보 보존 (DoD #2 narrative).
    RecoveredAfterRetry {
        returned_n: usize,
        first_failure: FailureInfo,
    },
    Failed(FailureInfo),
}

/// 1회 호출 + 재시도 정책(5xx/timeout/network 1회). 반환: (결과, 실제 API 호출 수).
async fn call_with_retry(port: &dyn SearchPort, query: &str, limit: usize) -> (CallOutcome, u32) {
    let first_res = port.search(query, limit).await;
    match first_res {
        Ok(items) => (CallOutcome::FirstOk(items.len()), 1),
        Err(e) => {
            let msg1 = format!("{e:?}");
            let cat1 = classify(&msg1);
            if !is_retryable(cat1) {
                let info = FailureInfo {
                    first_category: cat1,
                    first_message: msg1,
                    second_category: None,
                    second_message: None,
                    kind: judge_with_retry(cat1, None),
                };
                return (CallOutcome::Failed(info), 1);
            }
            // 재시도
            let second = port.search(query, limit).await;
            match second {
                Ok(items) => {
                    // 재시도 성공 → 회복 (Intermittent). 1차 정보 narrative 보존.
                    let info = FailureInfo {
                        first_category: cat1,
                        first_message: msg1,
                        second_category: None,
                        second_message: None,
                        kind: FailureKind::Intermittent,
                    };
                    (
                        CallOutcome::RecoveredAfterRetry {
                            returned_n: items.len(),
                            first_failure: info,
                        },
                        2,
                    )
                }
                Err(e2) => {
                    let msg2 = format!("{e2:?}");
                    let cat2 = classify(&msg2);
                    let info = FailureInfo {
                        first_category: cat1,
                        first_message: msg1,
                        second_category: Some(cat2),
                        second_message: Some(msg2),
                        kind: judge_with_retry(cat1, Some(Err(cat2))),
                    };
                    (CallOutcome::Failed(info), 2)
                }
            }
        }
    }
}

async fn append_cell_line(
    partial: &mut tokio::fs::File,
    cell: &CellResult,
) -> Result<(), AppError> {
    let line = format!(
        "{} | {} | {} | requested={} effective={} returned={} recall={}\n",
        cell.tag_korean,
        cell.variant.as_label(),
        cell.engine,
        cell.requested,
        cell.effective,
        cell.returned
            .map(|n| n.to_string())
            .unwrap_or_else(|| "FAIL".to_string()),
        cell.recall_returned
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".to_string()),
    );
    partial
        .write_all(line.as_bytes())
        .await
        .map_err(|e| AppError::Internal(format!("R-06 append 실패: {e}")))?;
    partial
        .flush()
        .await
        .map_err(|e| AppError::Internal(format!("R-06 flush 실패: {e}")))?;
    Ok(())
}

// =============== 테스트: T-05 mock SearchPort 통합 ===============

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::tempdir;

    /// 테스트 전용 partial 경로 헬퍼.
    fn partial_path_default(final_path: &Path) -> PathBuf {
        final_path.with_extension("md.partial")
    }

    /// 결정적 mock SearchPort: 호출 횟수 카운트 + 반환 결과 수 가변.
    struct MockSearch {
        name: &'static str,
        /// 각 호출마다 반환할 결과 수 (꼬리에서 pop). 비어 있으면 빈 결과 0.
        responses: std::sync::Mutex<Vec<usize>>,
        calls: AtomicUsize,
    }

    impl MockSearch {
        fn new(name: &'static str, responses: Vec<usize>) -> Self {
            Self {
                name,
                responses: std::sync::Mutex::new(responses),
                calls: AtomicUsize::new(0),
            }
        }
    }

    impl SearchPort for MockSearch {
        fn search(
            &self,
            _query: &str,
            _max_results: usize,
        ) -> Pin<
            Box<dyn std::future::Future<Output = Result<Vec<SearchResult>, AppError>> + Send + '_>,
        > {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let n = self.responses.lock().expect("mutex").pop().unwrap_or(0);
            Box::pin(async move {
                let items: Vec<SearchResult> = (0..n)
                    .map(|i| SearchResult {
                        title: format!("t-{i}"),
                        url: format!("https://x/{i}"),
                        snippet: None,
                        published_at: None,
                        image_url: None,
                    })
                    .collect();
                Ok(items)
            })
        }

        fn source_name(&self) -> &str {
            self.name
        }
    }

    /// 테스트용 RunParams 빌더 — 기본값을 채워 테스트 본문이 변하는 항목만 명시하게.
    fn make_params<'a>(
        tags: Vec<ActiveTag>,
        tavily: &'a dyn SearchPort,
        exa: &'a dyn SearchPort,
        partial_path: PathBuf,
        final_path: PathBuf,
    ) -> RunParams<'a> {
        RunParams {
            user_id: "u".to_string(),
            tags,
            limit_requested: 100,
            limits: EngineLimits {
                tavily_effective: 20,
                exa_effective: 100,
            },
            tavily,
            exa,
            tavily_quota: EngineQuotaInput {
                remaining_before: 1000,
            },
            exa_quota: EngineQuotaInput {
                remaining_before: 1000,
            },
            partial_path,
            final_path,
            engine_filter_notes: "n".to_string(),
            recall_threshold: 3,
        }
    }

    #[tokio::test]
    async fn end_to_end_two_tags_writes_data_md() {
        let tmp = tempdir().expect("tempdir");
        let final_path = tmp.path().join("M1_diagnosis_data.md");
        let partial_path = partial_path_default(&final_path);

        // 2 태그 × 5 변형 × 2 엔진 = 20 호출 (recall 없도록 모두 10 반환)
        let tavily = MockSearch::new("tavily", vec![10; 100]);
        let exa = MockSearch::new("exa", vec![10; 100]);

        let mut params = make_params(
            vec![
                ActiveTag {
                    korean_name: "AI/ML".to_string(),
                    created_at: "2026-04-01T00:00:00Z".to_string(),
                },
                ActiveTag {
                    korean_name: "보안".to_string(),
                    created_at: "2026-04-02T00:00:00Z".to_string(),
                },
            ],
            &tavily,
            &exa,
            partial_path.clone(),
            final_path.clone(),
        );
        params.user_id = "00000000-0000-0000-0000-000000000001".to_string();
        params.engine_filter_notes =
            "Tavily: time_range=week / Exa: startPublishedDate=now-7d".to_string();
        run(params).await.expect("run ok");

        // partial은 정리되었어야 함, final이 존재해야 함
        assert!(!partial_path.exists());
        let body = fs::read_to_string(&final_path).await.expect("read final");
        assert!(body.contains("AUTO-GENERATED"));
        assert!(body.contains("AI/ML"));
        assert!(body.contains("보안"));
        // 측정 row 수: 2 × 5 × 2 = 20
        let row_count = body.matches("baseline").count()
            + body.matches("simple").count()
            + body.matches("year2026").count()
            + body.matches("announcement").count()
            + body.matches("english_baseline").count();
        // 표 헤더의 1회 + 매핑 표 일부 fragment 영향 가능 → 최소 검증
        assert!(row_count >= 20);
    }

    #[tokio::test]
    async fn empty_tags_returns_error() {
        let tmp = tempdir().unwrap();
        let final_path = tmp.path().join("data.md");
        let partial_path = partial_path_default(&final_path);
        let tavily = MockSearch::new("tavily", vec![]);
        let exa = MockSearch::new("exa", vec![]);
        let res = run(make_params(vec![], &tavily, &exa, partial_path, final_path)).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn unknown_tag_aborts_before_calls() {
        let tmp = tempdir().unwrap();
        let final_path = tmp.path().join("data.md");
        let partial_path = partial_path_default(&final_path);
        let tavily = MockSearch::new("tavily", vec![]);
        let exa = MockSearch::new("exa", vec![]);
        let res = run(make_params(
            vec![ActiveTag {
                korean_name: "신규알수없는태그".to_string(),
                created_at: "2026-05-01T00:00:00Z".to_string(),
            }],
            &tavily,
            &exa,
            partial_path,
            final_path,
        ))
        .await;
        assert!(res.is_err());
        assert_eq!(tavily.calls.load(Ordering::SeqCst), 0);
        assert_eq!(exa.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn small_result_triggers_recall() {
        let tmp = tempdir().unwrap();
        let final_path = tmp.path().join("data.md");
        let partial_path = partial_path_default(&final_path);
        // tavily: 모든 호출 1개 반환 → 매 셀마다 recall 1회 추가
        let tavily = MockSearch::new("tavily", vec![1; 100]);
        // exa: 충분히 많이 반환 → recall 없음
        let exa = MockSearch::new("exa", vec![50; 100]);

        let params = make_params(
            vec![ActiveTag {
                korean_name: "AI/ML".to_string(),
                created_at: "2026-04-01T00:00:00Z".to_string(),
            }],
            &tavily,
            &exa,
            partial_path,
            final_path.clone(),
        );
        run(params).await.unwrap();

        // tavily: 5 변형 × (1차 1 + recall 1) = 10
        assert_eq!(tavily.calls.load(Ordering::SeqCst), 10);
        // exa: 5 변형 × 1 = 5 (recall 미발생)
        assert_eq!(exa.calls.load(Ordering::SeqCst), 5);

        let body = fs::read_to_string(&final_path).await.unwrap();
        // recall 컬럼에 "1" 존재 (tavily 셀들)
        assert!(body.contains(" 1 |"));
    }

    /// 재시도 후 성공: tavily 1차 5xx → 2차 성공. 셀은 성공으로 기록.
    #[tokio::test]
    async fn retry_succeeds_for_5xx_then_ok() {
        // 별도 mock: 첫 호출은 Err, 이후는 Ok(5)
        struct FlakyTavily {
            calls: AtomicUsize,
        }
        impl SearchPort for FlakyTavily {
            fn search(
                &self,
                _q: &str,
                _m: usize,
            ) -> Pin<
                Box<
                    dyn std::future::Future<Output = Result<Vec<SearchResult>, AppError>>
                        + Send
                        + '_,
                >,
            > {
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if n == 0 {
                        Err(AppError::Internal(
                            "Tavily returned 503: Service Unavailable".to_string(),
                        ))
                    } else {
                        Ok(vec![
                            SearchResult {
                                title: "t".to_string(),
                                url: "https://x".to_string(),
                                snippet: None,
                                published_at: None,
                                image_url: None,
                            };
                            5
                        ])
                    }
                })
            }
            fn source_name(&self) -> &str {
                "tavily"
            }
        }

        let tmp = tempdir().unwrap();
        let final_path = tmp.path().join("data.md");
        let partial_path = partial_path_default(&final_path);

        let tavily = FlakyTavily {
            calls: AtomicUsize::new(0),
        };
        // exa 5 변형 모두 Ok(10)
        let exa = MockSearch::new("exa", vec![10; 5]);

        let params = make_params(
            vec![ActiveTag {
                korean_name: "AI/ML".to_string(),
                created_at: "2026-04-01T00:00:00Z".to_string(),
            }],
            &tavily,
            &exa,
            partial_path,
            final_path.clone(),
        );
        run(params).await.unwrap();
        // tavily: 1차(503) + 2차(ok=5) = 2 호출 — 변형 1개. 그 후 변형 2~5 각 1 호출 → 총 1+1+1+1+1+1=6? 변형 1만 재시도, 변형 2~5는 1차 ok.
        // 실제 mock은 매 호출 (n>=1) ok이므로 변형 1만 2호출, 변형 2~5는 각 1호출 → 2 + 4 = 6
        assert_eq!(tavily.calls.load(Ordering::SeqCst), 6);

        let body = fs::read_to_string(&final_path).await.unwrap();
        assert!(body.contains("AI/ML"));
        // High-2 회복 셀이 실패 로그에 narrative 보존 (Intermittent + 회복? yes)
        assert!(body.contains("Intermittent"));
        assert!(body.contains("yes"));
    }

    /// High-3: 기존 partial 발견 시 자동 삭제 금지 — 즉시 중단.
    #[tokio::test]
    async fn existing_partial_aborts_with_clear_message() {
        let tmp = tempdir().unwrap();
        let final_path = tmp.path().join("data.md");
        let partial_path = partial_path_default(&final_path);
        // 기존 partial 시뮬레이션
        fs::write(&partial_path, b"<!-- old partial -->\n")
            .await
            .unwrap();

        let tavily = MockSearch::new("tavily", vec![10; 100]);
        let exa = MockSearch::new("exa", vec![10; 100]);
        let params = make_params(
            vec![ActiveTag {
                korean_name: "AI/ML".to_string(),
                created_at: "2026-04-01T00:00:00Z".to_string(),
            }],
            &tavily,
            &exa,
            partial_path.clone(),
            final_path,
        );
        let res = run(params).await;
        let err = res.expect_err("should abort on existing partial");
        let msg = format!("{err}");
        assert!(
            msg.contains("기존 partial 발견"),
            "expected abort message, got: {msg}"
        );
        // 호출 0회 보장 (즉시 중단)
        assert_eq!(tavily.calls.load(Ordering::SeqCst), 0);
        // 기존 partial은 그대로 남아 있어야 함 (자동 삭제 금지)
        assert!(partial_path.exists());
    }

    /// 재시도 비대상(401) — 즉시 실패 카운트.
    #[tokio::test]
    async fn auth_failure_records_permanent_no_retry() {
        struct UnauthTavily;
        impl SearchPort for UnauthTavily {
            fn search(
                &self,
                _q: &str,
                _m: usize,
            ) -> Pin<
                Box<
                    dyn std::future::Future<Output = Result<Vec<SearchResult>, AppError>>
                        + Send
                        + '_,
                >,
            > {
                Box::pin(async {
                    Err(AppError::Internal(
                        "Tavily returned 401 Unauthorized".to_string(),
                    ))
                })
            }
            fn source_name(&self) -> &str {
                "tavily"
            }
        }
        let tmp = tempdir().unwrap();
        let final_path = tmp.path().join("data.md");
        let partial_path = partial_path_default(&final_path);
        let tavily = UnauthTavily;
        let exa = MockSearch::new("exa", vec![10; 5]);
        let params = make_params(
            vec![ActiveTag {
                korean_name: "AI/ML".to_string(),
                created_at: "2026-04-01T00:00:00Z".to_string(),
            }],
            &tavily,
            &exa,
            partial_path,
            final_path.clone(),
        );
        run(params).await.unwrap();
        let body = fs::read_to_string(&final_path).await.unwrap();
        assert!(body.contains("PermanentNoRetry"));
        assert!(body.contains("auth"));
    }
}
