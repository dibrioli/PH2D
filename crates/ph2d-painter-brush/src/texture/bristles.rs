//! The **Bristles** kind (plano 27 D1): the Wet Paint engine's bristle tile, built by THE engine's
//! generator from the slot's four sliders and sampled exactly as the engine samples it — nearest texel,
//! wrapped, in canvas pixels from the dab centre (`ph2d_wet_paint::brush::sample_bristle`).

use super::{
    MAX_TEX_PARAMS, TEX_SIZE_MAX, TEX_SIZE_MIN, TexDabBasis, TextureMapping, TextureSettings,
};
use ph2d_wet_paint::brush::{
    BRISTLE_SIZE, BristleKnobs, bristle_index, bristle_round, build_bristle_texture, sample_bristle,
};
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

/// The bristles of a live dab at canvas pixel `(px, py)` — [`bristles_at`] with 1 texel = 1 px, clamped,
/// and **swept** over the dab's path step `D` (tile texels): the MEAN of the tile over the `n = máx|D|`
/// texels the dab travelled, centred (`off_k = round(k·D/n) − D/2`, `k < n`). Each hair lays its ink
/// along the streak instead of the same dot once per dab (doc 31 §2.1, §3): the `n` dabs a 1-px Spacing
/// would put there leave, summed, the exposure this one dab leaves — the ink of today, spread evenly.
/// ⛔ The MAX over the sweep was measured and refused (CV 0,15 at Spacing 0,1): a hair wider than one
/// texel is crossed by two neighbouring sweeps, and the overlap paints twice with the step's period.
/// At `n = 1` (Spacing ≈ 1 px), no step (the first dab, a Static frame), Tiled or Flow (anchored, nothing
/// repeats) ⇒ exactly the unswept sample. The Wet Paint host reads [`bristles_at`], never this: its water joins the
/// hairs on its own.
#[must_use]
pub(super) fn sample_dab(
    s: &TextureSettings,
    b: &TexDabBasis,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
) -> f32 {
    let p = [px as f64 + 0.5, py as f64 + 0.5];
    let d = [p[0] - f64::from(center[0]), p[1] - f64::from(center[1])];
    let q = tile_coord(s, b, d, p, 1.0, radius);
    let v = match b.sweep {
        [0, 0] => sample_bristle(&bristle_tile(&s.params), q[0], q[1]),
        sweep => swept(&s.params, sweep, bristle_round(q[0]), bristle_round(q[1])),
    };
    v.clamp(0.0, 1.0) as f32
}

/// The sweep is bounded by the tile's PERIOD ([`BRISTLE_SIZE`] texels per axis): a longer one wraps onto
/// texels it already crossed, so it only costs reads.
const SWEEP_MAX: i32 = BRISTLE_SIZE as i32;

/// The dab's path step in tile texels (the slot's LINEAR map — rotor, Angle, Size — applied to
/// [`crate::Dab::step`]), rounded; once per dab, by [`super::shape_basis`]. A sweep of one texel is the
/// unswept read (its one offset is `0`), so it answers `[0, 0]` and keeps today's path.
pub(super) fn sweep_texels(s: &TextureSettings, b: &TexDabBasis) -> [i32; 2] {
    let st = b.frame.step();
    if st == [0.0, 0.0] || s.mapping == TextureMapping::Tiled || s.flow {
        return [0, 0];
    }
    // The rotor is a rotation, so the radius cancels: any unit gives the same linear map.
    let z = tile_coord(s, b, [0.0, 0.0], [0.0, 0.0], 1.0, 1.0);
    let e = tile_coord(s, b, st.map(f64::from), [0.0, 0.0], 1.0, 1.0);
    let mut v = [e[0] - z[0], e[1] - z[1]];
    let m = v[0].abs().max(v[1].abs());
    if m > f64::from(SWEEP_MAX) {
        v = v.map(|c| c * f64::from(SWEEP_MAX) / m);
    }
    let v = v.map(|c| c.round() as i32);
    if v[0].abs().max(v[1].abs()) <= 1 {
        [0, 0]
    } else {
        v
    }
}

/// A swept tile: the texel offsets of the centred sweep and the texels filled so far — a texel is
/// filled when its mark equals the slot's generation, so a new sweep in the slot costs one increment,
/// not a whole-tile clear.
struct Varrido {
    key: ([u32; 4], [i32; 2]),
    offs: Vec<[i32; 2]>,
    texels: Vec<f32>,
    marca: Vec<u32>,
    geracao: u32,
}

/// Swept tiles kept per thread — a curve, or a symmetry's mirrored copies, alternate a few sweeps.
const VARRIDOS: usize = 8;

thread_local! {
    /// Filled texel by texel on demand: a straight stroke reuses one sweep for every dab, a curve pays
    /// only the texels its dabs touch — never a whole-tile bake per dab.
    static SWEPT: RefCell<(Vec<Varrido>, usize)> = const { RefCell::new((Vec::new(), 0)) };
}

