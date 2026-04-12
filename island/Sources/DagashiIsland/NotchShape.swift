import SwiftUI

struct NotchShape: Shape, Animatable {
    var topRadius: CGFloat
    var bottomRadius: CGFloat

    var animatableData: AnimatablePair<CGFloat, CGFloat> {
        get { AnimatablePair(topRadius, bottomRadius) }
        set { topRadius = newValue.first; bottomRadius = newValue.second }
    }

    static let closed = NotchShape(topRadius: 6, bottomRadius: 20)
    static let opened = NotchShape(topRadius: 22, bottomRadius: 36)

    func path(in rect: CGRect) -> Path {
        var path = Path()
        let w = rect.width
        let h = rect.height
        let tr = min(topRadius, min(w, h) / 2)
        let br = min(bottomRadius, min(w, h) / 2)

        // Start top-left with concave corner (notch edge)
        path.move(to: CGPoint(x: 0, y: 0))
        path.addLine(to: CGPoint(x: tr, y: 0))
        path.addQuadCurve(
            to: CGPoint(x: 0, y: tr),
            control: CGPoint(x: 0, y: 0)
        )

        // Left side down to bottom-left convex corner
        path.addLine(to: CGPoint(x: 0, y: h - br))
        path.addQuadCurve(
            to: CGPoint(x: br, y: h),
            control: CGPoint(x: 0, y: h)
        )

        // Bottom to bottom-right convex corner
        path.addLine(to: CGPoint(x: w - br, y: h))
        path.addQuadCurve(
            to: CGPoint(x: w, y: h - br),
            control: CGPoint(x: w, y: h)
        )

        // Right side up to top-right concave corner
        path.addLine(to: CGPoint(x: w, y: tr))
        path.addQuadCurve(
            to: CGPoint(x: w - tr, y: 0),
            control: CGPoint(x: w, y: 0)
        )

        path.closeSubpath()
        return path
    }
}
