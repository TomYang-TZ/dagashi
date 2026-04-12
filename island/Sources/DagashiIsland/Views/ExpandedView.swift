import SwiftUI
import WebKit

struct ExpandedView: View {
    @Bindable var model: AppModel

    var body: some View {
        VStack(spacing: 0) {
            // ASCII art area with dark background
            WebViewWrapper(model: model)
                .frame(height: model.webViewHeight > 0 ? model.webViewHeight : 200)
                .background(Color(red: 0.1, green: 0.09, blue: 0.06))
                .clipShape(RoundedRectangle(cornerRadius: 6))
                .padding(.horizontal, 8)
                .padding(.top, 8)

            // Character info bar
            if let pull = model.currentPull {
                HStack(spacing: 6) {
                    Text(pull.rarity.uppercased())
                        .font(.system(size: 6, weight: .bold, design: .monospaced))
                        .foregroundColor(.white)
                        .padding(.horizontal, 5)
                        .padding(.vertical, 2)
                        .background(rarityColor(pull.rarity))
                        .clipShape(RoundedRectangle(cornerRadius: 2))

                    Text(pull.character)
                        .font(.system(size: 8, weight: .bold, design: .monospaced))
                        .foregroundColor(Color(red: 0.35, green: 0.28, blue: 0.18))

                    Spacer()

                    Text(pull.anime_title)
                        .font(.system(size: 6, design: .monospaced))
                        .foregroundColor(Color(red: 0.35, green: 0.28, blue: 0.18).opacity(0.5))
                        .lineLimit(1)
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 8)
            }
        }
    }

    private func rarityColor(_ rarity: String) -> Color {
        switch rarity.lowercased() {
        case "legendary": return Color(red: 1.0, green: 0.84, blue: 0.0)
        case "epic":      return Color(red: 0.6, green: 0.3, blue: 0.9)
        case "rare":      return Color(red: 0.2, green: 0.8, blue: 0.8)
        case "uncommon":  return Color(red: 0.3, green: 0.8, blue: 0.3)
        default:          return Color(red: 0.6, green: 0.6, blue: 0.6)
        }
    }
}

struct WebViewWrapper: NSViewRepresentable {
    let model: AppModel

    func makeNSView(context: Context) -> WKWebView {
        let config = WKWebViewConfiguration()
        config.preferences.setValue(true, forKey: "allowFileAccessFromFileURLs")

        let webView = WKWebView(frame: .zero, configuration: config)
        webView.setValue(false, forKey: "drawsBackground")

        // Listen for "ready" message from JS
        let handler = context.coordinator
        config.userContentController.add(handler, name: "dagashi")
        handler.webView = webView
        handler.model = model

        let widgetPath = findWidgetHTML()
        if let url = widgetPath {
            webView.loadFileURL(url, allowingReadAccessTo: url.deletingLastPathComponent())
        }

        return webView
    }

    func updateNSView(_ webView: WKWebView, context: Context) {
        // Send frames when WebView is ready and we have data
        context.coordinator.sendFramesIfReady()
    }

    func makeCoordinator() -> Coordinator { Coordinator() }

    class Coordinator: NSObject, WKScriptMessageHandler {
        var webView: WKWebView?
        var model: AppModel?
        var isReady = false
        var lastSentJson: String?

        func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
            guard let body = message.body as? String, message.name == "dagashi" else { return }

            if body == "ready" {
                isReady = true
                sendFramesIfReady()
            } else if body.hasPrefix("height:"), let h = Double(body.dropFirst(7)) {
                DispatchQueue.main.async {
                    self.model?.webViewHeight = CGFloat(h) + 8 // small padding
                }
            }
        }

        func sendFramesIfReady() {
            guard isReady, let webView = webView, let model = model else { return }

            if let json = model.framesJson, json != lastSentJson {
                lastSentJson = json
                // Write JSON to a temp file and load via fetch — avoids escaping issues
                let tmpURL = FileManager.default.temporaryDirectory.appendingPathComponent("dagashi-frames.json")
                try? json.data(using: .utf8)?.write(to: tmpURL)

                let js = """
                fetch('\(tmpURL.absoluteString)')
                    .then(r => r.json())
                    .then(data => loadFrames(data))
                    .catch(e => console.error('Frame load error:', e));
                """
                webView.evaluateJavaScript(js)
            }

            if model.isLoading {
                webView.evaluateJavaScript("showLoading()")
            }
        }
    }

    private func findWidgetHTML() -> URL? {
        let candidates = [
            URL(fileURLWithPath: NSHomeDirectory())
                .appendingPathComponent("dagashi/src/widget.html"),
            Bundle.main.url(forResource: "widget", withExtension: "html"),
        ]

        for candidate in candidates {
            if let url = candidate, FileManager.default.fileExists(atPath: url.path) {
                return url
            }
        }
        return nil
    }
}
