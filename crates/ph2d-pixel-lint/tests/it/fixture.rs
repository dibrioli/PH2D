//! As fixtures do linter, construídas em código (P13 §7): `vicios_p13` (16×16, UM caso de cada
//! vício numa célula conhecida) e dois sprites limpos da DB32 (caixa e bola).

use ph2d_pixel_core::palette::DEFAULT as DB32;
use ph2d_pixel_core::{NewSprite, Op, Patch, Sprite, apply};
use ph2d_pixel_lint::Regra;
use std::collections::BTreeMap;

pub type Cel = [u32; 2];

/// Um RGBA `w × h` transparente onde se pinta à mão.
pub struct Tela {
    pub w: u32,
    pub h: u32,
    pub px: Vec<u8>,
}

impl Tela {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            px: vec![0; (w * h * 4) as usize],
        }
    }
    pub fn p(&mut self, x: u32, y: u32, c: [u8; 4]) {
        let i = ((y * self.w + x) * 4) as usize;
        self.px[i..i + 4].copy_from_slice(&c);
    }
    pub fn cor(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * self.w + x) * 4) as usize;
        [self.px[i], self.px[i + 1], self.px[i + 2], self.px[i + 3]]
    }
    /// O documento: um sprite RGBA com este cel na camada e no quadro de nascença.
    pub fn sprite(&self) -> Sprite {
        let mut s = Sprite::new(NewSprite {
            width: self.w,
            height: self.h,
            ..NewSprite::default()
        })
        .expect("tamanho");
        let r = s.bounds();
        let op = Op::Patch(Patch {
            layer: s.layers()[0].id,
            frame: s.frames()[0].id,
            bbox: Some(r),
            rect: r,
            pixels: self.px.clone(),
        });
        apply(&mut s, op).expect("patch");
        s
    }
}

/// A célula onde cada vício está: o gabarito.
pub type Gabarito = BTreeMap<Regra, Vec<Vec<Cel>>>;

/// `vicios_p13`. Cores da DB32; a do órfão é única (o «cores a mais» com limite 7 aponta-a).
pub fn vicios_p13() -> (Tela, Gabarito) {
    let mut t = Tela::new(16, 16);
    let mut g = Gabarito::new();
    // Órfão: uma migalha solta.
    t.p(1, 1, DB32[28]);
    g.insert(Regra::Orfao, vec![vec![[1, 1]]]);
    g.insert(Regra::CoresAMais, vec![vec![[1, 1]]]);
    // Canto duplo: 4 px, o degrau em Z (7,3)+(7,4), 4 px.
    let linha = DB32[1];
    for x in 4..=7 {
        t.p(x, 3, linha);
    }
    for x in 7..=10 {
        t.p(x, 4, linha);
    }
    g.insert(Regra::CantoDuplo, vec![vec![[7, 3], [7, 4]]]);
    // Degrau: corridas 3-1-3; o 1 do meio quebra o ritmo.
    for x in 0..=2 {
        t.p(x, 6, linha);
    }
    t.p(3, 7, linha);
    for x in 4..=6 {
        t.p(x, 8, linha);
    }
    g.insert(Regra::Degrau, vec![vec![[3, 7]]]);
    // Faixa: claro / médio de 1 px / escuro, fronteiras paralelas.
    for x in 1..=4 {
        t.p(x, 10, DB32[7]);
        t.p(x, 11, DB32[6]);
        t.p(x, 12, DB32[4]);
    }
    g.insert(Regra::Faixa, vec![(1..=4).map(|x| [x, 11]).collect()]);
    // Almofada: 8×8, contorno / sombra a 2–3 px de toda a borda / miolo.
    let mut anel = Vec::new();
    for y in 8..=15u32 {
        for x in 8..=15u32 {
            let d = 1 + (x - 8).min(15 - x).min(y - 8).min(15 - y);
            let c = match d {
                1 => DB32[2],
                2 | 3 => {
                    anel.push([x, y]);
                    DB32[16]
                }
                _ => DB32[18],
            };
            t.p(x, y, c);
        }
    }
    anel.sort_by_key(|c| (c[1], c[0]));
    g.insert(Regra::Almofada, vec![anel]);
    (t, g)
}

/// Uma caixa de madeira 12×10: contorno, tampo claro, sombra de 2 px à direita e em baixo (luz de
/// cima à esquerda), um brilho de 1 px e a junta entre duas tábuas.
pub fn caixa_limpa() -> Tela {
    let mut t = Tela::new(16, 16);
    for y in 3..=12 {
        for x in 2..=13 {
            let borda = x == 2 || x == 13 || y == 3 || y == 12;
            let sombra = x >= 11 || y >= 10;
            t.p(
                x,
                y,
                if borda {
                    DB32[1]
                } else if sombra {
                    DB32[4]
                } else {
                    DB32[5]
                },
            );
        }
    }
    // A junta das tábuas: um traço ESCURO de 1 px entre duas madeiras (não é tom de transição).
    for x in 3..=10 {
        t.p(x, 7, DB32[3]);
        t.p(x, 8, DB32[6]);
        t.p(x, 9, DB32[6]);
    }
    t.p(4, 5, DB32[7]);
    t
}

/// Uma bola 16×16: disco de raio 6,5, contorno (4-fronteira sem os L de 45°), sombra no quarto de
/// baixo à direita, um brilho de 1 px. (Com os L, o linter acusa os 4 cantos duplos — visto 09/11.)
pub fn bola_limpa() -> Tela {
    let mut t = Tela::new(16, 16);
    let dentro = |x: i32, y: i32| {
        let (dx, dy) = (f64::from(x) - 7.5, f64::from(y) - 7.5);
        dx * dx + dy * dy <= 42.25
    };
    for y in 0..16i32 {
        for x in 0..16i32 {
            if !dentro(x, y) {
                continue;
            }
            let borda = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|&(dx, dy)| !dentro(x + dx, y + dy));
            let sombra = x + y >= 18;
            let c = if borda {
                DB32[12]
            } else if sombra {
                DB32[10]
            } else {
                DB32[9]
            };
            t.p(x as u32, y as u32, c);
        }
    }
    // Pixel-perfect: a 4-fronteira do disco tem um L em cada diagonal de 45°; o artista apaga-o.
    for (x, y) in [(3, 3), (12, 3), (3, 12), (12, 12)] {
        t.p(x, y, [0; 4]);
    }
    t.p(5, 5, DB32[21]);
    t
}
