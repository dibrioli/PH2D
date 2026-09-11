//! The accuracy contract of the tabulated sRGB transfer
//! (`colorops::transfer`), which the two EXPERIMENTAL knobs run on.
//!
//! The table buys ~11x on the only transcendental in the model. What it costs
//! is exactness, so the cost is PINNED here in the units that decide whether
//! it is visible: one byte level is 1/255 = 3.92e-3 of the 0..1 sRGB scale,
//! or 1.0 of the engine's 0..255 float scale.
//!
//! These are not "close enough" assertions. Each bar is a measured number
//! with two decimal orders of slack, so a table that silently loses nodes, a
//! split moved to the wrong place, or an index rescaled wrongly all bleed
//! here rather than in a screenshot nobody diffs.

use ph2d_wet_paint::colorops::ColorMix;
use ph2d_wet_paint::colorops::transfer::{
    ks_of_srgb255, ks_of_srgb255_exact, linear_to_srgb, linear_to_srgb_exact, srgb_to_linear,
    srgb_to_linear_exact, srgb255_of_linear, srgb255_to_linear,
};

/// One byte level on the 0..1 sRGB scale — the threshold below which an error
/// cannot survive the renderer's `clamp_u8`.
const BYTE_LEVEL: f64 = 1.0 / 255.0;

/// Dense sweep: the forward transfer never strays a thousandth of a byte.
#[test]
fn the_forward_transfer_tracks_libm() {
    let mut worst = 0.0f64;
    let mut at = 0.0f64;
    for k in 0..=2_000_000u32 {
        let x = f64::from(k) / 2_000_000.0;
        let e = (srgb_to_linear(x) - srgb_to_linear_exact(x)).abs();
        if e > worst {
            worst = e;
            at = x;
        }
    }
    println!("forward max |table - libm| = {worst:.3e} at {at:.6}");
    assert!(
        worst < BYTE_LEVEL / 1000.0,
        "forward transfer drifted {worst:.3e} at {at} (bar {:.3e})",
        BYTE_LEVEL / 1000.0
    );
}

/// Dense sweep: the inverse transfer, whose curvature is concentrated at the
/// dark end, never strays a thousandth of a byte either — that is what the
/// segmented table is FOR, and a single uniform table of the same node count
/// measured 1.6e-5, which would fail this bar.
#[test]
fn the_inverse_transfer_tracks_libm() {
    let mut worst = 0.0f64;
    let mut at = 0.0f64;
    for k in 0..=2_000_000u32 {
        let x = f64::from(k) / 2_000_000.0;
        let e = (linear_to_srgb(x) - linear_to_srgb_exact(x)).abs();
        if e > worst {
            worst = e;
            at = x;
        }
    }
    println!("inverse max |table - libm| = {worst:.3e} at {at:.6}");
    assert!(
        worst < BYTE_LEVEL / 1000.0,
        "inverse transfer drifted {worst:.3e} at {at} (bar {:.3e})",
        BYTE_LEVEL / 1000.0
    );
}

/// The fused K/S door must answer what its own exact chain answers. K/S is
/// unbounded-ish (0 at white, ~126.5 at the reflectance floor), so the bar is
/// RELATIVE — an absolute one would be meaningless at both ends at once.
#[test]
fn the_ks_door_tracks_its_exact_chain() {
    let mut worst = 0.0f64;
    let mut at = 0.0f64;
    for k in 0..=2_000_000u32 {
        let c = f64::from(k) / 2_000_000.0 * 255.0;
        let a = ks_of_srgb255(c);
        let b = ks_of_srgb255_exact(c);
        let e = (a - b).abs() / b.max(1e-9);
        if e > worst {
            worst = e;
            at = c;
        }
    }
    println!("K/S door max relative error = {worst:.3e} at c={at:.4}");
    assert!(
        worst < 1e-4,
        "K/S door drifted {worst:.3e} relative at c={at} (bar 1e-4)"
    );
}

/// The 0..255 forward door is the clamped `c/255` chain in one step: outside
/// the gamut it must clamp exactly as the renderer's old `clamp_byte` did.
#[test]
fn the_255_door_clamps_the_gamut_like_clamp_byte() {
    for c in [-40.0, -1e-9, 0.0] {
        assert_eq!(srgb255_to_linear(c), srgb_to_linear_exact(0.0));
    }
    for c in [255.0, 255.0 + 1e-9, 400.0] {
        assert_eq!(srgb255_to_linear(c), srgb_to_linear(1.0));
    }
    // And inside the gamut it agrees with the two-step form it replaced.
    let mut worst = 0.0f64;
    for k in 0..=500_000u32 {
        let c = f64::from(k) / 500_000.0 * 255.0;
        let e = (srgb255_to_linear(c) - srgb_to_linear_exact(c / 255.0)).abs();
        worst = worst.max(e);
    }
    println!("0..255 door max |door - exact| = {worst:.3e}");
    assert!(
        worst < BYTE_LEVEL / 1000.0,
        "0..255 door drifted {worst:.3e}"
    );
}

