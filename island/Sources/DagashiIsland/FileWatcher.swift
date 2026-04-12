import Foundation

class FileWatcher {
    weak var model: AppModel?
    private var source: DispatchSourceFileSystemObject?
    private var fileDescriptor: Int32 = -1
    private var lastPullCount: Int = 0

    private var collectionPath: URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".dagashi")
            .appendingPathComponent("collection.json")
    }

    func startWatching() {
        lastPullCount = model?.pullCount ?? 0
        watch()
    }

    func stopWatching() {
        source?.cancel()
        source = nil
        if fileDescriptor >= 0 {
            close(fileDescriptor)
            fileDescriptor = -1
        }
    }

    private func watch() {
        stopWatching()

        let path = collectionPath.path
        fileDescriptor = open(path, O_EVTONLY)
        guard fileDescriptor >= 0 else {
            // File doesn't exist yet — retry in 5s
            DispatchQueue.main.asyncAfter(deadline: .now() + 5) { [weak self] in
                self?.watch()
            }
            return
        }

        source = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fileDescriptor,
            eventMask: [.write, .rename],
            queue: .main
        )

        source?.setEventHandler { [weak self] in
            self?.onFileChanged()
        }

        source?.setCancelHandler { [weak self] in
            guard let self = self else { return }
            if self.fileDescriptor >= 0 {
                close(self.fileDescriptor)
                self.fileDescriptor = -1
            }
        }

        source?.resume()
    }

    private func onFileChanged() {
        guard let model = model else { return }

        // Small delay to let the write complete
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) { [weak self] in
            guard let self = self else { return }
            model.loadLatestPull()

            // New pull detected?
            if model.pullCount > self.lastPullCount {
                self.lastPullCount = model.pullCount
                model.onNewPull()
            }
        }
    }

    deinit {
        stopWatching()
    }
}
