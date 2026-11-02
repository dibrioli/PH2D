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
}

/// O gesto: a imagem de trabalho do cel e as células que já mudaram.
pub struct Smudge {
    kind: Kind,
    img: Source,
    touched: Vec<bool>,
    rng: SplitMix32,
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
                };
                Some((y as usize * before.width() as usize + x as usize, c))
            })
            .collect();
        for (i, c) in new {
            self.img.set_index(i, c);
            self.touched[i] = true;
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
