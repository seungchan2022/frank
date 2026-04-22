import ProjectDescription

let project = Project(
    name: "Frank",
    targets: [
        .target(
            name: "Frank",
            destinations: .iOS,
            product: .app,
            bundleId: "dev.frank.app",
            deploymentTargets: .iOS("26.0"),
            infoPlist: .extendingDefault(
                with: [
                    "UILaunchScreen": [
                        "UIColorName": "",
                        "UIImageName": "",
                    ],
                    // xcconfig에서 주입 — Config.xcconfig (git ignore됨)
                    "SUPABASE_URL": "$(SUPABASE_URL)",
                    "SUPABASE_ANON_KEY": "$(SUPABASE_ANON_KEY)",
                    "SERVER_URL": "$(SERVER_URL)",
                    // HTTP 로컬 서버 허용 (ATS localhost 예외)
                    "NSAppTransportSecurity": [
                        "NSAllowsLocalNetworking": true,
                        "NSAllowsArbitraryLoads": true,
                    ],
                ]
            ),
            sources: ["Frank/Sources/**"],
            resources: ["Frank/Resources/**"],
            entitlements: "Frank/Frank.entitlements",
            dependencies: [
                .external(name: "Supabase"),
            ],
            settings: .settings(
                base: [
                    "DEVELOPMENT_TEAM": "$(DEVELOPMENT_TEAM)",
                    "CODE_SIGN_STYLE": "Automatic",
                ],
                configurations: [
                    .debug(name: "Debug", xcconfig: "Config.xcconfig"),
                    .release(name: "Release", xcconfig: "Config.xcconfig"),
                ]
            )
        ),
        .target(
            name: "FrankTests",
            destinations: .iOS,
            product: .unitTests,
            bundleId: "dev.frank.app.tests",
            deploymentTargets: .iOS("26.0"),
            infoPlist: .default,
            sources: ["FrankTests/**"],
            resources: [],
            dependencies: [.target(name: "Frank")],
            settings: .settings(
                // Swift Testing 모듈 경로 활성화 → SourceKit LSP "No such module 'Testing'" 오탐 제거
                base: ["ENABLE_TESTING_SEARCH_PATHS": "YES"]
            )
        ),
        .target(
            name: "FrankUITests",
            destinations: .iOS,
            product: .uiTests,
            bundleId: "dev.frank.app.uitests",
            deploymentTargets: .iOS("26.0"),
            infoPlist: .default,
            sources: ["FrankUITests/**"],
            resources: [],
            dependencies: [.target(name: "Frank")]
        ),
    ]
)
