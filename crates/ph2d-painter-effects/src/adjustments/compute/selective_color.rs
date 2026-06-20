//! Selective Color — per-color-group CMYK adjustment in display space (six
//! chromatic groups + three achromatic groups) plus its bespoke per-bucket slider
//! params. Split out of the former monolithic `compute.rs` (pure move).

use super::*;

/// The CMYK adjustment of Selective-Color color-group `bucket` (0 = Reds,
/// 1 = Yellows, 2 = Greens, 3 = Cyans, 4 = Blues, 5 = Magentas, 6 = Whites,
/// 7 = Neutrals, 8 = Blacks).
fn selcolor_bucket(p: &SelectiveColorParams, bucket: usize) -> CmykAdjust {
    match bucket {
        0 => p.reds,
        1 => p.yellows,
        2 => p.greens,
        3 => p.cyans,
        4 => p.blues,
        5 => p.magentas,
        6 => p.whites,
        7 => p.neutrals,
        _ => p.blacks,
    }
}

/// Mutable [`selcolor_bucket`].
fn selcolor_bucket_mut(p: &mut SelectiveColorParams, bucket: usize) -> &mut CmykAdjust {
    match bucket {
        0 => &mut p.reds,
        1 => &mut p.yellows,
        2 => &mut p.greens,
        3 => &mut p.cyans,
        4 => &mut p.blues,
        5 => &mut p.magentas,
        6 => &mut p.whites,
        7 => &mut p.neutrals,
        _ => &mut p.blacks,
    }
}

/// The 9 Selective-Color group labels (the bucket-selector order).
pub const SELCOLOR_BUCKETS: [&str; 9] = [
    "Reds", "Yellows", "Greens", "Cyans", "Blues", "Magentas", "Whites", "Neutrals", "Blacks",
];

/// Selective Color — a CMYK adjustment applied per color group in DISPLAY space.
/// Each pixel is weighted into the 6 chromatic groups (the RGB hue hexagon, like
/// Black & White) plus 3 achromatic groups (Whites/Neutrals/Blacks via luma tonal
/// masks, biased toward low-chroma pixels); the matching groups' CMYK shifts are
/// accumulated and applied (C/M/Y subtract R/G/B, K darkens all). `Relative`
/// scales the shift by the channel's existing value; `Absolute` is a flat shift.
/// `acc` is straight LINEAR f32 RGBA (alpha preserved). All-zero groups
/// early-return an exact identity.
pub(crate) fn apply_selective_color(p: &SelectiveColorParams, acc: &mut [[f32; 4]]) {
    let buckets: [CmykAdjust; 9] = core::array::from_fn(|i| selcolor_bucket(p, i));
    if buckets
        .iter()
        .all(|c| c.cyan == 0.0 && c.magenta == 0.0 && c.yellow == 0.0 && c.black == 0.0)
    {
        return;
    }
    let relative = matches!(p.method, SelectiveMethod::Relative);
    for px in acc.iter_mut() {
        let (r, g, b) = (
            linear_to_srgb_f32(px[0]),
            linear_to_srgb_f32(px[1]),
            linear_to_srgb_f32(px[2]),
        );
        let m = r.min(g).min(b);
        let chroma = r.max(g).max(b) - m;
        let (rr, gg, bb) = (r - m, g - m, b - m);
        // 6 chromatic group weights (hue-hexagon decomposition).
        let (reds, yellows, greens, cyans, blues, magentas) = if b <= r && b <= g {
            let y = rr.min(gg);
            (rr - y, y, gg - y, 0.0, 0.0, 0.0)
        } else if r <= g && r <= b {
            let c = gg.min(bb);
            (0.0, 0.0, gg - c, c, bb - c, 0.0)
        } else {
            let mg = bb.min(rr);
            (rr - mg, 0.0, 0.0, 0.0, bb - mg, mg)
        };
        // 3 achromatic group weights (luma tonal masks, biased to low chroma).
        let luma = 0.299 * r + 0.587 * g + 0.114 * b;
        let achroma = 1.0 - chroma;
        let blacks = (1.0 - luma) * (1.0 - luma) * achroma;
        let whites = luma * luma * achroma;
        let mid = 2.0 * luma - 1.0;
        let neutrals = (1.0 - mid * mid) * achroma;
        let w = [
            reds, yellows, greens, cyans, blues, magentas, whites, neutrals, blacks,
        ];
        let (mut tc, mut tm, mut ty, mut tk) = (0.0, 0.0, 0.0, 0.0);
        for (wi, bc) in w.iter().zip(buckets.iter()) {
            tc += wi * bc.cyan;
            tm += wi * bc.magenta;
            ty += wi * bc.yellow;
            tk += wi * bc.black;
        }
        let (nr, ng, nb) = if relative {
            (r - (tc + tk) * r, g - (tm + tk) * g, b - (ty + tk) * b)
        } else {
            (r - (tc + tk), g - (tm + tk), b - (ty + tk))
        };
        px[0] = srgb_to_linear_f32(nr.clamp(0.0, 1.0));
        px[1] = srgb_to_linear_f32(ng.clamp(0.0, 1.0));
        px[2] = srgb_to_linear_f32(nb.clamp(0.0, 1.0));
    }
}

/// The 4 CMYK sliders (`Cyan/Magenta/Yellow/Black`, each `(label, value01)`,
/// `-1..1 → 0..1`) of Selective-Color group `bucket` — what the bespoke editor
/// renders for the active bucket tab. Inverse of [`set_selective_color_param`].
#[must_use]
pub fn selective_color_slider_params(
    p: &SelectiveColorParams,
    bucket: usize,
) -> Vec<(&'static str, f32)> {
    let c = selcolor_bucket(p, bucket);
    let s = |v: f32| (v.clamp(-1.0, 1.0) + 1.0) * 0.5;
    vec![
        ("Cyan", s(c.cyan)),
        ("Mag", s(c.magenta)),
        ("Yel", s(c.yellow)),
        ("Blk", s(c.black)),
    ]
}

/// Set CMYK slider `slot` (0 = C, 1 = M, 2 = Y, 3 = K) of Selective-Color group
/// `bucket` from a normalized `0..1` value (`→ -1..1`). Inverse of
/// [`selective_color_slider_params`]. Out-of-range slots no-op.
pub fn set_selective_color_param(
    p: &mut SelectiveColorParams,
    bucket: usize,
    slot: usize,
    value01: f32,
) {
    let v = value01.clamp(0.0, 1.0) * 2.0 - 1.0;
    let c = selcolor_bucket_mut(p, bucket);
    match slot {
        0 => c.cyan = v,
        1 => c.magenta = v,
        2 => c.yellow = v,
        3 => c.black = v,
        _ => {}
    }
}
