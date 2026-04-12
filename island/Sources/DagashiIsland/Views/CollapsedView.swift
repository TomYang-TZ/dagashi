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

// Simple Japanese tile roof — slanted with overhang
struct PixelRoof: View {
    let tileColor = Color(red: 0.35, green: 0.33, blue: 0.38)
    let tileDark  = Color(red: 0.25, green: 0.23, blue: 0.28)
    let tileEdge  = Color(red: 0.45, green: 0.42, blue: 0.46)

    var body: some View {
        Canvas { context, size in
            let w = size.width
            let h = size.height

            // Main roof slope — wider than the store, slight overhang
            let roofPath = Path { p in
                p.move(to: CGPoint(x: -2, y: h))           // bottom-left (overhang)
                p.addLine(to: CGPoint(x: w * 0.15, y: 1))  // top-left slope
                p.addLine(to: CGPoint(x: w * 0.85, y: 1))  // top-right
                p.addLine(to: CGPoint(x: w + 2, y: h))     // bottom-right (overhang)
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

    var body: some View {
        GeometryReader { geo in
            let h = geo.size.height      // 38
            let groundTop = h - groundHeight  // 30

            ZStack(alignment: .topLeading) {
                // Warm sky
                Color(red: 0.95, green: 0.89, blue: 0.76)

                // Ground / road
                Rectangle()
                    .fill(Color(red: 0.78, green: 0.72, blue: 0.60))
                    .frame(height: groundHeight)
                    .offset(y: groundTop)

                // Road dashes (static)
                HStack(spacing: 16) {
                    ForEach(0..<15, id: \.self) { _ in
                        Rectangle()
                            .fill(Color(red: 0.70, green: 0.64, blue: 0.52).opacity(0.5))
                            .frame(width: 6, height: 1)
                    }
                }
                .offset(y: groundTop + 4)

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
                        PixelRoof()
                            .frame(width: 32, height: 7)
                    }
                    .offset(x: shopX, y: groundTop - 24)
                }

                // Sprites — walking on road (centered on ground strip)
                ForEach(people) { person in
                    spriteView(person)
                        .offset(
                            x: person.x,
                            y: groundTop - spriteHeight(person) + 3
                        )
                }

                // Pull count badge
                if model.pullCount > 0 {
                    Text("\(model.pullCount)")
                        .font(.system(size: 6, weight: .bold, design: .monospaced))
                        .foregroundColor(.black)
                        .padding(.horizontal, 3)
                        .padding(.vertical, 1)
                        .background(Color(red: 0.77, green: 0.64, blue: 0.35))
                        .clipShape(Capsule())
                        .offset(x: geo.size.width - 24, y: 4)
                }
            }
        }
        .onAppear { startAnimation() }
        .onDisappear { timer?.invalidate() }
    }

    private func spriteHeight(_ person: PixelPerson) -> CGFloat {
        switch person.kind {
        case .walker: return 10 // head(3) + body(4) + legs(3)
        case .biker:  return 10 // rider(5) + bike(5)
        }
    }

    @ViewBuilder
    private func spriteView(_ person: PixelPerson) -> some View {
        switch person.kind {
        case .walker:
            VStack(spacing: 0) {
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

    private func startAnimation() {
        timer = Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { _ in
            tick += 1

            for i in people.indices {
                people[i].x += people[i].speed * people[i].direction
            }

            people.removeAll { p in p.x < -20 || p.x > 240 }

            // Spawn less frequently — max ~3 on screen
            if tick % 60 == 0 && people.filter({ $0.kind == .walker }).count < 3 {
                spawnPerson()
            }
            if tick % 150 == 0 && people.filter({ $0.kind == .biker }).count < 1 {
                spawnBiker()
            }

            // Seed one walker early
            if tick == 10 { spawnPerson() }
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
