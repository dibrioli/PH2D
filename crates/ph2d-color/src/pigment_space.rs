//! `PigmentLinearSrgb` — working-space marker for Mixbox pigment mixing.
//!
//! Sochorová & Jamriška, *Practical Pigment Mixing for Digital Painting*
//! (SIGGRAPH Asia 2021), defines its 12-coefficient Kubelka-Munk
//! reduced model in **linear-light sRGB (D65)**. PH2D follows the same
//! convention: every `mixbox_lerp_*` entry point in the `mixbox` module
//! of `ph2d-painter-brush` takes inputs and returns outputs in this
//! space. (Plain reference — `ph2d-color` cannot link there with
//! intra-doc syntax without cycling the dep graph.)
//!
//! This module exists so that brush-engine call sites name the working
//! space explicitly — `PigmentLinearSrgb::new(r, g, b, a)` — and so that
//! a future cross-gamut conversion (when ADR-0051 lands the
//! `ColorProfile` pipeline) has a single place to attach the boundary
//! transforms (Display-P3 → LinearSrgb in, LinearSrgb → output profile
//! out).
//!
//! ## Why an alias and not a newtype
//!
//! Mixbox internals (and the painter brush GPU shader, ADR-0044 §2.9)
//! consume the same `[f32; 4]` channel layout as [`crate::LinearRgba`].
//! Wrapping in a newtype would force every call site to `.0` the inner
//! value before passing to math; given that the **only** legal input
//! to Mixbox is already linear sRGB, the alias is sufficient to mark
//! intent without runtime cost or syntactic friction. If a future ADR
//! adds Display-P3 working space variants, this becomes the place to
//! split into newtypes.
//!
//! ## Reference
//!
//! - Sochorová, Š., & Jamriška, O. (2021). *Practical Pigment Mixing
//!   for Digital Painting.* ACM TOG 40(6) (SIGGRAPH Asia).
//! - Open-source impl: <https://github.com/scrtwpns/mixbox>
//! - ADR-0044 §2.5 (PH2D adoption + det-mode exclusion).

use crate::LinearRgba;

/// The canonical working color space for Mixbox pigment mixing in PH2D.
///
/// Every `mixbox_lerp_srgb8` / `mixbox_lerp_linear` entry point in
/// `ph2d-painter-brush` takes and returns values typed as
/// `PigmentLinearSrgb`. Callers convert from sRGB bytes, OKLab, or other
/// working spaces at the call boundary — never inside Mixbox.
///
/// **Premultiplication:** Mixbox operates on *straight* alpha
/// (non-premultiplied). The brush engine applies the
/// shape×grain×flow×pressure attenuation *after* Mixbox returns, so
/// pigment math sees the full chromaticity unclamped by alpha.
pub type PigmentLinearSrgb = LinearRgba;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_is_linear_rgba() {
        // Compile-time guarantee: alias resolves to the canonical
        // linear-light type. If this ever breaks (e.g. someone changes
        // the alias to LinearP3 or wraps it), this test catches the
        // semantic drift — Mixbox's LUT only makes sense in
        // linear sRGB D65.
        let m: PigmentLinearSrgb = LinearRgba::white();
        assert_eq!(m.r(), 1.0);
        assert_eq!(m.a(), 1.0);
    }

    #[test]
    fn construction_via_alias_compiles() {
        // Call sites use `PigmentLinearSrgb::new(...)` directly — same
        // surface as `LinearRgba`.
        let c = PigmentLinearSrgb::new(0.2, 0.5, 0.9, 1.0);
        assert_eq!(c.g(), 0.5);
    }
}
