//! Gates for the two per-stroke coverage laws ([`super`]). They pin the properties the two media
//! disagree about, and the byte-identity of the pigment one.

use super::{StrokeCoverLaw, cover_add};

/// Run a stroke's worth of dab weights through a law and return the coverage it leaves.
fn run(law: StrokeCoverLaw, weights: &[f32], g: f32, coverage: f32) -> f32 {
    let mut m = 0.0_f32;
    for &w in weights {
        if let Some(add) = cover_add(law, m, w, g, coverage, false) {
            m += add;
        }
    }
    m
}

/// The dab weights a texel at the given distance sees as a brush is dragged past it at `spacing`,
/// with a smooth (`1 − t²`-ish) profile — the shape of the real thing, so the fixture contains the
/// phenomenon rather than a single dab.
fn pass_weights(dist: f32, radius: f32, spacing: f32) -> Vec<f32> {
    let n = (2.0 * radius / spacing).ceil() as i32;
    (-n..=n)
        .filter_map(|k| {
            let along = k as f32 * spacing;
            let d = (dist * dist + along * along).sqrt() / radius;
            (d < 1.0).then(|| {
                let t = 1.0 - d;
                t * t * (3.0 - 2.0 * t) // smoothstep shoulder
            })
        })
        .collect()
}

#[test]
fn the_pigment_law_is_the_arithmetic_that_shipped() {
    // Pure code motion: the extracted `cover_add` must reproduce the two in-line branches EXACTLY
    // (same operations, same order, same 1e-4 guards) — the pigment path's byte-identity rests on it.
    for &m in &[0.0_f32, 0.1, 0.5, 0.9, 1.0] {
        for &w in &[0.0_f32, 0.25, 0.75, 1.0] {
            for &g in &[0.3_f32, 1.0] {
                for &cov in &[0.4_f32, 1.0] {
                    // Non-AA branch, as it read in `bands.rs`.
                    let want = {
                        let cap = (g * cov).min(1.0);
                        if m >= cap { None } else { Some(w * (cap - m)) }
                    };
                    assert_eq!(
                        cover_add(StrokeCoverLaw::BuildUp, m, w, g, cov, false),
                        want,
                        "non-AA build-up drifted at m={m} w={w} g={g} cov={cov}"
                    );
                    // AA branch.
                    let want_aa = {
                        let cap = (w * cov).min(1.0);
                        if m >= cap {
                            None
                        } else {
                            Some((w * g * cov) * (1.0 - m / cap.max(1e-4)))
                        }
                    };
                    assert_eq!(
                        cover_add(StrokeCoverLaw::BuildUp, m, w, g, cov, true),
                        want_aa,
                        "AA build-up drifted at m={m} w={w} g={g} cov={cov}"
                    );
                }
            }
        }
    }
}

#[test]
fn the_envelope_keeps_the_deepest_dab_and_ignores_the_rest() {
    // A max: the final coverage is the largest target the stroke laid, whatever the order, and a
    // repeat adds NOTHING (that is the whole point — re-crossing must be inert).
    let ws = [0.2_f32, 0.9, 0.4, 0.9, 0.1];
    let up = run(StrokeCoverLaw::Envelope, &ws, 1.0, 1.0);
    let mut down = ws;
    down.reverse();
    let dn = run(StrokeCoverLaw::Envelope, &down, 1.0, 1.0);
    assert!(
        (up - 0.9).abs() < 1e-6 && (dn - 0.9).abs() < 1e-6,
        "envelope must equal the max target regardless of order, got {up} and {dn}"
    );
    assert_eq!(
        cover_add(StrokeCoverLaw::Envelope, 0.9, 0.9, 1.0, 1.0, false),
        None,
        "a dab that re-lays what is already there must add nothing"
    );
}

#[test]
fn the_envelope_shoulder_survives_a_pass_where_the_build_up_shoulder_collapses() {
    // THE discriminating property, at the texel that decides the look: the soft shoulder. Build-up
    // converges it toward full coverage because MANY dabs cross it (so the edge marches outward and
    // the band thins); the envelope leaves it at the profile's own value, which is what a feather IS.
    let (r, spacing) = (10.0_f32, 2.0_f32);
    let shoulder = pass_weights(8.0, r, spacing); // 80 % of the radius out: the feather
    let peak = shoulder.iter().copied().fold(0.0_f32, f32::max);
    let env = run(StrokeCoverLaw::Envelope, &shoulder, 1.0, 1.0);
    let bld = run(StrokeCoverLaw::BuildUp, &shoulder, 1.0, 1.0);
    assert!(
        (env - peak).abs() < 1e-6,
        "the envelope must equal the profile peak ({peak}), got {env}"
    );
    // A RATIO, not a margin: the overshoot is a multiple of the profile (measured 2.8× at this
    // shoulder), and a fixed margin would just encode this fixture's radius.
    assert!(
        bld > env * 2.0,
        "fixture must contain the phenomenon: build-up should overshoot the profile several-fold \
         (peak {peak}, build-up {bld}, envelope {env})"
    );
    // And the envelope is blind to how finely the path was sampled, while build-up is not.
    let dense = run(
        StrokeCoverLaw::Envelope,
        &pass_weights(8.0, r, 0.5),
        1.0,
        1.0,
    );
    let dense_bld = run(
        StrokeCoverLaw::BuildUp,
        &pass_weights(8.0, r, 0.5),
        1.0,
        1.0,
    );
    assert!(
        (dense - env).abs() < 0.02,
        "the envelope must be a fact of the PATH, not of the spacing ({env} vs {dense})"
    );
    assert!(
        dense_bld > bld,
        "control: build-up must grow with denser sampling ({bld} -> {dense_bld})"
    );
}

#[test]
fn the_envelope_still_honours_strength_and_grain() {
    // The target is the dab's FULL weight — Strength/Flow (`coverage`) and the Grain scale it, so a
    // half-strength mask stroke tops out at half coverage and a grain texel at its own value.
    assert!((run(StrokeCoverLaw::Envelope, &[1.0], 1.0, 0.5) - 0.5).abs() < 1e-6);
    assert!((run(StrokeCoverLaw::Envelope, &[1.0], 0.25, 1.0) - 0.25).abs() < 1e-6);
    // …and never past full coverage, however many dabs.
    assert!(run(StrokeCoverLaw::Envelope, &[1.0; 8], 1.0, 1.0) <= 1.0);
}
