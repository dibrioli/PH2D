//! **The joint overlay** — the sibling of `physics_overlay`, split off it when
//! the two together crossed the HR-18 shell cap.
//!
//! The colliders and the joints answer two different questions with the same
//! technique: *what shape is this, physically?* and *what is this attached to?*
//! Keeping them in one file is what pushed it over; keeping them as neighbours
//! is what keeps the screen-space rule (see the parent module's header) in one
//! place for both.

use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, Point};

use super::physics_overlay::CIRCLE_SEGS;

/// Radius of the ring drawn at each joint anchor, screen px. Big enough to
/// see, small enough not to hide the art under a chain of them.
const JOINT_DOT_PX: f64 = 3.0; // LITERAL-PX-OK: chrome de overlay, raio de tela

/// Joints — amber, so they read as a third thing next to the green scenery and
/// the cyan movers rather than as one of them.
pub(super) const JOINT_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 0.9]; // LITERAL-COLOR-OK: overlay de joint

/// **The joint overlay.** A joint is even less visible than a collider: it is
/// a *relationship*, with no geometry at all. Two objects that are pinned
/// together and two that merely touch look identical until one of them moves.
///
/// Drawn as the segment between the two anchors, with a dot at each end.
///
/// ⚠️ **The anchors come from the SOLVER, not from the joint entity's
/// `Transform`.** The Transform is the *authored* anchor and nothing writes it
/// back during play, so drawing from it would pin the marker where the artist
/// left it while the chain it belongs to swings away — a picture that is wrong
/// in exactly the situation the artist is watching. The live pair also tells
/// the truth about strain: a pin's two anchors coincide until the constraint
/// is fighting, and a rope's are apart by design.
pub(super) fn joint_marks(
    show: bool,
    anchors: &[([f32; 2], [f32; 2])],
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<BezPath> {
    if !show {
        return Vec::new();
    }
    anchors
        .iter()
        .map(|&(a, b)| {
            let to_screen = |wx: f32, wy: f32| {
                let (sx, sy) = camera.world_to_screen([wx, wy], window);
                Point::new(f64::from(sx), f64::from(sy))
            };
            let pa = to_screen(a[0], a[1]);
            let pb = to_screen(b[0], b[1]);
            let mut path = BezPath::new();
            // The link itself.
            path.move_to(pa);
            path.line_to(pb);
            // A small ring at each end, so a joint whose anchors coincide (a
            // pin at rest — the common case) still reads as something rather
            // than as a zero-length line, which draws nothing at all.
            for p in [pa, pb] {
                path.move_to(Point::new(p.x + JOINT_DOT_PX, p.y));
                for i in 1..=CIRCLE_SEGS {
                    let th = std::f64::consts::TAU * f64::from(i) / f64::from(CIRCLE_SEGS);
                    path.line_to(Point::new(
                        p.x + JOINT_DOT_PX * th.cos(),
                        p.y + JOINT_DOT_PX * th.sin(),
                    ));
                }
            }
            path
        })
        .collect()
}

#[cfg(test)]
mod joint_tests {
    //! **A joint is a relationship, and a relationship has no geometry.**
    //!
    //! Two objects pinned together look exactly like two that merely touch —
    //! until one of them moves. Same argument as the collider outline, one
    //! level more abstract.

    use super::super::physics_overlay::CIRCLE_SEGS;
    use super::joint_marks;
    use ph2d_host::WindowSize;
    use ph2d_render::Camera2d;
    use ph2d_vector::PathEl;

    fn window() -> WindowSize {
        WindowSize {
            width: 1000,
            height: 1000,
        }
    }

    fn camera() -> Camera2d {
        Camera2d {
            center: [0.0, 0.0],
            height_world: 10.0,
            ..Camera2d::default()
        }
    }

    fn points(path: &ph2d_vector::BezPath) -> Vec<(f64, f64)> {
        path.elements()
            .iter()
            .filter_map(|el| match el {
                PathEl::MoveTo(p) | PathEl::LineTo(p) => Some((p.x, p.y)),
                _ => None,
            })
            .collect()
    }

    /// **The link is drawn between the two anchors it was given.**
    #[test]
    fn the_joint_mark_spans_its_two_anchors() {
        let marks = joint_marks(true, &[([-1.0, 0.0], [1.0, 0.0])], &camera(), window());
        assert_eq!(marks.len(), 1);
        let pts = points(&marks[0]);
        let (ax, ay) = camera().world_to_screen([-1.0, 0.0], window());
        let (bx, by) = camera().world_to_screen([1.0, 0.0], window());
        assert_eq!(pts[0], (f64::from(ax), f64::from(ay)));
        assert_eq!(pts[1], (f64::from(bx), f64::from(by)));
    }

    /// **A pin at rest still draws something.**
    ///
    /// Its two anchors are the SAME world point — that is what a pin is — so
    /// the segment between them has zero length and paints nothing at all. The
    /// rings are what keep the most common joint in the editor from being
    /// invisible.
    #[test]
    fn a_joint_whose_anchors_coincide_is_still_visible() {
        let marks = joint_marks(true, &[([2.0, 1.0], [2.0, 1.0])], &camera(), window());
        let pts = points(&marks[0]);
        let (cx, cy) = camera().world_to_screen([2.0, 1.0], window());
        let (cx, cy) = (f64::from(cx), f64::from(cy));
        let spread = pts
            .iter()
            .map(|(x, y)| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt())
            .fold(0.0f64, f64::max);
        assert!(
            spread > 1.0,
            "a pin at rest painted {} points all within {spread:.3} px of its \
             anchor — a zero-length segment draws nothing, and the most common \
             joint in the editor would be invisible",
            pts.len()
        );
        // Two rings, one per anchor, plus the (degenerate) segment.
        assert_eq!(pts.len(), 2 + 2 * (CIRCLE_SEGS as usize + 1));
    }

    /// **Switched off draws nothing** — the same toggle the colliders obey.
    #[test]
    fn the_toggle_silences_the_joint_marks_too() {
        assert!(joint_marks(false, &[([0.0, 0.0], [1.0, 1.0])], &camera(), window()).is_empty());
    }

    /// **A scene with no joints costs nothing.**
    #[test]
    fn no_joints_no_paths() {
        assert!(joint_marks(true, &[], &camera(), window()).is_empty());
    }
}
