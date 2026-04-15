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
        let controller = OverlayController(model: model)
        controller.show()
        overlayController = controller

        model.onHideIsland = { [weak controller] in
            controller?.hide()
        }
        model.onShowIsland = { [weak controller] in
            controller?.reshowPanel()
        }

        // Start watching for pull changes and weather
        model.fileWatcher.startWatching()
        model.weatherService.startMonitoring()
    }
}
