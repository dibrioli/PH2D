//! **Sprite ▸ Color Mode**: RGB ↔ Grayscale ↔ Indexed, com dither. Devolve o documento inteiro
//! novo ([`Whole`]) — aplicado por [`crate::apply`], o desfazer devolve o de antes ao byte.
//!
//! Leis: cinza = [`ToGray`] (medido no Aseprite); a cor de um índice é a da paleta e o índice
//! transparente vale alfa 0; RGB/cinza → índice = a cor MAIS PRÓXIMA exata (distância euclidiana
//! RGBA, empate → o menor índice), fora o transparente numa camada que não é a Background. ⛔ O
//! «Default (Octree)» do Aseprite é uma aproximação (medido: 6/216 de erro contra qualquer
//! distância exata) — o exato é o que o artista espera da paleta que escolheu.
//! O dither é ANCORADO NA TELA (a célula `(x, y)` do sprite, nunca a do cel).

use std::collections::BTreeMap;

use crate::color::ToGray;
use crate::op::Whole;
use crate::sprite::{Cel, CelKey, ColorMode, Sprite};

/// A matriz de Bayer de um dither ordenado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bayer {
    M2,
    M4,
    M8,
}

impl Bayer {
    /// O lado da matriz.
    #[must_use]
    pub const fn side(self) -> usize {
        match self {
            Bayer::M2 => 2,
            Bayer::M4 => 4,
            Bayer::M8 => 8,
        }
    }

    /// O limiar inteiro `0..side²` na célula `(x, y)` da tela.
    #[must_use]
    pub fn at(self, x: i32, y: i32) -> u32 {
        let n = self.side() as i32;
        let (i, j) = (x.rem_euclid(n) as usize, y.rem_euclid(n) as usize);
        match self {
            Bayer::M2 => BAYER2[j * 2 + i],
            Bayer::M4 => BAYER4[j * 4 + i],
            Bayer::M8 => BAYER8[j * 8 + i],
        }
    }
}

const BAYER2: [u32; 4] = [0, 2, 3, 1];
const BAYER4: [u32; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
const BAYER8: [u32; 64] = [
    0, 32, 8, 40, 2, 34, 10, 42, 48, 16, 56, 24, 50, 18, 58, 26, 12, 44, 4, 36, 14, 46, 6, 38, 60,
    28, 52, 20, 62, 30, 54, 22, 3, 35, 11, 43, 1, 33, 9, 41, 51, 19, 59, 27, 49, 17, 57, 25, 15,
    47, 7, 39, 13, 45, 5, 37, 63, 31, 55, 23, 61, 29, 53, 21,
];

/// O dither da conversão para Indexed (as opções do diálogo do Aseprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dither {
    #[default]
    None,
    /// Ordenado: a cor desloca-se pelo limiar da matriz antes de procurar a mais próxima.
    Ordered(Bayer),
    /// O «Old» do Aseprite: escolhe entre a mais próxima e a segunda pela matriz.
    Old(Bayer),
    /// Difusão de erro de Floyd–Steinberg (7/16, 3/16, 5/16, 1/16), linha a linha.
    FloydSteinberg,
}

/// As opções de Sprite ▸ Color Mode ▸ More Options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Convert {
    pub dither: Dither,
    pub to_gray: ToGray,
}

/// O sprite convertido para `mode` (a paleta fica; o transparente fica). `None` se já está nele.
#[must_use]
pub fn convert(sprite: &Sprite, mode: ColorMode, opt: Convert) -> Option<Whole> {
    if sprite.mode == mode {
        return None;
    }
    let mut cels = BTreeMap::new();
    for (&key, cel) in &sprite.cels {
        let background = sprite.layer(key.0).is_some_and(|l| l.background);
        let rgba: Vec<[u8; 4]> = cel
            .data
            .chunks_exact(sprite.mode.bpp())
            .map(|b| sprite.rgba_of_value(sprite.read_value(b)))
            .collect();
        let data = match mode {
            ColorMode::Rgba => rgba.concat(),
            ColorMode::Gray => rgba
                .iter()
                .flat_map(|&c| [opt.to_gray.of(c), c[3]])
                .collect(),
            ColorMode::Indexed => {
                let mut m = Mapper::new(&sprite.palette, sprite.transparent, background);
                to_indexed(&mut m, &rgba, cel, opt.dither)
            }
        };
        cels.insert(key, Cel { bbox: cel.bbox, data });
    }
    Some(Whole {
        mode,
        palette: sprite.palette.clone(),
        transparent: sprite.transparent,
        cels,
    })
}

fn to_indexed(m: &mut Mapper, rgba: &[[u8; 4]], cel: &Cel, dither: Dither) -> Vec<u8> {
    let w = cel.bbox.w as usize;
    let at = |i: usize| (cel.bbox.x + (i % w) as i32, cel.bbox.y + (i / w) as i32);
    match dither {
        Dither::None => rgba.iter().map(|&c| m.index(c)).collect(),
        Dither::Ordered(b) => rgba
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                let (x, y) = at(i);
                m.ordered(c, b, x, y)
            })
            .collect(),
        Dither::Old(b) => rgba
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                let (x, y) = at(i);
                m.old(c, b, x, y)
            })
            .collect(),
        Dither::FloydSteinberg => floyd(m, rgba, w),
    }
}

