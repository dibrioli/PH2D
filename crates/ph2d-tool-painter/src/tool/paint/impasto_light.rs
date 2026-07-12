//! **Impasto** — the light pass: the relief made visible.
//!
//! The height field ([`super::impasto`]) is the *material*; this is the *look*. A surface normal is
//! taken from the height by central differences and shaded (Lambert + a Blinn-Phong highlight), and
//! the result modulates the composited pixels.
//!
//! ## The one property everything hangs on: flat paint is UNTOUCHED
//!
//! The shading is **relative**, not absolute: a pixel's diffuse response is divided by the response
//! of a *flat* surface, and the specular has the flat surface's own highlight subtracted. So where
//! the canvas has no relief the pass multiplies by exactly 1 and adds exactly 0 — the pixels come out
//! **byte-identical**.
//!
//! That is not a nicety, it is the whole contract. An absolute shading model (the naive
//! `rgb × (N·L)`) would darken the entire painting the moment the light came on, because a flat
//! surface lit from 45° returns 0.7, not 1. Half the emboss filters ever written have that bug. It
//! also gives the gates their teeth: "no relief ⇒ not one byte moves" is checkable, and it is checked.
//!
//! ## Why the pass runs on a freshly-composited region
//!
//! Lighting is spatially non-local (a pixel's normal reads its neighbours), and it is **not**
//! idempotent — lighting an already-lit pixel lights it twice. The dirty-rect fast lane re-composites
//! its region from the layers (un-lit) and lights *that*, so the cached composite holds lit pixels but
//! every one of them was lit exactly once, from scratch. Applying the pass in place on the cache would
//! have been the obvious thing, and it would have compounded the shading a little more every frame.

use super::Region;
use crate::tool::PainterTool;

/// Slope gain at Amount = 1: how steep a wall a height step of 1.0 across one pixel becomes.
///
/// **Measured, not guessed.** The first cut used `8.0`, chosen by taste, and the relief came out
/// FLAT — because a real stroke's steepest slope is only ~0.026 height-units per pixel, which at
/// gain 8 tilts the normal about 6°: nothing. Calibrated against an actual dragged stroke
/// (`flat_probe_exact_smoke_arming`): at 40 the shading moves the pixels ~90 levels, which reads as
/// thick paint. The relative-shading bounds (`AMBIENT`..2×) keep it from blowing out. // CLAMP-OK
const SLOPE_GAIN: f32 = 40.0;

/// Blinn-Phong shininess exponent. Tight enough that the highlight rides the *crest* of a ridge
/// instead of washing over its flanks. // CLAMP-OK
const SHININESS: f32 = 24.0;

/// Entries in the specular LUT — `pow` is a transcendental, so it is baked once per pass and looked up
/// per pixel (HR-5; the precedent is `watercolor_lut.rs`). // CLAMP-OK
const SPEC_LUT: usize = 256;

/// Lowest light elevation the pass will use, in degrees. At elevation 0 the light grazes the canvas,
/// the flat-surface response goes to zero and the relative shading divides by ~0. // CLAMP-OK
const MIN_ELEV_DEG: u16 = 5;

/// **Ambient** floor of the diffuse term: what a face turned fully AWAY from the light still returns.
///
/// Paint in shadow is darker; it is not black. Without this the shading floors at `0` and multiplies
/// the pixel to zero — which is exactly what put the black smudges on the stroke ends of Enio's smoke
/// (a stroke's cap is where the height falls from full to nothing over a pixel, so it is the steepest
/// slope on the canvas, so it is the first place a zero-floor bites). Folded so a FLAT surface still
/// returns exactly `1.0` — the byte-identity contract survives. // CLAMP-OK
const AMBIENT: f32 = 0.35;

/// Height (in deposit units) below which the relief is a *film*, not a *body*.
///
/// The normal comes from the **slope**, not the height — so a vanishingly thin layer of paint whose
/// grain swings per texel has micro-slopes as steep as a real ridge's, and would be shaded just as
/// hard, drawing a halo of shadow over paint the eye cannot even see. The `body` factor below this
/// height scales the SLOPE (a film drapes; it does not stand up in ridges) *and* fades the effect, so
/// the tail dies quadratically. Calibrated: with the slope scaled and this at 0.20, exactly zero
/// unpainted pixels are shaded, while the relief still reads at full strength — fading the effect
/// alone could not do both (`impasto_light_does_not_shade_paint_that_is_not_there`). // CLAMP-OK
const BODY_MIN: f32 = 0.20;

