import Foundation

protocol CollectPort: Sendable {
    func triggerCollect() async throws -> Int
}
