import SwiftUI
import Foundation

struct IslandView: View {
    @Bindable var model: AppModel

    private var isOpen: Bool {
        model.notchStatus == .opened || model.notchStatus == .popping
    }

    private var shape: NotchShape {
        isOpen ? .opened : .closed
    }

    private var contentSize: CGSize {
        if isOpen {
            let artH = model.webViewHeight > 0 ? model.webViewHeight : 200
            return CGSize(width: 400, height: artH + 50)
        }
        return CGSize(width: 224, height: 38)
    }

    var body: some View {
        // Single container that animates size, clipped once
        ZStack(alignment: .top) {
            // Warm background
            Color(red: 0.96, green: 0.92, blue: 0.86)

            // Collapsed content — pinned to top
            CollapsedView(model: model)
                .frame(width: 224, height: 38)
                .frame(maxWidth: .infinity, alignment: .center)
                .opacity(isOpen ? 0 : 1)

            // Expanded content — fixed at full width, only opacity animates
            ExpandedView(model: model)
                .frame(width: 400)
                .opacity(isOpen ? 1 : 0)

            // Cursor halo
            if model.isHovering && !isOpen {
                CursorHalo()
                    .frame(width: 224, height: 38)
                    .frame(maxWidth: .infinity, alignment: .center)
            }
        }
        .frame(width: contentSize.width, height: contentSize.height, alignment: .top)
        .clipShape(shape)
        .shadow(color: .black.opacity(isOpen ? 0.15 : 0), radius: 16, y: 8)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .animation(.spring(response: 0.45, dampingFraction: 0.88), value: model.notchStatus)
        .animation(.easeInOut(duration: 0.2), value: model.isHovering)
    }
}

struct CursorHalo: View {
    @State private var pulse: Bool = false

    var body: some View {
        ZStack {
            RoundedRectangle(cornerRadius: 16)
                .fill(
                    RadialGradient(
                        colors: [
                            Color.white.opacity(pulse ? 0.12 : 0.06),
                            Color.white.opacity(0)
                        ],
                        center: .center,
                        startRadius: 5,
                        endRadius: 80
                    )
                )

            RoundedRectangle(cornerRadius: 16)
                .strokeBorder(Color.white.opacity(pulse ? 0.15 : 0.08), lineWidth: 1)
        }
        .onAppear {
            withAnimation(.easeInOut(duration: 1.0).repeatForever(autoreverses: true)) {
                pulse = true
            }
        }
    }
}
