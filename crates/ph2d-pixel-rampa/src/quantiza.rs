//! **Cor → índice em OKLab** (P13 R3, `docs/PixelArt/P13_plano.md` §4): as duas métricas que o
//! `Mapper` do núcleo recebe pela porta [`Metrica`] — a OKLab pura e [`PreservaRampas`].
//!
//! `PreservaRampas`: a cor vai à rampa de MATIZ mais próxima (a distância no plano `ab`, que pesa a
//! matiz pelo croma e não se perde nos cinzentos) e, dentro dela, ao degrau de claridade `L` mais
//! próxima — uma escolha monótona em `L`, por isso a ordem de claridade da origem nunca se
//! inverte numa rampa. Se a cor OKLab mais próxima de todas é uma cor SOLTA, ela ganha.

use ph2d_pixel_core::convert::{Convert, Metrica, convert_com};
use ph2d_pixel_core::{ColorMode, Sprite, Whole};

use crate::oklab_de;

fn lab_alfa(c: [u8; 4]) -> [f32; 4] {
    let [l, a, b] = oklab_de(c);
    [l, a, b, f32::from(c[3]) / 255.0]
}

fn d2(p: [f32; 4], q: [f32; 4]) -> f32 {
    (0..4).map(|k| (p[k] - q[k]).powi(2)).sum()
}

fn mais_proxima_lab(lab: &[[f32; 4]], candidates: &[u8], c: [f32; 4]) -> u8 {
    let mut best = (f32::INFINITY, candidates[0]);
    for &i in candidates {
        let d = d2(c, lab[usize::from(i)]);
        if d < best.0 {
            best = (d, i);
        }
    }
    best.1
}

/// A distância euclidiana em OKLab (+ alfa em `0..1`), empate → o menor índice.
#[derive(Debug, Clone)]
pub struct Oklab {
    lab: Vec<[f32; 4]>,
}

impl Oklab {
    #[must_use]
    pub fn new(palette: &[[u8; 4]]) -> Self {
        Self {
            lab: palette.iter().map(|&c| lab_alfa(c)).collect(),
        }
    }
}

impl Metrica for Oklab {
    fn mais_proxima(&self, _palette: &[[u8; 4]], candidates: &[u8], c: [u8; 4]) -> u8 {
        mais_proxima_lab(&self.lab, candidates, lab_alfa(c))
    }
}

/// Converte mantendo as rampas da paleta do sprite (ver o módulo).
#[derive(Debug, Clone)]
pub struct PreservaRampas {
    lab: Vec<[f32; 4]>,
    /// Os degraus de cada rampa, ordenados por `L` (estável).
    rampas: Vec<Vec<u8>>,
    em_rampa: [bool; 256],
}

impl PreservaRampas {
    #[must_use]
    pub fn new(sprite: &Sprite) -> Self {
        let lab: Vec<[f32; 4]> = sprite.palette().iter().map(|&c| lab_alfa(c)).collect();
        let mut em_rampa = [false; 256];
        let rampas = sprite
            .ramps()
            .ramps
            .iter()
            .map(|r| {
                let mut v: Vec<u8> = r
                    .indices
                    .iter()
                    .copied()
                    .filter(|&i| usize::from(i) < lab.len())
                    .collect();
                v.sort_by(|&a, &b| lab[usize::from(a)][0].total_cmp(&lab[usize::from(b)][0]));
                for &i in &v {
                    em_rampa[usize::from(i)] = true;
                }
                v
            })
            .collect();
        Self {
            lab,
            rampas,
            em_rampa,
        }
    }

    /// A posição de `idx` na sua rampa pela ordem de `L` (`0` = o mais escuro).
    #[must_use]
    pub fn degrau(&self, idx: u8) -> Option<(usize, usize)> {
        self.rampas
            .iter()
            .enumerate()
            .find_map(|(r, v)| v.iter().position(|&i| i == idx).map(|p| (r, p)))
    }
}

impl Metrica for PreservaRampas {
    fn mais_proxima(&self, _palette: &[[u8; 4]], candidates: &[u8], c: [u8; 4]) -> u8 {
        let o = lab_alfa(c);
        let n = mais_proxima_lab(&self.lab, candidates, o);
        if !self.em_rampa[usize::from(n)] {
            return n;
        }
        let ab = |p: [f32; 4]| (p[1] - o[1]).powi(2) + (p[2] - o[2]).powi(2);
        let mut rampa: Option<(f32, &Vec<u8>)> = None;
        for v in &self.rampas {
            for &i in v.iter().filter(|i| candidates.contains(i)) {
                let d = ab(self.lab[usize::from(i)]);
                if rampa.is_none_or(|(m, _)| d < m) {
                    rampa = Some((d, v));
                }
            }
        }
        let Some((_, v)) = rampa else { return n };
        let mut best = (f32::INFINITY, n);
        for &i in v.iter().filter(|i| candidates.contains(i)) {
            let d = (self.lab[usize::from(i)][0] - o[0]).abs();
            if d < best.0 {
                best = (d, i);
            }
        }
        best.1
    }
}

/// **Color Mode ▸ Indexed ▸ Keep Ramps**: a conversão com [`PreservaRampas`]. A paleta e as
/// rampas ficam (o `Op::Whole` aceita-a sem remapear: nenhum índice muda de cor).
#[must_use]
pub fn converte_preservando_rampas(sprite: &Sprite, opt: Convert) -> Option<Whole> {
    convert_com(
        sprite,
        ColorMode::Indexed,
        opt,
        &PreservaRampas::new(sprite),
    )
}

/// **Color Mode ▸ Indexed ▸ Perceptual**: a conversão pela cor mais próxima em OKLab.
#[must_use]
pub fn converte_oklab(sprite: &Sprite, opt: Convert) -> Option<Whole> {
    convert_com(
        sprite,
        ColorMode::Indexed,
        opt,
        &Oklab::new(sprite.palette()),
    )
}
