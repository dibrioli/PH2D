//! **Impasto** — the RIG, and how it shades one pixel.
//!
//! Split out of `impasto_light.rs` (the workspace LOC cap) and cohesive on its own: this is the
//! *optics*, and its sibling is the *plumbing* (which pixels, which layers, which region). Everything
//! here is a pure function of a normal, a material and the lamps — no canvas, no layers, no I/O.
//!
//! ## The one property everything hangs on: the shading is RELATIVE
//!
//! A pixel's response is divided by the response of a **flat surface of the SAME MATERIAL**, and the
//! specular has that flat surface's own highlight subtracted. So flat paint multiplies by exactly 1 and
//! adds exactly 0, and comes out byte-identical — at every material. An absolute model would darken the
//! whole painting the moment the light came on (a flat surface lit from 45° returns 0.7, not 1); half
//! the emboss filters ever written have that bug, and every new knob here is a fresh door for it. Wax
//! wraps the diffuse, so it wraps the DIVISOR too. Roughness moves the exponent, so it moves the flat
//! specular too. That is not symmetry for its own sake — it is the contract.

use super::impasto_light::{AMBIENT, DEPTH_UNIT_PX};
use ph2d_painter_brush::material::MaterialBytes;

pub(super) struct Lamp {
    /// Unit light direction (x, y, z); `z > 0` points out of the canvas.
    dir: [f32; 3],
    /// Half-vector between the light and the (orthographic) view direction `(0, 0, 1)`.
    half: [f32; 3],
    /// Weighted colour: `intensity × colour`. Weight and hue never appear apart again — every sum below
    /// is over this one vector, which is why a rig of coloured lamps costs exactly what a rig of white
    /// ones does.
    tint: [f32; 3],
}

/// A material, RESOLVED against the rig — everything that depends on both, computed once for a given
/// material rather than per pixel.
///
/// It exists because the flat response is no longer a property of the rig alone. Roughness sets the
/// specular exponent and Wax wraps the diffuse, so *what a flat surface returns* — the divisor the whole
/// pass is relative to — is now a function of the **material at that pixel**. Recomputing it per texel
/// would put a division and a LUT fetch per lamp back in the inner loop.
///
/// It is cheap because of a fact about paint: a material is CONSTANT across a stroke (it comes off the
/// brush). So the canvas is piecewise-constant in material and a one-entry cache keyed on the stored
/// bytes hits essentially always — [`Rig::resolve`] runs a handful of times per pass, not 16 million.
pub(super) struct MatShade {
    /// The 4 canvas bytes this was resolved from — the cache key.
    pub(super) key: MaterialBytes,
    /// Row of the shared specular table this roughness reads.
    level: usize,
    shine: f32,
    metallic: f32,
    wax: f32,
    /// The FILTER on the scattered light ([`ph2d_painter_brush::material::Material::wax_color`]). White
    /// (the default) leaves the physics alone: the light comes back wearing the paint's own colour. The
    /// effective tint is `albedo × wax_color` — a filter, never a replacement, because the paint's own
    /// colour is what the light actually picks up on the way through, and because a replacement has no
    /// neutral you could put in a swatch.
    wax_color: [f32; 3],
    /// The flat surface's diffuse, in TWO parts — because the wrapped light is TINTED by the paint and
    /// the paint is per pixel, while this is cached per material.
    ///
    /// `flat[c] = flat_base[c] + albedo[c] · flat_wax[c]`, which is exact and costs one multiply-add per
    /// channel at the texel. `flat_base` is `Σ tintᵢ[c] · Lᵢ.z` (the un-wrapped flat response — the old
    /// divisor); `flat_wax` is `Σ tintᵢ[c] · (wrap(Lᵢ.z, wax) − Lᵢ.z)`, i.e. the light that WRAPPED onto
    /// a flat surface, which is the light that went *through* the paint and so comes back wearing its
    /// colour. Splitting it is what lets the divisor stay cached while the tint stays per-pixel.
    flat_base: [f32; 3],
    /// See [`Self::flat_base`]. Zero at `wax = 0`, which is what makes the neutral material the old code.
    flat_wax: [f32; 3],
    /// Each lamp's specular response on a FLAT surface, at THIS roughness. Subtracted per lamp and
    /// clamped at zero: sum the raw speculars and subtract the flat total instead, and a lamp facing
    /// away from a slope contributes a NEGATIVE term — it borrows headroom from a lamp facing it, and
    /// the "highlight" darkens the paint (`the_glint_only_ever_adds_light`).
    flat_spec: [f32; super::impasto_rig::MAX_LIGHTS],
    /// Every lamp is grey AND the paint is neither metallic NOR waxy ⇒ the three channels are the same
    /// float, and the shade computes it once instead of three times.
    ///
    /// Metal breaks it because a metal's highlight takes the PAINT's colour. **Wax breaks it for the
    /// same reason, one term over**: the light that scatters through paint comes back wearing the
    /// paint's colour, and that is per-channel by construction. So turning Wax on costs the coloured
    /// path — which is honest: a knob that tints the light cannot be had on a lane that assumes it does
    /// not. At `wax = 0` the tinted term is identically zero and the fast lane is the old code.
    achromatic: bool,
}

