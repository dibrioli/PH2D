//! Gradient Map — luma → gradient color (256-entry RGB LUT) plus the bespoke
//! N-stop editor mutations. Split out of the former monolithic `compute.rs`
//! (pure move).

use super::*;

/// A single gradient stop's color (`[u8;4]` sRGB) → linear RGB.
fn stop_linear(color: [u8; 4]) -> [f32; 3] {
    [
        srgb_to_linear_f32(color[0] as f32 / 255.0),
        srgb_to_linear_f32(color[1] as f32 / 255.0),
        srgb_to_linear_f32(color[2] as f32 / 255.0),
    ]
}

/// Sample a gradient (`stops` ASCENDING by offset) at `offset` (`0..=1`) →
/// linear RGB. Outside the stop span the endpoints extend flat; `Smooth` applies
/// a smoothstep to the inter-stop `t`. Empty stops fall back to a black→white
/// ramp (so the LUT is well-defined even for a degenerate gradient).
fn gradient_sample(stops: &[ColorStop], interp: GradientInterp, offset: f32) -> [f32; 3] {
    if stops.is_empty() {
        let v = srgb_to_linear_f32(offset.clamp(0.0, 1.0));
        return [v, v, v];
    }
    let n = stops.len();
    if offset <= stops[0].offset {
        return stop_linear(stops[0].color);
    }
    if offset >= stops[n - 1].offset {
        return stop_linear(stops[n - 1].color);
    }
    let mut i = 0;
    while i + 1 < n && stops[i + 1].offset < offset {
        i += 1;
    }
    let (s0, s1) = (&stops[i], &stops[i + 1]);
    let span = s1.offset - s0.offset;
    let mut t = if span > 1e-6 {
        ((offset - s0.offset) / span).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if matches!(interp, GradientInterp::Smooth) {
        t = t * t * (3.0 - 2.0 * t);
    }
    let (c0, c1) = (stop_linear(s0.color), stop_linear(s1.color));
    core::array::from_fn(|ch| c0[ch] + (c1[ch] - c0[ch]) * t)
}

/// The 256-entry luma→linear-RGB table a [`GradientMapParams`] resolves to — the
/// real-time strategy (handoff §2.5): build the gradient ONCE, then the per-pixel
/// inner loop is a luma + table lookup. **The GPU-mandate deliverable's math**:
/// this is an RGB-OUTPUT LUT (3 channels from ONE luma input), NOT the per-channel
/// `adj_luts` transfer Curves uses, so the GPU needs a new 256×RGB binding mode
/// (Coord — see the W4 handoff §GPU-COORD-GM). Stops are sorted here so the table
/// is correct regardless of authoring order.
#[must_use]
pub fn gradient_map_lut(p: &GradientMapParams) -> [[f32; 3]; 256] {
    let mut stops = p.stops.clone();
    stops.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    core::array::from_fn(|i| gradient_sample(&stops, p.interpolation, i as f32 / 255.0))
}

/// The 3 RGB sliders (`(label, value01)`, `0..255 → 0..1`) of gradient `stop` —
/// what the bespoke editor renders for the selected stop. Out-of-range stops
/// return black. Inverse of [`set_gradient_stop_color_param`].
#[must_use]
pub fn gradient_stop_color_params(p: &GradientMapParams, stop: usize) -> Vec<(&'static str, f32)> {
    let c = p.stops.get(stop).map(|s| s.color).unwrap_or([0, 0, 0, 255]);
    vec![
        ("Red", c[0] as f32 / 255.0),
        ("Green", c[1] as f32 / 255.0),
        ("Blue", c[2] as f32 / 255.0),
    ]
}

/// Set RGB slider `slot` (0 = R, 1 = G, 2 = B) of gradient `stop` from a
/// normalized `0..1` value. Inverse of [`gradient_stop_color_params`]. Out-of-range
/// stops/slots no-op.
pub fn set_gradient_stop_color_param(
    p: &mut GradientMapParams,
    stop: usize,
    slot: usize,
    value01: f32,
) {
    let byte = (value01.clamp(0.0, 1.0) * 255.0).round() as u8;
    if let Some(s) = p.stops.get_mut(stop)
        && slot < 3
    {
        s.color[slot] = byte;
    }
}

/// Move gradient `stop` to `offset` (clamped `0..=1`). The stops keep their Vec
/// order (a stable index per editor handle, so a drag never re-binds to a
/// different stop); [`gradient_map_lut`] sorts a copy at sample time, so stops may
/// cross freely. No-op for an out-of-range index.
pub fn move_gradient_stop(p: &mut GradientMapParams, stop: usize, offset: f32) {
    if let Some(s) = p.stops.get_mut(stop) {
        s.offset = offset.clamp(0.0, 1.0);
    }
}

/// Insert a stop at the midpoint of the widest offset gap, its color sampled ON
/// the current gradient (so the rendered map is unchanged until the new stop is
/// recolored). Returns the inserted index, or `None` at the ≤16-stop cap or for a
/// degenerate (<1-stop) gradient. Mirror of `add_curve_point`.
pub fn add_gradient_stop(p: &mut GradientMapParams) -> Option<usize> {
    const MAX_STOPS: usize = 16;
    let n = p.stops.len();
    if !(1..MAX_STOPS).contains(&n) {
        return None;
    }
    // Widest gap between adjacent (sorted) stops, else after the last stop.
    let mut stops = p.stops.clone();
    stops.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    let (mut best_gap, mut new_off) = (-1.0_f32, 0.5_f32);
    for w in stops.windows(2) {
        let gap = w[1].offset - w[0].offset;
        if gap > best_gap {
            best_gap = gap;
            new_off = (w[0].offset + w[1].offset) * 0.5;
        }
    }
    let lin = gradient_sample(&stops, p.interpolation, new_off);
    let color = [
        (linear_to_srgb_f32(lin[0]) * 255.0).round() as u8,
        (linear_to_srgb_f32(lin[1]) * 255.0).round() as u8,
        (linear_to_srgb_f32(lin[2]) * 255.0).round() as u8,
        255,
    ];
    p.stops.push(ColorStop {
        offset: new_off,
        color,
    });
    Some(p.stops.len() - 1)
}

/// Remove gradient `stop`. No-op when only two stops remain (a gradient needs ≥2)
/// or `stop` is out of range. Mirror of `remove_curve_point`.
pub fn remove_gradient_stop(p: &mut GradientMapParams, stop: usize) {
    if p.stops.len() > 2 && stop < p.stops.len() {
        p.stops.remove(stop);
    }
}

/// Gradient Map — remaps each pixel's DISPLAY-space luma (Rec.601, like Threshold)
/// to a color along the gradient ([`gradient_map_lut`], the same table the GPU
/// binds). Builds the LUT once, then the per-pixel loop is a luma + lerped lookup.
/// `acc` is straight LINEAR f32 RGBA (alpha preserved). Always applies (a fresh
/// Gradient Map is a visible remap, like Posterize).
pub(crate) fn apply_gradient_map(p: &GradientMapParams, acc: &mut [[f32; 4]]) {
    let lut = gradient_map_lut(p);
    let encode = build_lut(linear_to_srgb_f32); // luma is computed in display space
    for px in acc.iter_mut() {
        let luma = 0.299 * sample_lut(&encode, px[0])
            + 0.587 * sample_lut(&encode, px[1])
            + 0.114 * sample_lut(&encode, px[2]);
        let t = luma.clamp(0.0, 1.0) * 255.0;
        let i = t as usize;
        let frac = t - i as f32;
        let a = lut[i.min(255)];
        let b = lut[(i + 1).min(255)];
        for ch in 0..3 {
            px[ch] = a[ch] + (b[ch] - a[ch]) * frac;
        }
    }
}
