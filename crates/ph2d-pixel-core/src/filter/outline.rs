//! **Edit ▸ FX ▸ Outline** (Shift+O). Trial: `oracle_trial_outline.json`, 16 casos, 0 células.
//! Fora: uma célula de FUNDO com algum vizinho da matriz na forma ganha a cor. Dentro: uma célula
//! da FORMA com algum vizinho da matriz no fundo ganha a cor. Lê o original (não propaga).

use super::{Edge, Filter, Source};

/// O que é fundo: o transparente (*Mask*, alfa 0) ou uma cor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Background {
    #[default]
    Mask,
    Color([u8; 4]),
}

impl Background {
    fn is(self, c: [u8; 4]) -> bool {
        match self {
            Self::Mask => c[3] == 0,
            Self::Color(b) => c == b,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Place {
    #[default]
    Outside,
    Inside,
}

/// A matriz 3×3 dos vizinhos (`[linha][coluna]`, o centro não conta) e os presets do diálogo.
pub type Matrix = [[bool; 3]; 3];
pub const CIRCLE: Matrix = [
    [false, true, false],
    [true, false, true],
    [false, true, false],
];
pub const SQUARE: Matrix = [[true, true, true], [true, false, true], [true, true, true]];
pub const HORIZONTAL: Matrix = [[false; 3], [true, false, true], [false; 3]];
pub const VERTICAL: Matrix = [[false, true, false], [false; 3], [false, true, false]];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outline {
    pub color: [u8; 4],
    pub background: Background,
    pub matrix: Matrix,
    pub place: Place,
    /// *Tiled*: a vizinhança dá a volta; desligado, fora da imagem não há vizinho.
    pub tiled: bool,
}

impl Filter for Outline {
    fn at(&self, src: &Source, x: i32, y: i32) -> [u8; 4] {
        let c = src.get(x, y).unwrap_or_default();
        let here_bg = self.background.is(c);
        if here_bg != (self.place == Place::Outside) {
            return c;
        }
        let hit = (0..3).any(|r| {
            (0..3).any(|k| {
                if !self.matrix[r][k] || (r, k) == (1, 1) {
                    return false;
                }
                let (nx, ny) = (x + k as i32 - 1, y + r as i32 - 1);
                let n = if self.tiled {
                    Some(src.at_edge(nx, ny, Edge::Wrap))
                } else {
                    src.get(nx, ny)
                };
                n.is_some_and(|n| self.background.is(n) != here_bg)
            })
        });
        if hit { self.color } else { c }
    }
}
