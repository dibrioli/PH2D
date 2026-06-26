//! The stroke **heading** — a stable per-dab direction for "rotation follows the stroke" (the texture
//! **Rake**). Computed in the engine, where the path tangent is known, then stamped on every [`crate::Dab`].
//!
//! Behavioural reference (clean-room, no code copied): MyPaint's `direction` state input filters the
//! velocity *vector* with a length-weighted low-pass, then recovers the angle; Krita's "Fade" ties that
//! length scale to the brush size; Blender's rake holds the last heading when motion stalls. We take the
//! best of each: a **length-weighted exponential moving average of the unit path tangent, in vector
//! space** (wrap-safe — a vector lerp never spins the long way round a 180° flip the way an angle lerp
//! would), with the smoothing length proportional to the brush diameter.
//!
//! Why length-weighted (not per-dab / per-event): the smoothing must depend on distance travelled, not on
//! how many dabs the spacing happens to place, so the same physical stroke reads identically whether the
//! brush is sparse or dense. Why in the engine: the raw inter-dab chord is only ~3px (dabs ride a
//! stabilizer-smoothed spline) so its *direction* is noise; the engine sees the path geometry directly.
//!
//! HR-5: transcendental-free. `sqrt` (for normalisation) is allowed; the EMA blend `α = Δs/(Δs + L)` is a
//! pure rational, so no `exp`/`atan2`/`sin`/`cos`. Deterministic — no RNG.

/// The smoothing length `L` (px) as a fraction of the brush **diameter**: the heading re-aims over
/// roughly a TENTH of a brush-width of travel — snappy/responsive (the Rake should track the stroke,
/// not lag a whole brush-width behind it). Larger ⇒ steadier but laggier; smaller ⇒ snappier but shakier.
/// Brush-size-relative (Krita's "Fade"), so the feel is the same at any zoom.
const SMOOTH_LEN_FRACTION: f32 = 0.1;
/// Floor for the smoothing length (px) so a tiny brush still filters the ~3px chord noise rather than
/// snapping to every wiggle. Kept small so the Rake stays responsive on small brushes.
const SMOOTH_LEN_MIN_PX: f32 = 3.0;

/// The smoothing length `L` for a brush of `diameter_px`: `max(0.1·diameter, MIN)`.
#[must_use]
pub fn smooth_len(diameter_px: f32) -> f32 {
    (diameter_px * SMOOTH_LEN_FRACTION).max(SMOOTH_LEN_MIN_PX)
}

/// Rotate the 2-D vector `v` by the unit rotor `r` (a complex multiply `v · r`). Composes the Rake
/// heading with the texture **Angle** rotor and the per-dab **Jitter Rotate** rotor; both args unit ⇒
/// the result is unit. Transcendental-free.
#[must_use]
pub fn rotate(v: [f32; 2], r: [f32; 2]) -> [f32; 2] {
    [v[0] * r[0] - v[1] * r[1], v[0] * r[1] + v[1] * r[0]]
}

/// Advance the smoothed `heading` by a path step of length `step_len` (px) whose unit tangent is `tangent`.
/// `heading` is a unit vector carried across calls; `[0, 0]` means "no heading yet" (stroke start).
///
/// Length-weighted EMA in vector space: `α = step_len / (step_len + smooth_len)` (a longer step pulls the
/// heading further toward the new tangent — so behaviour is independent of how the path was chopped into
/// steps), then `heading ← normalize(heading + α·(tangent − heading))`.
///
/// Edge cases:
/// - **Start** (`heading == [0, 0]`): snap to `tangent` (no lerp from a zero vector).
/// - **~180° reversal**: the vector lerp passes through near-zero length; we snap to `tangent` rather than
///   normalising an unstable near-zero vector (a vector lerp can't spin the long way like an angle would).
/// - **Degenerate `tangent`** (`step_len ≈ 0`): no motion, hold the established `heading`.
pub fn advance(heading: [f32; 2], tangent: [f32; 2], step_len: f32, smooth_len: f32) -> [f32; 2] {
    // No motion → keep the current heading (Blender's "hold last angle" on a stall).
    if step_len <= f32::EPSILON {
        return heading;
    }
    // First travel → snap (no heading to lerp from).
    if heading == [0.0, 0.0] {
        return tangent;
    }
    let denom = step_len + smooth_len.max(f32::EPSILON);
    let alpha = step_len / denom;
    let mixed = [
        heading[0] + (tangent[0] - heading[0]) * alpha,
        heading[1] + (tangent[1] - heading[1]) * alpha,
    ];
    let len = (mixed[0] * mixed[0] + mixed[1] * mixed[1]).sqrt();
    if len < 1e-4 {
        // Near-180° reversal cancelled the vector → snap to the new tangent instead of normalising noise.
        tangent
    } else {
        [mixed[0] / len, mixed[1] / len]
    }
}

