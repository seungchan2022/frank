<script lang="ts">
	import type { WrongAnswer } from '$lib/types/quiz';

	interface Props {
		item: WrongAnswer;
		onDelete: (id: string) => void;
	}

	let { item, onDelete }: Props = $props();

	const correctOption = $derived(item.options[item.correctIndex]);
	const userOption = $derived(item.options[item.userIndex]);
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

	<!-- 삭제 버튼 -->
	<div class="flex justify-end">
		<button
			onclick={() => onDelete(item.id)}
			class="text-xs text-gray-400 hover:text-red-500 transition-colors"
		>
			삭제
		</button>
	</div>
</div>
