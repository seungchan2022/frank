import Foundation

struct Profile: Equatable, Sendable {
    let id: UUID
    let displayName: String?
    let onboardingCompleted: Bool
    /// MVP15 M3: 직업 한 줄 (최대 50자). nil = 미설정.
    let occupation: String?
}
