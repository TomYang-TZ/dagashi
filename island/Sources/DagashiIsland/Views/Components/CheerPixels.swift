import SwiftUI

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
