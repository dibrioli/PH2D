//! The **Bristles** kind (plano 27 D1): the Wet Paint engine's bristle tile, built by THE engine's
//! generator from the slot's four sliders and sampled exactly as the engine samples it — nearest texel,
//! wrapped, in canvas pixels from the dab centre (`ph2d_wet_paint::brush::sample_bristle`).

use super::{
    MAX_TEX_PARAMS, TEX_SIZE_MAX, TEX_SIZE_MIN, TexDabBasis, TextureMapping, TextureSettings,
};
use ph2d_wet_paint::brush::{BRISTLE_SIZE, BristleKnobs, build_bristle_texture, sample_bristle};
use std::cell::RefCell;
use std::rc::Rc;

/// The footprint unit radius, in tile texels, for the routes that only speak footprint units (the panel
/// previews): one whole tile spans the `[-1, 1]` footprint.
const UNIT_TEXELS: f64 = BRISTLE_SIZE as f64 / 2.0;

/// The engine knobs the slot's sliders encode (slots `0..4`, order [`ph2d_wet_paint::brush::BRISTLE_UNIT_KNOBS`]).
#[must_use]
pub fn bristle_knobs(params: &[f32; MAX_TEX_PARAMS]) -> BristleKnobs {
    BristleKnobs::from_unit([params[0], params[1], params[2], params[3]])
}

/// The cached tile and the sliders' bits it was built from.
type Ladrilho = Option<([u32; 4], Rc<[f32]>)>;

thread_local! {
    /// The last tile built on this thread, keyed by the sliders' bits — a per-pixel sampler can't
    /// rebuild ~2 000 tips, and a shared lock would serialise the parallel stamp walks.
    static TILE: RefCell<Ladrilho> = const { RefCell::new(None) };
}

/// The tile for these sliders (built once per thread per change).
#[must_use]
pub fn bristle_tile(params: &[f32; MAX_TEX_PARAMS]) -> Rc<[f32]> {
    let key = [params[0], params[1], params[2], params[3]].map(f32::to_bits);
    TILE.with(|c| {
        let mut c = c.borrow_mut();
        if let Some((k, t)) = c.as_ref()
            && *k == key
        {
            return Rc::clone(t);
        }
        let t: Rc<[f32]> = build_bristle_texture(bristle_knobs(params)).into();
        *c = Some((key, Rc::clone(&t)));
        t
    })
}

/// The bristle coverage at an offset of `(dx, dy)` canvas px from the dab centre.
#[must_use]
fn sample_px(params: &[f32; MAX_TEX_PARAMS], dx: f64, dy: f64) -> f32 {
    sample_bristle(&bristle_tile(params), dx, dy).clamp(0.0, 1.0) as f32
}

/// The bristles of a live dab under the slot's WHOLE transform, measured in PIXELS (never in the tip's
/// scale): the dab frame (Flatten & Rotate, Jitter Rotate, the Rake/Flow tangent), the slot Angle, Size
/// along the pattern's own axes, then Offset (and the Random mapping's jitter) in [`UNIT_TEXELS`] per unit.
/// Flow lays the tile along the stroke's arc length, Tiled anchors it to the canvas. At the neutral
/// transform every step is exact (`× 1`, `+ 0`), so the dab reads the engine's tile at its raw offset.
#[must_use]
pub(super) fn sample_dab(
    s: &TextureSettings,
    b: &TexDabBasis,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
) -> f32 {
    let d = [
        px as f64 + 0.5 - f64::from(center[0]),
        py as f64 + 0.5 - f64::from(center[1]),
    ];
    let rel = if s.mapping == TextureMapping::Tiled {
        [px as f64 + 0.5, py as f64 + 0.5]
    } else if b.footprint.is_identity() && !s.flow {
        d
    } else {
        // The footprint speaks radius units (its bend is a function of them): in, deform, back to px.
        let r = f64::from(radius.max(1e-3));
        let f = b.footprint.apply([(d[0] / r) as f32, (d[1] / r) as f32]);
        let f = [f64::from(f[0]) * r, f64::from(f[1]) * r];
        if s.flow {
            [f64::from(b.frame.parts().0) + f[0], f[1]]
        } else {
            f
        }
    };
    let (u, v) = (b.u.map(f64::from), b.v.map(f64::from));
    let sx = f64::from(s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX));
    let sy = f64::from(s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX));
    let shift = |o: f32, j: f32| (f64::from(o) + f64::from(j)) * UNIT_TEXELS;
    sample_px(
        &s.params,
        (rel[0] * u[0] + rel[1] * u[1]) * sx + shift(s.offset[0], b.jitter[0]),
        (rel[0] * v[0] + rel[1] * v[1]) * sy + shift(s.offset[1], b.jitter[1]),
    )
}

