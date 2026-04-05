import Foundation

@MainActor
final class AppDependencies {
    let auth: any AuthPort
    let tag: any TagPort
    let article: any ArticlePort
    let collect: any CollectPort

    init(
        auth: any AuthPort,
        tag: any TagPort,
        article: any ArticlePort,
        collect: any CollectPort
    ) {
        self.auth = auth
        self.tag = tag
        self.article = article
        self.collect = collect
    }
}
