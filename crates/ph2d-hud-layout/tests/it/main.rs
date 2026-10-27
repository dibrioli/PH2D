//! Os gates do HUD que se arruma (docs/BlenderConstraints/06_W5_resto.md §4, ADR-0194).

mod bit;
mod historia;
mod oraculo;

use ph2d_hud_layout::{Caixa, Elemento};

pub fn caixa(x: f64, y: f64, w: f64, h: f64) -> Caixa {
    Caixa { x, y, w, h }
}

pub fn el(x: f64, y: f64, w: f64, h: f64, conteudo: Option<f64>) -> Elemento {
    Elemento { caixa: caixa(x, y, w, h), conteudo }
}

pub fn bits(caixas: &[Caixa]) -> Vec<[u64; 4]> {
    caixas.iter().map(|c| [c.x.to_bits(), c.y.to_bits(), c.w.to_bits(), c.h.to_bits()]).collect()
}
