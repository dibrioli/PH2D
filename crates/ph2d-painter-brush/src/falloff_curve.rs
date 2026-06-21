//! Editable custom falloff curve — the [`Falloff::Custom`](crate::Falloff::Custom) profile.
//!
//! Behavioural reference (clean-room, no code copied): Blender's brush falloff
//! "Custom" `CurveMapping` (`blenkernel/intern/colortools.cc`
//! `BKE_curvemapping_evaluate`, sampled into the per-texel dab mask by
//! `editors/sculpt_paint/mesh/paint_image_2d_curve_mask.cc`). `x` is the
//! normalized distance from the dab centre (`0` = centre, `1` = rim); `y` the
//! strength.
//!
//! Each control point carries a **stable id** so a handle stays "grabbed" while
//! the artist drags it PAST a neighbour (the points re-sort by x, but the id —
//! and thus the widget the pointer holds — does not change). Points also carry a
//! [`HandleType`] (Blender's per-point handle): `Auto` is a smooth Catmull-Rom
//! tangent (free to bump, unlike a clamped tone curve), `Vector` makes the
//! adjacent segments run straight into the point (a sharp corner) — the
//! Hermite-form equivalent of Blender's `HD_VECT` handle pointing at the
//! neighbour.

/// Maximum control points in a custom falloff curve (keeps [`FalloffCurve`]
/// `Copy` and alloc-free).
pub const MAX_FALLOFF_POINTS: usize = 8;

/// Per-point handle (Blender's `HD_AUTO` / `HD_VECT`). Drives the spline tangent
/// at the point: smooth vs. a straight-line corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum HandleType {
    /// Smooth auto tangent (Catmull-Rom central difference). Blender `HD_AUTO`.
    #[default]
    Auto = 0,
    /// Straight handle — the adjacent segments run linearly into this point,
    /// making a sharp corner. Blender `HD_VECT`.
    Vector = 1,
}

/// Number of handle types (for the right-click menu's wire range).
pub const MAX_HANDLE_TYPES: u8 = 2;

impl HandleType {
    /// The stable wire discriminant (`0` = Auto, `1` = Vector).
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Handle type for a wire discriminant; out-of-range → [`Self::Auto`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Vector,
            _ => Self::Auto,
        }
    }

    /// Display name for the right-click handle menu (English UI per HR-15).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Vector => "Vector",
        }
    }
}

/// One control point of a [`FalloffCurve`]: a stable `id`, its `[x, y]` in
/// `[0,1]²` (`x` = distance, `y` = strength), and its [`HandleType`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FalloffPoint {
    /// Stable identity — survives the re-sort when a point is dragged past
    /// another, so the dragged widget stays grabbed.
    pub id: u8,
    /// Distance from the dab centre, `0..=1`.
    pub x: f32,
    /// Strength, `0..=1`.
    pub y: f32,
    /// Smooth (`Auto`) or sharp-corner (`Vector`) handle.
    pub handle: HandleType,
}

impl Default for FalloffPoint {
    fn default() -> Self {
        Self {
            id: 0,
            x: 0.0,
            y: 0.0,
            handle: HandleType::Auto,
        }
    }
}

/// An editable custom falloff profile: 2..=[`MAX_FALLOFF_POINTS`] control points,
/// ascending by `x`, evaluated as a cubic-Hermite spline with per-point handles.
/// Cheap to copy — the stroke engine reads it per dab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FalloffCurve {
    points: [FalloffPoint; MAX_FALLOFF_POINTS],
    len: u8,
}

impl Default for FalloffCurve {
    /// A linear ramp from full strength at the centre to zero at the rim
    /// (`(0,1) → (1,0)`, both `Auto`) — the clearest editable starting shape.
    fn default() -> Self {
        let mut points = [FalloffPoint::default(); MAX_FALLOFF_POINTS];
        points[0] = FalloffPoint {
            id: 0,
            x: 0.0,
            y: 1.0,
            handle: HandleType::Auto,
        };
        points[1] = FalloffPoint {
            id: 1,
            x: 1.0,
            y: 0.0,
            handle: HandleType::Auto,
        };
        Self { points, len: 2 }
    }
}

impl FalloffCurve {
    /// The active control points, ascending by `x`.
    #[must_use]
    pub fn points(&self) -> &[FalloffPoint] {
        &self.points[..self.len as usize]
    }

