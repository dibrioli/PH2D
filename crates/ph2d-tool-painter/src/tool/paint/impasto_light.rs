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
use crate::layers::ReliefComposite;
use crate::tool::PainterTool;

/// How many CANVAS PIXELS of physical paint height `h = 1.0` (one full-Depth stroke) represents.
///
/// This is the industry's decomposition of the height-to-slope gain — *a physical height over the
/// texel size*, the Blender Bump node's `Distance`, Substance's "Height Depth (cm)", HDRP's
/// amplitude-in-centimetres (`docs/Painter/17_impasto_deposito_pesquisa2.md` §4) — instead of a
/// unitless fudge. The slope is then geometry: `∇h × DEPTH_UNIT_PX` height-pixels per pixel, no
/// gain knob multiplying it (Substance Painter exposes none either; the deposit value is the knob).
///
/// The number itself is perceptual and MEASURED on the probe (`probe`-family tests): the default
/// soft brush at r=40 lays its wall over the falloff's `W_TAIL..W_SOLID` band, ~11 px — so at 16,
/// a Depth 0.7 stroke is ~11 px of paint falling over ~11 px of wall: the classic 45° bevel, and
/// the probe's shading peak sits ON that wall (42% → edge band) instead of inside the stroke. It
/// has a way to be wrong now: if strokes read taller or flatter than `DEPTH_UNIT_PX` pixels of
/// real paint would, this constant is lying. // CLAMP-OK
const DEPTH_UNIT_PX: f32 = 16.0;

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

/// How the relief's effect scales with how much PAINT is at a pixel.
///
/// The pass multiplies the composited pixel — and at a stroke's translucent edge that pixel is mostly
/// **paper showing through the paint**. Shading it in full means shading the paper: on a white canvas
/// that bleach was the halo Enio photographed (2026-07-12; 81 levels at the 20–60%-ink edge vs 55 at
/// the core, gone on black canvas). So the effect is weighted by the paint's own coverage, and bare
/// paper gets exactly none — the pass stays a strict no-op there.
///
/// But the weight is the deposit's own [`ph2d_painter_brush::height::body_profile`], on the SAME
/// thresholds: nothing over the stain (≤ `W_TAIL` coverage — where the wall does not stand, the
/// light does not push), full over solid paint (≥ `W_SOLID`), the ramp between. One curve, one
/// definition of "solid paint", both sides of the pipeline. The first cut instead weighted linearly
/// to 100% AND multiplied the slope by the same factor — a quadratic mute that melted the shoulder
/// of every soft brush (the "hard to adjust" verdict; measured in §10 of the plan). The slope no
/// longer carries any mute: the body curve already ends the relief where the paint gets thin, so
/// the geometry is real wherever it is nonzero.
#[inline]
fn paint_body(cover: f32) -> f32 {
    ph2d_painter_brush::height::body_profile(cover)
}

/// How the SPECULAR scales with the paint.
///
/// It is the **same body curve** the diffuse uses ([`paint_body`]) — and that is not laziness, it is
/// the only place the glint can live. The relief's slope exists exactly where the wall is, i.e. over
/// the coverage band `W_TAIL..W_SOLID`; gating the highlight *above* `W_SOLID` (the first cut, to kill
/// the halo) therefore allowed it only on the plateau, which is FLAT — the pass early-outs there and
/// adds nothing. Measured: Shine 0 → 1 moved the brightest pixel by **1 level**. A knob that does
/// nothing (Enio: *"shine não funciona"*), and the exact species the 2026-07-12 sweep exterminated.
///
/// The halo the gate is guarding against is a bleach of the PAPER seen through translucent paint, and
/// the body curve already refuses that: it is zero over the stain and only reaches full on solid
/// paint. So the glint climbs the wall with the body and peaks on the crest — where oil paint glints —
/// while the rim, which has barely any body, gets barely any highlight.
///
/// (`impasto_shine_glints_on_the_wall_without_bleaching_the_rim` pins BOTH halves: the glint must be
/// visible, and it must not bleach. Fixing one by breaking the other is how this knob died the first
/// time.)
#[inline]
fn gloss_body(cover: f32) -> f32 {
    paint_body(cover)
}

/// One layer's contribution to the composed relief, in z-order.
struct ReliefLayer<'a> {
    /// The committed height plane. `None` for the active layer when it has only a live stroke on it.
    height: Option<&'a [f32]>,
    /// Its paint coverage, when it has one.
    cover: Option<&'a [u8]>,
    /// The layer's own **Impasto depth** (`Layer::impasto_depth`) — `1` composites as sculpted, `0`
    /// mutes, negative inverts the relief.
    depth: f32,
    /// How this layer meets the relief below it.
    composite: ReliefComposite,
    /// This is the ACTIVE layer, so the open stroke rides on it (`live_h`/`live_c` fold in here — at
    /// this layer's z, under this layer's depth, not on top of the whole pile).
    active: bool,
}

