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
                    "SUPABASE_URL": "$(SUPABASE_URL)",
                    "SUPABASE_ANON_KEY": "$(SUPABASE_ANON_KEY)",
                ]
            ),
            sources: ["Frank/Sources/**"],
            resources: ["Frank/Resources/**"],
            entitlements: "Frank/Frank.entitlements",
            dependencies: [
                .external(name: "Supabase"),
            ]
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
            dependencies: [.target(name: "Frank")]
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
