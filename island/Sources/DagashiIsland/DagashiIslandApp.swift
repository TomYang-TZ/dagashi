import SwiftUI

@main
struct DagashiIslandApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    var body: some Scene {
        // MenuBarExtra keeps the app alive without a dock icon
        MenuBarExtra("Dagashi", systemImage: "sparkles") {
            Button("Quit") { NSApplication.shared.terminate(nil) }
        }
    }
}

class AppDelegate: NSObject, NSApplicationDelegate {
    var overlayController: OverlayController?

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Hide dock icon
        NSApp.setActivationPolicy(.accessory)

        let model = AppModel()
        overlayController = OverlayController(model: model)
        overlayController?.show()

        // Start watching for pull changes and weather
        model.fileWatcher.startWatching()
        model.weatherService.startMonitoring()
    }
}
