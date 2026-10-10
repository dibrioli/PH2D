//! **A lente inteira na CPU** — a referência do passe da placa, pixel a pixel, na ordem fixa.

use crate::aberracao::{self, Aberracao};
use crate::cubo::Cubo;
use crate::grade::{self, Grade};
use crate::grao::{self, Grao};
use crate::vinheta::{self, Vinheta};
use ph2d_color::srgb::{linear_to_srgb_unit, srgb_to_linear_unit};

/// Os quatro efeitos.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lente {
    pub aberracao: Aberracao,
    pub vinheta: Vinheta,
    pub grade: Grade,
    pub grao: Grao,
}

impl Lente {
    /// Nada a fazer: o passe não corre e o quadro é byte-idêntico.
    #[must_use]
    pub fn neutra(&self, tem_lut: bool) -> bool {
        self.aberracao.neutra()
            && self.vinheta.neutra()
            && self.grade.neutra(tem_lut)
            && self.grao.neutra()
    }
}

/// Uma imagem RGBA em luz LINEAR, alfa reto, linha a linha.
#[derive(Clone, Debug, PartialEq)]
pub struct Imagem {
    pub w: u32,
    pub h: u32,
    pub px: Vec<[f32; 4]>,
}

impl Imagem {
    #[must_use]
    pub fn em(&self, x: i32, y: i32) -> [f32; 4] {
        self.px[(y as u32 * self.w + x as u32) as usize]
    }
}

/// **A lente sobre a imagem** — aberração → vinheta → grade → grão. `tique` é o do relógio do
/// Motion (o grão animado anda com ele).
#[must_use]
pub fn aplica_cpu(src: &Imagem, l: &Lente, lut: Option<&Cubo>, tique: u32) -> Imagem {
    let (w, h) = (src.w as i32, src.h as i32);
    let quadro = [src.w as f32, src.h as f32];
    let centro = [quadro[0] * 0.5, quadro[1] * 0.5];
    let meia_diag = 0.5 * quadro[0].hypot(quadro[1]);
    let geo = vinheta::geometria(&l.vinheta, quadro);
    let balanco = grade::matriz_balanco(l.grade.temperatura, l.grade.matiz);
    let semente = grao::semente(&l.grao, tique);
    let mut px = Vec::with_capacity(src.px.len());
    for y in 0..h {
        for x in 0..w {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            let base = src.em(x, y);
            let mut c = [base[0], base[1], base[2]];
            if !l.aberracao.neutra() {
                let pos = aberracao::posicoes(p, centro, meia_diag, &l.aberracao);
                c[0] = aberracao::bilinear(|i, j| src.em(i, j)[0], w, h, pos[0]);
                c[2] = aberracao::bilinear(|i, j| src.em(i, j)[2], w, h, pos[2]);
            }
            if !l.vinheta.neutra() {
                c = vinheta::aplica(c, p, &l.vinheta, &geo);
            }
            if !l.grade.neutra(lut.is_some()) {
                c = grade::aplica(c, &l.grade, &balanco, lut);
            }
            if !l.grao.neutra() {
                let cod = c.map(|v| linear_to_srgb_unit(v.clamp(0.0, 1.0)));
                c = grao::aplica(cod, p, &l.grao, semente)
                    .map(|v| srgb_to_linear_unit(v.clamp(0.0, 1.0)));
            }
            px.push([c[0], c[1], c[2], base[3]]);
        }
    }
    Imagem {
        w: src.w,
        h: src.h,
        px,
    }
}
