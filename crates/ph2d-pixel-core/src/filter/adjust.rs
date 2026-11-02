//! **Edit ▸ Invert e Edit ▸ Adjustments** — filtros por célula. Trial: `oracle_trial_adjust.json`.

use super::{Filter, Source};

/// **Edit ▸ Invert**: `255 − v` em R, G e B; o alfa fica (Trial: 2 048 células, 0 desvios).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Invert;

impl Filter for Invert {
    fn at(&self, src: &Source, x: i32, y: i32) -> [u8; 4] {
        let c = src.get(x, y).unwrap_or_default();
        [255 - c[0], 255 - c[1], 255 - c[2], c[3]]
    }
}