    /// Number of active control points (`2..=MAX_FALLOFF_POINTS`).
    #[must_use]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// A falloff curve always keeps ≥2 points, so this is never `true`; provided
    /// only to satisfy `clippy::len_without_is_empty`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The handle type of point `id`, if present.
    #[must_use]
    pub fn handle_of(&self, id: u8) -> Option<HandleType> {
        self.points().iter().find(|p| p.id == id).map(|p| p.handle)
    }

    /// Strength at normalized distance `t` (`0` = centre, `1` = rim). Forced to
    /// `0` at/after the rim (`t >= 1`) so a dab never paints beyond its radius,
    /// matching the preset [`Falloff::weight`](crate::Falloff::weight).
    #[must_use]
    pub fn weight(&self, t: f32) -> f32 {
        if t >= 1.0 {
            return 0.0;
        }
        eval_falloff_curve(self.points(), t.max(0.0))
    }

    /// Move control point `id` to `(x, y)` — clamped to `[0,1]²` only, NOT
    /// between its neighbours, then the points re-sort by `x`. Dragging a point
    /// past another therefore reshapes the curve; the `id` keeps the dragged
    /// widget grabbed through the re-sort. No-op for an unknown `id`.
    pub fn set_point(&mut self, id: u8, x: f32, y: f32) {
        let n = self.len as usize;
        let Some(i) = self.points[..n].iter().position(|p| p.id == id) else {
            return;
        };
        self.points[i].x = x.clamp(0.0, 1.0);
        self.points[i].y = y.clamp(0.0, 1.0);
        self.sort_by_x();
    }

    /// Set the handle type of point `id`. No-op for an unknown `id`.
    pub fn set_handle(&mut self, id: u8, handle: HandleType) {
        let n = self.len as usize;
        if let Some(p) = self.points[..n].iter_mut().find(|p| p.id == id) {
            p.handle = handle;
        }
    }

    /// Insert a control point at the midpoint of the widest distance-gap, its
    /// strength sampled ON the current curve (so the profile is unchanged until
    /// the new point is dragged). Returns the new point's stable id, or `None` at
    /// the [`MAX_FALLOFF_POINTS`] cap.
    pub fn add_point(&mut self) -> Option<u8> {
        let n = self.len as usize;
        if n < 2 {
            return None;
        }
        let mut best_gap = -1.0_f32;
        let mut new_x = 0.5_f32;
        for i in 0..n - 1 {
            let gap = self.points[i + 1].x - self.points[i].x;
            if gap > best_gap {
                best_gap = gap;
                new_x = (self.points[i].x + self.points[i + 1].x) * 0.5;
            }
        }
        self.add_point_at(new_x, eval_falloff_curve(self.points(), new_x))
    }

    /// Insert a control point at `(x, y)` (clamped to `[0,1]²`), re-sorting by
    /// `x`. Returns the new point's stable id, or `None` at the cap. Used by a
    /// click on the curve canvas (add where the artist clicked).
    pub fn add_point_at(&mut self, x: f32, y: f32) -> Option<u8> {
        let n = self.len as usize;
        if n >= MAX_FALLOFF_POINTS {
            return None;
        }
        let id = self.alloc_id();
        self.points[n] = FalloffPoint {
            id,
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
            handle: HandleType::Auto,
        };
        self.len += 1;
        self.sort_by_x();
        Some(id)
    }

    /// Remove control point `id`. No-op when only the two endpoints remain (a
    /// curve needs ≥2 points) or for an unknown `id`.
    pub fn remove_point(&mut self, id: u8) {
        let n = self.len as usize;
        if n <= 2 {
            return;
        }
        let Some(i) = self.points[..n].iter().position(|p| p.id == id) else {
            return;
        };
        for j in i..n - 1 {
            self.points[j] = self.points[j + 1];
        }
        self.points[n - 1] = FalloffPoint::default();
        self.len -= 1;
    }

    /// Smallest id not currently in use (≤8 points → always one free in `0..=255`).
    fn alloc_id(&self) -> u8 {
        let n = self.len as usize;
        (0u8..=u8::MAX)
            .find(|id| !self.points[..n].iter().any(|p| p.id == *id))
            .unwrap_or(0)
    }

