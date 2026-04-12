import AppKit
import SwiftUI

class OverlayController {
    let model: AppModel
    var panel: NotchPanel?
    var clickMonitor: Any?
    var clickLocalMonitor: Any?
    var keyMonitor: Any?
    var moveMonitor: Any?

    let closedSize = NSSize(width: 224, height: 38)
    let openedSize = NSSize(width: 400, height: 260)

    init(model: AppModel) {
        self.model = model
    }

    func show() {
        guard let screen = NSScreen.main else { return }

        let size = openedSize
        let origin = NSPoint(
            x: screen.frame.midX - size.width / 2,
            y: screen.frame.maxY - size.height
        )

        let panel = NotchPanel(contentRect: NSRect(origin: origin, size: size))

        let hostView = NSHostingView(rootView: IslandView(model: model))
        hostView.frame = panel.contentView?.bounds ?? .zero
        hostView.autoresizingMask = [.width, .height]
        panel.contentView?.addSubview(hostView)

        panel.orderFrontRegardless()
        self.panel = panel

        setupMouseMonitors()
    }

    private func setupMouseMonitors() {
        guard let panel = panel else { return }

        // Use NSEvent.mouseLocation for screen coordinates
        // Hover near notch → halo effect
        moveMonitor = NSEvent.addGlobalMonitorForEvents(matching: .mouseMoved) { [weak self] _ in
            guard let self = self else { return }
            let mouse = NSEvent.mouseLocation // screen coordinates
            let isNear = panel.frame.contains(mouse)

            DispatchQueue.main.async {
                if isNear && self.model.notchStatus == .closed {
                    if !self.model.isHovering { self.model.isHovering = true }
                } else if !isNear && self.model.isHovering && self.model.notchStatus == .closed {
                    self.model.isHovering = false
                }
            }
        }

        // Click outside → collapse (global catches clicks on other apps)
        clickMonitor = NSEvent.addGlobalMonitorForEvents(matching: .leftMouseDown) { [weak self] _ in
            guard let self = self else { return }
            // Global events are clicks NOT on our panel
            DispatchQueue.main.async {
                if self.model.notchStatus == .opened {
                    self.model.notchStatus = .closed
                }
            }
        }

        // Click on island → expand (local catches clicks on our panel)
        clickLocalMonitor = NSEvent.addLocalMonitorForEvents(matching: .leftMouseDown) { [weak self] event in
            guard let self = self, let screen = NSScreen.main else { return event }
            let mouse = NSEvent.mouseLocation

            // Collapsed pill rect (centered at top of screen)
            let pillW: CGFloat = 224
            let pillH: CGFloat = 38
            let pillRect = NSRect(
                x: screen.frame.midX - pillW / 2,
                y: screen.frame.maxY - pillH,
                width: pillW,
                height: pillH
            )

            DispatchQueue.main.async {
                if self.model.notchStatus == .closed && pillRect.contains(mouse) {
                    self.model.notchStatus = .opened
                    self.model.isHovering = false
                }
            }
            return event
        }

        // ESC to collapse
        keyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard let self = self else { return event }
            if event.keyCode == 53 && self.model.notchStatus == .opened { // 53 = ESC
                DispatchQueue.main.async {
                    self.model.notchStatus = .closed
                }
                return nil // consume the event
            }
            return event
        }
    }

    deinit {
        if let m = clickMonitor { NSEvent.removeMonitor(m) }
        if let m = clickLocalMonitor { NSEvent.removeMonitor(m) }
        if let m = keyMonitor { NSEvent.removeMonitor(m) }
        if let m = moveMonitor { NSEvent.removeMonitor(m) }
    }
}
