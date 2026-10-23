//! O bake do preview de uma Color Ramp (Grain e Shape): a MESMA `ColorRamp` que o tool pinta, em LUT sRGB.

use ph2d_tool_painter::{
    ColorRamp, RampColorMode, RampInterp, RampStop, linear_to_srgb_byte, srgb_to_linear_byte,
};

/// Bake the **exact** `ColorRamp` from `stops` (display sRGB → linear), honouring `mode` / `interp`,
/// then the `bw` luminance filter, into `out` as a 256-entry **sRGB-straight RGBA** LUT — the same bake
/// the tool paints with, so a preview is faithful. Returns `false` (→ grayscale scalar) when off / empty.
/// Shared by the Grain ([`crate::paint_texture`]) + Shape ([`crate::paint_shape`]) previews.
pub(crate) fn build_preview_lut(
    enabled: bool,
    bw: bool,
    mode: u8,
    interp: u8,
    stops: &[[f32; 6]],
    out: &mut [[f32; 4]; 256],
) -> bool {
    if !enabled || stops.is_empty() {
        return false;
    }
    let s2l = |c: f32| srgb_to_linear_byte((c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8); // LITERAL-PX-OK: sRGB 8-bit normalize
    let rs: Vec<RampStop> = stops
        .iter()
        .map(|s| RampStop::new(s[0], [s2l(s[1]), s2l(s[2]), s2l(s[3]), s[4]]))
        .collect();
    let ramp = ColorRamp::new(
        rs,
        RampColorMode::from_u8(mode),
        RampInterp::from_u8(interp),
    );
    ramp.bake_into(out); // linear RGBA in the chosen interp/colour space
    for c in out.iter_mut() {
        if bw {
            let l = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]; // LITERAL-PX-OK: Rec.709 luma weights
            c[0] = l;
            c[1] = l;
            c[2] = l;
        }
        c[0] = f32::from(linear_to_srgb_byte(c[0])) / 255.0; // LITERAL-PX-OK: sRGB 8-bit normalize
        c[1] = f32::from(linear_to_srgb_byte(c[1])) / 255.0; // LITERAL-PX-OK: sRGB 8-bit normalize
        c[2] = f32::from(linear_to_srgb_byte(c[2])) / 255.0; // LITERAL-PX-OK: sRGB 8-bit normalize
    }
    true
}
