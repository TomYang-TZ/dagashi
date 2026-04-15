import SwiftUI

struct WeatherParticles: View {
    let weather: SceneWeather
    let tick: Int
    let width: CGFloat
    let height: CGFloat

    var body: some View {
        Canvas { context, size in
            switch weather {
            case .rainy:
                for i in 0..<20 {
                    let x = CGFloat((i * 37 + tick * 3) % Int(width))
                    let y = CGFloat((i * 23 + tick * 5) % Int(height))
                    context.fill(
                        Path(CGRect(x: x, y: y, width: 1, height: 2)),
                        with: .color(Color(red: 0.5, green: 0.6, blue: 0.8).opacity(0.5))
                    )
                }
            case .snowy:
                for i in 0..<12 {
                    let x = CGFloat((i * 41 + tick * 1) % Int(width))
                    let y = CGFloat((i * 19 + tick * 2) % Int(height))
                    let drift = CGFloat(Darwin.sin(Double(tick + i * 7) * 0.1)) * 2
                    context.fill(
                        Path(CGRect(x: x + drift, y: y, width: 1.5, height: 1.5)),
                        with: .color(.white.opacity(0.7))
                    )
                }
            case .stormy:
                for i in 0..<30 {
                    let x = CGFloat((i * 31 + tick * 6) % Int(width + 20)) - 10
                    let y = CGFloat((i * 17 + tick * 8) % Int(height))
                    var ray = Path()
                    ray.move(to: CGPoint(x: x, y: y))
                    ray.addLine(to: CGPoint(x: x + 2, y: y + 3))
                    context.stroke(ray, with: .color(Color(red: 0.6, green: 0.65, blue: 0.8).opacity(0.35)), lineWidth: 0.5)
                }
                for i in 0..<5 {
                    let x = CGFloat((i * 47 + tick * 3) % Int(width + 40)) - 20
                    let y = CGFloat(5 + i * 5)
                    context.stroke(
                        Path { p in p.move(to: CGPoint(x: x, y: y)); p.addLine(to: CGPoint(x: x + 10, y: y)) },
                        with: .color(.white.opacity(0.08)),
                        lineWidth: 0.5
                    )
                }
                if (tick / 10) % 50 < 2 {
                    context.fill(
                        Path(CGRect(x: 0, y: 0, width: size.width, height: size.height)),
                        with: .color(.white.opacity(0.2))
                    )
                }
            case .cloudy:
                for i in 0..<3 {
                    let x = CGFloat((i * 80 + tick / 2) % Int(width + 30)) - 15
                    let y = CGFloat(3 + i * 5)
                    context.fill(
                        Path(CGRect(x: x, y: y, width: 12, height: 3)),
                        with: .color(.white.opacity(0.4))
                    )
                    context.fill(
                        Path(CGRect(x: x + 2, y: y - 1, width: 8, height: 2)),
                        with: .color(.white.opacity(0.3))
                    )
                }
            case .night:
                for i in 0..<8 {
                    let x = CGFloat((i * 53) % Int(width))
                    let y = CGFloat((i * 7) % Int(height - 5)) + 2
                    let twinkle = (tick + i * 4) % 10 < 6
                    if twinkle {
                        context.fill(
                            Path(CGRect(x: x, y: y, width: 1, height: 1)),
                            with: .color(Color(red: 1.0, green: 0.95, blue: 0.7).opacity(0.6))
                        )
                    }
                }
            case .sunny:
                break
            }
        }
        .frame(width: width, height: height)
        .allowsHitTesting(false)
    }
}
