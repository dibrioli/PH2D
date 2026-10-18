//! A porta do RELEVO dos fios (W3, [`crate::bristles_relevo`]): um dab penteia a altura da camada activa.

use super::Engine;

impl Engine {
    /// Penteia [`crate::grid::Grid::altura`] sob o dab de centro `(x, y)` e raio `r` (células): cada célula
    /// vai para `alvo(cx, cy)` (px) pela cobertura `sil(cx, cy)` da silhueta ([`crate::bristles_relevo::penteia`]).
    /// O alcance é o da silhueta do motor (o do Shape, como o depósito).
    pub fn pentear_relevo(
        &mut self,
        x: f64,
        y: f64,
        r: f64,
        sil: crate::brush::CellFn<'_>,
        alvo: crate::brush::CellFn<'_>,
    ) {
        let alcance = (r * self.silhouette_reach[0].max(1.0)).ceil() as i32 + 1;
        let g = &mut self.layers[self.active_layer].grid;
        let (cx, cy) = (x.round() as i32, y.round() as i32);
        let (x0, x1) = ((cx - alcance).max(1), (cx + alcance).min(g.w as i32));
        let (y0, y1) = ((cy - alcance).max(1), (cy + alcance).min(g.h as i32));
        if x1 < x0 || y1 < y0 {
            return;
        }
        let s = g.s;
        let campo = crate::grid::altura::garante(g);
        for py in y0..=y1 {
            for px in x0..=x1 {
                let w = sil(px, py);
                if w <= 0.0 {
                    continue;
                }
                let i = py as usize * s + px as usize;
                campo[i] = crate::bristles_relevo::penteia(campo[i], alvo(px, py), w);
            }
        }
        self.merge_dirty(x0, y0, x1, y1);
    }
}
