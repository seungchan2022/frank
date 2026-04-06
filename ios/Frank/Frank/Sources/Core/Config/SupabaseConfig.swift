import Foundation

struct SupabaseConfig: Sendable {
    let url: URL
    let anonKey: String

    static var live: SupabaseConfig {
        // 1) Info.plist에서 읽기 시도
        if let urlString = Bundle.main.infoDictionary?["SUPABASE_URL"] as? String,
           !urlString.isEmpty,
           urlString.hasPrefix("https://"),
           let url = URL(string: urlString),
           let anonKey = Bundle.main.infoDictionary?["SUPABASE_ANON_KEY"] as? String,
           !anonKey.isEmpty {
            return SupabaseConfig(url: url, anonKey: anonKey)
        }

        // 2) Secrets.plist 파일에서 읽기 시도
        if let path = Bundle.main.path(forResource: "Secrets", ofType: "plist"),
           let dict = NSDictionary(contentsOfFile: path),
           let urlString = dict["SUPABASE_URL"] as? String,
           let url = URL(string: urlString),
           let anonKey = dict["SUPABASE_ANON_KEY"] as? String {
            return SupabaseConfig(url: url, anonKey: anonKey)
        }

        Log.app.warning("Supabase config not found — using placeholder")
        // swiftlint:disable:next force_unwrapping
        guard let placeholderURL = URL(string: "https://placeholder.supabase.co") else {
            fatalError("Invalid placeholder URL literal — this is a programmer error")
        }
        return SupabaseConfig(
            url: placeholderURL,
            anonKey: "placeholder"
        )
    }
}
