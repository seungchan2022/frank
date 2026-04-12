<script lang="ts">
	import type { QuizQuestion } from '$lib/types/quiz';

	interface Props {
		questions: QuizQuestion[];
		onClose: () => void;
	}

	let { questions, onClose }: Props = $props();

	let currentIndex = $state(0);
	let selectedIndex = $state<number | null>(null);
	let confirmed = $state(false);
	let score = $state(0);
	let finished = $state(false);

	const currentQuestion = $derived(questions[currentIndex]);
	const isCorrect = $derived(
		selectedIndex !== null && selectedIndex === currentQuestion?.answer_index
	);

	function selectOption(index: number) {
		if (confirmed) return; // 확인 후에는 변경 불가
		selectedIndex = index;
	}

	function confirm() {
		if (selectedIndex === null || confirmed) return;
		if (selectedIndex === currentQuestion.answer_index) {
			score += 1;
		}
		confirmed = true;
	}

	function nextQuestion() {
		if (currentIndex + 1 >= questions.length) {
			finished = true;
		} else {
			currentIndex += 1;
			selectedIndex = null;
			confirmed = false;
		}
	}
</script>

<!-- 모달 오버레이 -->
<div
	class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
	role="dialog"
	aria-modal="true"
	aria-label="퀴즈"
>
	<div class="w-full max-w-lg rounded-xl bg-white shadow-xl">
		{#if finished}
			<!-- 최종 점수 화면 -->
			<div class="p-6 text-center">
				<div class="mb-4 text-5xl">
					{#if score === questions.length}
						🎉
					{:else if score >= questions.length / 2}
						👍
					{:else}
						📚
					{/if}
				</div>
				<h2 class="mb-2 text-xl font-bold text-gray-900">퀴즈 완료!</h2>
				<p class="mb-6 text-3xl font-bold text-indigo-600">
					{score} / {questions.length}
				</p>
				<p class="mb-6 text-sm text-gray-500">
					{#if score === questions.length}
						완벽합니다! 기사 내용을 완전히 이해했습니다.
					{:else if score >= questions.length / 2}
						잘 이해했습니다. 틀린 문제를 다시 확인해보세요.
					{:else}
						기사를 다시 읽고 복습해보세요.
					{/if}
				</p>
				<button
					onclick={onClose}
					class="w-full rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700"
				>
					닫기
				</button>
			</div>
		{:else}
			<!-- 문제 화면 -->
			<div class="p-6">
				<!-- 헤더 -->
				<div class="mb-4 flex items-center justify-between">
					<span class="text-sm font-medium text-gray-500">
						문제 {currentIndex + 1} / {questions.length}
					</span>
					<div class="h-1.5 flex-1 mx-4 rounded-full bg-gray-100">
						<div
							class="h-1.5 rounded-full bg-indigo-500 transition-all"
							style="width: {((currentIndex + 1) / questions.length) * 100}%"
						></div>
					</div>
					<button
						onclick={onClose}
						aria-label="닫기"
						class="text-gray-400 hover:text-gray-600"
					>
						✕
					</button>
				</div>

				<!-- 질문 -->
				<h3 class="mb-5 text-base font-semibold text-gray-900 leading-snug">
					{currentQuestion.question}
				</h3>

				<!-- 보기 -->
				<ul class="space-y-2 mb-4">
					{#each currentQuestion.options as option, i (i)}
						<li>
							<button
								onclick={() => selectOption(i)}
								disabled={confirmed}
								class={[
									'w-full rounded-lg border px-4 py-3 text-left text-sm transition-colors',
									!confirmed && selectedIndex === null
										? 'border-gray-200 bg-white hover:border-indigo-300 hover:bg-indigo-50'
										: !confirmed && selectedIndex === i
											? 'border-indigo-400 bg-indigo-50 text-indigo-800'
											: confirmed
												? selectedIndex === i
													? i === currentQuestion.answer_index
														? 'border-green-400 bg-green-50 text-green-800'
														: 'border-red-400 bg-red-50 text-red-800'
													: i === currentQuestion.answer_index
														? 'border-green-400 bg-green-50 text-green-800'
														: 'border-gray-200 bg-white text-gray-400'
												: 'border-gray-200 bg-white'
								].join(' ')}
							>
								<span class="font-medium mr-2">{String.fromCharCode(65 + i)}.</span>
								{option}
							</button>
						</li>
					{/each}
				</ul>

				<!-- 확인 버튼 (선택 후, 확인 전) -->
				{#if selectedIndex !== null && !confirmed}
					<button
						onclick={confirm}
						class="w-full rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700"
					>
						확인
					</button>
				{/if}

				<!-- 해설 + 다음 버튼 (확인 후 표시) -->
				{#if confirmed}
					<div
						class={[
							'rounded-lg border p-3 mb-4 text-sm',
							isCorrect
								? 'border-green-200 bg-green-50 text-green-800'
								: 'border-red-200 bg-red-50 text-red-800'
						].join(' ')}
					>
						<span class="font-semibold">{isCorrect ? '✓ 정답!' : '✗ 오답'}</span>
						<p class="mt-1 text-gray-700">{currentQuestion.explanation}</p>
					</div>

					<button
						onclick={nextQuestion}
						class="w-full rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700"
					>
						{currentIndex + 1 >= questions.length ? '결과 보기' : '다음 문제'}
					</button>
				{/if}
			</div>
		{/if}
	</div>
</div>