/// The lighting environment, resolved once per pass.
struct Light {
    /// Unit light direction (x, y, z); `z > 0` points out of the canvas.
    dir: [f32; 3],
    /// The diffuse response of a FLAT surface (`= dir.z`) — the divisor that keeps flat paint at 1.0.
    flat_diffuse: f32,
    /// The specular response of a FLAT surface — subtracted so flat paint gains no highlight.
    flat_spec: f32,
    /// `pow(x, SHININESS)` for `x ∈ [0, 1]`, baked (HR-5).
    spec_lut: [f32; SPEC_LUT],
    /// Half-vector between the light and the (orthographic) view direction `(0, 0, 1)`.
    half: [f32; 3],
    /// Slope gain (Amount × [`SLOPE_GAIN`]).
    gain: f32,
    /// Specular strength (Shine).
    shine: f32,
}

impl Light {
    fn new(angle_deg: u16, elev_deg: u16, amount: f32, shine: f32) -> Self {
        // Transcendental-free (HR-5): the shared 1°-step rotor, the same one the brush's Jitter Rotate
        // and per-slot Angle are built from.
        let elev = elev_deg.clamp(MIN_ELEV_DEG, 90);
        let az = ph2d_painter_brush::texture::rotate_by_degrees(angle_deg % 360);
        let el = ph2d_painter_brush::texture::rotate_by_degrees(elev);
        let (cos_e, sin_e) = (el[0], el[1]);
        let dir = [cos_e * az[0], cos_e * az[1], sin_e];
        let half = {
            let h = [dir[0], dir[1], dir[2] + 1.0]; // view = (0, 0, 1), orthographic
            let len = (h[0] * h[0] + h[1] * h[1] + h[2] * h[2]).sqrt().max(1e-6);
            [h[0] / len, h[1] / len, h[2] / len]
        };
        let mut spec_lut = [0.0f32; SPEC_LUT];
        for (i, e) in spec_lut.iter_mut().enumerate() {
            let x = i as f32 / (SPEC_LUT - 1) as f32;
            *e = x.powf(SHININESS);
        }
        let lut = |x: f32| {
            let i = (x.clamp(0.0, 1.0) * (SPEC_LUT - 1) as f32) as usize;
            spec_lut[i]
        };
        // A flat surface's own response — the reference the whole pass is relative to.
        let flat_spec = lut(half[2]); // N_flat = (0, 0, 1) ⇒ N·H = half.z
        Self {
            dir,
            flat_diffuse: sin_e.max(1e-3),
            flat_spec,
            spec_lut,
            half,
            gain: amount.clamp(0.0, 1.0) * SLOPE_GAIN,
            shine: shine.clamp(0.0, 1.0),
        }
    }

    /// Shade one pixel from its height gradient and the amount of paint (`h`) actually there. Returns
    /// `(multiply, add)`: the composite's RGB is `rgb × multiply + add`. A FLAT pixel — or one with no
    /// real body of paint — returns exactly `(1.0, 0.0)`.
    #[inline]
    fn shade(&self, h: f32, dhx: f32, dhy: f32) -> (f32, f32) {
        if dhx == 0.0 && dhy == 0.0 {
            return (1.0, 0.0); // flat paint is untouched, to the byte
        }
        // How much of a BODY is here. The normal comes from the slope, so without this a film of paint
        // one thousandth deep — but with a per-texel grain — would shade as hard as a real ridge, and
        // draw shadows over the brush's invisible falloff tail. No body, no relief.
        let body = (h.abs() / BODY_MIN).min(1.0);
        if body <= 0.0 {
            return (1.0, 0.0);
        }
        // Surface normal from the gradient: a rising slope tilts the normal AGAINST the rise. The slope
        // is scaled by the BODY — a film of paint drapes, it does not stand up in ridges — so the
        // brush's invisible falloff tail flattens out quadratically instead of catching hard shadows.
        let n = {
            let v = [-dhx * self.gain * body, -dhy * self.gain * body, 1.0];
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
            [v[0] / len, v[1] / len, v[2] / len]
        };
        let ndl = n[0] * self.dir[0] + n[1] * self.dir[1] + n[2] * self.dir[2];
        // RELATIVE diffuse, with an AMBIENT floor: exactly 1.0 on flat ground (so flat paint stays
        // byte-identical), above 1 on a face turned toward the light, and down to `AMBIENT` — never 0 —
        // on a face turned away. Paint in shadow is dark, not black.
        let ratio = (ndl.max(0.0) / self.flat_diffuse).clamp(0.0, 2.0);
        let mut mul = AMBIENT + (1.0 - AMBIENT) * ratio;
        let ndh = n[0] * self.half[0] + n[1] * self.half[1] + n[2] * self.half[2];
        let i = (ndh.clamp(0.0, 1.0) * (SPEC_LUT - 1) as f32) as usize;
        let mut add = self.shine * (self.spec_lut[i] - self.flat_spec).max(0.0);
        // Fade the whole effect in with the body, so the pass is a strict no-op on bare canvas.
        mul = 1.0 + (mul - 1.0) * body;
        add *= body;
        (mul, add)
    }
}