/// The two halves of the K–M round trip live in different modules and each
/// floors reflectance at 1/255. If those floors ever disagree the mixing
/// would quietly stop being an involution at the dark end — assert they are
/// the SAME number by exercising the composition where the floor bites.
#[test]
fn both_halves_floor_reflectance_at_the_same_place() {
    // Below the floor every colour maps to the same K/S, so the round trip
    // must land them all on the same colour too.
    let a = srgb255_of_linear(reflect(ks_of_srgb255(0.0)));
    let b = srgb255_of_linear(reflect(ks_of_srgb255(10.0)));
    assert!(
        (a - b).abs() < 1e-9,
        "the floor is not shared: {a} vs {b} — one half floors elsewhere"
    );
    // And just above the floor the round trip must start tracking the input.
    let c = srgb255_of_linear(reflect(ks_of_srgb255(60.0)));
    assert!(
        c > a + 1.0,
        "the floor swallowed a colour it should not: {c}"
    );
}

fn reflect(ks: f64) -> f64 {
    1.0 + ks - (ks * ks + 2.0 * ks).sqrt()
}

/// A colour that goes into K/S space and comes back must be the colour it
/// was, to well under a byte. This is the invariant every K–M site leans on:
/// `ColorMix::Km` with an endpoint weight, and any wash whose settled and
/// suspended pigment have converged to the same colour.
#[test]
fn the_km_round_trip_returns_the_colour() {
    let mut worst = 0.0f64;
    let mut at = 0.0f64;
    for k in 0..=255_000u32 {
        let c = f64::from(k) / 1000.0;
        let back = srgb255_of_linear(reflect(ks_of_srgb255(c)));
        // Below the reflectance floor the round trip CANNOT return the input
        // (every colour there shares one K/S) — that is the reference's own
        // behaviour, so measure where the floor does not bite.
        if ks_of_srgb255_exact(c) >= ks_of_srgb255_exact(0.0) - 1e-9 {
            continue;
        }
        let e = (back - c).abs();
        if e > worst {
            worst = e;
            at = c;
        }
    }
    println!("K–M round trip max error = {worst:.4} of 255 at c={at:.3}");
    assert!(
        worst < 0.01,
        "round trip lost {worst:.4} of a level at c={at} (bar 0.01)"
    );
}

/// THE SPEC'S OWN WORRY, measured: "a wet cell can re-mix thousands of times".
///
/// A still wash — settled and suspended already the same pigment — is re-mixed
/// by every drying pass.
///
/// ⚠️ The oracle is the EXACT chain iterated identically, never "the colour
/// stays put". Mixing a colour with itself does NOT return it in this model:
/// below the reflectance floor every colour shares one K/S, so `libm` walks
/// c=12 to c=12.7 on the first mix too. A gate that asserted stillness would
/// be measuring the reference's own physics and calling it the table's error.
#[test]
fn a_still_wash_tracks_the_exact_transfer_over_thousands_of_re_mixes() {
    let mut worst = 0.0f64;
    let mut worst_at = 0.0f64;
    for start in [12.0f64, 60.0, 128.0, 200.0, 250.0, 254.0] {
        let mut tab = start;
        let mut exact = start;
        let mut out = [0.0f64; 3];
        for _ in 0..5000 {
            ColorMix::Km.mix(tab, tab, tab, tab, tab, tab, 0.37, &mut out);
            tab = out[0];
            exact = exact_km_mix(exact, exact, 0.37);
        }
        let drift = (tab - exact).abs();
        println!(
            "still wash from {start}: 5000 re-mixes -> table {tab:.5}, exact {exact:.5}, \
             apart {drift:.3e} of 255"
        );
        if drift > worst {
            worst = drift;
            worst_at = start;
        }
    }
    // Measured worst 0.016 of a byte level; the bar is 3x that. The walk is
    // bounded by construction — it stops at a zero of the table's own
    // interpolation error, which is within a couple of table cells — so this
    // number shrinks with the node count (0.469 at 1024 nodes, 0.140 at 2048,
    // 0.055 at 8192) rather than growing with the pass count.
    assert!(
        worst < 0.05,
        "table and libm drifted {worst:.3e} of a level apart from {worst_at} \
         over 5000 re-mixes (bar 0.05)"
    );
}

/// The exact K–M channel mix, spelled out with `libm` so no oracle in this
/// file can inherit the table's own answer.
fn exact_km_mix(d: f64, s: f64, w: f64) -> f64 {
    let ks = |c: f64| {
        let r = srgb_to_linear_exact(c / 255.0);
        let rr = if r < 1.0 / 255.0 { 1.0 / 255.0 } else { r };
        ((1.0 - rr) * (1.0 - rr)) / (2.0 * rr)
    };
    linear_to_srgb_exact(reflect((1.0 - w) * ks(d) + w * ks(s))) * 255.0
}

/// The other half of the same worry: a cell being re-mixed by PASSING fronts
/// (alternating pigments at a light weight) must land where the exact
/// transfer lands it.
#[test]
fn a_re_mixed_cell_lands_where_the_exact_transfer_lands_it() {
    let (mut a_ref, mut a_tab) = (200.0f64, 200.0f64);
    let mut out = [0.0f64; 3];
    for k in 0..5000 {
        let src = if k % 2 == 0 { 12.0 } else { 190.0 };
        a_ref = exact_km_mix(a_ref, src, 0.03);
        ColorMix::Km.mix(a_tab, a_tab, a_tab, src, src, src, 0.03, &mut out);
        a_tab = out[0];
    }
    let drift = (a_ref - a_tab).abs();
    println!("re-mixed cell: exact {a_ref:.6}  table {a_tab:.6}  drift {drift:.3e} of 255");
    assert!(
        drift < 0.01,
        "table and libm diverged {drift:.3e} of a level over 5000 re-mixes"
    );
}
