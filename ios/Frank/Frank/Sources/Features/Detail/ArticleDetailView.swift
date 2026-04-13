import SwiftUI

/// MVP5 M3: ArticleDetailView — 온디맨드 요약 + 즐겨찾기 토글 UI.
/// MVP7 M2: LikesFeature 주입 — 헤더 하트 버튼.
/// MVP7 M4: QuizFeature 주입 — 즐겨찾기 기사에서 퀴즈 버튼 표시.
/// MVP8 M2: 연관 기사 섹션 제거 (RelatedFeature 제거). QuizPort 기본값 제거 — AppDependencies 통해 주입.
/// - 요약하기 버튼: idle/loading/done/failed 상태에 따라 UI 전환
/// - 즐겨찾기 버튼: isLiked 상태에 따라 채워진/빈 하트 아이콘
/// - 좋아요 버튼: LikesFeature 공유 (FeedView와 상태 동기화)
/// - 퀴즈 버튼: 즐겨찾기 상태일 때만 표시, QuizFeature로 관리
struct ArticleDetailView: View {
    let feedItem: FeedItem
    let favoritesFeature: FavoritesFeature
    let likesFeature: LikesFeature
    private let summarizePort: any SummarizePort
    /// MVP9 M2: 오답 시트 로드용으로 보관
    private let wrongAnswerPort: any WrongAnswerPort
    @State private var feature: ArticleDetailFeature
    @State private var quizFeature: QuizFeature
    @State private var favoriteLoading: Bool = false
    @State private var showQuiz = false
    @State private var showSafari = false
    /// MVP9 M2: 오답 보기 시트 표시 여부
    @State private var showWrongAnswerSheet = false
    /// MVP9 M2: 시트에 표시할 오답 목록
    @State private var sheetWrongAnswers: [WrongAnswer] = []
    /// MVP9 M2: 오답 시트 로딩 중
    @State private var sheetLoading = false

    init(
        feedItem: FeedItem,
        summarize: any SummarizePort,
        favoritesFeature: FavoritesFeature,
        likesFeature: LikesFeature,
        quiz: any QuizPort,
        wrongAnswer: any WrongAnswerPort = MockWrongAnswerAdapter(),
        favorites: any FavoritesPort = MockFavoritesAdapter()
    ) {
        self.feedItem = feedItem
        self.favoritesFeature = favoritesFeature
        self.likesFeature = likesFeature
        self.summarizePort = summarize
        self.wrongAnswerPort = wrongAnswer
        self._feature = State(initialValue: ArticleDetailFeature(feedItem: feedItem, summarize: summarize))
        self._quizFeature = State(initialValue: QuizFeature(
            quiz: quiz,
            wrongAnswer: wrongAnswer,
            favorites: favorites
        ))
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                headerSection
                Divider()
                snippetSection
                if let errMsg = favoritesFeature.operationError {
                    Text(errMsg)
                        .font(.footnote)
                        .foregroundStyle(.red)
                        .padding(.horizontal, 4)
                        .onTapGesture { favoritesFeature.clearOperationError() }
                }
                actionButtons
                summarySection
            }
            .padding(.horizontal, 20)
            .padding(.vertical, 16)
        }
        .navigationBarTitleDisplayMode(.inline)
        .sheet(isPresented: $showSafari) {
            SafariView(url: feedItem.url)
        }
        .sheet(isPresented: $showQuiz) {
            QuizView(
                questions: quizFeature.questions,
                onClose: {
                    showQuiz = false
                    quizFeature.reset()
                },
                onWrongAnswer: { question, userIndex in
                    quizFeature.saveWrongAnswer(question: question, userIndex: userIndex)
                },
                onQuizCompleted: {
                    quizFeature.markQuizCompleted()
                }
            )
        }
        .sheet(isPresented: $showWrongAnswerSheet) {
            NavigationStack {
                Group {
                    if sheetLoading {
                        ProgressView()
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                    } else if sheetWrongAnswers.isEmpty {
                        Text("이 기사의 오답 기록이 없어요")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                    } else {
                        List(sheetWrongAnswers) { item in
                            WrongAnswerRow(item: item)
                        }
                        .listStyle(.plain)
                    }
                }
                .navigationTitle("오답 보기")
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button("닫기") {
                            showWrongAnswerSheet = false
                        }
                    }
                }
            }
        }
        .onChange(of: quizFeature.phase) { _, newPhase in
            if case .done = newPhase {
                showQuiz = true
            }
        }
    }
}

