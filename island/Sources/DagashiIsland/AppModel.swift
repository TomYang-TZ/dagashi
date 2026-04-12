import SwiftUI

enum CrowdState {
    case idle       // normal sparse traffic
    case gathering  // pull in progress — people rush to shop
    case cheering   // pull complete — crowd bounces
    case dispersing // user collapsed — crowd walks away
}

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
    var crowdState: CrowdState = .idle
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

    func onPullStarted() {
        isLoading = true
        crowdState = .gathering
        // After people arrive, start cheering
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
            if self.crowdState == .gathering {
                self.crowdState = .cheering
            }
        }
    }

    func onNewPull() {
        isLoading = false
        loadLatestPull()

        // Keep cheering briefly, then disperse and expand
        crowdState = .cheering
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
            self.crowdState = .dispersing
            // Expand after crowd starts leaving
            DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
                self.notchStatus = .opened
                self.crowdState = .idle
            }
        }
    }

    func onCollapse() {
        crowdState = .dispersing
        // Back to idle after crowd disperses
        DispatchQueue.main.asyncAfter(deadline: .now() + 4) {
            if self.crowdState == .dispersing {
                self.crowdState = .idle
            }
        }
    }
}
