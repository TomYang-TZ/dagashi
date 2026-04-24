import Foundation

class FileWatcher {
    weak var model: AppModel?
    private var collectionSource: DispatchSourceFileSystemObject?
    private var pullingSource: DispatchSourceFileSystemObject?
    private var collectionFd: Int32 = -1
    private var pullingFd: Int32 = -1
    private var lastPullCount: Int = 0
    private var pullingTimer: Timer?
    private var hourlyTimer: Timer?
    private var lastHourKey: String = ""

    private var dagashiDir: URL {
        FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent(".dagashi")
    }

    func startWatching() {
        lastPullCount = model?.pullCount ?? 0
        lastHourKey = currentHourKey()
        watchCollection()
        startPullingPoller()
        startHourlyTrigger()
    }

    func stopWatching() {
        collectionSource?.cancel()
        pullingSource?.cancel()
        pullingTimer?.invalidate()
        hourlyTimer?.invalidate()
        if collectionFd >= 0 { close(collectionFd); collectionFd = -1 }
        if pullingFd >= 0 { close(pullingFd); pullingFd = -1 }
    }

    private var wasPulling = false

    // Poll for signal files
    private func startPullingPoller() {
        pullingTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            guard let self = self, let model = self.model else { return }

            // Check if main app wants to show the island
            let showFile = self.dagashiDir.appendingPathComponent("show-island")
            if FileManager.default.fileExists(atPath: showFile.path) {
                try? FileManager.default.removeItem(at: showFile)
                model.onShowIsland?()
            }

            let pullingFile = self.dagashiDir.appendingPathComponent("pulling")
            let isPulling = FileManager.default.fileExists(atPath: pullingFile.path)

            if isPulling && !self.wasPulling {
                // Pull started
                model.onPullStarted()
            } else if !isPulling && self.wasPulling {
                // Pull ended (success or failure) — always clear loading state
                model.isLoading = false
                if model.crowdState == .gathering || model.crowdState == .cheering {
                    model.onCollapse()
                }
            }
            self.wasPulling = isPulling
        }
    }

    private func currentHourKey() -> String {
        let now = Date()
        let cal = Calendar.current
        let y = cal.component(.year, from: now)
        let m = cal.component(.month, from: now)
        let d = cal.component(.day, from: now)
        let h = cal.component(.hour, from: now)
        return String(format: "%04d-%02d-%02d-%02d", y, m, d, h)
    }

    private func startHourlyTrigger() {
        hourlyTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            guard let self = self else { return }
            let key = self.currentHourKey()
            if key != self.lastHourKey {
                self.lastHourKey = key
                let trigger = self.dagashiDir.appendingPathComponent("trigger-pull")
                try? "1".write(to: trigger, atomically: true, encoding: .utf8)
            }
        }
    }

    private func watchCollection() {
        let path = dagashiDir.appendingPathComponent("collection.json").path
        collectionFd = open(path, O_EVTONLY)
        guard collectionFd >= 0 else {
            DispatchQueue.main.asyncAfter(deadline: .now() + 5) { [weak self] in
                self?.watchCollection()
            }
            return
        }

        collectionSource = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: collectionFd,
            eventMask: [.write, .rename],
            queue: .main
        )

        collectionSource?.setEventHandler { [weak self] in
            self?.onCollectionChanged()
        }

        collectionSource?.setCancelHandler { [weak self] in
            guard let self = self else { return }
            if self.collectionFd >= 0 { close(self.collectionFd); self.collectionFd = -1 }
        }

        collectionSource?.resume()
    }

    private func onCollectionChanged() {
        guard let model = model else { return }

        DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) { [weak self] in
            guard let self = self else { return }

            let oldCount = self.lastPullCount
            model.loadLatestPull()

            if model.pullCount > oldCount {
                self.lastPullCount = model.pullCount
                model.onNewPull()
            }
        }
    }

    deinit {
        stopWatching()
    }
}