/// The relief + coverage the light reads, sampled straight out of the layer store — no composed buffer,
/// no per-frame allocation. See [`PainterTool::impasto_fields`].
struct ReliefFields<'a> {
    /// Every visible layer that carries relief, **bottom-up** — the order it composites in. The order
    /// is load-bearing now: [`ReliefComposite::Level`] buries what is *under* it, and until it existed
    /// the fold was a commutative sum that could iterate in any order at all.
    layers: Vec<ReliefLayer<'a>>,
    /// The open stroke's relief on the active layer (`None` outside a stroke, or while erasing — an
    /// erase mutates the layer in place, so there is nothing separate to add).
    live_h: Option<&'a [f32]>,
    /// That stroke's PAINT envelope (`0..1`) — the same plane the relief is derived from, read here as
    /// the live coverage. The committed layers carry theirs as `u8`; a stroke in progress has it in
    /// full precision, and there is no reason to round-trip it.
    live_c: Option<&'a [f32]>,
    width: usize,
    height: usize,
}

impl ReliefFields<'_> {
    /// Height at a canvas pixel, clamped to the canvas (so the central difference can read across the
    /// dirty rect's edge exactly as a full recompose would) — and to the glass ceiling, so the LIVE
    /// stroke over an already-full pile shows the same paint the commit is about to store (no pop at
    /// pen-up), and stacked layers top out exactly as stacked strokes do.
    ///
    /// The fold walks the layers bottom-up, each one scaled by its own depth and joined to the pile
    /// under it by its own composite mode.
    #[inline]
    fn height_at(&self, x: i64, y: i64) -> f32 {
        let i = self.index(x, y);
        let mut h = 0.0f32;
        for l in &self.layers {
            let mut own = l.height.map_or(0.0, |f| f[i]);
            if l.active {
                own += self.live_h.map_or(0.0, |s| s[i]);
            }
            own *= l.depth;
            match l.composite {
                ReliefComposite::Add => h += own,
                // Bury, in proportion to this layer's own paint: solid paint IS the surface, bare paint
                // shows the pile below untouched. Anything else would make an empty region of a `Level`
                // layer flatten the whole painting.
                ReliefComposite::Level => {
                    let c = self.layer_cover_at(l, i);
                    h = h * (1.0 - c) + own * c;
                }
            }
        }
        h.clamp(-super::impasto::H_CEIL, super::impasto::H_CEIL)
    }

    /// One layer's own paint coverage at `i` (`0..1`) — including the open stroke when it is the active
    /// one. This is the `Level` fold's weight; it is NOT [`Self::cover_at`], which is the whole
    /// canvas's.
    #[inline]
    fn layer_cover_at(&self, l: &ReliefLayer<'_>, i: usize) -> f32 {
        let mut c = l.cover.map_or(0.0, |cv| f32::from(cv[i]) / 255.0);
        if l.active {
            c = c.max(self.live_c.map_or(0.0, |s| s[i]));
        }
        c.clamp(0.0, 1.0)
    }

    /// Paint coverage at a canvas pixel (`0..1`) — the MAX over the layers, not the sum: it is a
    /// presence, not a quantity (two layers of paint over one pixel do not make it 200% paint).
    ///
    /// Deliberately NOT scaled by the layers' Impasto depth: this is what the light uses to decide it is
    /// looking at *paint* rather than paper ([`paint_body`]), and muting a layer's relief does not make
    /// its pigment any less present on the canvas.
    #[inline]
    fn cover_at(&self, x: i64, y: i64) -> f32 {
        let i = self.index(x, y);
        let mut c = self.live_c.map_or(0.0, |l| l[i]);
        for l in &self.layers {
            if let Some(cv) = l.cover {
                c = c.max(f32::from(cv[i]) / 255.0);
            }
        }
        c.clamp(0.0, 1.0)
    }

    #[inline]
    fn index(&self, x: i64, y: i64) -> usize {
        let cx = x.clamp(0, self.width as i64 - 1) as usize;
        let cy = y.clamp(0, self.height as i64 - 1) as usize;
        cy * self.width + cx
    }
}

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
    /// Specular strength (Shine).
    shine: f32,
}

