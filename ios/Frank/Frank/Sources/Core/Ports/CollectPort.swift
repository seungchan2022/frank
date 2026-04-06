import Foundation

protocol CollectPort: Sendable {
    func triggerCollect() async throws -> Int
    func triggerSummarize() async throws -> Int
}
