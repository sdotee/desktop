import SwiftUI
import SwiftData

enum SidebarItem: String, CaseIterable, Identifiable {
    case shortLinks
    case textSharing
    case fileUpload
    case tags
    case usage
    case settings

    var id: String { rawValue }

    var title: String {
        switch self {
        case .shortLinks: String(localized: "Short Links")
        case .textSharing: String(localized: "Text Sharing")
        case .fileUpload: String(localized: "File Upload")
        case .tags: String(localized: "Tags")
        case .usage: String(localized: "Usage")
        case .settings: String(localized: "Settings")
        }
    }

    var icon: String {
        switch self {
        case .shortLinks: "link"
        case .textSharing: "doc.text"
        case .fileUpload: "arrow.up.doc"
        case .tags: "tag"
        case .usage: "chart.bar"
        case .settings: "gearshape"
        }
    }
}

struct ContentView: View {
    @State private var selectedItem: SidebarItem? = .shortLinks
    @State private var hasAPIKey = KeychainService.getAPIKey() != nil

    var body: some View {
        if hasAPIKey {
            mainContent
        } else {
            OnboardingView(hasAPIKey: $hasAPIKey)
        }
    }

    @ViewBuilder
    private var mainContent: some View {
        #if os(macOS)
        NavigationSplitView {
            List(SidebarItem.allCases, selection: $selectedItem) { item in
                Label(item.title, systemImage: item.icon)
                    .tag(item)
            }
            .listStyle(.sidebar)
            .navigationSplitViewColumnWidth(min: 180, ideal: 200, max: 280)
        } detail: {
            detailView(for: selectedItem)
        }
        .frame(minWidth: 700, minHeight: 450)
        #else
        TabView {
            Tab(String(localized: "Short Links"), systemImage: "link") {
                NavigationStack {
                    ShortLinkListView()
                }
            }
            Tab(String(localized: "Text"), systemImage: "doc.text") {
                NavigationStack {
                    TextShareListView()
                }
            }
            Tab(String(localized: "Upload"), systemImage: "arrow.up.doc") {
                NavigationStack {
                    FileUploadView()
                }
            }
            Tab(String(localized: "More"), systemImage: "ellipsis") {
                NavigationStack {
                    MoreView(hasAPIKey: $hasAPIKey)
                }
            }
        }
        #endif
    }

    @ViewBuilder
    private func detailView(for item: SidebarItem?) -> some View {
        switch item {
        case .shortLinks:
            ShortLinkListView()
        case .textSharing:
            TextShareListView()
        case .fileUpload:
            FileUploadView()
        case .tags:
            TagsView()
        case .usage:
            UsageView()
        case .settings:
            SettingsView(hasAPIKey: $hasAPIKey)
        case nil:
            Text(String(localized: "Select an item from the sidebar"))
                .foregroundStyle(.secondary)
        }
    }
}

// MARK: - Onboarding View

struct OnboardingView: View {
    @Binding var hasAPIKey: Bool
    @State private var apiKey = ""
    @State private var baseURL = UserDefaults.standard.string(forKey: Constants.baseURLKey) ?? Constants.defaultBaseURL
    @State private var isValidating = false
    @State private var errorMessage: String?
    @State private var showSuccess = false
    @FocusState private var focusedField: OnboardingField?

    private enum OnboardingField {
        case baseURL, apiKey
    }

    var body: some View {
        #if os(macOS)
        macOSLayout
        #else
        iOSLayout
        #endif
    }

    // MARK: - App Icon

    #if os(iOS)
    private var appIconImage: Image {
        if let icons = Bundle.main.infoDictionary?["CFBundleIcons"] as? [String: Any],
           let primaryIcon = icons["CFBundlePrimaryIcon"] as? [String: Any],
           let iconFiles = primaryIcon["CFBundleIconFiles"] as? [String],
           let iconName = iconFiles.last,
           let uiImage = UIImage(named: iconName) {
            return Image(uiImage: uiImage)
        }
        return Image(systemName: "app.fill")
    }
    #endif

    // MARK: - macOS Layout