// MARK: - Header

extension ArticleDetailView {
    private var headerSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            // 제목 + 좋아요 버튼 행
            HStack(alignment: .top, spacing: 8) {
                Text(feedItem.title)
                    .font(.title2)
                    .fontWeight(.bold)
                    .frame(maxWidth: .infinity, alignment: .leading)

                // 좋아요 하트 버튼
                Button {
                    Task { await likesFeature.like(feedItem: feedItem) }
                } label: {
                    Image(
                        systemName: likesFeature.isLiked(feedItem.url.absoluteString) ? "heart.fill" : "heart"
                    )
                    .foregroundStyle(
                        likesFeature.isLiked(feedItem.url.absoluteString) ? .red : .secondary
                    )
                    .font(.system(size: 22))
                }
                .accessibilityLabel(
                    likesFeature.isLiked(feedItem.url.absoluteString) ? "좋아요 완료" : "좋아요"
                )
            }

            HStack(spacing: 4) {
                Text(feedItem.source)
                Text("\u{00B7}")
                Text(ArticleCardView.relativeTimeText(feedItem.publishedAt))
            }
            .font(.caption)
            .foregroundStyle(.secondary)
        }
    }
}

// MARK: - Snippet

extension ArticleDetailView {
    @ViewBuilder
    private var snippetSection: some View {
        if let snippet = feedItem.snippet {
            VStack(alignment: .leading, spacing: 8) {
                Text("기사 소개")
                    .font(.subheadline)
                    .fontWeight(.bold)
                    .foregroundStyle(.secondary)

                Text(snippet)
            }

            Divider()
        }
    }
}

// MARK: - Action Buttons

extension ArticleDetailView {
    private var actionButtons: some View {
        VStack(spacing: 10) {
            // 원문 보기
            Button {
                showSafari = true
            } label: {
                HStack {
                    Image(systemName: "safari")
                    Text("원문 보기")
                }
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.bordered)

            // 요약하기
            summarizeButton

            // 즐겨찾기 토글 버튼
            favoriteButton

            // 퀴즈 버튼 (즐겨찾기 상태일 때만 표시)
            if favoritesFeature.isLiked(feedItem.url.absoluteString) {
                quizButton
            }
        }
    }

    @ViewBuilder
    private var quizButton: some View {
        // MVP9 M2: quizCompleted 상태일 때 "다시 풀기" + "오답 보기" 분리 버튼
        if favoritesFeature.isQuizCompleted(feedItem.url.absoluteString) {
            completedQuizButtons
        } else {
            notCompletedQuizButtons
        }
    }

    @ViewBuilder
    private var completedQuizButtons: some View {
        HStack(spacing: 10) {
            Button {
                Task {
                    await quizFeature.generateQuiz(url: feedItem.url.absoluteString, title: feedItem.title)
                }
            } label: {
                HStack {
                    Image(systemName: "arrow.clockwise")
                    Text("다시 풀기")
                }
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.bordered)
            .tint(.indigo)

            Button {
                Task { await loadWrongAnswerSheet() }
            } label: {
                HStack {
                    Image(systemName: "list.bullet.rectangle")
                    Text("오답 보기")
                }
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.bordered)
            .tint(.orange)
        }
    }