    /// Stable insertion sort by `x` (≤8 points). Keeps ids attached to points.
    fn sort_by_x(&mut self) {
        let n = self.len as usize;
        for i in 1..n {
            let mut j = i;
            while j > 0 && self.points[j - 1].x > self.points[j].x {
                self.points.swap(j - 1, j);
                j -= 1;
            }
        }
    }
}

/// Secant slope between points `a` and `b` (0 for a vertical/degenerate gap).
fn secant(points: &[FalloffPoint], a: usize, b: usize) -> f32 {
    let dx = points[b].x - points[a].x;
    if dx.abs() <= 1e-9 {
        0.0
    } else {
        (points[b].y - points[a].y) / dx
    }
}

/// Smooth auto tangent at point `i` (Catmull-Rom central difference; free to
/// overshoot so a lifted middle point makes a bump, like a general curve editor).
fn auto_tangent(points: &[FalloffPoint], i: usize) -> f32 {
    let n = points.len();
    if n < 2 {
        return 0.0;
    }
    if i == 0 {
        secant(points, 0, 1)
    } else if i == n - 1 {
        secant(points, n - 2, n - 1)
    } else {
        let dx = points[i + 1].x - points[i - 1].x;
        if dx.abs() <= 1e-9 {
            0.0
        } else {
            (points[i + 1].y - points[i - 1].y) / dx
        }
    }
}

/// Tangent at point `i` for the segment toward neighbour `toward`. A `Vector`
/// handle points straight at the neighbour (secant) → a sharp linear corner; an
/// `Auto` handle uses the smooth [`auto_tangent`].
fn tangent_for_segment(points: &[FalloffPoint], i: usize, toward: usize) -> f32 {
    if points[i].handle == HandleType::Vector {
        secant(points, i, toward)
    } else {
        auto_tangent(points, i)
    }
}

