//! **As ferramentas Blur e Jumble** (tecla R alterna). Cada carimbo lê o estado deixado pelo
//! anterior (o 2.º clique lê o 1.º) e, dentro do carimbo, o estado de ANTES dele. Trial:
//! `oracle_trial_blur.json`.
//! - Blur: `⌊média 3×3⌋` por canal.
//! - Jumble: a cor de um vizinho 3×3 sorteado — só cores que já existem; a semente é do gesto.

use super::{Edge, Source};
use crate::brush::SplitMix32;
use crate::raster::Cell;
use crate::sprite::{CelKey, Sprite};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    #[default]
    Blur,
    Jumble,
    /// Empurra: cada célula do carimbo recebe a cor da célula ANTERIOR do traço (o mesmo desvio que
    /// o cursor andou), misturada em RGBA com a força `forca / 255`; em Indexed volta à paleta.
    Push {
        forca: u8,
    },
}

/// O gesto: a imagem de trabalho do cel e as células que já mudaram.
pub struct Smudge {
    kind: Kind,
    img: Source,
    touched: Vec<bool>,
    rng: SplitMix32,
    /// O centro do carimbo anterior (o Push lê o desvio a partir dele).
    last: Option<Cell>,
}

impl Smudge {
    #[must_use]
    pub fn new(s: &Sprite, key: CelKey, kind: Kind, seed: u32) -> Self {
        let img = Source::of(s, key);
        let touched = vec![false; img.width() as usize * img.height() as usize];
        Self {
            kind,
            img,
            touched,
            rng: SplitMix32::new(seed),
            last: None,
        }
    }

    /// Um carimbo sobre `cells` (as do pincel, já cortadas à selecção pelo chamador).
    pub fn dab(&mut self, cells: &[Cell]) {
        let before = &self.img;
        let new: Vec<(usize, [u8; 4])> = cells
            .iter()
            .filter_map(|&(x, y)| {
                before.get(x, y)?;
                let c = match self.kind {
                    Kind::Blur => {
                        let mut sum = [0u32; 4];
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                let p = before.at_edge(x + dx, y + dy, Edge::Clamp);
                                for k in 0..4 {
                                    sum[k] += u32::from(p[k]);
                                }
                            }
                        }
                        sum.map(|v| (v / 9) as u8)
                    }
                    Kind::Jumble => {
                        let r = self.rng.next_u32() % 9;
                        let (dx, dy) = ((r % 3) as i32 - 1, (r / 3) as i32 - 1);
                        before.at_edge(x + dx, y + dy, Edge::Clamp)
                    }
                    Kind::Push { .. } => return None,
                };
                Some((y as usize * before.width() as usize + x as usize, c))
            })
            .collect();
        for (i, c) in new {
            self.img.set_index(i, c);
            self.touched[i] = true;
        }
    }

    /// Um carimbo centrado em `at`: o Push arrasta do carimbo anterior; Blur e Jumble são [`Self::dab`].
    pub fn arrasta(&mut self, s: &Sprite, at: Cell, cells: &[Cell]) {
        let Kind::Push { forca } = self.kind else {
            self.dab(cells);
            return;
        };
        let Some(prev) = self.last.replace(at) else {
            return;
        };
        let (dx, dy) = (at.0 - prev.0, at.1 - prev.1);
        let f = u32::from(forca);
        let before = &self.img;
        let new: Vec<(usize, [u8; 4])> = cells
            .iter()
            .filter_map(|&(x, y)| {
                let cur = before.get(x, y)?;
                let from = before.at_edge(x - dx, y - dy, Edge::Clamp);
                let mut c = [0u8; 4];
                for k in 0..4 {
                    c[k] = ((u32::from(from[k]) * f + u32::from(cur[k]) * (255 - f) + 127) / 255)
                        as u8;
                }
                let c = s.rgba_of_value(s.value_of_rgba(c));
                Some((y as usize * before.width() as usize + x as usize, c))
            })
            .collect();
        for (i, c) in new {
            if self.img.get(
                (i % self.img.width() as usize) as i32,
                (i / self.img.width() as usize) as i32,
            ) != Some(c)
            {
                self.touched[i] = true;
            }
            self.img.set_index(i, c);
        }
    }

    /// A cor de trabalho da célula (a pré-visualização).
    #[must_use]
    pub fn color_at(&self, x: i32, y: i32) -> Option<[u8; 4]> {
        self.img.get(x, y)
    }

    /// Os valores que o gesto escreve (no modo do sprite) — para [`crate::Stroke::with_values`].
    #[must_use]
    pub fn values(&self, s: &Sprite) -> Vec<(Cell, u32)> {
        let w = self.img.width() as usize;
        self.touched
            .iter()
            .enumerate()
            .filter(|&(_, &t)| t)
            .map(|(i, _)| {
                let (x, y) = ((i % w) as i32, (i / w) as i32);
                let c = self.img.get(x, y).unwrap_or_default();
                ((x, y), s.value_of_rgba(c))
            })
            .collect()
    }
}
