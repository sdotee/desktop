import SwiftUI
import SwiftData
#if os(macOS)
import Sparkle
#endif

#if os(macOS)
final class CheckForUpdatesViewModel: ObservableObject {
    @Published var canCheckForUpdates = false

    init(updater: SPUUpdater) {
        updater.publisher(for: \.canCheckForUpdates)
            .assign(to: &$canCheckForUpdates)
    }
}

struct CheckForUpdatesView: View {
    @ObservedObject var checkForUpdatesViewModel: CheckForUpdatesViewModel
    let updater: SPUUpdater

    var body: some View {
        Button(String(localized: "Check for Updates…")) {
            updater.checkForUpdates()
        }
        .disabled(!checkForUpdatesViewModel.canCheckForUpdates)
    }
}
#endif

@main
struct SEEApp: App {
    #if os(macOS)
    private let updaterController: SPUStandardUpdaterController

    init() {
        updaterController = SPUStandardUpdaterController(
            startingUpdater: true,
            updaterDelegate: nil,
            userDriverDelegate: nil
        )
    }
    #endif

    var sharedModelContainer: ModelContainer = {
        let schema = Schema([
            ShortLink.self,
            TextShare.self,
            UploadedFile.self,
        ])
        let modelConfiguration = ModelConfiguration(schema: schema, isStoredInMemoryOnly: false)
        do {
            return try ModelContainer(for: schema, configurations: [modelConfiguration])
        } catch {
            fatalError("Could not create ModelContainer: \(error)")
        }
    }()

    var body: some Scene {
        WindowGroup(id: "main") {
            ContentView()
        }
        .modelContainer(sharedModelContainer)
        #if os(macOS)
        .windowToolbarStyle(.unified)
        .defaultSize(width: 900, height: 600)
        .commands {
            CommandGroup(after: .appInfo) {
                CheckForUpdatesView(
                    checkForUpdatesViewModel: CheckForUpdatesViewModel(updater: updaterController.updater),
                    updater: updaterController.updater
                )
            }
            CommandGroup(after: .newItem) {
                Button(String(localized: "New Short Link")) {
                    NotificationCenter.default.post(name: .createShortLink, object: nil)
                }
                .keyboardShortcut("n", modifiers: [.command])

                Button(String(localized: "New Text Share")) {
                    NotificationCenter.default.post(name: .createTextShare, object: nil)
                }
                .keyboardShortcut("n", modifiers: [.command, .shift])
            }
        }
        #endif

        #if os(macOS)
        Settings {
            SettingsView()
                .modelContainer(sharedModelContainer)
        }

        MenuBarExtra("S.EE", image: "MenuBarIcon") {
            MenuBarView()
                .modelContainer(sharedModelContainer)
        }
        .menuBarExtraStyle(.window)
        #endif
    }
}

// MARK: - Notification Names

extension Notification.Name {
    static let createShortLink = Notification.Name("createShortLink")
    static let createTextShare = Notification.Name("createTextShare")
}