#[cfg(test)]
mod tests {
    use super::{advance, rotate, smooth_len};

    fn unit(v: [f32; 2]) -> [f32; 2] {
        let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
        [v[0] / l, v[1] / l]
    }
    fn is_unit(v: [f32; 2]) -> bool {
        ((v[0] * v[0] + v[1] * v[1]).sqrt() - 1.0).abs() < 1e-4
    }

    #[test]
    fn start_snaps_to_first_tangent() {
        let h = advance([0.0, 0.0], [1.0, 0.0], 3.0, 25.0);
        assert_eq!(h, [1.0, 0.0]);
    }

    #[test]
    fn no_motion_holds_heading() {
        let prev = [0.6, 0.8];
        assert_eq!(advance(prev, [0.0, 1.0], 0.0, 25.0), prev);
    }

    #[test]
    fn output_is_always_unit() {
        let mut h = [1.0, 0.0];
        for _ in 0..50 {
            h = advance(h, unit([1.0, 0.3]), 3.0, 25.0);
            assert!(is_unit(h), "heading stays unit-length");
        }
    }

    #[test]
    fn converges_toward_a_steady_tangent() {
        // A long run of +y tangent pulls the heading from +x toward +y and holds there.
        let mut h = [1.0, 0.0];
        for _ in 0..200 {
            h = advance(h, [0.0, 1.0], 3.0, 25.0);
        }
        assert!(h[1] > 0.999 && h[0].abs() < 0.03, "settled on +y: {h:?}");
    }

    #[test]
    fn straight_run_is_stable_dab_to_dab() {
        // Constant tangent → the heading never drifts (a straight stroke gives a constant direction).
        let t = unit([1.0, 1.0]);
        let mut h = advance([0.0, 0.0], t, 3.0, 25.0); // snaps to t
        for _ in 0..100 {
            let next = advance(h, t, 3.0, 25.0);
            let d = ((next[0] - h[0]).powi(2) + (next[1] - h[1]).powi(2)).sqrt();
            assert!(d < 1e-5, "no drift on a straight run");
            h = next;
        }
    }

    #[test]
    fn longer_steps_pull_harder_length_weighting() {
        // The same start/tangent, but a step that is 4× longer moves the heading 4×-ish more per call,
        // so behaviour tracks DISTANCE, not call count.
        let small = advance([1.0, 0.0], [0.0, 1.0], 3.0, 30.0);
        let big = advance([1.0, 0.0], [0.0, 1.0], 12.0, 30.0);
        assert!(
            big[1] > small[1],
            "a longer step re-aims further toward the tangent"
        );
    }

    #[test]
    fn reversal_snaps_not_spins() {
        // Heading +x, an exact −x tangent: the vector lerp can collapse to ~zero; we snap to −x rather
        // than normalising noise. With α far from 0.5 it just eases, still toward −x along the short arc.
        let h = advance([1.0, 0.0], [-1.0, 0.0], 30.0, 30.0); // α = 0.5 → exact cancellation → snap
        assert!(
            (h[0] + 1.0).abs() < 1e-4 && h[1].abs() < 1e-4,
            "snapped to -x: {h:?}"
        );
    }

    #[test]
    fn rotate_composes_heading_with_angle_offset() {
        // The Rake heading rotated by the Angle rotor: +x ∘ +90° (rotor [0,1]) = +y, stays unit.
        let h = rotate([1.0, 0.0], [0.0, 1.0]);
        assert!(
            is_unit(h) && h[0].abs() < 1e-6 && (h[1] - 1.0).abs() < 1e-6,
            "+x ∘ 90° = +y: {h:?}"
        );
        // Angle 0 (rotor [1,0]) is the identity — a rake brush with Angle 0 keeps the bare heading.
        assert_eq!(rotate([0.6, 0.8], [1.0, 0.0]), [0.6, 0.8]);
    }

    #[test]
    fn smooth_len_is_brush_relative_with_a_floor() {
        assert_eq!(smooth_len(100.0), 10.0); // 0.1 · diameter (responsive)
        assert_eq!(smooth_len(4.0), super::SMOOTH_LEN_MIN_PX); // floor for tiny brushes
    }
}