/// The same tile read at a footprint coordinate (`u, v` in footprint units — previews only).
#[must_use]
pub(super) fn sample_unit(params: &[f32; MAX_TEX_PARAMS], u: f32, v: f32) -> f32 {
    sample_px(
        params,
        f64::from(u) * UNIT_TEXELS,
        f64::from(v) * UNIT_TEXELS,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture::{self, TexDabBasis, TextureKind, TextureSettings, param_specs};

    fn params_de(k: BristleKnobs) -> [f32; MAX_TEX_PARAMS] {
        let u = k.to_unit();
        let mut p = [0.5; MAX_TEX_PARAMS];
        p[..4].copy_from_slice(&u);
        p
    }

    /// ⭐ **O ladrilho do Shape é o do motor, AO BIT**, para os knobs dos 12 presets de fábrica e o padrão do
    /// motor — um só gerador. E os sliders de fábrica do tipo são as cerdas do Wet Round com a Strength 2.
    #[test]
    fn o_ladrilho_do_shape_e_o_do_motor_ao_bit() {
        let fabrica = [
            (0.0, 900.0, 2.0, 4.0),
            (0.05, 2000.0, 2.0, 1.0),
            (0.0, 4000.0, 2.0, 0.5),
            (0.0, 2000.0, 1.0, 2.0),
            (0.03, 2000.0, 1.0, 2.0),
            (0.03, 1500.0, 1.0, 1.5),
            (0.03, 3000.0, 1.0, 3.0),
            (0.01, 950.0, 1.0, 1.0),
        ];
        for (felt, count, strength, size) in fabrica {
            let k = BristleKnobs {
                felt,
                bristle_count: count,
                bristle_strength: strength,
                bristle_size: size,
            };
            let motor: Vec<u32> = build_bristle_texture(k)
                .iter()
                .map(|v| v.to_bits())
                .collect();
            let shape: Vec<u32> = bristle_tile(&params_de(k))
                .iter()
                .map(|v| v.to_bits())
                .collect();
            assert_eq!(shape, motor, "felt {felt} · {count} · {strength} · {size}");
        }
        let mut p = [0.5; MAX_TEX_PARAMS];
        for (i, s) in param_specs(TextureKind::Bristles).iter().enumerate() {
            p[i] = s.default;
        }
        let k = bristle_knobs(&p);
        assert_eq!(
            [k.bristle_count, k.bristle_strength, k.bristle_size, k.felt],
            [2000.0, 2.0, 2.0, 0.03]
        );
    }

    /// **A lei é em PÍXEIS:** o mesmo píxel a partir do centro lê o mesmo texel com raio 4 e com 60, e é o
    /// `sample_bristle` do motor nesse desvio. CONTROLO: um tipo procedural (Noise) muda com o raio.
    #[test]
    fn as_cerdas_amostram_em_pixeis_e_nao_na_escala_da_ponta() {
        let s = TextureSettings {
            kind: TextureKind::Bristles,
            params: params_de(BristleKnobs::default()),
            ..TextureSettings::default()
        };
        let b = TexDabBasis::identity();
        let c = [100.25f32, 80.75];
        let tile = build_bristle_texture(BristleKnobs::default());
        let mut iguais = 0;
        let mut noise_muda = 0;
        let noise = TextureSettings {
            kind: TextureKind::Noise,
            ..TextureSettings::default()
        };
        for py in 40..120i64 {
            for px in 60..140i64 {
                let a = texture::sample(&s, &b, px, py, c, 4.0, None);
                let z = texture::sample(&s, &b, px, py, c, 60.0, None);
                let m =
                    sample_bristle(&tile, px as f64 + 0.5 - 100.25, py as f64 + 0.5 - 80.75) as f32;
                assert_eq!(
                    (a.to_bits(), z.to_bits()),
                    (m.to_bits(), m.to_bits()),
                    "({px}, {py})"
                );
                iguais += 1;
                noise_muda += usize::from(
                    texture::sample(&noise, &b, px, py, c, 4.0, None)
                        != texture::sample(&noise, &b, px, py, c, 60.0, None),
                );
            }
        }
        assert_eq!(iguais, 6400);
        assert!(noise_muda > 3000, "o controlo não mede: {noise_muda}");
    }

    /// Nenhum carimbo cacheado (independente do raio) pode guardar as cerdas — nem no Grain nem no Shape.
    #[test]
    fn as_cerdas_nunca_vao_para_o_carimbo_cacheado() {
        let mut spec = crate::BrushSpec::default();
        assert!(
            spec.dab_mask_cacheable(false),
            "o controlo: o pincel liso é cacheável"
        );
        spec.shape.kind = TextureKind::Bristles;
        assert!(!spec.dab_mask_cacheable(false));
        spec.shape.kind = TextureKind::None;
        spec.texture.kind = TextureKind::Bristles;
        assert!(!spec.texture.is_cacheable());
        spec.texture.mapping = crate::texture::TextureMapping::Tiled;
        assert!(!spec.texture.is_canvas_cacheable());
        spec.texture.kind = TextureKind::Noise;
        assert!(
            spec.texture.is_canvas_cacheable(),
            "o controlo: o Noise ladrilhado é"
        );
    }
}
