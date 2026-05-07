<script lang="ts">
	import type { WrongAnswer } from '$lib/types/quiz';

	interface Props {
		item: WrongAnswer;
		onDelete: (id: string) => void;
	}

	let { item, onDelete }: Props = $props();

	const correctOption = $derived(item.options[item.correctIndex]);
	const userOption = $derived(item.options[item.userIndex]);

	/** http/https scheme만 허용 — XSS(javascript:) 및 data: URL 방지 */
	const safeArticleUrl = $derived.by(() => {
		if (!item.articleUrl) return null;
		try {
			const url = new URL(item.articleUrl);
			return url.protocol === 'http:' || url.protocol === 'https:' ? item.articleUrl : null;
		} catch {
			return null;
		}
	});
</script>

<div class="rounded-lg border border-red-100 bg-white p-4 shadow-sm">
	<!-- 기사 제목 -->
	<p class="mb-2 text-xs font-medium text-gray-400 line-clamp-1">{item.articleTitle}</p>

	<!-- 문제 -->
	<p class="mb-3 text-sm font-semibold text-gray-900 leading-snug">{item.question}</p>

	<!-- 내 답 vs 정답 -->
	<div class="mb-3 space-y-1">
		<div class="flex items-start gap-2 text-xs">
			<span class="flex-shrink-0 rounded bg-red-100 px-1.5 py-0.5 font-medium text-red-700">내 답</span>
			<span class="text-gray-600">{userOption}</span>
		</div>
		<div class="flex items-start gap-2 text-xs">
			<span class="flex-shrink-0 rounded bg-green-100 px-1.5 py-0.5 font-medium text-green-700">정답</span>
			<span class="text-gray-600">{correctOption}</span>
		</div>
	</div>

	<!-- 해설 -->
	{#if item.explanation}
		<p class="mb-3 text-xs text-gray-500 leading-relaxed">{item.explanation}</p>
	{/if}

	<!-- 원문 보기 + 삭제 버튼 -->
	<!-- safeArticleUrl: http/https scheme 검증 통과한 URL만 링크 표시 (XSS 방지) -->
	<div class="flex items-center justify-between">
		{#if safeArticleUrl}
		<a
			href={safeArticleUrl}
			target="_blank"
			rel="noopener noreferrer"
			class="flex items-center gap-1 text-xs text-blue-500 hover:text-blue-700 transition-colors"
		>
			<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
				<path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" />
				<path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" />
			</svg>
			원문 보기
		</a>
		{/if}
		<button
			onclick={() => onDelete(item.id)}
			class="text-xs text-gray-400 hover:text-red-500 transition-colors"
		>
			삭제
		</button>
	</div>
</div>