/// Evaluate a falloff curve (points ascending by `x`) at `t`. Empty → `1`; one
/// point → its strength; ≥2 → a cubic-Hermite spline with per-point handles,
/// endpoints extended flat. The panel plots the live preview with this, so the
/// graph matches the dab.
#[must_use]
pub fn eval_falloff_curve(points: &[FalloffPoint], t: f32) -> f32 {
    match points.len() {
        0 => return 1.0,
        1 => return points[0].y.clamp(0.0, 1.0),
        _ => {}
    }
    let t = t.clamp(0.0, 1.0);
    let n = points.len();
    if t <= points[0].x {
        return points[0].y.clamp(0.0, 1.0);
    }
    if t >= points[n - 1].x {
        return points[n - 1].y.clamp(0.0, 1.0);
    }
    let mut i = 0;
    while i + 1 < n && points[i + 1].x < t {
        i += 1;
    }
    let (x0, y0) = (points[i].x, points[i].y);
    let (x1, y1) = (points[i + 1].x, points[i + 1].y);
    let h = x1 - x0;
    if h <= 1e-9 {
        return y1.clamp(0.0, 1.0);
    }
    let m0 = tangent_for_segment(points, i, i + 1);
    let m1 = tangent_for_segment(points, i + 1, i);
    let s = (t - x0) / h;
    let (s2, s3) = (s * s, s * s * s);
    let h00 = 2.0 * s3 - 3.0 * s2 + 1.0;
    let h10 = s3 - 2.0 * s2 + s;
    let h01 = -2.0 * s3 + 3.0 * s2;
    let h11 = s3 - s2;
    (h00 * y0 + h10 * h * m0 + h01 * y1 + h11 * h * m1).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_linear_ramp() {
        let c = FalloffCurve::default();
        assert_eq!(c.len(), 2);
        assert!((c.weight(0.0) - 1.0).abs() < 1e-6, "centre full");
        assert_eq!(c.weight(1.0), 0.0, "rim zero (forced)");
        assert!((c.weight(0.5) - 0.5).abs() < 1e-6, "linear midpoint");
    }

    #[test]
    fn ids_are_stable_and_unique() {
        let mut c = FalloffCurve::default();
        let id = c.add_point_at(0.5, 0.5).expect("inserted");
        assert_eq!(c.len(), 3);
        // ids unique.
        let ids: Vec<u8> = c.points().iter().map(|p| p.id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len(), "ids unique");
        // The new point keeps its id after a re-sorting drag.
        c.set_point(id, 0.9, 0.2);
        assert!(c.points().iter().any(|p| p.id == id), "id survives re-sort");
    }

    #[test]
    fn drag_past_neighbour_reorders_and_keeps_id() {
        let mut c = FalloffCurve::default();
        let mid = c.add_point_at(0.5, 0.5).unwrap(); // points: 0.0, 0.5, 1.0
        // Drag the middle point past the right endpoint (x=1.0 → clamps to 1.0,
        // tying it). Points stay ascending; the dragged id is still present.
        c.set_point(mid, 1.0, 0.3);
        let xs: Vec<f32> = c.points().iter().map(|p| p.x).collect();
        for w in xs.windows(2) {
            assert!(w[0] <= w[1] + 1e-6, "points stay ascending: {xs:?}");
        }
        assert!(c.points().iter().any(|p| p.id == mid));
    }

    #[test]
    fn lifted_middle_point_makes_a_bump() {
        // A general curve editor (not a clamped tone curve): a middle point above
        // BOTH neighbours must create a hump (non-monotone). Lower the left
        // endpoint first so the middle can rise above both ends.
        let mut c = FalloffCurve::default();
        c.set_point(0, 0.0, 0.2); // left endpoint down
        let mid = c.add_point_at(0.5, 0.8).unwrap(); // peak above both ends
        assert_eq!(c.handle_of(mid), Some(HandleType::Auto));
        // (0,0.2) → (0.5,0.8) → (1,0): the curve rises to the peak then falls.
        assert!(c.weight(0.5) > c.weight(0.1), "rises toward the peak");
        assert!(c.weight(0.5) > c.weight(0.9), "falls after the peak");
    }

    #[test]
    fn vector_handle_makes_a_straight_corner() {
        // Slope just left and just right of a control point at (0.5, 0.9). A small
        // step so an Auto (C1) point reads as continuous — only a Vector corner
        // shows a slope jump.
        let slopes = |c: &FalloffCurve| {
            let l = (c.weight(0.5) - c.weight(0.49)) / 0.01;
            let r = (c.weight(0.51) - c.weight(0.5)) / 0.01;
            (l, r)
        };
        let mut c = FalloffCurve::default();
        let mid = c.add_point_at(0.5, 0.9).unwrap();

        // Auto handle: the spline is C1 → the slope is continuous (no corner).
        let (al, ar) = slopes(&c);
        assert!((al - ar).abs() < 0.3, "Auto is smooth: {al} vs {ar}");

        // Vector handle: both adjacent segments run straight into the point, with
        // DIFFERENT slopes → a real sharp corner (slope discontinuity). Left ≈
        // −0.2 (line (0,1)→(0.5,0.9)), right ≈ −1.8 (line (0.5,0.9)→(1,0)).
        c.set_handle(mid, HandleType::Vector);
        assert!(
            (c.weight(0.25) - 0.95).abs() < 0.02,
            "left segment straight, got {}",
            c.weight(0.25)
        );
        assert!(
            (c.weight(0.75) - 0.45).abs() < 0.02,
            "right segment straight, got {}",
            c.weight(0.75)
        );
        let (vl, vr) = slopes(&c);
        assert!(
            (vl - vr).abs() > 1.0,
            "Vector is a sharp corner: {vl} vs {vr}"
        );
    }

    #[test]
    fn weight_forces_zero_at_and_past_rim() {
        let mut c = FalloffCurve::default();
        c.set_point(1, 1.0, 1.0);
        assert_eq!(c.weight(1.0), 0.0);
        assert_eq!(c.weight(1.5), 0.0);
    }

    #[test]
    fn add_caps_and_remove_keeps_two() {
        let mut c = FalloffCurve::default();
        while c.add_point().is_some() {}
        assert_eq!(c.len(), MAX_FALLOFF_POINTS);
        assert!(c.add_point().is_none(), "no insert past the cap");
        let ids: Vec<u8> = c.points().iter().map(|p| p.id).collect();
        for id in ids {
            c.remove_point(id);
        }
        assert_eq!(c.len(), 2, "a curve never drops below 2 points");
    }

    #[test]
    fn handle_wire_round_trips() {
        for v in 0..MAX_HANDLE_TYPES {
            assert_eq!(HandleType::from_u8(v).to_u8(), v);
            assert!(!HandleType::from_u8(v).name().is_empty());
        }
        assert_eq!(HandleType::from_u8(99), HandleType::Auto, "OOR → Auto");
    }
}
