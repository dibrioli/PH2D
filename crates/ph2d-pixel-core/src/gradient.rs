//! **A ferramenta Gradient** (sub-ferramenta do Paint Bucket, Shift+G; P8_plano §2.1): a região que
//! o balde pintaria a partir de onde o arrasto começa, colorida de fg (t = 0) a bg (t = 1).
//! Trial: `oracle_trial_gradient.json`.
//!
//! - Linear: `t` = projecção do centro da célula sobre o arrasto `a → b`.
//! - Radial: elipse inscrita na caixa do arrasto (centro = ponto médio, raios = metade dos lados);
//!   um raio nulo põe tudo em bg.
//! - Solid: `v = ⌊255·t⌋` mistura as cores. Com matriz, a célula é bg quando `t·(n²+1) ≥ M+1` —
//!   em `t`, não em `v` (em `v` erram 10 células do Bayer 8); a matriz de [`Dither`] presa ao
//!   canvas, com estado próprio da ferramenta (não o da dinâmica).

use crate::dither::Dither;
use crate::fill::{Bucket, bucket_cells};
use crate::raster::Cell;
use crate::sprite::{CelKey, Sprite};

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

/// `⌊255·t⌋` — o nível da mistura Solid.
#[must_use]
pub fn level(t: f64) -> u8 {
    (255.0 * t.clamp(0.0, 1.0)).floor() as u8
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
            return mix(self.fg, self.bg, level(t));
        }
        if dithered_to_bg(self.dither, p.0, p.1, t) {
            self.bg
        } else {
            self.fg
        }
    }

    /// Os valores (no modo do sprite) que o arrasto escreve no cel `key`: a região do balde a
    /// partir de `from`, dentro da selecção.
    #[must_use]
    pub fn values(&self, s: &Sprite, key: CelKey, bucket: Bucket) -> Vec<(Cell, u32)> {
        bucket_cells(s, key, self.from, bucket)
            .into_iter()
            .filter(|&(x, y)| s.in_selection(x, y))
            .map(|p| (p, s.value_of_rgba(self.color_at(p))))
            .collect()
    }
}

/// fg → bg ao nível `v` por canal: `a + ⌊(b − a)·v/255⌋` (com fg/bg 0 → 255 dá `v`, o medido).
fn mix(a: [u8; 4], b: [u8; 4], v: u8) -> [u8; 4] {
    std::array::from_fn(|k| {
        let d = i32::from(b[k]) - i32::from(a[k]);
        (i32::from(a[k]) + (d * i32::from(v)).div_euclid(255)) as u8
    })
}
