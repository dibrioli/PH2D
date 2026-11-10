//! **As rampas do Pixel Art em OKLCH** (P13, `docs/PixelArt/P13_plano.md` §4): gerar uma rampa a
//! partir de uma cor base com desvio de matiz (a sombra vai para o frio, a luz para o quente),
//! croma máximo no meio e corte de gama SÓ no croma — a claridade e a matiz planeadas ficam.
//!
//! Oráculo: coloraide 8.14.1 (MIT), `fit('srgb', method='minde-chroma', pspace='oklch', jnd=0)`,
//! fixture `tests/fixtures/coloraide_rampas.json` (gerada por
//! `docs/PixelArt/ferramentas/oraculo_rampas_coloraide.py`).

use ph2d_color::OklchColor;
use ph2d_color::perceptual::{from_oklab, oklab_to_oklch, oklch_to_oklab, to_oklab};
use ph2d_color::srgb::{linear_to_srgb_unit, srgb_to_linear_unit};
use ph2d_pixel_core::{DocOp, Op};

/// A resolução da bissecção do croma (a mesma da `ph2d_color::perceptual::gamut_map`).
const RESOLUCAO_CROMA: f32 = 1e-6;

/// O diálogo Palette ▸ New Ramp.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfigRampa {
    /// Quantos degraus (≥ 1).
    pub passos: usize,
    /// Quanto a matiz roda entre o meio e cada ponta, em graus: a ponta clara vai para o
    /// amarelo, a escura para o lado oposto.
    pub desvio_matiz_graus: f32,
    /// `0` = croma igual em todo degrau; `1` = croma zero nas pontas. Parábola, máximo no meio.
    pub curva_croma: f32,
    /// A claridade OKLab do degrau mais escuro e do mais claro.
    pub faixa_l: [f32; 2],
}

/// A matiz OKLCH (graus) do amarelo sRGB puro: o «quente» para onde a luz roda.
#[must_use]
pub fn matiz_quente() -> f32 {
    oklab_to_oklch(to_oklab([1.0, 1.0, 0.0]))[2] * 360.0
}

/// sRGB de 8 bits → OKLCH (matiz em graus).
#[must_use]
pub fn oklch_de(rgba: [u8; 4]) -> OklchColor {
    let lin = [0, 1, 2].map(|i| srgb_to_linear_unit(f32::from(rgba[i]) / 255.0));
    let [l, c, h] = oklab_to_oklch(to_oklab(lin));
    OklchColor::new(l, c, h * 360.0, f32::from(rgba[3]) / 255.0)
}

/// O degrau PLANEADO (antes do corte de gama).
#[must_use]
pub fn planeada(base: OklchColor, cfg: ConfigRampa) -> Vec<OklchColor> {
    let n = cfg.passos.max(1);
    let d = (matiz_quente() - base.h + 540.0).rem_euclid(360.0) - 180.0;
    let sentido = if d >= 0.0 { 1.0 } else { -1.0 };
    (0..n)
        .map(|i| {
            let t = if n == 1 {
                0.5
            } else {
                i as f32 / (n - 1) as f32
            };
            let u = 2.0 * t - 1.0;
            let l = cfg.faixa_l[0] + (cfg.faixa_l[1] - cfg.faixa_l[0]) * t;
            let c = base.c * (1.0 - cfg.curva_croma * u * u).max(0.0);
            let h = (base.h + sentido * cfg.desvio_matiz_graus * u).rem_euclid(360.0);
            OklchColor::new(l, c, h, base.a)
        })
        .collect()
}

fn rgb_linear(p: OklchColor) -> [f32; 3] {
    from_oklab(oklch_to_oklab([p.l, p.c, p.h / 360.0]))
}

fn na_gama(c: [f32; 3]) -> bool {
    c.iter().all(|v| (0.0..=1.0).contains(v))
}

/// O corte de gama SÓ no croma: a maior croma (até à resolução) que cabe no sRGB com a mesma
/// claridade e matiz. Claridade fora de `0..=1` vira branco/preto.
#[must_use]
pub fn na_gama_pelo_croma(p: OklchColor) -> OklchColor {
    if na_gama(rgb_linear(p)) {
        return p;
    }
    let (mut min, mut max) = (0.0_f32, p.c);
    while max - min > RESOLUCAO_CROMA {
        let c = (min + max) * 0.5;
        if na_gama(rgb_linear(OklchColor { c, ..p })) {
            min = c;
        } else {
            max = c;
        }
    }
    OklchColor { c: min, ..p }
}

/// A rampa em OKLCH, já dentro da gama, do escuro para o claro (antes do arredondamento).
#[must_use]
pub fn gera_rampa_oklch(base: OklchColor, cfg: ConfigRampa) -> Vec<OklchColor> {
    planeada(base, cfg)
        .into_iter()
        .map(na_gama_pelo_croma)
        .collect()
}

/// **Palette ▸ New Ramp**: a rampa em sRGB de 8 bits, do escuro para o claro.
#[must_use]
pub fn gera_rampa(base: [u8; 4], cfg: ConfigRampa) -> Vec<[u8; 4]> {
    gera_rampa_oklch(oklch_de(base), cfg)
        .into_iter()
        .map(|p| {
            let [r, g, b] = rgb_linear(p).map(|v| byte(linear_to_srgb_unit(v.clamp(0.0, 1.0))));
            [r, g, b, base[3]]
        })
        .collect()
}

fn byte(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// OKLab de uma cor de 8 bits (para quem mede a rampa gravada).
#[must_use]
pub fn oklab_de(rgba: [u8; 4]) -> [f32; 3] {
    to_oklab([0, 1, 2].map(|i| srgb_to_linear_unit(f32::from(rgba[i]) / 255.0)))
}

/// Palette ▸ New Ramp, como UM passo: as `cores` entram no fim da paleta e viram uma rampa.
/// `None` se a paleta passaria de 256 cores.
#[must_use]
pub fn op_rampa_nova(s: &ph2d_pixel_core::Sprite, nome: &str, cores: &[[u8; 4]]) -> Option<Op> {
    let mut colors = s.palette().to_vec();
    let first = colors.len();
    colors.extend_from_slice(cores);
    if colors.len() > ph2d_pixel_core::palette::MAX_COLORS {
        return None;
    }
    let indices = (first..colors.len()).map(|i| i as u8).collect();
    let add = DocOp::AddRamp {
        name: nome.to_owned(),
        indices,
    };
    Some(Op::Batch(vec![
        Op::Palette {
            colors,
            transparent: s.transparent_index(),
        },
        Op::Doc(Box::new(add)),
    ]))
}

pub mod quantiza;
pub use quantiza::{Oklab, PreservaRampas, converte_oklab, converte_preservando_rampas};

/// OKLCH → sRGB de 8 bits, sem corte de gama (o chamador garante a gama).
#[must_use]
pub fn srgb8_de_oklch(p: OklchColor) -> [u8; 4] {
    let [r, g, b] = rgb_linear(p).map(|v| byte(linear_to_srgb_unit(v.clamp(0.0, 1.0))));
    [r, g, b, byte(p.a)]
}
