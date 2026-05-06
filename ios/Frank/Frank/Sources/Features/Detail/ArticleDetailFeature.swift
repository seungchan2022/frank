import Foundation
import Observation

/// M2 디테일 뷰 phase.
enum DetailPhase: Equatable {
    case idle
    case loading
    case done(SummaryResult)
    case failed(String)

    var summaryResult: SummaryResult? {
        guard case .done(let result) = self else { return nil }
        return result
    }

    var errorMessage: String? {
        guard case .failed(let msg) = self else { return nil }
        return msg
    }
}

/// MVP15 M3: 재작성 phase.
enum RewritePhase: Equatable {
    case idle
    case loading
    case done(RewriteResult)
    case failed(String)

    var rewriteResult: RewriteResult? {
        guard case .done(let result) = self else { return nil }
        return result
    }
}

/// MVP5 M2: ArticleDetailFeature — FeedItem 보유 + 온디맨드 요약.
/// MVP15 M3: 재작성(rewrite) 기능 추가.
/// - `phase`: idle → loading → done | failed
/// - `rewritePhase`: idle → loading → done | failed
/// - `loadSummary()`: 캐시 히트 시 API 호출 없이 즉시 반환
@Observable
@MainActor
final class ArticleDetailFeature {

    // MARK: - Data

    let feedItem: FeedItem
    private(set) var phase: DetailPhase = .idle
    /// MVP15 M3: 재작성 phase
    private(set) var rewritePhase: RewritePhase = .idle
    /// MVP15 M3: 현재 사용자 프로필 (occupation 여부 확인용)
    private(set) var userProfile: Profile?

    // MARK: - Dependencies

    private let summarize: any SummarizePort
    private let rewrite: any RewritePort
    private let auth: any AuthPort
    private let cache: SummarySessionCache
    private let rewriteCache: RewriteSessionCache

    // MARK: - Init

    init(
        feedItem: FeedItem,
        summarize: any SummarizePort,
        rewrite: any RewritePort,
        auth: any AuthPort,
        cache: SummarySessionCache? = nil,
        rewriteCache: RewriteSessionCache? = nil
    ) {
        self.feedItem = feedItem
        self.summarize = summarize
        self.rewrite = rewrite
        self.auth = auth
        // nil이면 @MainActor 컨텍스트 안에서 .shared 해결 (Swift 6 default-param nonisolated 경고 방지)
        let resolvedCache = cache ?? SummarySessionCache.shared
        self.cache = resolvedCache
        let resolvedRewriteCache = rewriteCache ?? RewriteSessionCache.shared
        self.rewriteCache = resolvedRewriteCache
        // 캐시 히트 시 즉시 done 상태로 시작 — 즐겨찾기에서 진입 시 버튼 없이 요약 바로 표시
        if let cached = resolvedCache.get(feedItem.url.absoluteString) {
            self.phase = .done(cached)
        }
        // MVP15 M3: 재작성 캐시 히트 시 즉시 done 상태로 시작
        // MVP16 M3 (C2-bug): occupation 불일치 시 idle 유지 → 버튼 재활성화
        // userProfile은 init 시점에 아직 로드 전이므로 occupation 검증은 loadUserProfile() 후 수행
        if let cachedRewrite = resolvedRewriteCache.get(feedItem.url.absoluteString) {
            self.rewritePhase = .done(cachedRewrite)
        }
    }

    // MARK: - Actions

    func loadSummary() async {
        let url = feedItem.url.absoluteString

        // 캐시 히트 — API 호출 없이 즉시 반환
        if let cached = cache.get(url) {
            phase = .done(cached)
            return
        }

        phase = .loading

        do {
            let result = try await summarize.summarize(url: url, title: feedItem.title)
            cache.set(url, result)
            phase = .done(result)
        } catch {
            phase = .failed(summarizeErrorMessage(from: error))
        }
    }

    /// MVP15 M3: 사용자 프로필 로드 (occupation 여부 확인용).
    /// MVP16 M3 (C2-bug): 로드 후 캐시된 rewrite의 occupation과 현재 occupation 비교.
    /// 불일치 시 rewritePhase를 idle로 되돌려 버튼 재활성화.
    func loadUserProfile() async {
        userProfile = try? await auth.currentProfile()

        // occupation 변경 감지: 캐시된 rewrite가 있고 occupation이 다르면 idle로 리셋
        let url = feedItem.url.absoluteString
        if case .done = rewritePhase,
           let cached = rewriteCache.getWithOccupation(url) {
            let cachedOcc = cached.occupation
            let currentOcc = userProfile?.occupation
            if cachedOcc != currentOcc {
                rewritePhase = .idle
            }
        }
    }

    /// MVP15 M3: 직업 시각으로 재작성 요청.
    /// loading 중이면 중복 호출 무시.
    /// MVP16 M3 (C2-bug): occupation과 함께 캐시 저장.
    func loadRewrite() async {
        if case .loading = rewritePhase { return }

        rewritePhase = .loading

        do {
            let result = try await rewrite.rewrite(url: feedItem.url.absoluteString, title: feedItem.title)
            // MVP16 M3: occupation과 함께 캐시 저장
            rewriteCache.set(feedItem.url.absoluteString, result, occupation: userProfile?.occupation)
            rewritePhase = .done(result)
        } catch {
            rewritePhase = .failed(rewriteErrorMessage(from: error))
        }
    }

    // MARK: - Private

    private func summarizeErrorMessage(from error: Error) -> String {
        if isTimeoutError(error, domainCase: (error as? APISummarizeError) == .timeout) {
            return "요약 요청이 시간을 초과했습니다. 다시 시도해주세요."
        }
        return "요약을 불러오지 못했습니다. 다시 시도해주세요."
    }

    private func rewriteErrorMessage(from error: Error) -> String {
        if (error as? APIRewriteError) == .occupationRequired {
            return "직업을 먼저 설정해 주세요."
        }
        if isTimeoutError(error, domainCase: (error as? APIRewriteError) == .timeout) {
            return "재작성 요청이 시간을 초과했습니다. 다시 시도해주세요."
        }
        return "재작성에 실패했습니다. 다시 시도해주세요."
    }

    /// URLError.timedOut 또는 도메인별 타임아웃 케이스를 통합 감지한다.
    private func isTimeoutError(_ error: Error, domainCase: Bool) -> Bool {
        domainCase || (error as? URLError)?.code == .timedOut
    }
}
