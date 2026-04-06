import SwiftUI

struct SettingsView: View {
    let feature: SettingsFeature
    let authFeature: AuthFeature
    var onTagsSaved: (() -> Void)?
    @Environment(\.dismiss) private var dismiss
    @State private var showSignOutAlert = false

    var body: some View {
        NavigationStack {
            List {
                Section("관리") {
                    NavigationLink {
                        TagManagementView(feature: feature, onTagsSaved: onTagsSaved)
                    } label: {
                        Label("태그 관리", systemImage: "tag")
                    }
                }

                Section("계정") {
                    Button(role: .destructive) {
                        showSignOutAlert = true
                    } label: {
                        Label("로그아웃", systemImage: "rectangle.portrait.and.arrow.right")
                    }
                }
            }
            .navigationTitle("설정")
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("닫기") { dismiss() }
                }
            }
            .alert("로그아웃", isPresented: $showSignOutAlert) {
                Button("취소", role: .cancel) {}
                Button("로그아웃", role: .destructive) {
                    Task {
                        await authFeature.send(.signOut)
                        dismiss()
                    }
                }
            } message: {
                Text("정말 로그아웃하시겠습니까?")
            }
            .task {
                await feature.send(.loadTags)
            }
        }
    }
}
