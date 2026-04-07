import Foundation

struct Profile: Equatable, Sendable {
    let id: UUID
    let displayName: String?
    let onboardingCompleted: Bool
}
