import SwiftUI

struct PixelPerson: Identifiable {
    let id = UUID()
    var x: CGFloat
    var speed: CGFloat
    var direction: CGFloat  // 1 = right, -1 = left
    var color: Color
    var kind: SpriteKind

    enum SpriteKind {
        case walker
        case biker
    }
}

// Cheering sound effect pixels — tiny dots that pop above heads like retro game effects
struct CheerPixels: View {
    let tick: Int
    let index: Int

    var body: some View {
        let phase = (tick + index * 5) % 12
        let show1 = phase < 4
        let show2 = phase >= 3 && phase < 8
        let show3 = phase >= 6 && phase < 11

        Canvas { context, size in
            let cx = size.width / 2
            if show1 {
                context.fill(Path(CGRect(x: cx - 2, y: 1, width: 1, height: 1)),
                    with: .color(Color(red: 1.0, green: 0.8, blue: 0.2)))
            }
            if show2 {
                context.fill(Path(CGRect(x: cx + 1, y: 0, width: 1, height: 1)),
                    with: .color(Color(red: 1.0, green: 0.5, blue: 0.3)))
            }
            if show3 {
                context.fill(Path(CGRect(x: cx, y: 2, width: 1, height: 1)),
                    with: .color(Color(red: 0.3, green: 0.8, blue: 1.0)))
            }
        }
        .frame(width: 8, height: 5)
    }
}

// Weather particle effects
struct WeatherParticles: View {
    let weather: SceneWeather
    let tick: Int
    let width: CGFloat
    let height: CGFloat

    var body: some View {
        Canvas { context, size in
            switch weather {
            case .rainy:
                // Falling rain drops
                for i in 0..<20 {
                    let x = CGFloat((i * 37 + tick * 3) % Int(width))
                    let y = CGFloat((i * 23 + tick * 5) % Int(height))
                    context.fill(
                        Path(CGRect(x: x, y: y, width: 1, height: 2)),
                        with: .color(Color(red: 0.5, green: 0.6, blue: 0.8).opacity(0.5))
                    )
                }
            case .snowy:
                // Floating snowflakes
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
                // Heavy angled rain (wind-blown)
                for i in 0..<30 {
                    let x = CGFloat((i * 31 + tick * 6) % Int(width + 20)) - 10
                    let y = CGFloat((i * 17 + tick * 8) % Int(height))
                    var ray = Path()
                    ray.move(to: CGPoint(x: x, y: y))
                    ray.addLine(to: CGPoint(x: x + 2, y: y + 3)) // angled
                    context.stroke(ray, with: .color(Color(red: 0.6, green: 0.65, blue: 0.8).opacity(0.35)), lineWidth: 0.5)
                }
                // Wind streaks
                for i in 0..<5 {
                    let x = CGFloat((i * 47 + tick * 3) % Int(width + 40)) - 20
                    let y = CGFloat(5 + i * 5)
                    context.stroke(
                        Path { p in p.move(to: CGPoint(x: x, y: y)); p.addLine(to: CGPoint(x: x + 10, y: y)) },
                        with: .color(.white.opacity(0.08)),
                        lineWidth: 0.5
                    )
                }
                // Lightning flash every ~5 seconds
                if (tick / 10) % 50 < 2 {
                    context.fill(
                        Path(CGRect(x: 0, y: 0, width: size.width, height: size.height)),
                        with: .color(.white.opacity(0.2))
                    )
                }
            case .cloudy:
                // Drifting pixel clouds
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
                // Twinkling stars
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
                break // sun handled separately
            }
        }
        .frame(width: width, height: height)
        .allowsHitTesting(false)
    }
}

