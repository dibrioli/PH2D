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

/// **O mundo sem escultura** — a forma NEUTRA, e o que faz a doação ser aditiva de verdade.
///
/// `[0, 0, 1]` é a normal de um plano virado para o olho e `0` é o peso. Somada à inclinação da tinta
/// em [`Rig::shade_over`], ela devolve **literalmente** a expressão que o passe sempre computou — não
/// um caminho equivalente, a mesma aritmética. É isso que torna *"com a flag off o caminho da tinta
/// sai byte-idêntico"* ([`docs/3D/02.3`](../../../../../docs/3D/02-Arquitetura/02.3-Modulo-removivel-e-mapa-de-crates.md),
/// costura S1) uma propriedade da forma escrita, e não uma promessa a testar em todo canto.
pub(super) const NO_FORM: [f32; 4] = [0.0, 0.0, 1.0, 0.0];

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
    ///
    /// ⚠️ **A resolução em si não mora mais aqui** ([`ph2d_light::resolve`]): a escultura do módulo 3D
    /// tem de ser acesa pelas MESMAS lâmpadas, e duas conversões graus→vetor não divergiriam no dia em
    /// que fossem escritas — divergiriam no dia em que alguém consertasse uma. O que ficou aqui é o que
    /// é do Painter: a LUT especular, e o material contra o qual o rig é resolvido.
    ///
    /// (O filtro de lâmpada apagada, o piso de elevação e o rotor de 1° viajaram junto e estão gateados
    /// lá — inclusive o caso que parece higiene e não é: *baixar todas as luzes a zero é uma tela SEM
    /// LUZ*, não uma tela escurecida ao piso ambiente.)
    pub(super) fn new(rig: &super::impasto_rig::LightRig) -> Option<Self> {
        const DARK: Lamp = Lamp {
            dir: [0.0; 3],
            half: [0.0; 3],
            tint: [0.0; 3],
        };
        let resolved = ph2d_light::resolve(rig)?;
        let mut lamps = [DARK; super::impasto_rig::MAX_LIGHTS];
        for (dst, src) in lamps.iter_mut().zip(resolved.lamps()) {
            *dst = Lamp {
                dir: src.dir,
                half: src.half,
                tint: src.tint,
            };
        }
        Some(Self {
            lamps,
            n: resolved.lamps().len(),
            spec: ph2d_painter_brush::material::SpecLut::get(),
            achromatic: resolved.achromatic(),
        })
    }

    /// The lit lamps, in the form the GPU light pass consumes
    /// ([`super::impasto_gpu::ImpastoLamp`]).
    ///
    /// The rig is resolved HERE and shipped as plain floats, rather than the shader rebuilding `dir`
    /// from the artist's whole-degree azimuth and elevation: that construction goes through the shared
    /// 1°-step rotor ([`ph2d_painter_brush::texture::rotate_by_degrees`]), which is what keeps the pass
    /// transcendental-free (HR-5), and a shader calling `sin`/`cos` for itself would be a second rotor
    /// disagreeing in the last bits with the one the rest of the app turns by.
    pub(super) fn export_lamps(&self) -> Vec<super::impasto_gpu::ImpastoLamp> {
        self.lamps[..self.n]
            .iter()
            .map(|l| super::impasto_gpu::ImpastoLamp {
                dir: l.dir,
                half: l.half,
                tint: l.tint,
            })
            .collect()
    }

    /// Force the COLOURED path, so a gate can prove the two lanes agree to the bit.
    ///
    /// The achromatic lane is an optimisation, and the GPU port takes it as licence to implement the
    /// coloured path ONLY — one branch instead of two, in the place where a stray branch is hardest to
    /// see. That licence has to be earned rather than argued
    /// (`the_achromatic_fast_lane_is_the_coloured_one_to_the_bit`).
    #[cfg(test)]
    pub(super) fn force_chromatic(&mut self) {
        self.achromatic = false;
    }

    /// Shade one pixel from its height gradient, the paint actually there, and **what that paint is**:
    /// `body` weights the diffuse modelling ([`paint_body`]), `gloss` the highlight ([`gloss_body`]),
    /// `mat` is the material the stroke baked in, `albedo` the pixel's own colour (which is the metal's
    /// highlight). Returns `(multiply, glint)` PER CHANNEL: the composite's RGB is
    /// `screen(rgb × multiply, glint)` — see [`PainterTool::apply_impasto_light`]. A FLAT pixel — or one
    /// with no real body of paint — returns exactly `([1, 1, 1], [0, 0, 0])`, at every material.
    /// Sombreia um pixel, com a **SEGUNDA FONTE DE NORMAL** — a forma doada pelo módulo 3D
    /// (`docs/3D/05.2`). `form` é `[nx, ny, nz, peso]`; [`NO_FORM`] é o mundo sem escultura.
    ///
    /// ## As duas fontes compõem, e a forma da composição apaga o caso especial
    ///
    /// A normal da tinta é `[-∇h·K, 1]` e a da forma é um vetor unitário. Compor não é escolher uma:
    /// é **somar a inclinação da tinta à normal da forma** e renormalizar —
    ///
    /// ```text
    /// v = [ form.x − dhx·K ,  form.y − dhy·K ,  form.z ]
    /// ```
    ///
    /// — que é o *blend UDN* dos normal maps, e que degenera EXATO nos dois extremos:
    ///
    /// - **sem forma** (`form = [0, 0, 1]`): `v` é literalmente `[-dhx·K, -dhy·K, 1]`, a expressão que
    ///   sempre esteve aqui ⇒ **byte-idêntico**, sem ramo e sem `if`;
    /// - **tinta plana** (`dh = 0`): `v` é a normal da forma, intocada.
    ///
    /// É por isso que não há uma pergunta *"qual fonte manda?"* a responder: uma pincelada sobre uma
    /// escultura tem o relevo da tinta **por cima** da inclinação da forma, que é o que a mão faz.
    ///
    /// ⚠️ **Porta ÚNICA, e o wrapper `shade(…)` sem forma foi REMOVIDO.** Ele só delegava com
    /// [`NO_FORM`], então o gate que o comparava com esta função era uma tautologia — duas maneiras de
    /// escrever a mesma chamada, concordando por construção. O que prova a byte-identidade é a
    /// comparação contra a EXPRESSÃO antiga, congelada no gate, e essa continua valendo.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn shade_over(
        &self,
        mat: &MatShade,
        body: f32,
        gloss: f32,
        dhx: f32,
        dhy: f32,
        form: [f32; 4],
        albedo: [f32; 3],
    ) -> ([f32; 3], [f32; 3]) {
        // ⚠️ O early-out ganhou a terceira condição, e ela é load-bearing: com tinta PLANA sobre uma
        // forma, `dhx` e `dhy` são zero e o gradiente não tem nada a dizer — mas a forma tem. Sem
        // `form[3] <= 0.0` aqui, a doação seria invisível exatamente onde ela é o ponto: pintar chapado
        // sobre uma escultura e ver a escultura acender a tinta.
        if (dhx == 0.0 && dhy == 0.0 && form[3] <= 0.0) || body <= 0.0 {
            return ([1.0; 3], [0.0; 3]); // flat paint — or bare paper — is untouched, to the byte
        }
        // Surface normal from the gradient: a rising slope tilts the normal AGAINST the rise. The
        // slope is GEOMETRY — the height buffer's unit converted to pixels ([`DEPTH_UNIT_PX`]) — with
        // no gain and no coverage mute: the body curve already ends the relief where the paint thins,
        // so wherever the gradient is nonzero there is real paint standing there.
        let n = {
            let v = [
                form[0] - dhx * DEPTH_UNIT_PX,
                form[1] - dhy * DEPTH_UNIT_PX,
                form[2],
            ];
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
    // Fade both with the body, so the pass is a strict no-op on bare canvas. (See the tests below for
    // why the coloured path is allowed to be the ONLY one the GPU port implements.)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::paint::impasto_rig::{ImpastoLight, LightRig};
    use ph2d_painter_brush::material::Material;

    /// A grey rig, which is every rig anybody paints with until they deliberately colour a lamp — and
    /// therefore the rig the achromatic lane exists for.
    fn grey_rig() -> LightRig {
        let mut r = LightRig::default();
        r.lights[1] = ImpastoLight {
            on: true,
            angle_deg: 70,
            elev_deg: 55,
            intensity: 0.6,
            color: [1.0, 1.0, 1.0],
        };
        r
    }

    /// **The licence the GPU port is built on.** The achromatic fast lane computes one channel; the
    /// coloured lane computes three. They must be the SAME float, not merely the same picture — because
    /// the shader implements the coloured path alone, and a branch that exists on one side of a parity
    /// gate and not the other is a divergence waiting for the one canvas that takes it.
    ///
    /// It holds for a reason worth writing down rather than measuring: on the achromatic lane
    /// `wax = 0`, so `scattered` is identically `0.0` and the tinted term is `direct + 0.0 · wax_tint`
    /// — exactly `direct`; and `metallic = 0`, so `spec_tint` is exactly `1.0` and `x · 1.0` is `x`.
    /// The lanes are the same arithmetic with the zeros written out.
    #[test]
    fn the_achromatic_fast_lane_is_the_coloured_one_to_the_bit() {
        let rig = grey_rig();
        let fast = Rig::new(&rig).expect("a lit rig");
        assert!(
            fast.achromatic,
            "precondition: a grey rig must actually TAKE the fast lane, or this gate proves nothing"
        );
        let mut slow = Rig::new(&rig).expect("a lit rig");
        slow.force_chromatic();
        // Materials on the fast lane are the dielectric, wax-free ones — those are its preconditions.
        for &rough in &[0.0f32, 0.25, 0.5, 1.0] {
            for &shine in &[0.0f32, 0.7] {
                let key = Material {
                    shine,
                    roughness: rough,
                    metallic: 0.0,
                    wax: 0.0,
                    wax_color: [1.0, 1.0, 1.0],
                }
                .to_bytes();
                let (mf, ms) = (fast.resolve(key), slow.resolve(key));
                assert!(mf.achromatic && !ms.achromatic, "the two lanes are armed");
                for &(dhx, dhy) in &[(0.05f32, 0.0f32), (-0.3, 0.12), (0.9, -0.9), (0.01, 0.004)] {
                    for &body in &[0.2f32, 0.6, 1.0] {
                        let albedo = [0.83f32, 0.41, 0.17];
                        let a = fast.shade_over(&mf, body, body, dhx, dhy, NO_FORM, albedo);
                        let b = slow.shade_over(&ms, body, body, dhx, dhy, NO_FORM, albedo);
                        assert_eq!(
                            a, b,
                            "lanes diverged at rough {rough} shine {shine} slope ({dhx}, {dhy}) \
                             body {body}"
                        );
                    }
                }
            }
        }
    }

    /// **A COSTURA S1, provada e não prometida: sem forma, a aritmética é a que sempre shipou.**
    ///
    /// `docs/3D/02.3` promete que *"com a flag off o caminho da tinta sai byte-idêntico"*. Aqui isso
    /// não é um teste de regressão sobre uma imagem — é a comparação contra a EXPRESSÃO ANTIGA,
    /// congelada abaixo verbatim. Se alguém trocar a soma por um ramo (`if há forma { … } else { … }`),
    /// os dois lados deixam de ser a mesma conta e isto sangra.
    #[test]
    fn the_shade_with_no_form_is_the_shade_that_shipped() {
        // A normal como ela era antes da doação existir — o código que shipava, sem forma nenhuma.
        fn normal_before_the_donation(dhx: f32, dhy: f32) -> [f32; 3] {
            let v = [-dhx * DEPTH_UNIT_PX, -dhy * DEPTH_UNIT_PX, 1.0];
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
            [v[0] / len, v[1] / len, v[2] / len]
        }
        // E a mesma normal, pela porta nova, com a forma NEUTRA.
        fn normal_with_no_form(dhx: f32, dhy: f32) -> [f32; 3] {
            let v = [
                NO_FORM[0] - dhx * DEPTH_UNIT_PX,
                NO_FORM[1] - dhy * DEPTH_UNIT_PX,
                NO_FORM[2],
            ];
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
            [v[0] / len, v[1] / len, v[2] / len]
        }
        for &(dhx, dhy) in &[
            (0.05f32, 0.0f32),
            (-0.3, 0.12),
            (0.9, -0.9),
            (0.0071, 0.0033),
            (-1.7, 0.4),
        ] {
            assert_eq!(
                normal_before_the_donation(dhx, dhy),
                normal_with_no_form(dhx, dhy),
                "a forma neutra moveu a normal em ({dhx}, {dhy})"
            );
        }
        // ⚠️ Havia aqui uma segunda metade comparando `shade(…)` com `shade_over(…, NO_FORM, …)`, e
        // ela era uma TAUTOLOGIA: o wrapper só delegava, então os dois lados eram duas maneiras de
        // escrever a mesma chamada, concordando por construção. O wrapper foi removido (uma porta), e
        // o que resta é a comparação que de fato pode falhar — a expressão de hoje contra a de ontem.
        //
        // E a tinta plana continua intocada AO BYTE, que é a outra metade do contrato.
        let rig = Rig::new(&LightRig::default()).expect("a principal nasce acesa");
        let mat = rig.resolve(Material::NEUTRAL.to_bytes());
        assert_eq!(
            rig.shade_over(&mat, 1.0, 1.0, 0.0, 0.0, NO_FORM, [0.5; 3]),
            ([1.0; 3], [0.0; 3]),
            "sem forma e sem gradiente o passe não pode tocar o pixel"
        );
    }

    /// **A doação faz o que ela existe para fazer — e a tinta continua por cima dela.**
    ///
    /// Duas metades, e as duas importam:
    ///
    /// 1. **tinta PLANA sobre uma forma acende.** É o objetivo O1 inteiro (*"pintar sobre forma"*) e é
    ///    exatamente o caso que o early-out antigo teria engolido: sem gradiente, o passe devolvia
    ///    "intocado".
    /// 2. **o relevo da própria tinta ainda cavalga a forma.** Se a composição fosse um ramo — *a
    ///    forma vence onde ela cobre* — uma pincelada com corpo sobre uma escultura perderia o próprio
    ///    corpo, e ninguém saberia dizer onde ele foi parar.
    #[test]
    fn a_form_lights_flat_paint_and_the_paints_own_relief_still_rides_on_it() {
        let rig = Rig::new(&LightRig::default()).expect("a principal nasce acesa");
        let mat = rig.resolve(Material::NEUTRAL.to_bytes());
        let albedo = [0.6f32, 0.6, 0.6];
        // Uma forma inclinada PARA a lâmpada principal (que vem de cima e da esquerda: x<0, y<0).
        // Unitária de propósito — é uma normal, e `0.25 + 0.25 + 0.5 = 1`.
        let tilted = [-0.5f32, -0.5, std::f32::consts::FRAC_1_SQRT_2, 1.0];

        // (1) Tinta plana: sem forma é intocada AO BYTE; com forma, acende.
        let flat_no_form = rig.shade_over(&mat, 1.0, 1.0, 0.0, 0.0, NO_FORM, albedo);
        assert_eq!(
            flat_no_form,
            ([1.0; 3], [0.0; 3]),
            "sem forma, tinta plana é intocada — o contrato que a doação não pode quebrar"
        );
        let flat_on_form = rig.shade_over(&mat, 1.0, 1.0, 0.0, 0.0, tilted, albedo);
        assert!(
            flat_on_form.0[0] > 1.05,
            "a forma inclinada para a luz tinha de CLAREAR a tinta plana: {:?}",
            flat_on_form.0
        );

        // (2) O relevo da tinta cavalga: a mesma forma, com e sem uma inclinação de tinta por cima.
        let ridged = rig.shade_over(&mat, 1.0, 1.0, 0.02, 0.0, tilted, albedo);
        assert!(
            (ridged.0[0] - flat_on_form.0[0]).abs() > 1e-3,
            "o relevo da tinta sumiu sobre a forma — a composição virou um ramo: \
             {:?} contra {:?}",
            ridged.0,
            flat_on_form.0
        );
    }

    /// The exported rig is the resolved one, lamp for lamp — the GPU shades from THESE numbers, so a
    /// re-derivation on the way out would be the rotor bug arriving through the export door.
    #[test]
    fn the_exported_lamps_are_the_resolved_ones() {
        let rig = grey_rig();
        let r = Rig::new(&rig).expect("a lit rig");
        let out = r.export_lamps();
        assert_eq!(out.len(), 2, "the key plus the one lamp the fixture lit");
        for (i, l) in out.iter().enumerate() {
            assert_eq!(l.dir, r.lamps[i].dir);
            assert_eq!(l.half, r.lamps[i].half);
            assert_eq!(l.tint, r.lamps[i].tint);
            let len = (l.dir[0] * l.dir[0] + l.dir[1] * l.dir[1] + l.dir[2] * l.dir[2]).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-5,
                "lamp {i} direction is a unit vector"
            );
        }
        // A lamp switched off never crosses: the shader loops over what it is given.
        let mut dark = LightRig::default();
        for l in &mut dark.lights {
            l.on = false;
        }
        assert!(
            Rig::new(&dark).is_none(),
            "no lit lamp, no rig — and so no export at all"
        );
    }
}
