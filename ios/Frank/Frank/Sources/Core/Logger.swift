import OSLog

enum Log {
    static let app = Logger(subsystem: "dev.frank.app", category: "app")
    static let router = Logger(subsystem: "dev.frank.app", category: "router")
    static let network = Logger(subsystem: "dev.frank.app", category: "network")
    static let feature = Logger(subsystem: "dev.frank.app", category: "feature")
}
