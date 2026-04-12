import SwiftUI

enum NotchStatus {
    case closed
    case opened
    case popping // brief pop animation for new pull
}

enum DisplayMode: CaseIterable {
    case colorClean, colorBlock, monoClean, monoBlock, original

    var label: String {
        switch self {
        case .colorClean: return "COLOR · CLEAN"
        case .colorBlock: return "COLOR · BLOCK"
        case .monoClean:  return "MONO · CLEAN"
        case .monoBlock:  return "MONO · BLOCK"
        case .original:   return "ORIGINAL"
        }
    }
}

@Observable
class AppModel {
    var notchStatus: NotchStatus = .closed
    var isHovering = false
    var currentPull: PullMeta?
    var framesJson: String?
    var isLoading = false
    var displayMode: DisplayMode = .colorClean
    var pullCount: Int = 0
    var webViewHeight: CGFloat = 0

    let fileWatcher: FileWatcher

    init() {
        self.fileWatcher = FileWatcher()
        self.fileWatcher.model = self
        loadLatestPull()
    }

    func loadLatestPull() {
        let dagashiDir = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".dagashi")
        let collectionPath = dagashiDir.appendingPathComponent("collection.json")

        guard let data = try? Data(contentsOf: collectionPath),
              let collection = try? JSONDecoder().decode(PullCollection.self, from: data),
              let latest = collection.pulls.last else {
            return
        }

        pullCount = collection.pulls.count
        currentPull = latest

        // Load frames
        let framesPath = dagashiDir
            .appendingPathComponent("pulls")
            .appendingPathComponent(latest.date)
            .appendingPathComponent("frames.json")

        if let framesData = try? Data(contentsOf: framesPath),
           let json = String(data: framesData, encoding: .utf8) {
            framesJson = json
        }
    }

    func onNewPull() {
        isLoading = false
        loadLatestPull()

        // Pop animation
        withAnimation(.spring(response: 0.3, dampingFraction: 0.5)) {
            notchStatus = .popping
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            withAnimation(.spring(response: 0.42, dampingFraction: 0.8)) {
                self.notchStatus = .opened
            }
        }
    }
}
