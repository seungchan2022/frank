import Foundation

protocol CollectPort: Sendable {
    func triggerCollect() async throws
    func triggerSummarize() async throws
}