/// The lighting environment, resolved once per pass: every lit lamp, plus the sums a FLAT surface
/// returns — which is what the whole pass divides by.
///
/// **The contract survives the rig, per channel** (`impasto_rig`): on flat paint `N·Lᵢ = Lᵢ.z` for every
/// lamp, so `diffuse[c] / flat[c] = 1` in every channel whatever the colours and intensities are. A
/// warm key and a cool fill therefore tint the paint exactly where it TILTS, and a flat painting under a
/// red lamp does not turn red.
pub(super) struct Rig {
    /// A fixed array, not a `Vec`: `shade` walks this once per TEXEL, and at 4096² the heap indirection
    /// on a rig of one lamp cost 0.4 ms/move all by itself.
    lamps: [Lamp; super::impasto_rig::MAX_LIGHTS],
    /// How many of [`Self::lamps`] are real.
    n: usize,
    // (There is deliberately NO rig-wide `flat_spec` total. The flat highlight is subtracted PER LAMP,
    // in `MatShade::flat_spec`, and clamped at zero there — see `the_glint_only_ever_adds_light`. A
    // total subtracted from a raw sum lets a lamp facing away from a slope borrow headroom from one
    // facing it, and the "highlight" comes out negative.)
    /// `pow(ndh, exponent(roughness))` for every roughness — the process-wide table
    /// ([`ph2d_painter_brush::material::SpecLut`]). Roughness is per-pixel now, so the old per-pass
    /// 256-entry row became a 2-D table with a second axis; it is baked ONCE for the process, which is
    /// what keeps the pass transcendental-free (HR-5) despite the extra freedom.
    spec: &'static ph2d_painter_brush::material::SpecLut,
    /// Every lamp is GREY (`r = g = b`). Then every channel's ratio is the same number, and the shade
    /// computes it **once** instead of three times.
    ///
    /// Not a micro-optimisation dressed up as one: this is the DEFAULT rig (one white key), and it is
    /// every rig anyone paints with until they deliberately colour a lamp. Without it, giving the pass a
    /// per-channel answer cost 0.4 ms/move at 4096² — a third of the impasto budget — to compute the same
    /// float three times for everybody.
    achromatic: bool,
}

