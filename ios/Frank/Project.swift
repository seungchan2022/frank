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
                ]
            ),
            sources: ["Frank/Sources/**"],
            resources: ["Frank/Resources/**"],
            dependencies: []
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
    ]
)
