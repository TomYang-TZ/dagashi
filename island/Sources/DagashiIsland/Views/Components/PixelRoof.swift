import SwiftUI

struct PixelRoof: View {
    var isStormy: Bool = false
    var tick: Int = 0

    let tileColor = Color(red: 0.35, green: 0.33, blue: 0.38)
    let tileDark  = Color(red: 0.25, green: 0.23, blue: 0.28)
    let tileEdge  = Color(red: 0.45, green: 0.42, blue: 0.46)

    var body: some View {
        Canvas { context, size in
            let w = size.width
            let h = size.height

            let windOffset = isStormy ? CGFloat(Darwin.sin(Double(tick) * 0.3)) * 1.5 : 0

            let roofPath = Path { p in
                p.move(to: CGPoint(x: -2 + windOffset, y: h))
                p.addLine(to: CGPoint(x: w * 0.15 + windOffset * 0.5, y: 1))
                p.addLine(to: CGPoint(x: w * 0.85 + windOffset * 0.5, y: 1))
                p.addLine(to: CGPoint(x: w + 2 + windOffset, y: h))
                p.closeSubpath()
            }
            context.fill(roofPath, with: .color(tileColor))

            let ridge = Path { p in
                p.move(to: CGPoint(x: w * 0.15, y: 1))
                p.addLine(to: CGPoint(x: w * 0.85, y: 1))
            }
            context.stroke(ridge, with: .color(tileEdge), lineWidth: 1.5)

            for i in 1..<4 {
                let y = CGFloat(i) * h / 4
                let inset = (1.0 - y / h) * w * 0.15
                let line = Path { p in
                    p.move(to: CGPoint(x: inset - 1, y: y))
                    p.addLine(to: CGPoint(x: w - inset + 1, y: y))
                }
                context.stroke(line, with: .color(tileDark), lineWidth: 0.5)
            }

            let eave = Path { p in
                p.move(to: CGPoint(x: -2, y: h - 0.5))
                p.addLine(to: CGPoint(x: w + 2, y: h - 0.5))
            }
            context.stroke(eave, with: .color(tileDark), lineWidth: 1)
        }
    }
}