impl Rig {
    /// Resolve `mat` against this rig — the flat response the pass divides by, which now depends on the
    /// PAINT (Roughness sets the exponent, Wax wraps the diffuse) and not on the lamps alone.
    pub(super) fn resolve(&self, key: MaterialBytes) -> MatShade {
        use ph2d_painter_brush::material::{Material, wrapped_ndl};
        let m = Material::from_bytes(key);
        let level = Material::rough_level(m.roughness);
        let mut flat_base = [0.0f32; 3];
        let mut flat_wax = [0.0f32; 3];
        let mut flat_spec = [0.0f32; super::impasto_rig::MAX_LIGHTS];
        for (i, l) in self.lamps[..self.n].iter().enumerate() {
            // Flat: N = (0, 0, 1) ⇒ N·L = L.z and N·H = half.z. The wrap is applied to the FLAT response
            // too — that is the line that keeps flat paint byte-identical at any Wax. Wrap the pixel and
            // not the divisor and every flat canvas would brighten the moment Wax left zero.
            let lz = l.dir[2].max(0.0);
            let wrapped = wrapped_ndl(l.dir[2], m.wax) - lz; // the light that went THROUGH the paint
            for c in 0..3 {
                flat_base[c] += l.tint[c] * lz;
                flat_wax[c] += l.tint[c] * wrapped;
            }
            flat_spec[i] = self.spec.spec(level, l.half[2]);
        }
        MatShade {
            key,
            level,
            shine: m.shine.clamp(0.0, 1.0),
            metallic: m.metallic.clamp(0.0, 1.0),
            wax: m.wax,
            wax_color: m.wax_color,
            flat_base,
            flat_wax,
            flat_spec,
            achromatic: self.achromatic && m.metallic <= 0.0 && m.wax <= 0.0,
        }
    }

    /// Resolve the rig. `None` when no lamp is lit — the caller then leaves the canvas alone rather
    /// than dividing by a zero flat response.
    pub(super) fn new(rig: &super::impasto_rig::LightRig) -> Option<Self> {
        const DARK: Lamp = Lamp {
            dir: [0.0; 3],
            half: [0.0; 3],
            tint: [0.0; 3],
        };
        let mut lamps = [DARK; super::impasto_rig::MAX_LIGHTS];
        let mut n = 0usize;
        for l in rig.lights.iter().filter(|l| l.on && l.intensity > 0.0) {
            // Transcendental-free (HR-5): the shared 1°-step rotor, the same one the brush's Jitter
            // Rotate and per-slot Angle are built from.
            let elev = l.elev_deg.clamp(super::impasto_rig::MIN_ELEV_DEG, 90);
            let az = ph2d_painter_brush::texture::rotate_by_degrees(l.angle_deg % 360);
            let el = ph2d_painter_brush::texture::rotate_by_degrees(elev);
            let (cos_e, sin_e) = (el[0], el[1]);
            let dir = [cos_e * az[0], cos_e * az[1], sin_e];
            let half = {
                let h = [dir[0], dir[1], dir[2] + 1.0]; // view = (0, 0, 1), orthographic
                let len = (h[0] * h[0] + h[1] * h[1] + h[2] * h[2]).sqrt().max(1e-6);
                [h[0] / len, h[1] / len, h[2] / len]
            };
            let w = l.intensity.clamp(0.0, 2.0);
            let tint = [
                w * l.color[0].clamp(0.0, 1.0),
                w * l.color[1].clamp(0.0, 1.0),
                w * l.color[2].clamp(0.0, 1.0),
            ];
            lamps[n] = Lamp { dir, half, tint };
            n += 1;
        }
        if n == 0 {
            return None;
        }
        // (The `intensity > 0` filter above is about the EMPTY rig: with every lamp at zero power, an
        // unfiltered `lamps` is non-empty, so this returns `Some`, the flat diffuse is all-zero, the
        // floor in `resolve` turns it into 1, the diffuse stays 0, and the ratio is 0 — which drives
        // every lit pixel to the AMBIENT floor. Turning the lights down to zero would DARKEN the
        // painting to 35% instead of leaving it unlit. The filter is what makes `n == 0` above true, and
        // the pass bail. `the_lights_turned_all_the_way_down_is_an_unlit_canvas` — MUT: drop it ⇒ RED.)
        let achromatic = lamps[..n]
            .iter()
            .all(|l| l.tint[0] == l.tint[1] && l.tint[1] == l.tint[2]);
        Some(Self {
            lamps,
            n,
            spec: ph2d_painter_brush::material::SpecLut::get(),
            achromatic,
        })
    }