// Pixel moon for night mode
struct PixelMoon: View {
    var body: some View {
        Canvas { context, size in
            let cx = size.width / 2
            let cy = size.height / 2
            // Crescent moon — circle with dark cutout
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

// Animated pixel sun — beams extend and shrink
struct PixelSun: View {
    let tick: Int

    // Two frames: short rays and long rays, alternating
    private var isExtended: Bool { (tick / 8) % 2 == 0 }

    var body: some View {
        Canvas { context, size in
            let cx = size.width / 2
            let cy = size.height / 2
            let sunColor = Color(red: 1.0, green: 0.85, blue: 0.3)
            let rayColor = Color(red: 1.0, green: 0.78, blue: 0.2)

            // Sun body — 3x3 pixel block
            context.fill(
                Path(CGRect(x: cx - 2, y: cy - 2, width: 4, height: 4)),
                with: .color(sunColor)
            )

            let len: CGFloat = isExtended ? 3 : 1

            // Cardinal rays (up, down, left, right)
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

            // Diagonal rays (shorter)
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

// Simple Japanese tile roof — slanted with overhang
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

            // Wind offset for stormy weather — awning sways
            let windOffset = isStormy ? CGFloat(Darwin.sin(Double(tick) * 0.3)) * 1.5 : 0

            // Main roof slope
            let roofPath = Path { p in
                p.move(to: CGPoint(x: -2 + windOffset, y: h))
                p.addLine(to: CGPoint(x: w * 0.15 + windOffset * 0.5, y: 1))
                p.addLine(to: CGPoint(x: w * 0.85 + windOffset * 0.5, y: 1))
                p.addLine(to: CGPoint(x: w + 2 + windOffset, y: h))
                p.closeSubpath()
            }
            context.fill(roofPath, with: .color(tileColor))

            // Tile ridge line at top
            let ridge = Path { p in
                p.move(to: CGPoint(x: w * 0.15, y: 1))
                p.addLine(to: CGPoint(x: w * 0.85, y: 1))
            }
            context.stroke(ridge, with: .color(tileEdge), lineWidth: 1.5)

            // Horizontal tile lines
            for i in 1..<4 {
                let y = CGFloat(i) * h / 4
                let inset = (1.0 - y / h) * w * 0.15
                let line = Path { p in
                    p.move(to: CGPoint(x: inset - 1, y: y))
                    p.addLine(to: CGPoint(x: w - inset + 1, y: y))
                }
                context.stroke(line, with: .color(tileDark), lineWidth: 0.5)
            }

            // Bottom edge (eave)
            let eave = Path { p in
                p.move(to: CGPoint(x: -2, y: h - 0.5))
                p.addLine(to: CGPoint(x: w + 2, y: h - 0.5))
            }
            context.stroke(eave, with: .color(tileDark), lineWidth: 1)
        }
    }
}

struct CollapsedView: View {
    @Bindable var model: AppModel
    @State private var people: [PixelPerson] = []
    @State private var timer: Timer?
    @State private var tick: Int = 0

    // Layout: total height is 38px
    // Ground strip: bottom 8px
    // Sprites stand ON the ground (feet touch top of ground strip)
    private let groundHeight: CGFloat = 8
    private let shopX: CGFloat = 4

    private var skyColor: Color {
        switch model.sceneWeather {
        case .sunny:  return Color(red: 0.95, green: 0.89, blue: 0.76)
        case .cloudy: return Color(red: 0.82, green: 0.82, blue: 0.82)
        case .rainy:  return Color(red: 0.65, green: 0.68, blue: 0.72)
        case .snowy:  return Color(red: 0.90, green: 0.92, blue: 0.95)
        case .stormy: return Color(red: 0.45, green: 0.48, blue: 0.55)
        case .night:  return Color(red: 0.10, green: 0.10, blue: 0.18)
        }
    }

    private var groundColor: Color {
        switch model.sceneWeather {
        case .snowy:  return Color(red: 0.92, green: 0.93, blue: 0.95)
        case .night:  return Color(red: 0.18, green: 0.17, blue: 0.22)
        case .rainy, .stormy: return Color(red: 0.55, green: 0.55, blue: 0.58)
        default:      return Color(red: 0.78, green: 0.72, blue: 0.60)
        }
    }

    var body: some View {
        GeometryReader { geo in
            let h = geo.size.height
            let groundTop = h - groundHeight

            ZStack(alignment: .topLeading) {
                // Sky
                skyColor

                // Ground / road
                Rectangle()
                    .fill(groundColor)
                    .frame(height: groundHeight)
                    .offset(y: groundTop)

                // Road dashes (static)
                HStack(spacing: 16) {
                    ForEach(0..<15, id: \.self) { _ in
                        Rectangle()
                            .fill(groundColor.opacity(0.7))
                            .frame(width: 6, height: 1)
                    }
                }
                .offset(y: groundTop + 4)

                // Weather particles
                WeatherParticles(weather: model.sceneWeather, tick: tick, width: geo.size.width, height: groundTop)

                // Rainy: puddles on road
                if model.sceneWeather == .rainy || model.sceneWeather == .stormy {
                    ForEach(0..<4, id: \.self) { i in
                        Ellipse()
                            .fill(Color(red: 0.45, green: 0.50, blue: 0.58).opacity(0.3))
                            .frame(width: CGFloat(6 + i % 3 * 2), height: 2)
                            .offset(x: CGFloat(50 + i * 45), y: groundTop + 2)
                    }
                }

                // Dagashi shop icon (left, sitting on ground)
                if let iconURL = Bundle.module.url(forResource: "dagashi-icon", withExtension: "png", subdirectory: "Resources"),
                   let nsImage = NSImage(contentsOf: iconURL) {
                    ZStack(alignment: .top) {
                        // Store body
                        Image(nsImage: nsImage)
                            .resizable()
                            .interpolation(.none)
                            .aspectRatio(contentMode: .fit)
                            .frame(height: 18)
                            .offset(y: 6)

                        // Tile roof
                        PixelRoof(isStormy: model.sceneWeather == .stormy, tick: tick)
                            .frame(width: 32, height: 7)

                        // Snow on roof
                        if model.sceneWeather == .snowy {
                            Canvas { ctx, size in
                                ctx.fill(
                                    Path(CGRect(x: 2, y: 0, width: 28, height: 2)),
                                    with: .color(.white.opacity(0.85))
                                )
                                ctx.fill(
                                    Path(CGRect(x: 4, y: -1, width: 24, height: 1)),
                                    with: .color(.white.opacity(0.6))
                                )
                            }
                            .frame(width: 32, height: 3)
                        }

                        // Night: warm glow from shop windows
                        if model.sceneWeather == .night {
                            Rectangle()
                                .fill(Color(red: 1.0, green: 0.85, blue: 0.4).opacity(0.15))
                                .frame(width: 26, height: 14)
                                .blur(radius: 3)
                                .offset(y: 10)
                        }
                    }
                    .offset(x: shopX, y: groundTop - 24)
                }

                // Sprites — walking on road
                ForEach(people) { person in
                    spriteView(person)
                        .offset(
                            x: person.x,
                            y: groundTop - spriteHeight(person) + 3
                        )
                }

                // Cheering effect pixels above crowd
                if (model.crowdState == .cheering || model.crowdState == .gathering) && !people.isEmpty {
                    let cheerCount = min(people.count, 8)
                    ForEach(0..<cheerCount, id: \.self) { i in
                        if i < people.count {
                            CheerPixels(tick: tick, index: i)
                                .offset(
                                    x: people[i].x + 1,
                                    y: groundTop - 14
                                )
                        }
                    }
                }

                // Sky element (top right) — sun, moon, or clouds
                if model.sceneWeather == .night {
                    PixelMoon()
                        .offset(x: geo.size.width - 22, y: 3)
                } else if model.sceneWeather == .sunny {
                    PixelSun(tick: tick)
                        .offset(x: geo.size.width - 22, y: 3)
                }
            }
        }
        .onAppear { startAnimation() }
        .onDisappear { timer?.invalidate() }
    }

    private func spriteHeight(_ person: PixelPerson) -> CGFloat {
        switch person.kind {
        case .walker: return hasUmbrellas ? 15 : 10
        case .biker:  return 10
        }
    }

    private var hasUmbrellas: Bool {
        model.sceneWeather == .rainy || model.sceneWeather == .stormy
    }

    @ViewBuilder
    private func spriteView(_ person: PixelPerson) -> some View {
        switch person.kind {
        case .walker:
            VStack(spacing: 0) {
                // Umbrella — wide, tilted, with handle gap
                if hasUmbrellas {
                    Canvas { ctx, size in
                        // Wide dome — tilted to one side
                        ctx.fill(Path(CGRect(x: 0, y: 2, width: 10, height: 1)),
                            with: .color(person.color))
                        ctx.fill(Path(CGRect(x: 1, y: 1, width: 8, height: 1)),
                            with: .color(person.color))
                        ctx.fill(Path(CGRect(x: 3, y: 0, width: 4, height: 1)),
                            with: .color(person.color))
                        // Handle stick
                        ctx.fill(Path(CGRect(x: 5, y: 3, width: 1, height: 2)),
                            with: .color(Color(red: 0.4, green: 0.35, blue: 0.3)))
                    }
                    .frame(width: 10, height: 5)
                    .offset(x: person.direction > 0 ? -2 : 1) // tilt based on direction
                }

                // Person body
                Circle()
                    .fill(Color(red: 0.9, green: 0.8, blue: 0.7))
                    .frame(width: 3, height: 3)
                Rectangle()
                    .fill(person.color)
                    .frame(width: 3, height: 4)
                HStack(spacing: 1) {
                    Rectangle()
                        .fill(Color(red: 0.35, green: 0.3, blue: 0.25))
                        .frame(width: 1, height: tick % 4 < 2 ? 3 : 2)
                    Rectangle()
                        .fill(Color(red: 0.35, green: 0.3, blue: 0.25))
                        .frame(width: 1, height: tick % 4 < 2 ? 2 : 3)
                }
            }
            .scaleEffect(x: person.direction, y: 1)

        case .biker:
            // Rider on top, bike frame in middle, wheels at bottom
            VStack(spacing: 0) {
                // Rider
                Circle()
                    .fill(Color(red: 0.9, green: 0.8, blue: 0.7))
                    .frame(width: 2, height: 2)
                Rectangle()
                    .fill(person.color)
                    .frame(width: 3, height: 3)
                // Bike frame bar
                Rectangle()
                    .fill(Color(red: 0.45, green: 0.42, blue: 0.38))
                    .frame(width: 8, height: 1)
                // Wheels side by side
                HStack(spacing: 3) {
                    RoundedRectangle(cornerRadius: 1)
                        .fill(Color(red: 0.35, green: 0.32, blue: 0.28))
                        .frame(width: 3, height: 3)
                    RoundedRectangle(cornerRadius: 1)
                        .fill(Color(red: 0.35, green: 0.32, blue: 0.28))
                        .frame(width: 3, height: 3)
                }
            }
            .scaleEffect(x: person.direction, y: 1)
        }
    }

    // Each person gets a target X when gathering (spread across shop front)
    private let shopAreaLeft: CGFloat = 30
    private let shopAreaRight: CGFloat = 120

    private func gatherTarget(for index: Int) -> CGFloat {
        // Spread evenly across the shop front area
        let spread = shopAreaRight - shopAreaLeft
        let slot = CGFloat(index % 10) / 10.0
        return shopAreaLeft + slot * spread
    }

    private func startAnimation() {
        timer = Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { _ in
            tick += 1
            let crowd = model.crowdState

            for i in people.indices {
                switch crowd {
                case .gathering, .cheering:
                    // Move toward spread-out target positions
                    let target = gatherTarget(for: i)
                    let dx = target - people[i].x
                    if abs(dx) > 3 {
                        people[i].x += (dx > 0 ? 1 : -1) * min(abs(dx) * 0.08, 1.5)
                    }
                    // Cheering bounce — hop up and down via direction field hack
                    if crowd == .cheering {
                        // Small side-to-side bounce
                        people[i].x += CGFloat((tick + i * 3) % 6 < 3 ? 1 : -1) * 0.3
                    }
                case .dispersing:
                    // Walk away — alternate directions
                    let awayDir: CGFloat = i % 2 == 0 ? -1 : 1
                    people[i].x += awayDir * 0.7
                case .idle:
                    people[i].x += people[i].speed * people[i].direction
                }
            }

            people.removeAll { p in p.x < -20 || p.x > 240 }

            switch crowd {
            case .gathering:
                if tick % 6 == 0 && people.count < 10 {
                    if tick % 18 == 0 { spawnBiker() } else { spawnPerson() }
                }
            case .cheering:
                if people.count < 8 { spawnPerson() }
            case .dispersing:
                break
            case .idle:
                if tick % 60 == 0 && people.filter({ $0.kind == .walker }).count < 3 {
                    spawnPerson()
                }
                if tick % 150 == 0 && people.filter({ $0.kind == .biker }).count < 1 {
                    spawnBiker()
                }
                if tick == 10 { spawnPerson() }
            }
        }
    }

    private func spawnPerson() {
        let fromLeft = Bool.random()
        let colors: [Color] = [
            Color(red: 0.8, green: 0.3, blue: 0.3),
            Color(red: 0.3, green: 0.5, blue: 0.8),
            Color(red: 0.3, green: 0.7, blue: 0.4),
            Color(red: 0.8, green: 0.7, blue: 0.3),
            Color(red: 0.6, green: 0.3, blue: 0.7),
        ]

        people.append(PixelPerson(
            x: fromLeft ? -10 : 230,
            speed: CGFloat.random(in: 0.3...0.6),
            direction: fromLeft ? 1 : -1,
            color: colors.randomElement()!,
            kind: .walker
        ))
    }

    private func spawnBiker() {
        let fromLeft = Bool.random()
        people.append(PixelPerson(
            x: fromLeft ? -15 : 240,
            speed: CGFloat.random(in: 1.0...1.5),
            direction: fromLeft ? 1 : -1,
            color: Color(red: 0.4, green: 0.4, blue: 0.45),
            kind: .biker
        ))
    }
}