    @ViewBuilder
    private var notCompletedQuizButtons: some View {
        switch quizFeature.phase {
        case .idle, .done:
            Button {
                Task {
                    await quizFeature.generateQuiz(url: feedItem.url.absoluteString, title: feedItem.title)
                }
            } label: {
                HStack {
                    Image(systemName: "brain.head.profile")
                    Text("퀴즈 풀기")
                }
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.bordered)
            .tint(.indigo)

        case .loading:
            LoadingTextView(initial: "퀴즈 생성 중…", after: "마무리 중…")

        case .failed(let message):
            VStack(spacing: 6) {
                Text(message)
                    .font(.caption)
                    .foregroundStyle(.red)
                Button {
                    Task {
                        await quizFeature.generateQuiz(url: feedItem.url.absoluteString, title: feedItem.title)
                    }
                } label: {
                    HStack {
                        Image(systemName: "arrow.clockwise")
                        Text("다시 시도")
                    }
                    .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .tint(.orange)
            }
        }
    }

    private func loadWrongAnswerSheet() async {
        sheetLoading = true
        showWrongAnswerSheet = true
        let all = (try? await wrongAnswerPort.list()) ?? []
        sheetWrongAnswers = all.filter { $0.articleUrl == feedItem.url.absoluteString }
        sheetLoading = false
    }

    @ViewBuilder
    private var favoriteButton: some View {
        let isLiked = favoritesFeature.isLiked(feedItem.url.absoluteString)
        Button {
            guard !favoriteLoading else { return }
            Task {
                favoriteLoading = true
                defer { favoriteLoading = false }
                if isLiked {
                    await favoritesFeature.removeFavorite(url: feedItem.url.absoluteString)
                } else {
                    // step-5 K: phase.done에서 summary/insight 꺼내 전달
                    let summary = feature.phase.summaryResult?.summary
                    let insight = feature.phase.summaryResult?.insight
                    await favoritesFeature.addFavorite(
                        feedItem: feedItem,
                        summary: summary,
                        insight: insight
                    )
                }
            }
        } label: {
            HStack {
                Image(systemName: isLiked ? "star.fill" : "star")
                    .foregroundStyle(isLiked ? .yellow : .primary)
                Text(isLiked ? "즐겨찾기 해제" : "즐겨찾기 추가")
            }
            .frame(maxWidth: .infinity)
        }
        .buttonStyle(.bordered)
        .disabled(favoriteLoading)
        .opacity(favoriteLoading ? 0.5 : 1.0)
    }

    @ViewBuilder
    private var summarizeButton: some View {
        switch feature.phase {
        case .idle:
            Button {
                Task { await feature.loadSummary() }
            } label: {
                HStack {
                    Image(systemName: "sparkles")
                    Text("요약하기")
                }
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)

        case .loading:
            LoadingTextView(initial: "요약 중…", after: "마무리 중…")

        case .done:
            Button {
                // 이미 done 상태 — 재요약 불필요
            } label: {
                HStack {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundStyle(.green)
                    Text("요약 완료")
                }
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.bordered)
            .disabled(true)

        case .failed:
            Button {
                Task { await feature.loadSummary() }
            } label: {
                HStack {
                    Image(systemName: "arrow.clockwise")
                    Text("다시 시도")
                }
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)
            .tint(.orange)
        }
    }
}

// MARK: - Summary Section

extension ArticleDetailView {
    @ViewBuilder
    private var summarySection: some View {
        switch feature.phase {
        case .done(let result):
            VStack(alignment: .leading, spacing: 20) {
                Divider()

                Text("AI 요약 및 인사이트")
                    .font(.headline)
                    .fontWeight(.semibold)

                VStack(alignment: .leading, spacing: 10) {
                    Text("요약")
                        .font(.caption)
                        .fontWeight(.semibold)
                        .foregroundStyle(.secondary)
                        .kerning(1.2)
                        .textCase(.uppercase)

                    paragraphView(result.summary)
                }

                Divider()

                VStack(alignment: .leading, spacing: 10) {
                    Text("인사이트")
                        .font(.caption)
                        .fontWeight(.semibold)
                        .foregroundStyle(.secondary)
                        .kerning(1.2)
                        .textCase(.uppercase)

                    paragraphView(result.insight, secondary: true)
                }
            }

        case .failed(let message):
            Text(message)
                .font(.caption)
                .foregroundStyle(.red)
                .padding(.top, 4)

        default:
            EmptyView()
        }
    }
}

// MARK: - Markdown Helper

/// 마크다운 텍스트를 AttributedString으로 렌더링.
/// 파싱 실패 시 plain text로 fallback.
private func markdownText(_ text: String) -> Text {
    if let attributed = try? AttributedString(
        markdown: text,
        options: .init(interpretedSyntax: .full)
    ) {
        return Text(attributed)
    }
    return Text(text)
}

/// 줄 단위로 분리해 VStack으로 렌더링 — 문단 간격 표현.
@ViewBuilder
private func paragraphView(_ text: String, secondary: Bool = false) -> some View {
    let lines = text
        .components(separatedBy: "\n")
        .map { $0.trimmingCharacters(in: .whitespaces) }
        .filter { !$0.isEmpty }

    VStack(alignment: .leading, spacing: 12) {
        ForEach(lines, id: \.self) { line in
            markdownText(line)
                .font(.body)
                .lineSpacing(5)
                .foregroundStyle(secondary ? AnyShapeStyle(.secondary) : AnyShapeStyle(.primary))
        }
    }
}
