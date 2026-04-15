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

struct WalkerSprite: View {
    let person: PixelPerson
    let tick: Int
    let hasUmbrellas: Bool

    var body: some View {
        VStack(spacing: 0) {
            if hasUmbrellas {
                Canvas { ctx, size in
                    ctx.fill(Path(CGRect(x: 0, y: 2, width: 10, height: 1)),
                        with: .color(person.color))
                    ctx.fill(Path(CGRect(x: 1, y: 1, width: 8, height: 1)),
                        with: .color(person.color))
                    ctx.fill(Path(CGRect(x: 3, y: 0, width: 4, height: 1)),
                        with: .color(person.color))
                    ctx.fill(Path(CGRect(x: 5, y: 3, width: 1, height: 2)),
                        with: .color(Color(red: 0.4, green: 0.35, blue: 0.3)))
                }
                .frame(width: 10, height: 5)
                .offset(x: person.direction > 0 ? -2 : 1)
            }

            Circle()
                .fill(Color(red: 0.9, green: 0.8, blue: 0.7))
                .overlay(Circle().stroke(Color(red: 0.35, green: 0.3, blue: 0.25), lineWidth: 0.5))
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
    }
}

struct BikerSprite: View {
    let person: PixelPerson

    var body: some View {
        VStack(spacing: 0) {
            Circle()
                .fill(Color(red: 0.9, green: 0.8, blue: 0.7))
                .overlay(Circle().stroke(Color(red: 0.35, green: 0.3, blue: 0.25), lineWidth: 0.5))
                .frame(width: 2, height: 2)
            Rectangle()
                .fill(person.color)
                .frame(width: 3, height: 3)
            Rectangle()
                .fill(Color(red: 0.45, green: 0.42, blue: 0.38))
                .frame(width: 8, height: 1)
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