    /// Shade one pixel from its height gradient, the paint actually there, and **what that paint is**:
    /// `body` weights the diffuse modelling ([`paint_body`]), `gloss` the highlight ([`gloss_body`]),
    /// `mat` is the material the stroke baked in, `albedo` the pixel's own colour (which is the metal's
    /// highlight). Returns `(multiply, glint)` PER CHANNEL: the composite's RGB is
    /// `screen(rgb × multiply, glint)` — see [`PainterTool::apply_impasto_light`]. A FLAT pixel — or one
    /// with no real body of paint — returns exactly `([1, 1, 1], [0, 0, 0])`, at every material.
    #[inline]
    pub(super) fn shade(
        &self,
        mat: &MatShade,
        body: f32,
        gloss: f32,
        dhx: f32,
        dhy: f32,
        albedo: [f32; 3],
    ) -> ([f32; 3], [f32; 3]) {
        if (dhx == 0.0 && dhy == 0.0) || body <= 0.0 {
            return ([1.0; 3], [0.0; 3]); // flat paint — or bare paper — is untouched, to the byte
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
        // How much of the highlight takes the PAINT's colour instead of the LAMP's — the one line that
        // separates a conductor from a dielectric in every PBR model. `metallic = 0` leaves it at 1 in
        // every channel, so the dielectric path is the old code, to the float.
        let spec_tint = [
            1.0 + (albedo[0] - 1.0) * mat.metallic,
            1.0 + (albedo[1] - 1.0) * mat.metallic,
            1.0 + (albedo[2] - 1.0) * mat.metallic,
        ];
        // What the light that went THROUGH the paint comes back wearing. The paint's own colour — that
        // is the physics, and it is free — times the artist's FILTER. White (the default) is the physics
        // untouched; a warm filter over white paint is alabaster, which is the case the pigment alone
        // can never say (white scatters white).
        let wax_tint = [
            albedo[0] * mat.wax_color[0],
            albedo[1] * mat.wax_color[1],
            albedo[2] * mat.wax_color[2],
        ];
        // Sum every lamp's tinted response at this normal. One pass over 1..4 lamps — the single-lamp
        // rig (the default, and every canvas nobody has opened the rig on) is one iteration.
        let (mut diffuse, mut spec) = ([0.0f32; 3], [0.0f32; 3]);
        for (i, l) in self.lamps[..self.n].iter().enumerate() {
            let raw = n[0] * l.dir[0] + n[1] * l.dir[1] + n[2] * l.dir[2];
            // The diffuse splits in two, and the split IS the physics. `direct` is the light that
            // bounced off the surface — `max(N·L, 0)`, the plain Lambert. `scattered` is the light that
            // went INTO the paint and came back out nearby: the wrap
            // (`ph2d_painter_brush::material::wrapped_ndl`), minus the direct term it sits on top of.
            //
            // And the scattered light is COLOURED — by the paint it travelled through. That is not a
            // stylistic choice, it is what makes an ear red against the sun and marble warm in shadow:
            // the medium doing the absorbing IS the pigment, so what comes back is what the pigment did
            // not eat. Which means the tint is not a knob to pick — it is already in the pixel
            // (`albedo`), free, per-pixel, and correct by construction. (It comes back MORE saturated
            // than the paint, too, and that also falls out: the pass MULTIPLIES the pixel, which is the
            // albedo, so the scattered term lands as albedo² — a longer path, more absorbed, exactly
            // what Beer-Lambert says. Enio, 2026-07-13: *"seria possível ter cor para wax?"*)
            let direct = raw.max(0.0);
            let scattered = ph2d_painter_brush::material::wrapped_ndl(raw, mat.wax) - direct;
            let ndh = n[0] * l.half[0] + n[1] * l.half[1] + n[2] * l.half[2];
            let s = self.spec.spec(mat.level, ndh);
            // Each lamp's highlight is relative to ITS OWN flat response, and clamped at zero there:
            // summing the raw speculars and subtracting the flat total would let a lamp facing away
            // borrow headroom from one facing the paint, and the pass would glint on flat ground.
            let ds = (s - mat.flat_spec[i]).max(0.0);
            if mat.achromatic {
                // Grey lamps, dielectric paint, no wax ⇒ the three channels are the same float, and
                // `scattered` is identically zero. Compute it once.
                diffuse[0] += l.tint[0] * direct;
                spec[0] += l.tint[0] * ds;
            } else {
                for c in 0..3 {
                    diffuse[c] += l.tint[c] * (direct + scattered * wax_tint[c]);
                    spec[c] += l.tint[c] * ds * spec_tint[c];
                }
            }
        }
        if mat.achromatic {
            let flat = mat.flat_base[0]; // wax is 0 on this lane, so the tinted half is 0 too
            let m = channel(mat, body, diffuse[0], spec[0], gloss, flat);
            return ([m.0; 3], [m.1; 3]);
        }
        let (mut mul, mut add) = ([0.0f32; 3], [0.0f32; 3]);
        for c in 0..3 {
            // The divisor wears the SAME tint the pixel does — filter included. That is what keeps a
            // flat surface at ratio exactly 1 in every channel: on flat ground the pixel's own sum IS
            // this sum, term for term. Tint the pixel and not the divisor and the paint would glow in
            // its own colour everywhere, flat or not.
            let flat = mat.flat_base[c] + wax_tint[c] * mat.flat_wax[c];
            let (m, a) = channel(mat, body, diffuse[c], spec[c], gloss, flat);
            mul[c] = m;
            add[c] = a;
        }
        (mul, add)
    }
}

/// One channel's `(multiply, glint)` from its summed diffuse and specular. The shared core of both
/// paths in [`Rig::shade`] — so the achromatic fast lane cannot drift from the coloured one.
#[inline]
fn channel(
    mat: &MatShade,
    body: f32,
    diffuse: f32,
    spec: f32,
    gloss: f32,
    flat: f32,
) -> (f32, f32) {
    // RELATIVE diffuse, with an AMBIENT floor: exactly 1.0 on flat ground (so flat paint stays
    // byte-identical), above 1 on a face turned toward the light, and down to `AMBIENT` — never 0 — on
    // a face turned away. Paint in shadow is dark, not black.
    //
    // The divisor is the flat response OF THIS MATERIAL (`MatShade::flat_diffuse`), not of the rig: Wax
    // wraps the pixel AND the flat surface, so the ratio on flat ground is still exactly 1 at any Wax.
    // Wrap only the pixel and every flat canvas brightens the moment the knob leaves zero — the same
    // absolute-shading bug the whole pass is built to avoid, arriving through a new door.
    // A channel a rig gives NO light to (a pure-red lamp alone, on the blue channel) would divide by
    // zero. Its diffuse is zero too, so the ratio is 0/0 — and the answer that keeps the contract is 1
    // (the channel is untouched), which is what this floor produces. It is applied HERE, not in
    // `resolve`, because the divisor is only whole once the pixel's own colour has been folded in.
    let flat = if flat <= 1e-4 { 1.0 } else { flat };
    let ratio = (diffuse / flat).clamp(0.0, 2.0);
    let m = AMBIENT + (1.0 - AMBIENT) * ratio;
    // Fade both with the body, so the pass is a strict no-op on bare canvas.
    //
    // (Tried and rejected, 2026-07-12: weighting the BRIGHTENING more harshly than the shadow, to stop
    // the light washing out translucent paint. It bought 5 points of chroma and cost the modelling
    // entirely — the lit flank came out exactly as bright as the paint under it, which is what "no
    // relief" looks like. The washing-out is not a bug to weight away: brightening a pixel whose pigment
    // channel is already at the ceiling ALWAYS costs saturation, in paint as in physics. The line that
    // must hold is the other one — the light may not touch paint with no body at all — and that is what
    // `body` here, and the gates, enforce.)
    (1.0 + (m - 1.0) * body, mat.shine * spec * gloss)
}
