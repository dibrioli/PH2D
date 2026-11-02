//! **A ferramenta Gradient** (sub-ferramenta do Paint Bucket, Shift+G; P8_plano §2.1): a [`region`]
//! — a selecção inteira, ou o cel inteiro sem selecção — colorida de fg (t = 0) a bg (t = 1). Não usa
//! a contiguidade do balde: um gradiente SOBREPÕE-SE ao anterior (ordem do dono, smoke P8 02/11; o
//! Trial mediu sobre canvas branco, onde as duas leis coincidem). Trial: `oracle_trial_gradient.json`.
//!
//! - Linear: `t` = projecção do centro da célula sobre o arrasto `a → b`.
//! - Radial: elipse inscrita na caixa do arrasto (centro = ponto médio, raios = metade dos lados);
//!   um raio nulo põe tudo em bg.
//! - Solid: `⌊a + (b − a)·t⌋` por canal. Com matriz, a célula é bg quando `t·(n²+1) ≥ M+1` —
//!   em `t`, não em `v` (em `v` erram 10 células do Bayer 8); a matriz de [`Dither`] presa ao
//!   canvas, com estado próprio da ferramenta (não o da dinâmica).

use crate::dither::Dither;
use crate::raster::Cell;
use crate::sprite::Sprite;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shape {
    #[default]
    Linear,
    Radial,
}

/// O gradiente de um arrasto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gradient {
    pub shape: Shape,
    pub from: Cell,
    pub to: Cell,
    pub fg: [u8; 4],
    pub bg: [u8; 4],
    pub dither: Dither,
}

/// A fracção do caminho na célula `p`, presa a `[0, 1]`.
#[must_use]
pub fn t_at(shape: Shape, a: Cell, b: Cell, p: Cell) -> f64 {
    let (ax, ay, bx, by) = (
        f64::from(a.0),
        f64::from(a.1),
        f64::from(b.0),
        f64::from(b.1),
    );
    let (px, py) = (f64::from(p.0), f64::from(p.1));
    let t = match shape {
        Shape::Linear => {
            let (dx, dy) = (bx - ax, by - ay);
            let len2 = dx * dx + dy * dy;
            if len2 == 0.0 {
                return 1.0;
            }
            ((px - ax) * dx + (py - ay) * dy) / len2
        }
        Shape::Radial => {
            let (rx, ry) = ((bx - ax).abs() / 2.0, (by - ay).abs() / 2.0);
            if rx == 0.0 || ry == 0.0 {
                return 1.0;
            }
            let (cx, cy) = ((ax + bx) / 2.0, (ay + by) / 2.0);
            ((px - cx) / rx).hypot((py - cy) / ry)
        }
    };
    t.clamp(0.0, 1.0)
}

/// A célula `(x, y)` na fracção `t` cai em bg pela matriz `d`.
#[must_use]
pub fn dithered_to_bg(d: Dither, x: i32, y: i32, t: f64) -> bool {
    let n2 = d.side() * d.side();
    t * f64::from(n2 + 1) >= f64::from(d.level(x, y) + 1)
}

impl Gradient {
    /// A cor da célula `p`.
    #[must_use]
    pub fn color_at(&self, p: Cell) -> [u8; 4] {
        let t = t_at(self.shape, self.from, self.to, p);
        if self.dither == Dither::None {
            return mix(self.fg, self.bg, t);
        }
        if dithered_to_bg(self.dither, p.0, p.1, t) {
            self.bg
        } else {
            self.fg
        }
    }

    /// Os valores (no modo do sprite) que o arrasto escreve: a [`region`] inteira.
    #[must_use]
    pub fn values(&self, s: &Sprite) -> Vec<(Cell, u32)> {
        self.paint(s, |p| self.color_at(p))
    }

    /// A cor da célula `p` caminhando pela `ramp` (opção «Palette ramp», spec §13.3 do MVP): `t`
    /// cai entre os degraus `k = ⌊t·(n−1)⌋` e `k+1`, e a fracção entre eles escolhe o degrau pela
    /// matriz (sem matriz: o mais próximo). As pontas ficam cravadas em `ramp[0]`/`ramp[n−1]`, e
    /// toda célula é uma cor da rampa.
    #[must_use]
    pub fn ramp_color_at(&self, ramp: &[[u8; 4]], p: Cell) -> [u8; 4] {
        let n = ramp.len();
        if n < 2 {
            return ramp.first().copied().unwrap_or(self.fg);
        }
        let f = t_at(self.shape, self.from, self.to, p) * (n - 1) as f64;
        let k = (f.floor() as usize).min(n - 2);
        let frac = f - k as f64;
        let up = if self.dither == Dither::None {
            frac >= 0.5
        } else {
            dithered_to_bg(self.dither, p.0, p.1, frac)
        };
        ramp[k + usize::from(up)]
    }

    /// [`Self::values`] pela `ramp`.
    #[must_use]
    pub fn ramp_values(&self, s: &Sprite, ramp: &[[u8; 4]]) -> Vec<(Cell, u32)> {
        self.paint(s, |p| self.ramp_color_at(ramp, p))
    }

    fn paint(&self, s: &Sprite, color: impl Fn(Cell) -> [u8; 4]) -> Vec<(Cell, u32)> {
        region(s)
            .into_iter()
            .map(|p| (p, s.value_of_rgba(color(p))))
            .collect()
    }
}

/// A região do Gradient: as células do sprite cobertas pela selecção, ou todas sem selecção
/// (o cel inteiro), em ordem de linhas. Não depende do conteúdo do cel.
#[must_use]
pub fn region(s: &Sprite) -> Vec<Cell> {
    let (w, h) = (
        i32::try_from(s.width()).unwrap_or(i32::MAX),
        i32::try_from(s.height()).unwrap_or(i32::MAX),
    );
    (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| s.in_selection(x, y))
        .collect()
}

/// A rampa «Palette ramp»: as entradas da paleta de `from` a `to`, na ordem da paleta (para trás
/// se `to < from`), pontas incluídas.
#[must_use]
pub fn palette_ramp(palette: &[[u8; 4]], from: usize, to: usize) -> Vec<[u8; 4]> {
    let ix: Vec<usize> = if from <= to {
        (from..=to).collect()
    } else {
        (to..=from).rev().collect()
    };
    ix.into_iter()
        .filter_map(|i| palette.get(i).copied())
        .collect()
}

/// fg → bg por canal: `⌊a + (b − a)·t⌋` (Trial, #204060 → #c08010: 0 desvios; arredondar deixa 27).
fn mix(a: [u8; 4], b: [u8; 4], t: f64) -> [u8; 4] {
    std::array::from_fn(|k| {
        let (a, b) = (f64::from(a[k]), f64::from(b[k]));
        (a + (b - a) * t).floor().clamp(0.0, 255.0) as u8
    })
}