/// `Σ_k tile(q + off_k) / n` over the centred sweep's texels, memoised per `(sliders, sweep)`.
fn swept(params: &[f32; MAX_TEX_PARAMS], sweep: [i32; 2], x: i32, y: i32) -> f64 {
    let key = (
        [params[0], params[1], params[2], params[3]].map(f32::to_bits),
        sweep,
    );
    SWEPT.with(|c| {
        let (lista, prox) = &mut *c.borrow_mut();
        let i = if let Some(i) = lista.iter().position(|v| v.key == key) {
            i
        } else {
            let n = sweep[0].abs().max(sweep[1].abs());
            let along = |k: i32, c: i32| (f64::from(k * c) / f64::from(n)).round() as i32 - c / 2;
            let offs = (0..n)
                .map(|k| [along(k, sweep[0]), along(k, sweep[1])])
                .collect();
            if lista.len() < VARRIDOS {
                lista.push(Varrido {
                    key,
                    offs,
                    texels: vec![0.0; BRISTLE_SIZE * BRISTLE_SIZE],
                    marca: vec![0; BRISTLE_SIZE * BRISTLE_SIZE],
                    geracao: 1,
                });
                lista.len() - 1
            } else {
                let i = *prox;
                *prox = (i + 1) % VARRIDOS;
                let v = &mut lista[i];
                (v.key, v.offs) = (key, offs);
                v.geracao = v.geracao.wrapping_add(1);
                if v.geracao == 0 {
                    v.marca.fill(0);
                    v.geracao = 1;
                }
                i
            }
        };
        let v = &mut lista[i];
        let idx = bristle_index(x, y);
        if v.marca[idx] != v.geracao {
            v.marca[idx] = v.geracao;
            let tile = bristle_tile(params);
            let soma: f32 = v
                .offs
                .iter()
                .map(|o| tile[bristle_index(x + o[0], y + o[1])])
                .sum();
            v.texels[idx] = soma / v.offs.len() as f32;
        }
        f64::from(v.texels[idx])
    })
}

/// ⭐ **The bristles of a live dab under the slot's WHOLE transform**, in TEXELS (never in the tip's
/// scale) — the one door the Digital stamp and the Wet Paint host both read. `d` is the offset from the
/// dab centre and `p` the absolute position, in texels; `px_per_texel` converts the stroke facts (radius,
/// arc length), `1` on the canvas and the grid ratio in the Wet Paint engine (1 texel = 1 cell there).
///
/// The order: the per-dab rotation ([`crate::FootprintDeform::so_o_rotor`]: Jitter Rotate and the Rake/Flow
/// tangent — NOT the brush's fixed Flatten & Rotate, so a flat preset keeps its round hairs; radius units,
/// so in, rotate, back), the slot Angle, Size along the pattern's own axes, then
/// Offset (and the Random mapping's jitter) in [`UNIT_TEXELS`] per unit; Flow lays the tile along the arc
/// length, Tiled anchors it to `p`. At the neutral transform every step is exact (`× 1`, `+ 0`): the raw
/// engine sample `sample_bristle(tile, d)`, unclamped like the engine's `texv`.
#[must_use]
pub fn bristles_at(
    s: &TextureSettings,
    b: &TexDabBasis,
    d: [f64; 2],
    p: [f64; 2],
    px_per_texel: f64,
    radius_px: f32,
) -> f64 {
    let q = tile_coord(s, b, d, p, px_per_texel, radius_px);
    sample_bristle(&bristle_tile(&s.params), q[0], q[1])
}

/// The tile coordinate [`bristles_at`] samples (before the nearest-texel rounding).
fn tile_coord(
    s: &TextureSettings,
    b: &TexDabBasis,
    d: [f64; 2],
    p: [f64; 2],
    px_per_texel: f64,
    radius_px: f32,
) -> [f64; 2] {
    let fp = b.footprint.so_o_rotor();
    let rel = if s.mapping == TextureMapping::Tiled {
        p
    } else if fp.is_identity() && !s.flow {
        d
    } else {
        let r = f64::from(radius_px.max(1e-3)) / px_per_texel;
        let f = fp.apply([(d[0] / r) as f32, (d[1] / r) as f32]);
        let f = [f64::from(f[0]) * r, f64::from(f[1]) * r];
        if s.flow {
            [f64::from(b.frame.parts().0) / px_per_texel + f[0], f[1]]
        } else {
            f
        }
    };
    let (u, v) = (b.u.map(f64::from), b.v.map(f64::from));
    let sx = f64::from(s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX));
    let sy = f64::from(s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX));
    let shift = |o: f32, j: f32| (f64::from(o) + f64::from(j)) * UNIT_TEXELS;
    [
        (rel[0] * u[0] + rel[1] * u[1]) * sx + shift(s.offset[0], b.jitter[0]),
        (rel[0] * v[0] + rel[1] * v[1]) * sy + shift(s.offset[1], b.jitter[1]),
    ]
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
#[path = "bristles_sweep_tests.rs"]
mod sweep_tests;

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