impl PainterTool {
    /// Whether the light pass has anything to do: it is switched on and some layer carries relief.
    /// Cheap enough to call per frame — the height map is empty for every document nobody has sculpted.
    #[must_use]
    pub fn impasto_visible(&self) -> bool {
        self.paint.impasto_show
            && (!self.heights.is_empty() || !self.paint.stroke_height.is_empty())
    }

    /// Whether `id` is visible *and every group above it is too*. Hiding a GROUP has to put out the
    /// light on everything inside it — checking only the layer's own flag would leave the relief of a
    /// hidden group's paint still catching the light, over pixels that are no longer there.
    fn layer_effectively_visible(&self, id: crate::tool::RtLayerId) -> bool {
        let mut cur = Some(id);
        while let Some(c) = cur {
            match self.layers.get(c) {
                Some(l) if l.visible => cur = self.layers.parent_of(c),
                _ => return false,
            }
        }
        true
    }

    /// Total relief over the VISIBLE layers, canvas-sized. `None` when nothing is lit.
    ///
    /// Fase 1 composites the heights by **Add** (per-layer Subtract / Replace / Ignore is named and
    /// deferred). A hidden layer contributes nothing — hide the layer and its paint stops catching the
    /// light, which is the only behaviour that would not surprise anyone.
    fn impasto_height_total(&self) -> Option<Vec<f32>> {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if n == 0 {
            return None;
        }
        // Only layers that were actually sculpted have an entry — the map is lazy, so this loop is empty
        // for every document nobody has used Impasto on. The in-progress stroke rides on the active
        // layer's entry (`layer_height_view`), so it must be considered even before the layer HAS one.
        let mut ids: Vec<crate::tool::RtLayerId> = self.heights.keys().copied().collect();
        if let Some(a) = self.layers.active()
            && !self.paint.stroke_height.is_empty()
            && !ids.contains(&a)
        {
            ids.push(a);
        }
        let mut total: Option<Vec<f32>> = None;
        for id in ids {
            if !self.layer_effectively_visible(id) {
                continue;
            }
            let Some(field) = self.layer_height_view(id) else {
                continue;
            };
            if field.len() != n {
                continue; // a field shaped for a document that is no longer bound
            }
            match total.as_mut() {
                None => total = Some(field),
                Some(acc) => {
                    for (a, b) in acc.iter_mut().zip(field.iter()) {
                        *a += b;
                    }
                }
            }
        }
        total.filter(|t| t.iter().any(|&v| v != 0.0))
    }

    /// Light `rgba` — the pixels of `region`, freshly composited and NOT yet lit (`rgba` is
    /// `region.w × region.h × 4`, straight sRGB8). No-op when the pass is off or nothing has relief.
    pub(crate) fn apply_impasto_light(&self, rgba: &mut [u8], region: Region) {
        if !self.impasto_visible() {
            return;
        }
        let Some(height) = self.impasto_height_total() else {
            return;
        };
        let (w, h) = self.source_size;
        if rgba.len() < (region.w as usize) * (region.h as usize) * 4 {
            return; // shape guard — bail rather than index out of a mis-sized buffer
        }
        let light = Light::new(
            self.paint.impasto_light_angle_deg,
            self.paint.impasto_light_elev_deg,
            self.paint.impasto_light_amount,
            self.paint.impasto_shine,
        );
        let at = |x: i64, y: i64| -> f32 {
            let cx = x.clamp(0, w as i64 - 1) as usize;
            let cy = y.clamp(0, h as i64 - 1) as usize;
            height[cy * (w as usize) + cx]
        };
        for ry in 0..region.h {
            let gy = i64::from(region.y + ry);
            for rx in 0..region.w {
                let gx = i64::from(region.x + rx);
                // Central differences — the normal reads across the region's edge into the canvas, so a
                // dirty-rect update lights its border exactly as a full recompose would.
                let dhx = (at(gx + 1, gy) - at(gx - 1, gy)) * 0.5;
                let dhy = (at(gx, gy + 1) - at(gx, gy - 1)) * 0.5;
                let (mul, add) = light.shade(at(gx, gy), dhx, dhy);
                if mul == 1.0 && add == 0.0 {
                    continue; // flat: byte-identical, and not even a rounding trip through f32
                }
                let i = ((ry as usize) * (region.w as usize) + rx as usize) * 4;
                for c in 0..3 {
                    let v = f32::from(rgba[i + c]) / 255.0;
                    let lit = (v * mul + add).clamp(0.0, 1.0);
                    rgba[i + c] = (lit * 255.0 + 0.5) as u8;
                }
            }
        }
    }
}
