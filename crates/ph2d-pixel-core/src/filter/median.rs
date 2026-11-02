//! **Edit ▸ FX ▸ Despeckle** (diálogo «Median Blur»): mediana por canal da janela `w × h`
//! centrada — o elemento `len/2` da lista ordenada. Trial: `oracle_trial_despeckle.json`.

use super::{Edge, Filter, Source};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Median {
    pub w: u32,
    pub h: u32,
    pub edge: Edge,
}

impl Filter for Median {
    fn at(&self, src: &Source, x: i32, y: i32) -> [u8; 4] {
        let (rx, ry) = ((self.w.max(1) / 2) as i32, (self.h.max(1) / 2) as i32);
        let (cw, ch) = (self.w.max(1) as i32, self.h.max(1) as i32);
        let mut win: Vec<[u8; 4]> = Vec::with_capacity((cw * ch) as usize);
        for dy in 0..ch {
            for dx in 0..cw {
                win.push(src.at_edge(x + dx - rx, y + dy - ry, self.edge));
            }
        }
        let mid = win.len() / 2;
        std::array::from_fn(|k| {
            let mut c: Vec<u8> = win.iter().map(|p| p[k]).collect();
            *c.select_nth_unstable(mid).1
        })
    }
}