impl Light {
    fn new(angle_deg: u16, elev_deg: u16, shine: f32) -> Self {
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
            shine: shine.clamp(0.0, 1.0),
        }
    }

    /// Shade one pixel from its height gradient and the paint actually there: `body` weights the
    /// diffuse modelling ([`paint_body`]), `gloss` the highlight ([`gloss_body`]). Returns
    /// `(multiply, glint)`: the composite's RGB is `screen(rgb × multiply, glint)` — see
    /// [`PainterTool::apply_impasto_light`]. A FLAT pixel — or one with no real body of paint —
    /// returns exactly `(1.0, 0.0)`.
    #[inline]
    fn shade(&self, body: f32, gloss: f32, dhx: f32, dhy: f32) -> (f32, f32) {
        if (dhx == 0.0 && dhy == 0.0) || body <= 0.0 {
            return (1.0, 0.0); // flat paint — or bare paper — is untouched, to the byte
        }
        // Surface normal from the gradient: a rising slope tilts the normal AGAINST the rise. The
        // slope is GEOMETRY — the height buffer's unit converted to pixels ([`DEPTH_UNIT_PX`]) — with
        // no gain and no coverage mute: the body curve already ends the relief where the paint thins,
        // so wherever the gradient is nonzero there is real paint standing there.
        let n = {
            let v = [-dhx * DEPTH_UNIT_PX, -dhy * DEPTH_UNIT_PX, 1.0];
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
        // Fade both with the body, so the pass is a strict no-op on bare canvas.
        //
        // (Tried and rejected, 2026-07-12: weighting the BRIGHTENING more harshly than the shadow, to
        // stop the light washing out translucent paint. It bought 5 points of chroma and cost the
        // modelling entirely — the lit flank came out exactly as bright as the paint under it, which is
        // what "no relief" looks like. The washing-out is not a bug to weight away: brightening a pixel
        // whose pigment channel is already at the ceiling ALWAYS costs saturation, in paint as in
        // physics. The line that must hold is the other one — the light may not touch paint with no
        // body at all — and that is what `body` here, and the gates, enforce.)
        mul = 1.0 + (mul - 1.0) * body;
        add *= gloss;
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

    /// The relief and the paint coverage the light must read, as BORROWED slices — never materialised.
    ///
    /// Building the composed fields into two canvas-sized buffers is the obvious thing, and it is what
    /// this did first: it cost an `O(canvas)` allocate-and-sum on **every frame**, while the pass itself
    /// only ever lights the dirty rect. At 2048² that is the difference between 3.9 ms and 2.1 ms per
    /// move — most of the impasto budget spent composing pixels nobody was going to look at.
    ///
    /// Heights **add** across layers (more paint IS thicker); coverage takes the **max** (it is a
    /// presence, not a quantity — two layers of paint over one pixel do not make it 200% paint). Per-layer
    /// Subtract / Replace / Ignore is named and deferred. A hidden layer contributes neither: hide it and
    /// its paint stops catching the light.
    fn impasto_fields(&self) -> Option<ReliefFields<'_>> {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if n == 0 {
            return None;
        }
        let active = self.layers.active();
        // The open stroke rides on the active layer — which may not have a committed entry yet.
        let live_visible = active.is_some_and(|a| self.layer_effectively_visible(a));
        let live_h = (live_visible && self.paint.stroke_height.len() == n && !self.paint.eraser)
            .then_some(self.paint.stroke_height.as_slice());
        let live_c = (live_visible && self.paint.stroke_paint.len() == n && !self.paint.eraser)
            .then_some(self.paint.stroke_paint.as_slice());

        // Bottom-up, because `Level` is not commutative. Only layers that were actually sculpted have
        // an entry — the map is lazy, so this is empty for every document nobody has used Impasto on
        // (plus the active layer, when a live stroke is in flight on a layer that has none yet).
        let mut layers: Vec<ReliefLayer<'_>> = Vec::new();
        for id in self.layers.z_order_bottom_up() {
            if !self.layer_effectively_visible(id) {
                continue;
            }
            let is_active = active == Some(id);
            let height = self
                .heights
                .get(&id)
                .map(Vec::as_slice)
                .filter(|f| f.len() == n);
            let carries_live = is_active && live_h.is_some();
            if height.is_none() && !carries_live {
                continue;
            }
            let Some(layer) = self.layers.get(id) else {
                continue;
            };
            layers.push(ReliefLayer {
                height,
                cover: self
                    .covers
                    .get(&id)
                    .map(Vec::as_slice)
                    .filter(|c| c.len() == n),
                depth: layer.impasto_depth,
                composite: layer.impasto_composite,
                active: is_active,
            });
        }
        if layers.is_empty() {
            return None;
        }
        Some(ReliefFields {
            layers,
            live_h,
            live_c,
            width: w as usize,
            height: h as usize,
        })
    }

    /// Light `rgba` — the pixels of `region`, freshly composited and NOT yet lit (`rgba` is
    /// `region.w × region.h × 4`, straight sRGB8). No-op when the pass is off or nothing has relief.
    pub(crate) fn apply_impasto_light(&self, rgba: &mut [u8], region: Region) {
        if !self.impasto_visible() {
            return;
        }
        let Some(fields) = self.impasto_fields() else {
            return;
        };
        if rgba.len() < (region.w as usize) * (region.h as usize) * 4 {
            return; // shape guard — bail rather than index out of a mis-sized buffer
        }
        let light = Light::new(
            self.paint.impasto_light_angle_deg,
            self.paint.impasto_light_elev_deg,
            self.paint.impasto_shine,
        );
        let at = |x: i64, y: i64| fields.height_at(x, y);
        let cover_at = |x: i64, y: i64| fields.cover_at(x, y);
        for ry in 0..region.h {
            let gy = i64::from(region.y + ry);
            for rx in 0..region.w {
                let gx = i64::from(region.x + rx);
                // Central differences — the normal reads across the region's edge into the canvas, so a
                // dirty-rect update lights its border exactly as a full recompose would.
                let dhx = (at(gx + 1, gy) - at(gx - 1, gy)) * 0.5;
                let dhy = (at(gx, gy + 1) - at(gx, gy - 1)) * 0.5;
                let cover = cover_at(gx, gy);
                let (mul, add) = light.shade(paint_body(cover), gloss_body(cover), dhx, dhy);
                if mul == 1.0 && add == 0.0 {
                    continue; // flat: byte-identical, and not even a rounding trip through f32
                }
                let i = ((ry as usize) * (region.w as usize) + rx as usize) * 4;
                for c in 0..3 {
                    let v = f32::from(rgba[i + c]) / 255.0;
                    let lit = light_pixel(v, mul, add);
                    rgba[i + c] = (lit * 255.0 + 0.5) as u8;
                }
            }
        }
    }
}

