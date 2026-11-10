//! **O G-buffer da dobra** (B15.3 + B15.1): o tique da lâmpada só SOMBREIA. A altura em moldura, o corpo e o
//! material de cada texel saem de [`ReliefFields::dobra_da_linha`] UMA vez, no 1.º tique depois da recomposição
//! inteira, e ficam com a composição não acesa ([`LuzGuardada`]): a mesma lista branca de eventos os mantém vivos,
//! e tudo o que a derruba derruba-os. O núcleo do sombreado é UM ([`sombreia_faixa`]), partilhado com
//! [`PainterTool::apply_impasto_light`] — logo o mesmo bit (`o_tique_de_luz_pelo_gbuffer_e_a_luz_inteira`).

use super::impasto_light::{FAIXA_LINHAS, gloss_body, light_pixel, paint_body};
use super::impasto_shade::{MatShade, Rig};
use super::relief_fields::ReliefFields;
use crate::tool::PainterTool;
use ph2d_painter_brush::material::{Material, MaterialBytes};
use rayon::prelude::*;
use std::sync::OnceLock;

/// A tela composta antes da luz e, preguiçosa, a dobra do relevo que a acende.
pub(crate) struct LuzGuardada {
    pub(crate) nao_aceso: Vec<u8>,
    dobra: OnceLock<Option<DobraDaTela>>,
}

impl LuzGuardada {
    pub(crate) fn new(nao_aceso: Vec<u8>) -> Self {
        Self {
            nao_aceso,
            dobra: OnceLock::new(),
        }
    }
}

/// A dobra da tela inteira, por faixas de [`FAIXA_LINHAS`]: `alt` tem a moldura de 1 px (`(w + 2) × (h + 2)`),
/// `cob` e `idx` o passo `w + 2` das linhas da dobra; o material é um índice na paleta da SUA faixa (u16, montada
/// na ordem da faixa: o mesmo índice em qualquer nº de fios).
pub(crate) struct DobraDaTela {
    alt: Vec<f32>,
    cob: Vec<f32>,
    idx: Vec<u16>,
    paletas: Vec<Vec<MaterialBytes>>,
    paper_body: f32,
}

/// Onde o sombreado vai buscar o material do texel `q` (índice nas linhas da dobra).
pub(super) trait MatFonte {
    fn mat(&mut self, light: &Rig, q: usize) -> &MatShade;
}

/// A cache de uma entrada sobre os bytes da dobra (o passe de sempre).
pub(super) struct PorBytes<'a> {
    pub(super) mats: &'a [MaterialBytes],
    pub(super) cache: MatShade,
}

impl MatFonte for PorBytes<'_> {
    #[inline]
    fn mat(&mut self, light: &Rig, q: usize) -> &MatShade {
        let key = self.mats[q];
        if key != self.cache.key {
            self.cache = light.resolve(key);
        }
        &self.cache
    }
}

/// A paleta da faixa, já resolvida contra o rig deste tique.
struct PorPaleta<'a> {
    idx: &'a [u16],
    res: &'a [MatShade],
}

impl MatFonte for PorPaleta<'_> {
    #[inline]
    fn mat(&mut self, _: &Rig, q: usize) -> &MatShade {
        &self.res[usize::from(self.idx[q])]
    }
}

/// Sombreia uma faixa de `n = faixa.len() / (w·4)` linhas: `alt` = as `n + 2` linhas em moldura, `cob` as `n`
/// linhas da dobra (passo `w + 2`).
#[inline]
pub(super) fn sombreia_faixa(
    light: &Rig,
    paper_body: f32,
    alt: &[f32],
    cob: &[f32],
    faixa: &mut [u8],
    w: usize,
    mut mats: impl MatFonte,
) {
    let fw = w + 2;
    let n = faixa.len() / (w * 4);
    for ry in 0..n {
        let c = (ry + 1) * fw + 1; // o píxel (0, ry) na moldura
        for rx in 0..w {
            let k = c + rx;
            let q = ry * fw + 1 + rx; // o mesmo píxel nas linhas da dobra
            let dhx = (alt[k + 1] - alt[k - 1]) * 0.5;
            let dhy = (alt[k + fw] - alt[k - fw]) * 0.5;
            let cover = cob[q];
            let i = (ry * w + rx) * 4;
            let albedo = [
                f32::from(faixa[i]) / 255.0,
                f32::from(faixa[i + 1]) / 255.0,
                f32::from(faixa[i + 2]) / 255.0,
            ];
            let mat = mats.mat(light, q);
            let body = paint_body(cover).max(paper_body);
            let (mul, add) = light.shade(mat, body, gloss_body(cover), dhx, dhy, albedo);
            if mul == [1.0; 3] && add == [0.0; 3] {
                continue; // flat: byte-identical, and not even a rounding trip through f32
            }
            for c in 0..3 {
                let lit = light_pixel(albedo[c], mul[c], add[c]);
                faixa[i + c] = (lit * 255.0 + 0.5) as u8;
            }
        }
    }
}

