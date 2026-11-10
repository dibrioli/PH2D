//! **A luz na rampa** (P13 L, `docs/PixelArt/P13_plano.md` §4.L): o Pixel Sprite iluminado
//! continua na paleta. A luz não multiplica a cor: escolhe outro degrau da MESMA rampa.
//!
//! Lei: `L_alvo = L(cor) · lum(luz)`, com `lum(luz) = L` OKLab da luz linear (a luz neutra
//! `(1,1,1)` dá `lum ≈ 1`); o degrau é o de L mais próximo dentro da rampa (as pontas param, como
//! no Shading); empate fica com o mais escuro. Texel fora de rampa ⇒ a lei de hoje (multiplica),
//! que é do receptor e não daqui ([`LutDaLuz::degrau_de`] devolve `None`).
//!
//! Este é o GABARITO da placa: `ph2d-render` (`band_blit.wgsl`, `luz_receber_rampa`) lê a LUT de
//! [`LutDaLuz::texeis`] e faz a mesma conta com o `pc_to_oklab` da `ph2d-color`.

use ph2d_color::perceptual::to_oklab;
use ph2d_color::srgb::srgb_to_linear_unit;
use ph2d_pixel_core::PaletteRamps;

/// Um degrau da rampa como a luz o lê: a claridade OKLab e a cor da paleta.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DegrauDaLuz {
    pub l: f32,
    pub cor: [u8; 4],
}

/// A claridade OKLab de uma cor sRGB de 8 bits.
#[must_use]
pub fn claridade(cor: [u8; 4]) -> f32 {
    to_oklab([0, 1, 2].map(|i| srgb_to_linear_unit(f32::from(cor[i]) / 255.0)))[0]
}

/// A força da luz (RGB linear, `1` = neutra) na escala da claridade.
#[must_use]
pub fn lum(luz_rgb: [f32; 3]) -> f32 {
    to_oklab(luz_rgb.map(|v| v.max(0.0)))[0]
}

/// O degrau que a luz escolhe na rampa (`pos` = o degrau do texel).
#[must_use]
pub fn degrau_aceso(rampa: &[DegrauDaLuz], pos: usize, luz_rgb: [f32; 3]) -> usize {
    let Some(d) = rampa.get(pos) else {
        return pos;
    };
    let alvo = d.l * lum(luz_rgb);
    let mut melhor = 0;
    let mut dist = f32::INFINITY;
    for (i, g) in rampa.iter().enumerate() {
        let e = (g.l - alvo).abs();
        if e < dist {
            dist = e;
            melhor = i;
        }
    }
    melhor
}

/// A cor do texel `cor`, no degrau `pos` da rampa, acesa por `luz_rgb`. Sempre uma cor da rampa.
#[must_use]
pub fn luz_na_rampa(
    cor: [u8; 4],
    rampa_oklab: &[DegrauDaLuz],
    pos: usize,
    luz_rgb: [f32; 3],
) -> [u8; 4] {
    match rampa_oklab.get(degrau_aceso(rampa_oklab, pos, luz_rgb)) {
        Some(d) => [d.cor[0], d.cor[1], d.cor[2], cor[3]],
        None => cor,
    }
}

/// As rampas de uma paleta na ordem do documento (o «slot» é o id da banda de rampa).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LutDaLuz {
    pub rampas: Vec<Vec<DegrauDaLuz>>,
}

impl LutDaLuz {
    #[must_use]
    pub fn da_paleta(paleta: &[[u8; 4]], ramps: &PaletteRamps) -> Self {
        let rampas = ramps
            .ramps
            .iter()
            .map(|r| {
                r.indices
                    .iter()
                    .filter_map(|&i| paleta.get(usize::from(i)))
                    .map(|&cor| DegrauDaLuz {
                        l: claridade(cor),
                        cor,
                    })
                    .collect()
            })
            .collect();
        Self { rampas }
    }

    /// `(slot, pos)` da cor na LUT (pela cor RGB, como a banda a vê); `None` = fora de rampa.
    #[must_use]
    pub fn degrau_de(&self, cor: [u8; 4]) -> Option<(usize, usize)> {
        self.rampas.iter().enumerate().find_map(|(s, r)| {
            r.iter()
                .position(|d| d.cor[..3] == cor[..3])
                .map(|p| (s, p))
        })
    }

    /// A largura da textura LUT (o degrau mais longo; ≥ 1).
    #[must_use]
    pub fn largura(&self) -> usize {
        self.rampas.iter().map(Vec::len).max().unwrap_or(0).max(1)
    }

    /// A LUT para a placa, linha a linha (`altura = rampas.len()`): `(L, r, g, b)` com a cor em
    /// sRGB `0..1`; o texel depois do último degrau tem `L = -1` (fim da rampa).
    #[must_use]
    pub fn texeis(&self) -> Vec<[f32; 4]> {
        let w = self.largura();
        let mut out = Vec::with_capacity(w * self.rampas.len());
        for r in &self.rampas {
            for i in 0..w {
                out.push(match r.get(i) {
                    Some(d) => [
                        d.l,
                        f32::from(d.cor[0]) / 255.0,
                        f32::from(d.cor[1]) / 255.0,
                        f32::from(d.cor[2]) / 255.0,
                    ],
                    None => [-1.0, 0.0, 0.0, 0.0],
                });
            }
        }
        out
    }

    /// Acende um cel RGBA8 com uma luz por texel; fora de rampa (ou transparente) fica `None`,
    /// para o receptor aplicar a lei de hoje.
    #[must_use]
    pub fn acende(&self, cor: [u8; 4], luz_rgb: [f32; 3]) -> Option<[u8; 4]> {
        if cor[3] == 0 {
            return None;
        }
        let (s, p) = self.degrau_de(cor)?;
        Some(luz_na_rampa(cor, &self.rampas[s], p, luz_rgb))
    }
}