/// Erro de Floyd–Steinberg em inteiros (×16), propagado só a células opacas.
fn floyd(m: &mut Mapper, rgba: &[[u8; 4]], w: usize) -> Vec<u8> {
    let mut err = vec![[0i32; 3]; rgba.len()];
    let mut out = vec![0u8; rgba.len()];
    for i in 0..rgba.len() {
        let c = rgba[i];
        if c[3] == 0 {
            out[i] = m.index(c);
            continue;
        }
        let mut want = c;
        for k in 0..3 {
            want[k] = (i32::from(c[k]) + err[i][k].div_euclid(16)).clamp(0, 255) as u8;
        }
        let idx = m.index(want);
        out[i] = idx;
        let got = m.palette[usize::from(idx)];
        let e: [i32; 3] = std::array::from_fn(|k| i32::from(want[k]) - i32::from(got[k]));
        let (x, last_row) = (i % w, i + w >= rgba.len());
        let mut push = |j: usize, f: i32| {
            for k in 0..3 {
                err[j][k] += e[k] * f;
            }
        };
        if x + 1 < w {
            push(i + 1, 7);
        }
        if !last_row {
            if x > 0 {
                push(i + w - 1, 3);
            }
            push(i + w, 5);
            if x + 1 < w {
                push(i + w + 1, 1);
            }
        }
    }
    out
}

/// A procura da cor mais próxima, com memória por cor (o resultado é o mesmo sem ela).
struct Mapper<'a> {
    palette: &'a [[u8; 4]],
    transparent: u8,
    /// Os índices que uma célula opaca pode receber.
    candidates: Vec<u8>,
    /// `(cor + 1, índice)`; 0 = vazio (a cor `u32::MAX` é o branco opaco, não um vazio).
    memo: Vec<(u64, u8)>,
}

const MEMO_BITS: u32 = 14;

impl<'a> Mapper<'a> {
    fn new(palette: &'a [[u8; 4]], transparent: u8, background: bool) -> Self {
        let candidates = (0..palette.len() as u16)
            .map(|i| i as u8)
            .filter(|&i| background || i != transparent)
            .collect::<Vec<_>>();
        let candidates = if candidates.is_empty() {
            vec![transparent]
        } else {
            candidates
        };
        Self {
            palette,
            transparent,
            candidates,
            memo: vec![(0, 0); 1 << MEMO_BITS],
        }
    }

    fn dist(a: [u8; 4], b: [u8; 4]) -> u32 {
        (0..4)
            .map(|k| {
                let d = i32::from(a[k]) - i32::from(b[k]);
                (d * d) as u32
            })
            .sum()
    }

    /// As duas mais próximas (a segunda = a melhor das outras).
    fn two_nearest(&self, c: [u8; 4]) -> (u8, u8) {
        let mut best = (u32::MAX, self.candidates[0]);
        let mut second = (u32::MAX, self.candidates[0]);
        for &i in &self.candidates {
            let d = Self::dist(c, self.palette[usize::from(i)]);
            if d < best.0 {
                second = best;
                best = (d, i);
            } else if d < second.0 {
                second = (d, i);
            }
        }
        (best.1, second.1)
    }

    fn index(&mut self, c: [u8; 4]) -> u8 {
        if c[3] == 0 && !self.candidates.contains(&self.transparent) {
            return self.transparent;
        }
        let key = u32::from_le_bytes(c);
        let slot = (key.wrapping_mul(0x9e37_79b1) >> (32 - MEMO_BITS)) as usize;
        if self.memo[slot].0 == u64::from(key) + 1 {
            return self.memo[slot].1;
        }
        let i = self.two_nearest(c).0;
        self.memo[slot] = (u64::from(key) + 1, i);
        i
    }

    /// Ordenado: `c + (limiar/n² − ½)·S` por canal, `S = 255 / (cores − 1)` (o passo médio da
    /// paleta), e a mais próxima.
    fn ordered(&mut self, c: [u8; 4], b: Bayer, x: i32, y: i32) -> u8 {
        if c[3] == 0 {
            return self.index(c);
        }
        let n2 = (b.side() * b.side()) as i32;
        let steps = (self.candidates.len() as i32 - 1).max(1);
        // (2t + 1 − n²) / (2n²) · 255 / steps, em inteiros.
        let t = b.at(x, y) as i32;
        let off = ((2 * t + 1 - n2) * 255).div_euclid(2 * n2 * steps);
        let mut d = c;
        for k in 0..3 {
            d[k] = (i32::from(c[k]) + off).clamp(0, 255) as u8;
        }
        self.index(d)
    }

    /// «Old»: entre a mais próxima `a` e a segunda `b`, a matriz escolhe `b` na fração do caminho
    /// que `c` andou de `a` para `b`.
    fn old(&mut self, c: [u8; 4], m: Bayer, x: i32, y: i32) -> u8 {
        if c[3] == 0 {
            return self.index(c);
        }
        let (a, b) = self.two_nearest(c);
        let (pa, pb) = (self.palette[usize::from(a)], self.palette[usize::from(b)]);
        let ab = Self::dist(pa, pb).max(1) as u64;
        // Projecção de c−a sobre b−a, em fracção de |b−a|², × n².
        let dot: i64 = (0..3)
            .map(|k| {
                (i64::from(c[k]) - i64::from(pa[k])) * (i64::from(pb[k]) - i64::from(pa[k]))
            })
            .sum();
        let n2 = (m.side() * m.side()) as u64;
        let frac = (dot.max(0) as u64 * n2 / ab).min(n2);
        if u64::from(m.at(x, y)) < frac { b } else { a }
    }
}