/// A dobra das linhas `gy0 - 1 ..= gy0 + n` (moldura incluída) a partir de `x0 - 1`: as de moldura só a altura.
pub(super) fn dobra_da_faixa(
    fields: &ReliefFields<'_>,
    gy0: i64,
    x0: i64,
    fw: usize,
    alt: &mut [f32],
    cob: &mut [f32],
    mats: &mut [MaterialBytes],
) {
    let n = cob.len() / fw;
    for (j, linha) in alt.chunks_exact_mut(fw).enumerate() {
        let gy = gy0 - 1 + j as i64;
        if j == 0 || j == n + 1 {
            for (i, a) in linha.iter_mut().enumerate() {
                *a = fields.height_at(x0 - 1 + i as i64, gy);
            }
        } else {
            let r = (j - 1) * fw..j * fw;
            fields.dobra_da_linha(gy, x0 - 1, linha, &mut cob[r.clone()], &mut mats[r]);
        }
    }
}

impl PainterTool {
    /// Acende a tela inteira `rgba` pelo G-buffer de `guardada` (montado aqui no 1.º tique). `false` = não há
    /// G-buffer possível (sem campos, ou uma faixa com mais de 65 536 materiais): o chamador usa o passe inteiro.
    pub(super) fn acende_pela_dobra(&self, guardada: &LuzGuardada, rgba: &mut [u8]) -> bool {
        if !self.impasto_visible() {
            return true;
        }
        let Some(light) = Rig::new(&self.paint.impasto_rig) else {
            return true; // every lamp switched off: the canvas comes back unlit, to the byte
        };
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 || rgba.len() < w * h * 4 {
            return true;
        }
        let Some(dobra) = guardada.dobra.get_or_init(|| self.dobra_da_tela()) else {
            return false;
        };
        let fw = w + 2;
        rgba[..w * h * 4]
            .par_chunks_mut(w * 4 * FAIXA_LINHAS)
            .enumerate()
            .for_each(|(b, faixa)| {
                let n = faixa.len() / (w * 4);
                let y = b * FAIXA_LINHAS;
                let res: Vec<MatShade> =
                    dobra.paletas[b].iter().map(|&k| light.resolve(k)).collect();
                let mats = PorPaleta {
                    idx: &dobra.idx[y * fw..(y + n) * fw],
                    res: &res,
                };
                let (alt, cob) = (
                    &dobra.alt[y * fw..(y + n + 2) * fw],
                    &dobra.cob[y * fw..(y + n) * fw],
                );
                sombreia_faixa(&light, dobra.paper_body, alt, cob, faixa, w, mats);
            });
        true
    }

    /// A dobra da tela inteira, faixa a faixa no pool (as faixas de [`Self::apply_impasto_light`]).
    fn dobra_da_tela(&self) -> Option<DobraDaTela> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let fields = self.impasto_fields_para(super::Region {
            x: 0,
            y: 0,
            w: w as u32,
            h: h as u32,
        })?;
        let fw = w + 2;
        let mut cob = vec![0.0f32; fw * h];
        let mut idx = vec![0u16; fw * h];
        let faixas: Vec<Option<(Vec<f32>, Vec<MaterialBytes>)>> = cob
            .par_chunks_mut(fw * FAIXA_LINHAS)
            .zip(idx.par_chunks_mut(fw * FAIXA_LINHAS))
            .enumerate()
            .map(|(b, (cob, idx))| {
                let n = cob.len() / fw;
                let mut alt = vec![0.0f32; fw * (n + 2)];
                let mut mats: Vec<MaterialBytes> = vec![[0; 7]; fw * n];
                let gy0 = (b * FAIXA_LINHAS) as i64;
                dobra_da_faixa(&fields, gy0, 0, fw, &mut alt, cob, &mut mats);
                let mut paleta = vec![Material::NEUTRAL.to_bytes()];
                let mut ult = 0usize;
                for (m, i) in mats.iter().zip(idx.iter_mut()) {
                    if paleta[ult] != *m {
                        ult = match paleta.iter().position(|k| k == m) {
                            Some(p) => p,
                            None => {
                                paleta.push(*m);
                                paleta.len() - 1
                            }
                        };
                    }
                    *i = u16::try_from(ult).ok()?;
                }
                Some((alt, paleta))
            })
            .collect();
        let mut alt = vec![0.0f32; fw * (h + 2)];
        let mut paletas = Vec::with_capacity(faixas.len());
        for (b, f) in faixas.into_iter().enumerate() {
            let (a, paleta) = f?;
            let y = b * FAIXA_LINHAS;
            // As linhas da faixa (moldura de cima incluída); a de baixo é a de cima da faixa seguinte, ou a última.
            alt[y * fw..y * fw + a.len()].copy_from_slice(&a);
            paletas.push(paleta);
        }
        Some(DobraDaTela {
            alt,
            cob,
            idx,
            paletas,
            paper_body: fields.paper_body(),
        })
    }
}