    #if os(macOS)
    private var macOSLayout: some View {
        VStack(spacing: 0) {
            Spacer()

            VStack(spacing: 32) {
                // Hero
                VStack(spacing: 12) {
                    Image(nsImage: NSApp.applicationIconImage)
                        .resizable()
                        .frame(width: 96, height: 96)

                    Text("S.EE")
                        .font(.system(size: 28, weight: .bold))

                    Text(String(localized: "URL Shortener, Text Sharing & File Hosting"))
                        .font(.title3)
                        .foregroundStyle(.secondary)
                }

                // API Token hint
                VStack(spacing: 6) {
                    Text(String(localized: "To get started, you need an API Token."))
                        .font(.body)
                        .foregroundStyle(.secondary)
                    Link(destination: URL(string: "https://s.ee/user/developers/")!) {
                        HStack(spacing: 4) {
                            Image(systemName: "arrow.up.right.square")
                                .font(.body)
                            Text(String(localized: "Get your API Token at s.ee"))
                                .font(.body.weight(.medium))
                        }
                    }
                    .focusEffectDisabled()
                }

                // Form
                VStack(spacing: 20) {
                    Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 16) {
                        GridRow {
                            Text(String(localized: "Base URL"))
                                .font(.body)
                                .gridColumnAlignment(.trailing)
                            TextField("https://s.ee/api/v1/", text: $baseURL)
                                .textFieldStyle(.roundedBorder)
                                .focused($focusedField, equals: .baseURL)
                        }

                        GridRow {
                            Text(String(localized: "API Key"))
                                .font(.body)
                                .gridColumnAlignment(.trailing)
                            HStack(spacing: 8) {
                                SecureField(String(localized: "Enter your API key"), text: $apiKey)
                                    .textFieldStyle(.roundedBorder)
                                    .focused($focusedField, equals: .apiKey)
                                Button(String(localized: "Paste")) {
                                    if let clipboard = ClipboardService.getString() {
                                        apiKey = clipboard
                                    }
                                }
                                .controlSize(.regular)
                            }
                        }
                    }

                    statusMessage
                }
                .frame(width: 420)

                // Action
                Button(action: validate) {
                    if isValidating {
                        ProgressView()
                            .controlSize(.small)
                            .frame(width: 140)
                    } else {
                        Text(String(localized: "Verify & Continue"))
                            .frame(width: 140)
                    }
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
                .disabled(apiKey.isEmpty || isValidating)
                .keyboardShortcut(.defaultAction)
            }

            Spacer()
        }
        .frame(minWidth: 560, minHeight: 460)
        .onSubmit {
            if focusedField == .baseURL {
                focusedField = .apiKey
            } else if focusedField == .apiKey && !apiKey.isEmpty {
                validate()
            }
        }
        .onAppear { focusedField = .apiKey }
    }
    #endif

    // MARK: - iOS Layout

    #if os(iOS)
    private var iOSLayout: some View {
        ScrollView {
            VStack(spacing: 32) {
                Spacer(minLength: 20)

                // Hero
                VStack(spacing: 12) {
                    appIconImage
                        .resizable()
                        .frame(width: 80, height: 80)
                        .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
                        .overlay(
                            RoundedRectangle(cornerRadius: 18, style: .continuous)
                                .strokeBorder(.quaternary, lineWidth: 0.5)
                        )

                    Text("S.EE")
                        .font(.title.bold())

                    Text(String(localized: "URL Shortener, Text Sharing & File Hosting"))
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.center)
                }

                // API Token hint
                VStack(spacing: 8) {
                    Text(String(localized: "To get started, you need an API Token."))
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                    Link(destination: URL(string: "https://s.ee/user/developers/")!) {
                        HStack(spacing: 4) {
                            Image(systemName: "arrow.up.right.square")
                                .font(.subheadline)
                            Text(String(localized: "Get your API Token at s.ee"))
                                .font(.subheadline.weight(.medium))
                        }
                    }
                }
                .multilineTextAlignment(.center)

                // Form
                VStack(spacing: 16) {
                    VStack(alignment: .leading, spacing: 6) {
                        Text(String(localized: "Base URL"))
                            .font(.subheadline.weight(.medium))
                            .foregroundStyle(.secondary)
                        TextField("https://s.ee/api/v1/", text: $baseURL)
                            .textFieldStyle(.roundedBorder)
                            .keyboardType(.URL)
                            .textInputAutocapitalization(.never)
                            .autocorrectionDisabled()
                            .focused($focusedField, equals: .baseURL)
                    }

                    VStack(alignment: .leading, spacing: 6) {
                        Text(String(localized: "API Key"))
                            .font(.subheadline.weight(.medium))
                            .foregroundStyle(.secondary)
                        HStack(spacing: 8) {
                            SecureField(String(localized: "Enter your API key"), text: $apiKey)
                                .textFieldStyle(.roundedBorder)
                                .textInputAutocapitalization(.never)
                                .autocorrectionDisabled()
                                .focused($focusedField, equals: .apiKey)
                            Button(String(localized: "Paste")) {
                                if let clipboard = ClipboardService.getString() {
                                    apiKey = clipboard
                                }
                            }
                            .buttonStyle(.bordered)
                        }
                    }

                    statusMessage
                }
                .frame(maxWidth: 500)

                // Action
                Button(action: validate) {
                    if isValidating {
                        ProgressView()
                            .frame(maxWidth: .infinity)
                            .frame(height: 20)
                    } else {
                        Text(String(localized: "Verify & Continue"))
                            .frame(maxWidth: .infinity)
                    }
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
                .disabled(apiKey.isEmpty || isValidating)
                .frame(maxWidth: 500)

                Spacer(minLength: 20)
            }
            .frame(maxWidth: .infinity)
            .padding(.horizontal, 24)
        }
        .scrollDismissesKeyboard(.interactively)
        .onSubmit {
            if focusedField == .baseURL {
                focusedField = .apiKey
            } else if focusedField == .apiKey && !apiKey.isEmpty {
                validate()
            }
        }
    }
    #endif

