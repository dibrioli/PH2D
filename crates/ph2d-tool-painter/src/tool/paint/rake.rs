//! The **Rake** heading solver — the texture-frame direction for a dab whose rotation follows the
//! stroke. Split from `stamp_cache.rs` (workspace LOC cap); the stamp paths call [`advance_rake`].

/// Easing applied each time the **Rake** heading re-aims at a fresh long-baseline direction.
const RAKE_LERP: f32 = 0.5;

/// Minimum travel (px) before the **Rake** heading re-aims, regardless of brush size — keeps a small
/// brush from re-aiming on a single noisy chord.
const RAKE_BASELINE_MIN_PX: f32 = 10.0;

/// The texture-frame direction for a dab under **Rake**. The raw inter-dab chord is only ~3px (dabs ride
/// a stabilizer-smoothed spline), so its DIRECTION is noise — easing toward it still random-walks (the
/// "anarchy" Enio reported 2026-06-24). Instead ACCUMULATE the chords (`accum`) across dabs/segments and
/// re-aim the carried `dir` only once travel clears a real baseline (~½ diameter), easing toward that
/// long-baseline heading. `dir`/`accum` persist in `PaintState` across batches. Returns the unit
/// direction for `dab_basis` (`[0,0]` only before any travel → `dab_basis` falls back to the Angle);
/// with Rake off the raw tangent passes through (ignored downstream).
pub(super) fn advance_rake(
    rake: bool,
    dir: &mut [f32; 2],
    accum: &mut [f32; 2],
    tangent: [f32; 2],
    diameter_px: f32,
) -> [f32; 2] {
    if !rake {
        return tangent;
    }
    accum[0] += tangent[0];
    accum[1] += tangent[1];
    let baseline = (diameter_px * 0.5).max(RAKE_BASELINE_MIN_PX);
    let alen = (accum[0] * accum[0] + accum[1] * accum[1]).sqrt();
    if alen >= baseline {
        let nt = [accum[0] / alen, accum[1] / alen];
        *dir = if *dir == [0.0, 0.0] {
            nt // first re-aim → snap (no lerp from a zero heading)
        } else {
            let d = [
                dir[0] + (nt[0] - dir[0]) * RAKE_LERP,
                dir[1] + (nt[1] - dir[1]) * RAKE_LERP,
            ];
            let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if l < 1e-4 { nt } else { [d[0] / l, d[1] / l] } // near-180° reversal → snap
        };
        *accum = [0.0, 0.0];
        return *dir;
    }
    // Below baseline: hold the established heading. Before the first re-aim, use the partial NET
    // displacement (already summed, far cleaner than one chord) so the stroke START tracks; only fall
    // back to the Angle (`[0,0]`) when there is no travel yet.
    if *dir != [0.0, 0.0] {
        *dir
    } else if alen >= 1e-3 {
        [accum[0] / alen, accum[1] / alen]
    } else {
        [0.0, 0.0]
    }
}

#[cfg(test)]
mod rake_tests {
    use super::advance_rake;

    #[test]
    fn advance_rake_uses_long_baseline_and_persists() {
        // Rake OFF → the raw tangent passes through, no state kept.
        let (mut dir, mut acc) = ([0.0, 0.0], [0.0, 0.0]);
        assert_eq!(
            advance_rake(false, &mut dir, &mut acc, [3.0, 4.0], 50.0),
            [3.0, 4.0]
        );
        assert_eq!((dir, acc), ([0.0, 0.0], [0.0, 0.0]));

        // Below baseline (½·50 = 25): a single +x chord does NOT re-aim the heading, but the partial
        // net displacement already points +x — so a noisy first chord can't whip the texture around.
        let (mut dir, mut acc) = ([0.0, 0.0], [0.0, 0.0]);
        let r = advance_rake(true, &mut dir, &mut acc, [3.0, 0.0], 50.0);
        assert_eq!(dir, [0.0, 0.0], "not re-aimed below baseline");
        assert!(
            (r[0] - 1.0).abs() < 1e-5 && r[1].abs() < 1e-5,
            "partial tracks +x"
        );

        // Accumulate +x past the baseline → the heading re-aims to +x.
        for _ in 0..12 {
            advance_rake(true, &mut dir, &mut acc, [3.0, 0.0], 50.0);
        }
        assert!(
            (dir[0] - 1.0).abs() < 1e-5 && dir[1].abs() < 1e-5,
            "heading re-aimed to +x"
        );

        // Parked (≈0 tangent) keeps the established heading (no fallback to the Angle).
        let prev = dir;
        assert_eq!(
            advance_rake(true, &mut dir, &mut acc, [0.0, 0.0], 50.0),
            prev
        );
    }
}
