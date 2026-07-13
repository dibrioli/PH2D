//! The paint's **material** — what the light finds when it arrives.
//!
//! The relief ([`crate::height`]) is the paint's *shape*; this is what the paint *is*. It is a property
//! of the PAINT, so it belongs to the brush that lays it down and is baked into the canvas with the
//! stroke — exactly like Depth and Body, and unlike the light rig (which is a property of the ROOM and
//! stays canvas-wide and live).
//!
//! ## Why not per-lamp (the question that produced this module)
//!
//! Enio, 2026-07-13: *"Shine parece ser a intensidade do brilho mas é global e não por lâmpada."* The
//! specular strength is a **BRDF** term: it belongs to the surface, not to the light. Giving every lamp
//! its own `ks` is what takes Krita's Phong Bumpmap to 24 controls for one material
//! (`docs/Painter/17_impasto_deposito_pesquisa2.md` §2.4). But the smell was real, from the other side:
//! `Shine` was *documented* as a property of the paint and *stored* per CANVAS, so one painting could
//! only ever have one material. The fix is not less specific (per-lamp) — it is **more**: per paint, and
//! paint is per pixel.
//!
//! ## The neutral material is TODAY, to the byte
//!
//! [`Material::NEUTRAL`] reproduces the pass exactly as it shaded before this module existed — that is
//! what lets every existing gate stay green and makes "no material set ⇒ nothing moved" checkable. It
//! falls out of the roughness mapping rather than being asserted: see [`Material::exponent`].

/// Specular exponent at **`roughness = 1`** — a broad, soft sheen (dry gouache, chalk). // CLAMP-OK
const EXP_MATTE: f32 = 6.0;
/// Specular exponent at **`roughness = 0`** — a tight, hard glint (wet varnish, fresh oil). // CLAMP-OK
const EXP_GLOSSY: f32 = 96.0;

/// Roughness levels the specular LUT is baked at. The stored material is a `u8`, so this quantises the
/// exponent — 64 steps over a geometric range is finer than the eye reads a highlight's width, and it
/// keeps the table at 66 KB instead of 256.
///
/// **65, not 64** — an ODD count, so the level grid (`level / (N-1)`) lands on `0.5` EXACTLY and the
/// neutral material reads the exponent-24 row it is defined to read. At 64 the grid is `level / 63`,
/// the neutral quantises to `r = 0.5079`, the exponent comes out **23.47**, and "the neutral material
/// is today, to the byte" becomes quietly false — every byte-identity gate drifting by a hair for a
/// reason no one would find. Caught by `the_neutral_lut_row_is_the_old_table`, which is exactly the job
/// of pinning the arithmetic instead of trusting it. // CLAMP-OK
pub const ROUGH_LEVELS: usize = 65;
/// Entries per roughness row — one per `N·H` bucket. Mirrors the old 1-D table. // CLAMP-OK
pub const SPEC_LUT: usize = 256;

/// The paint's optical properties at one pixel. All `0..1`, all per-BRUSH at deposit.
///
/// Four numbers, not more: every one of them is a distinct *percept*, and the section already learned
/// what a second gain over the same percept does to it (§16 of the plan — the pair that made Impasto
/// read as "hard to adjust").
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Material {
    /// **Shine** — how much the highlight is worth. `0` = matte paint that only models; up = wet oil.
    /// (Was canvas-global; it is the paint's, so it moved here.)
    pub shine: f32,
    /// **Roughness** — how BROAD the highlight is, which is a different question from how strong.
    /// `0` = a tight glint riding the crest (varnish); `1` = a wide soft sheen over the whole flank
    /// (chalk). This is the knob that did not exist: the exponent was the constant `SHININESS = 24`.
    pub roughness: f32,
    /// **Metallic** — whose colour the highlight takes. `0` (dielectric: oil, acrylic, gouache) = the
    /// LAMP's colour, so a white light glints white on red paint. `1` (metal: gold leaf, iridescent,
    /// metallic acrylic) = the PAINT's own colour, so gold glints gold. The one-line difference between
    /// a dielectric and a conductor in every PBR model, and ArtRage ships it as "Metallic".
    pub metallic: f32,
    /// **Wax** — the waxy, soft-terminator look of paint that light enters and leaves nearby.
    ///
    /// It is **wrap lighting** (half-Lambert), NOT subsurface scattering, and the distinction is the
    /// honest one: real SSS is a *diffusion* problem, this project has already measured what that costs
    /// (`project_wos_diffusion_over_budget` — 20-100× over budget), and a painting is opaque-backed, so
    /// there is no transmission to see anyway. The half of "SSS" that IS visible in paint — thin paint
    /// letting the ground through — is already modelled, by the film's Beer-Lambert opacity
    /// ([`crate::height_film`]). What is left is the soft terminator, and wrapping the diffuse delivers
    /// exactly that for zero cost. Naming it `Wax` rather than `SSS` is deliberate: it promises the
    /// percept it delivers.
    pub wax: f32,
}

