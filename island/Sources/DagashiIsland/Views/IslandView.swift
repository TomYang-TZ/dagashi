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
            let artH = model.webViewHeight > 0 ? min(model.webViewHeight, 400) : 200
            return CGSize(width: 420, height: artH + 50)
        }
        return CGSize(width: 224, height: 38)
    }

    private var bgColor: Color {
        switch model.sceneWeather {
        case .sunny:  return Color(red: 0.96, green: 0.92, blue: 0.86)
        case .cloudy: return Color(red: 0.85, green: 0.85, blue: 0.85)
        case .rainy:  return Color(red: 0.70, green: 0.72, blue: 0.76)
        case .snowy:  return Color(red: 0.92, green: 0.93, blue: 0.96)
        case .stormy: return Color(red: 0.50, green: 0.52, blue: 0.58)
        case .night:  return Color(red: 0.12, green: 0.12, blue: 0.20)
        }
    }

    var body: some View {
        bgColor
            .frame(width: contentSize.width, height: contentSize.height)
            .overlay(alignment: .top) {
                // Collapsed content
                CollapsedView(model: model)
                    .frame(width: 224, height: 38)
                    .opacity(isOpen ? 0 : 1)
            }
            .overlay(alignment: .top) {
                // Expanded content
                ExpandedView(model: model)
                    .frame(width: 420)
                    .opacity(isOpen ? 1 : 0)
            }
            .clipShape(shape)
            .shadow(color: .black.opacity(isOpen ? 0.15 : 0), radius: 16, y: 8)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
            .animation(.spring(response: 0.45, dampingFraction: 0.88), value: model.notchStatus)
    }
}
