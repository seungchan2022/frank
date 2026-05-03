/// POST /me/summarize 응답 타입
export interface SummaryResult {
	summary: string;
	/// MVP15 M3: occupation 있으면 직업 시각 인사이트, 없으면 null.
	insight: string | null;
}

/// POST /me/rewrite 응답 타입
export interface RewriteResult {
	rewrite: string;
}