    // MARK: - Shared Components

    @ViewBuilder
    private var statusMessage: some View {
        if let errorMessage {
            HStack(spacing: 6) {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(.red)
                Text(errorMessage)
            }
            .font(.caption)
            .foregroundStyle(.red)
            .frame(maxWidth: .infinity, alignment: .leading)
        }

        if showSuccess {
            HStack(spacing: 6) {
                Image(systemName: "checkmark.circle.fill")
                    .foregroundStyle(.green)
                Text(String(localized: "API key verified successfully!"))
            }
            .font(.caption)
            .foregroundStyle(.green)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    // MARK: - Validation

    private func loadDefaultDomains() async {
        if let response: APIResponse<DomainsResponse> = try? await APIClient.shared.request(.getDomains),
           let first = response.data?.domains.first,
           UserDefaults.standard.string(forKey: Constants.defaultShortLinkDomainKey)?.isEmpty != false {
            UserDefaults.standard.set(first, forKey: Constants.defaultShortLinkDomainKey)
        }

        if let response: APIResponse<DomainsResponse> = try? await APIClient.shared.request(.getTextDomains),
           let first = response.data?.domains.first,
           UserDefaults.standard.string(forKey: Constants.defaultTextDomainKey)?.isEmpty != false {
            UserDefaults.standard.set(first, forKey: Constants.defaultTextDomainKey)
        }

        if let response: APIResponse<DomainsResponse> = try? await APIClient.shared.request(.getFileDomains),
           let first = response.data?.domains.first,
           UserDefaults.standard.string(forKey: Constants.defaultFileDomainKey)?.isEmpty != false {
            UserDefaults.standard.set(first, forKey: Constants.defaultFileDomainKey)
        }
    }

    private func validate() {
        isValidating = true
        errorMessage = nil
        showSuccess = false

        UserDefaults.standard.set(baseURL, forKey: Constants.baseURLKey)
        KeychainService.setAPIKey(apiKey)

        Task {
            do {
                let _ = try await APIClient.shared.validateAPIKey()
                await loadDefaultDomains()
                showSuccess = true
                try? await Task.sleep(for: .seconds(0.5))
                hasAPIKey = true
            } catch {
                KeychainService.setAPIKey(nil)
                errorMessage = error.localizedDescription
            }
            isValidating = false
        }
    }
}

// MARK: - iOS More View

#if os(iOS)
struct MoreView: View {
    @Binding var hasAPIKey: Bool

    var body: some View {
        List {
            NavigationLink {
                TagsView()
            } label: {
                Label(String(localized: "Tags"), systemImage: "tag")
            }

            NavigationLink {
                UsageView()
            } label: {
                Label(String(localized: "Usage"), systemImage: "chart.bar")
            }

            NavigationLink {
                SettingsView(hasAPIKey: $hasAPIKey)
            } label: {
                Label(String(localized: "Settings"), systemImage: "gearshape")
            }
        }
        .navigationTitle(String(localized: "More"))
    }
}
#endif
