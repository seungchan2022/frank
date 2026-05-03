import SwiftUI

struct SettingsView: View {
    let feature: SettingsFeature
    let authFeature: AuthFeature
    var onTagsSaved: (() -> Void)?
    @Environment(\.dismiss) private var dismiss
    @State private var showSignOutAlert = false
    /// MVP15 M3: 직업 입력 필드 로컬 상태
    @State private var occupationInput: String = ""

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

                // MARK: MVP15 M3: 직업 설정 섹션
                Section {
                    VStack(alignment: .leading, spacing: 8) {
                        TextField("예: iOS 개발자, 프론트엔드 엔지니어", text: $occupationInput)
                            .textInputAutocapitalization(.never)
                            .autocorrectionDisabled()
                        if let errorMsg = feature.occupationError {
                            Text(errorMsg)
                                .font(.caption)
                                .foregroundStyle(.red)
                        }
                        if let successMsg = feature.occupationSuccess {
                            Text(successMsg)
                                .font(.caption)
                                .foregroundStyle(.green)
                        }
                        Button {
                            Task {
                                await feature.send(.saveOccupation(occupationInput.isEmpty ? nil : occupationInput))
                                // 이슈 C: 저장 성공 시 입력 필드를 서버에 반영된 값으로 동기화
                                if feature.occupationError == nil {
                                    occupationInput = feature.occupation ?? ""
                                }
                            }
                        } label: {
                            if feature.isSavingOccupation {
                                ProgressView()
                                    .progressViewStyle(.circular)
                                    .frame(maxWidth: .infinity, alignment: .center)
                            } else {
                                Text("저장")
                                    .frame(maxWidth: .infinity, alignment: .center)
                            }
                        }
                        .disabled(feature.isSavingOccupation)
                        .buttonStyle(.borderedProminent)
                    }
                    .padding(.vertical, 4)
                } header: {
                    Text("직업")
                } footer: {
                    Text("직업을 설정하면 기사 요약에 맞춤 인사이트가 제공됩니다. 최대 50자.")
                        .font(.caption)
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
                await feature.send(.loadOccupation)
                // 로드된 occupation으로 입력 필드 초기화 — 사용자가 먼저 타이핑한 경우 덮어쓰지 않음
                if occupationInput.isEmpty {
                    occupationInput = feature.occupation ?? ""
                }
            }
        }
    }
}
