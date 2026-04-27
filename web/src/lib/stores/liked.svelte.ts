/**
 * MVP7 M2: 좋아요 상태 관리 store.
 * Svelte 5 $state 기반 — Set 참조 교체 패턴 필수.
 *
 * likedUrls: 세션 동안 좋아요한 기사 URL 집합.
 * 서버 DB 기준이 아닌 클라이언트 세션 기준 (이벤트 누적 모델).
 */

export interface LikeArticleInput {
	url: string;
	title: string;
	snippet: string | null;
	tag_id?: string | null;
}

export interface LikeArticleResult {
	keywords: string[];
	total_likes: number;
}

/**
 * 팩토리 함수로 스토어 인스턴스 생성.
 * 테스트에서 독립 인스턴스를 생성할 수 있도록 팩토리 패턴 사용.
 */
export function createLikedStore() {
	// Svelte 5 $state — 참조 비교로 반응성 동작
	let likedUrls = $state<Set<string>>(new Set());

	/**
	 * 기사 좋아요 토글.
	 * - 이미 좋아요한 url이면 UI에서만 취소 (서버 unlike API 없음 — 이벤트 누적 모델).
	 * - 새 좋아요는 즉시 UI 반영 후 백그라운드 API 호출.
	 */
	function likeArticle(input: LikeArticleInput): void {
		if (likedUrls.has(input.url)) {
			// 토글 취소 — 클라이언트 상태만 변경 (서버 unlike API 미지원)
			likedUrls = new Set([...likedUrls].filter((u) => u !== input.url));
			return;
		}

		// 즉시 UI 반영 (fire-and-forget)
		likedUrls = new Set([...likedUrls, input.url]);

		// 백그라운드 API 호출 — 응답 기다리지 않음
		fetch('/api/articles/like', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				title: input.title,
				snippet: input.snippet ?? null,
				tag_id: input.tag_id ?? null
			})
		}).catch(() => {
			// 실패해도 UI 롤백 없음 — 이벤트 누적 모델
		});
	}

	/**
	 * 좋아요 여부 확인.
	 */
	function isLiked(url: string): boolean {
		return likedUrls.has(url);
	}

	/**
	 * 상태 초기화 (로그아웃/세션 전환 시).
	 */
	function reset(): void {
		likedUrls = new Set();
	}

	return {
		get likedUrls() {
			return likedUrls;
		},
		isLiked,
		likeArticle,
		reset
	};
}

// 싱글톤 인스턴스 — 앱 전역 공유
export const likedStore = createLikedStore();
