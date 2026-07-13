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
use super::impasto_shade::Rig;
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
pub(super) const DEPTH_UNIT_PX: f32 = 16.0;

// (The Blinn-Phong exponent used to be `const SHININESS: f32 = 24.0` right here. It is now the paint's
// **Roughness** — `ph2d_painter_brush::material` — because how BROAD a highlight is, is a property of
// the surface, not of the renderer. The old constant survives exactly, as the neutral material: the
// roughness→exponent map is geometric between 6 and 96 and `√(6·96) = 24`, so a default brush shades
// byte-identically to the build that had the constant. `Material::NEUTRAL` and its gates pin that.)

/// **Ambient** floor of the diffuse term: what a face turned fully AWAY from the light still returns.
///
/// Paint in shadow is darker; it is not black. Without this the shading floors at `0` and multiplies
/// the pixel to zero — which is exactly what put the black smudges on the stroke ends of Enio's smoke
/// (a stroke's cap is where the height falls from full to nothing over a pixel, so it is the steepest
/// slope on the canvas, so it is the first place a zero-floor bites). Folded so a FLAT surface still
/// returns exactly `1.0` — the byte-identity contract survives. // CLAMP-OK
pub(super) const AMBIENT: f32 = 0.35;

/// How the relief's effect scales with how much PAINT is at a pixel.
///
/// The pass multiplies the composited pixel — and at a stroke's translucent edge that pixel is mostly
/// **paper showing through the paint**. Shading it in full means shading the paper: on a white canvas
/// that bleach was the halo Enio photographed (2026-07-12; 81 levels at the 20–60%-ink edge vs 55 at
/// the core, gone on black canvas). So the effect is weighted by the paint's own coverage, and bare
/// paper gets exactly none — the pass stays a strict no-op there.
///
/// ## The weight IS the coverage — because the coverage is now the SOLID paint
///
/// It used to be `body_profile(cover)` over a `cover` that held the RAW paint (silhouette × dynamics),
/// and that put the dynamics INSIDE the body curve, where they could starve it: at
/// Flow × Strength × pressure ≈ `W_TAIL` the argument falls under the tail for every texel and the light
/// models **nothing anywhere on the stroke** — while the pigment, cut on the silhouette
/// ([`ph2d_painter_brush::height::film_coverage`]), is still perfectly there. Enio's haze, hiding behind
/// the mouse (which always presses at 1.0).
///
/// The layers now store the **solid paint** itself
/// ([`ph2d_painter_brush::height::solid_paint`] — `dynamics × body_profile(silhouette)`, the film's own
/// alpha), so the curve is already in the number and the weight is the number. At full dynamics the two
/// are the same value, so every stroke a mouse ever drew — and every gate that pins the look — is
/// unchanged; under a light touch the light now models a thinner film instead of refusing to see it.
///
/// One definition of "solid paint", still, on both sides of the pipeline: nothing over the stain
/// (`≤ W_TAIL` of silhouette — where the wall does not stand, the light does not push, and the brush no
/// longer lays pigment either), full over solid paint (`≥ W_SOLID`), the ramp between.
#[inline]
fn paint_body(cover: f32) -> f32 {
    cover
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
    /// Its MATERIAL plane, when it has one. `None` = a layer sculpted before materials existed; it
    /// reads as `Material::NEUTRAL`, which is the pass exactly as it shaded then.
    mat: Option<&'a [[u8; 4]]>,
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
    /// That stroke's **solid paint** plane — the same `u8` quantity the layer will store at stroke end
    /// (`PaintState::stroke_film`), so the light reads the identical number before and after the commit.
    /// (It used to be the raw f32 paint envelope, in "full precision" — and the precision was of the
    /// wrong quantity: the relief's ingredient, not the pigment's alpha.)
    live_c: Option<&'a [u8]>,
    /// The open stroke's MATERIAL — the BRUSH's, as a scalar, because a stroke's material is constant
    /// (it comes off the brush). No plane is needed for it, which is what makes the whole per-pixel
    /// material cost one merge at commit instead of a second buffer per stroke.
    live_mat: [u8; 4],
    /// `Material::NEUTRAL`, quantised ONCE — the ground the material fold starts from, and the material
    /// a layer with no entry reads as. It is a constant, and it is read per texel; deriving it in the
    /// loop is the kind of thing that costs half a millisecond and looks like nothing.
    neutral: [u8; 4],
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
            c = c.max(self.live_c.map_or(0.0, |s| f32::from(s[i]) / 255.0));
        }
        c.clamp(0.0, 1.0)
    }

    /// The MATERIAL at a canvas pixel — what the paint there IS.
    ///
    /// Folded bottom-up with **`over`**, weighted by each layer's own coverage: the paint on top is the
    /// paint you see, so an opaque layer's material replaces what is under it and a translucent one
    /// mixes. (Deliberately NOT `max` like [`Self::cover_at`] — coverage is a *presence* and material is
    /// an *identity*, and folding an identity by max would mean "the glossiest layer wins", which is not
    /// a thing paint does.)
    ///
    /// The fold starts at NEUTRAL, not at zero, and that is load-bearing: zero is `roughness = 0`, which
    /// is a MIRROR, so bare paper would be the shiniest thing on the canvas and every stroke's
    /// translucent rim would fade toward glass.
    #[inline]
    fn material_at(&self, x: i64, y: i64) -> [u8; 4] {
        let i = self.index(x, y);
        // Hoisted, and it matters: this runs once per TEXEL. Quantising the neutral material inside the
        // loop cost 0.5 ms/move at 2048² all by itself — the fold does four channels over up to four
        // layers, and `to_bytes` is four clamps and four multiplies each time it is asked.
        let neutral = self.neutral;
        let mut m = [
            f32::from(neutral[0]),
            f32::from(neutral[1]),
            f32::from(neutral[2]),
            f32::from(neutral[3]),
        ];
        for l in &self.layers {
            let a = self.layer_cover_at(l, i);
            if a <= 0.0 {
                continue;
            }
            let src = match l.mat {
                Some(mt) => mt[i],
                // A layer with relief but no material entry is a document from before materials existed:
                // it reads as the neutral material, which IS the pass as it shaded then.
                None => neutral,
            };
            for (c, v) in m.iter_mut().enumerate() {
                *v += (f32::from(src[c]) - *v) * a;
            }
        }
        // The live stroke is the topmost paint on the active layer: its material is the BRUSH's, and it
        // is what the artist is looking at while they turn the knob.
        if let Some(lc) = self.live_c {
            let a = f32::from(lc[i]) / 255.0;
            if a > 0.0 {
                for (c, v) in m.iter_mut().enumerate() {
                    *v += (f32::from(self.live_mat[c]) - *v) * a;
                }
            }
        }
        [
            (m[0] + 0.5) as u8,
            (m[1] + 0.5) as u8,
            (m[2] + 0.5) as u8,
            (m[3] + 0.5) as u8,
        ]
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
        let mut c = self.live_c.map_or(0.0, |l| f32::from(l[i]) / 255.0);
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

/// One resolved lamp: its direction, its half-vector, and what a FLAT surface returns to it. The whole
/// pass is relative to that flat response — see [`Rig`].
impl PainterTool {
    /// Whether the light pass has anything to do: it is switched on and some layer carries relief.
    /// Cheap enough to call per frame — the height map is empty for every document nobody has sculpted.
    #[must_use]
    pub fn impasto_visible(&self) -> bool {
        self.paint.impasto_show
            && (!self.heights.is_empty() || !self.paint.relief.stroke_height.is_empty())
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
        let live_h =
            (live_visible && self.paint.relief.stroke_height.len() == n && !self.paint.eraser)
                .then_some(self.paint.relief.stroke_height.as_slice());
        let live_c =
            (live_visible && self.paint.relief.stroke_film.len() == n && !self.paint.eraser)
                .then_some(self.paint.relief.stroke_film.as_slice());

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
                .map(|f| f.as_slice())
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
                    .map(|c| c.as_slice())
                    .filter(|c| c.len() == n),
                mat: self
                    .mats
                    .get(&id)
                    .map(|m| m.as_slice())
                    .filter(|m| m.len() == n),
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
            live_mat: self.paint.brush.material().to_bytes(),
            neutral: ph2d_painter_brush::material::Material::NEUTRAL.to_bytes(),
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
        let Some(light) = Rig::new(&self.paint.impasto_rig) else {
            return; // every lamp switched off: the canvas comes back unlit, to the byte
        };
        let at = |x: i64, y: i64| fields.height_at(x, y);
        let cover_at = |x: i64, y: i64| fields.cover_at(x, y);
        // Materials are piecewise-constant across a canvas (one per stroke), so a ONE-entry cache turns
        // the per-material resolve — which now owns the flat divisor, and so is not free — into a few
        // calls per pass instead of one per texel.
        let mut mat = light.resolve(ph2d_painter_brush::material::Material::NEUTRAL.to_bytes());
        for ry in 0..region.h {
            let gy = i64::from(region.y + ry);
            for rx in 0..region.w {
                let gx = i64::from(region.x + rx);
                // Central differences — the normal reads across the region's edge into the canvas, so a
                // dirty-rect update lights its border exactly as a full recompose would.
                let dhx = (at(gx + 1, gy) - at(gx - 1, gy)) * 0.5;
                let dhy = (at(gx, gy + 1) - at(gx, gy - 1)) * 0.5;
                let cover = cover_at(gx, gy);
                let i = ((ry as usize) * (region.w as usize) + rx as usize) * 4;
                // The pixel's own colour IS a metal's highlight, so it is an INPUT to the shade, not
                // only the thing the shade multiplies.
                let albedo = [
                    f32::from(rgba[i]) / 255.0,
                    f32::from(rgba[i + 1]) / 255.0,
                    f32::from(rgba[i + 2]) / 255.0,
                ];
                let key = fields.material_at(gx, gy);
                if key != mat.key {
                    mat = light.resolve(key);
                }
                let (mul, add) =
                    light.shade(&mat, paint_body(cover), gloss_body(cover), dhx, dhy, albedo);
                if mul == [1.0; 3] && add == [0.0; 3] {
                    continue; // flat: byte-identical, and not even a rounding trip through f32
                }
                for c in 0..3 {
                    let lit = light_pixel(albedo[c], mul[c], add[c]);
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
