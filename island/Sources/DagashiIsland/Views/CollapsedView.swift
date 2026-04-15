import SwiftUI

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

    private var closeButtonFg: Color {
        switch model.sceneWeather {
        case .night:          return Color(red: 0.7, green: 0.7, blue: 0.8)
        case .stormy:         return Color(red: 0.85, green: 0.85, blue: 0.85)
        case .rainy:          return Color(red: 0.3, green: 0.3, blue: 0.35)
        case .snowy:          return Color(red: 0.4, green: 0.4, blue: 0.5)
        default:              return Color(red: 0.4, green: 0.32, blue: 0.2)
        }
    }

    private var closeButtonBg: Color {
        switch model.sceneWeather {
        case .night:          return Color(red: 0.2, green: 0.2, blue: 0.3).opacity(0.7)
        case .stormy:         return Color(red: 0.3, green: 0.32, blue: 0.38).opacity(0.7)
        case .rainy:          return Color(red: 0.75, green: 0.77, blue: 0.8).opacity(0.7)
        case .snowy:          return Color.white.opacity(0.6)
        default:              return Color(red: 0.88, green: 0.82, blue: 0.7).opacity(0.7)
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

                // Close button — appears on hover, top-right
                if model.isHovering {
                    Button(action: {
                        model.onHideIsland?()
                    }) {
                        Text("×")
                            .font(.system(size: 9, weight: .bold, design: .monospaced))
                            .foregroundColor(closeButtonFg)
                            .frame(width: 14, height: 14)
                            .background(closeButtonBg)
                            .clipShape(RoundedRectangle(cornerRadius: 3))
                    }
                    .buttonStyle(.plain)
                    .offset(x: geo.size.width - 18, y: 3)
                    .transition(.opacity)
                }
            }
        }
        .animation(.easeInOut(duration: 0.15), value: model.isHovering)
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
            WalkerSprite(person: person, tick: tick, hasUmbrellas: hasUmbrellas)
        case .biker:
            BikerSprite(person: person)
        }
    }

    // Each person gets a target X when gathering (spread across shop front)
    private let shopAreaLeft: CGFloat = 30
    private let shopAreaRight: CGFloat = 120

    private func gatherTarget(for index: Int) -> CGFloat {
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
                    let target = gatherTarget(for: i)
                    let dx = target - people[i].x
                    if abs(dx) > 3 {
                        people[i].x += (dx > 0 ? 1 : -1) * min(abs(dx) * 0.08, 1.5)
                    }
                    if crowd == .cheering {
                        people[i].x += CGFloat((tick + i * 3) % 6 < 3 ? 1 : -1) * 0.3
                    }
                case .dispersing:
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