impl Material {
    /// The material the pass shaded with **before this module existed**: today's `SHININESS = 24`
    /// exponent, no metal, no wrap. Every pre-material gate is a gate on this.
    ///
    /// `roughness = 0.5` lands on 24 *exactly* — not by a fudge, but because the mapping is geometric
    /// and `√(EXP_MATTE · EXP_GLOSSY) = √(6 · 96) = √576 = 24`. The endpoints were chosen to make the
    /// midpoint the old constant, so the byte-identity is a property of the arithmetic and cannot drift.
    pub const NEUTRAL: Self = Self {
        shine: 0.0,
        roughness: 0.5,
        metallic: 0.0,
        wax: 0.0,
    };

    /// Quantise to the 4 bytes the canvas stores (`[shine, roughness, metallic, wax]`).
    #[must_use]
    pub fn to_bytes(self) -> [u8; 4] {
        let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        [
            q(self.shine),
            q(self.roughness),
            q(self.metallic),
            q(self.wax),
        ]
    }

    /// The material a canvas pixel stores. Inverse of [`Self::to_bytes`] up to the `u8` quantisation.
    #[must_use]
    pub fn from_bytes(b: [u8; 4]) -> Self {
        let d = |v: u8| f32::from(v) / 255.0;
        Self {
            shine: d(b[0]),
            roughness: d(b[1]),
            metallic: d(b[2]),
            wax: d(b[3]),
        }
    }

    /// The Blinn-Phong exponent this roughness means — **geometric** between [`EXP_MATTE`] and
    /// [`EXP_GLOSSY`], because the eye reads highlight width on a ratio scale, not a linear one (halving
    /// the exponent is one "step" of glossiness whether you start at 6 or at 60; a linear ramp would
    /// spend most of its travel in a range no one can tell apart).
    #[must_use]
    pub fn exponent(roughness: f32) -> f32 {
        let r = roughness.clamp(0.0, 1.0);
        EXP_GLOSSY * (EXP_MATTE / EXP_GLOSSY).powf(r)
    }

    /// Which row of the specular LUT this roughness reads.
    #[must_use]
    pub fn rough_level(roughness: f32) -> usize {
        let r = roughness.clamp(0.0, 1.0);
        ((r * (ROUGH_LEVELS - 1) as f32) + 0.5) as usize
    }
}

/// `pow(ndh, exponent(r))` for every `(roughness level, N·H)` pair — the whole specular, as a table.
///
/// **Baked once, ever** (not per pass): it is a pure function of nothing but itself, so a `OnceLock`
/// removes the `powf` from the hot path entirely and keeps the pass transcendental-free (HR-5) even
/// though roughness is now per-pixel. The old code baked a 256-entry row per pass for a single fixed
/// exponent; this is the same table with a second axis, and it costs the process one 16 k-entry init.
pub struct SpecLut {
    table: Vec<f32>,
}

impl SpecLut {
    /// The process-wide table. Cheap to call; the first call builds it.
    #[must_use]
    pub fn get() -> &'static Self {
        static LUT: std::sync::OnceLock<SpecLut> = std::sync::OnceLock::new();
        LUT.get_or_init(|| {
            let mut table = vec![0.0f32; ROUGH_LEVELS * SPEC_LUT];
            for level in 0..ROUGH_LEVELS {
                let e = Material::exponent(level as f32 / (ROUGH_LEVELS - 1) as f32);
                for i in 0..SPEC_LUT {
                    let x = i as f32 / (SPEC_LUT - 1) as f32;
                    table[level * SPEC_LUT + i] = x.powf(e);
                }
            }
            SpecLut { table }
        })
    }

    /// `pow(ndh, exponent(roughness))`, from the table.
    #[inline]
    #[must_use]
    pub fn spec(&self, level: usize, ndh: f32) -> f32 {
        let i = (ndh.clamp(0.0, 1.0) * (SPEC_LUT - 1) as f32) as usize;
        self.table[level.min(ROUGH_LEVELS - 1) * SPEC_LUT + i]
    }

    /// The raw row for a level — the shape the GPU port uploads.
    #[must_use]
    pub fn row(&self, level: usize) -> &[f32] {
        let l = level.min(ROUGH_LEVELS - 1);
        &self.table[l * SPEC_LUT..(l + 1) * SPEC_LUT]
    }
}

