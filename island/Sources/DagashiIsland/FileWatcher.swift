import Foundation

class FileWatcher {
    weak var model: AppModel?
    private var collectionSource: DispatchSourceFileSystemObject?
    private var pullingSource: DispatchSourceFileSystemObject?
    private var collectionFd: Int32 = -1
    private var pullingFd: Int32 = -1
    private var lastPullCount: Int = 0
    private var pullingTimer: Timer?

    private var dagashiDir: URL {
        FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent(".dagashi")
    }

    func startWatching() {
        lastPullCount = model?.pullCount ?? 0
        watchCollection()
        startPullingPoller()
    }

    func stopWatching() {
        collectionSource?.cancel()
        pullingSource?.cancel()
        pullingTimer?.invalidate()
        if collectionFd >= 0 { close(collectionFd); collectionFd = -1 }
        if pullingFd >= 0 { close(pullingFd); pullingFd = -1 }
    }

    private var wasPulling = false

    // Poll for the "pulling" signal file
    private func startPullingPoller() {
        pullingTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            guard let self = self, let model = self.model else { return }
            let pullingFile = self.dagashiDir.appendingPathComponent("pulling")
            let isPulling = FileManager.default.fileExists(atPath: pullingFile.path)

            if isPulling && !self.wasPulling {
                // Pull started
                model.onPullStarted()
            } else if !isPulling && self.wasPulling && model.crowdState != .idle {
                // Pull ended (success or failure) — disperse if crowd is still gathered
                if model.crowdState == .gathering || model.crowdState == .cheering {
                    model.onCollapse()
                }
            }
            self.wasPulling = isPulling
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
