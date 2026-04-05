import Foundation

struct Profile: Equatable, Sendable {
    let id: UUID
    let email: String
    let onboardingCompleted: Bool
}