/// Apply the shading to ONE channel: the diffuse **modulates**, the highlight is light **added
/// against the headroom that is left** (a screen), never a flat `+ add`.
///
/// The screen is not a taste call, it is the only form that keeps the paint's colour. Write it out:
/// `screen(v) = v·(1 − add) + add`, so for two channels of the same pixel
/// `screen(R) − screen(G) = (R − G)·(1 − add)` — the hue is EXACTLY preserved and the chroma merely
/// scales, for every `add < 1`. A flat `v + add` instead clamps: on a translucent rim the pigment's own
/// channel is already at the ceiling, so only the OTHER channels rise and the colour collapses to white
/// — the halo Enio photographed, arriving through the specular door. (It is also what a real highlight
/// does: approach white, never overshoot it.)
#[inline]
fn light_pixel(v: f32, mul: f32, add: f32) -> f32 {
    let lit = (v * mul).clamp(0.0, 1.0);
    (lit + add * (1.0 - lit)).clamp(0.0, 1.0)
}

#[cfg(test)]
impl PainterTool {
    /// The **composed** relief at a canvas pixel — depth-scaled, mode-folded, ceiling-clamped: exactly
    /// the number the light reads.
    ///
    /// Deliberately the light's OWN sampler, not a re-implementation of the fold for the gates to
    /// compare against. An oracle that re-derives the thing it is testing agrees with the bug
    /// ([[feedback_oracle_must_model_appearance_not_implementation]]); this one asks the pass what it
    /// sees. (`layer_height_view` is a different question — one layer's raw plane, before any of this.)
    pub(crate) fn composed_relief_at(&self, x: u32, y: u32) -> f32 {
        self.impasto_fields()
            .map_or(0.0, |f| f.height_at(i64::from(x), i64::from(y)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_highlight_scales_chroma_and_never_annihilates_it() {
        // The algebraic heart of the specular blend. A flat additive highlight (`v + add`) destroys the
        // paint's colour on any pixel whose pigment channel is already at the ceiling — which is every
        // pixel of red paint on white paper. The screen cannot: it scales both channels by `(1 − add)`
        // and lifts both by `add`, so the DIFFERENCE between them — the pigment — survives in exact
        // proportion. RED with `let lit = v * mul + add`: the chroma below goes to 0.
        let pigment = |r: f32, g: f32| (r - g).max(0.0);
        for &add in &[0.0f32, 0.25, 0.5, 0.9] {
            // A rim pixel: red at the ceiling, green half-way (pale pink over white paper).
            let (r, g) = (1.0f32, 0.6f32);
            let (lr, lg) = (light_pixel(r, 1.0, add), light_pixel(g, 1.0, add));
            let before = pigment(r, g);
            let after = pigment(lr, lg);
            let expected = before * (1.0 - add);
            assert!(
                (after - expected).abs() < 1e-6,
                "the highlight scales the pigment by (1 − add): add {add} ⇒ {after} vs {expected}"
            );
            assert!(
                add >= 1.0 || after > 0.0,
                "…and never annihilates it (add {add} left {after})"
            );
        }
        // The other half of the contract: no glint, no light ⇒ the pixel is untouched, to the float.
        assert_eq!(light_pixel(0.42, 1.0, 0.0), 0.42);
        // And the highlight approaches white without overshooting it.
        assert!(light_pixel(0.9, 1.0, 1.0) <= 1.0);
    }
}
