import SwiftUI

struct PixelMoon: View {
    var body: some View {
        Canvas { context, size in
            let cx = size.width / 2
            let cy = size.height / 2
            context.fill(
                Path(ellipseIn: CGRect(x: cx - 4, y: cy - 4, width: 8, height: 8)),
                with: .color(Color(red: 0.95, green: 0.92, blue: 0.75))
            )
            context.fill(
                Path(ellipseIn: CGRect(x: cx - 2, y: cy - 5, width: 7, height: 7)),
                with: .color(Color(red: 0.10, green: 0.10, blue: 0.18))
            )
        }
        .frame(width: 16, height: 16)
    }
}

struct PixelSun: View {
    let tick: Int

    private var isExtended: Bool { (tick / 8) % 2 == 0 }

    var body: some View {
        Canvas { context, size in
            let cx = size.width / 2
            let cy = size.height / 2
            let sunColor = Color(red: 1.0, green: 0.85, blue: 0.3)
            let rayColor = Color(red: 1.0, green: 0.78, blue: 0.2)

            context.fill(
                Path(CGRect(x: cx - 2, y: cy - 2, width: 4, height: 4)),
                with: .color(sunColor)
            )

            let len: CGFloat = isExtended ? 3 : 1

            for dir in [(0, -1), (0, 1), (-1, 0), (1, 0)] as [(Int, Int)] {
                let dx = CGFloat(dir.0)
                let dy = CGFloat(dir.1)
                for p in 0..<Int(len) {
                    let px = cx + dx * (3 + CGFloat(p)) - 0.5
                    let py = cy + dy * (3 + CGFloat(p)) - 0.5
                    context.fill(
                        Path(CGRect(x: px, y: py, width: 1, height: 1)),
                        with: .color(rayColor)
                    )
                }
            }

            let dLen: CGFloat = isExtended ? 2 : 1
            for dir in [(-1, -1), (1, -1), (-1, 1), (1, 1)] as [(Int, Int)] {
                let dx = CGFloat(dir.0)
                let dy = CGFloat(dir.1)
                for p in 0..<Int(dLen) {
                    let px = cx + dx * (3 + CGFloat(p)) - 0.5
                    let py = cy + dy * (3 + CGFloat(p)) - 0.5
                    context.fill(
                        Path(CGRect(x: px, y: py, width: 1, height: 1)),
                        with: .color(rayColor.opacity(0.7))
                    )
                }
            }
        }
        .frame(width: 16, height: 16)
    }
}