/// **Wrap lighting** (`Wax`): the diffuse term, softened so light reaches around the terminator.
///
/// `(N·L + w) / (1 + w)` — Valve's half-Lambert, the standard cheap stand-in for subsurface softness.
/// At `w = 0` it is `max(N·L, 0)` exactly, so the neutral material is the old code, to the float. The
/// normalisation by `1 + w` is what keeps a surface facing the light returning `1` at any wrap, so the
/// wrap widens the lit band instead of brightening the whole canvas.
#[inline]
#[must_use]
pub fn wrapped_ndl(ndl: f32, wax: f32) -> f32 {
    let w = wax.clamp(0.0, 1.0);
    ((ndl + w) / (1.0 + w)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The load-bearing arithmetic: the neutral material IS the old constant. If the endpoints are ever
    /// retuned, this goes red and the byte-identity gates go with it — which is the point of pinning it
    /// here, at the arithmetic, rather than discovering it downstream in a shading diff.
    #[test]
    fn the_neutral_material_is_the_old_shininess_exactly() {
        assert_eq!(
            Material::exponent(Material::NEUTRAL.roughness),
            24.0,
            "the neutral roughness must map to the SHININESS = 24 the pass shaded with before \
             materials existed — the geometric midpoint of {EXP_MATTE} and {EXP_GLOSSY} is sqrt(576)"
        );
    }

    /// …and the LUT row the neutral material reads must be the row the old 1-D table WAS. A mapping that
    /// is exact but a LUT that quantises the level away would be the same bug one layer down.
    #[test]
    fn the_neutral_lut_row_is_the_old_table() {
        let lut = SpecLut::get();
        let level = Material::rough_level(Material::NEUTRAL.roughness);
        for i in 0..SPEC_LUT {
            let x = i as f32 / (SPEC_LUT - 1) as f32;
            let old = x.powf(24.0); // what `Rig::new` baked, verbatim
            let new = lut.row(level)[i];
            assert!(
                (old - new).abs() <= f32::EPSILON * 4.0,
                "neutral LUT entry {i} drifted from the old pow(x, 24): {new} vs {old}"
            );
        }
    }

    /// Roughness is MONOTONE and actually moves the highlight's width — the failure mode this section
    /// keeps producing is a knob that is wired, painted, and does nothing (§17). A tight glint must fall
    /// off faster than a broad one at the same angle off the crest.
    #[test]
    fn roughness_broadens_the_highlight_and_is_monotone() {
        let lut = SpecLut::get();
        let off_crest = 0.9; // a little off the highlight's centre
        let glossy = lut.spec(Material::rough_level(0.0), off_crest);
        let mid = lut.spec(Material::rough_level(0.5), off_crest);
        let matte = lut.spec(Material::rough_level(1.0), off_crest);
        assert!(
            glossy < mid && mid < matte,
            "off the crest, a rougher paint must still be lit (a broader lobe): \
             glossy {glossy} < mid {mid} < matte {matte}"
        );
        // And on the crest itself every material is fully lit — roughness spreads the lobe, it does not
        // dim it. (A model where matte paint's peak is darker is a model that loses energy.)
        for r in [0.0f32, 0.5, 1.0] {
            assert_eq!(lut.spec(Material::rough_level(r), 1.0), 1.0);
        }
    }

    /// Wrap is EXACTLY the old `max(N·L, 0)` at the neutral material — the whole byte-identity contract
    /// rests on the diffuse being untouched when Wax is 0.
    #[test]
    fn wax_zero_is_the_old_lambert_to_the_float() {
        for ndl in [-1.0f32, -0.3, 0.0, 0.42, 1.0] {
            assert_eq!(wrapped_ndl(ndl, 0.0), ndl.max(0.0));
        }
        // And wrapping lights the far side of the terminator without brightening the front.
        assert!(
            wrapped_ndl(-0.2, 0.5) > 0.0,
            "wax reaches around the terminator"
        );
        assert_eq!(
            wrapped_ndl(1.0, 0.5),
            1.0,
            "…and never over-brightens a face-on surface"
        );
    }

    /// The canvas stores 4 bytes; a round-trip must not move the material enough to shift a pixel.
    #[test]
    fn the_stored_material_round_trips() {
        let m = Material {
            shine: 0.7,
            roughness: 0.25,
            metallic: 1.0,
            wax: 0.33,
        };
        let back = Material::from_bytes(m.to_bytes());
        for (a, b) in [
            (m.shine, back.shine),
            (m.roughness, back.roughness),
            (m.metallic, back.metallic),
            (m.wax, back.wax),
        ] {
            assert!((a - b).abs() <= 1.0 / 255.0, "{a} vs {b}");
        }
    }
}
