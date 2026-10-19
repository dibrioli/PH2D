//! A porta do RELEVO dos fios (W3, [`crate::bristles_relevo`]): um dab penteia a altura da camada activa.

use super::Engine;
use crate::par::Rows;

impl Engine {
    /// Penteia [`crate::grid::Grid::altura`] sob o dab de centro `(x, y)` cuja silhueta cabe na caixa de
    /// meia-extensão `meia` (células — a da PEGADA: a lâmina chata do óleo é ⅓ do quadrado do raio): cada
    /// célula vai para `alvo(cx, cy)` (a unidade do campo) pela cobertura `sil(cx, cy)`
    /// ([`crate::bristles_relevo::penteia`]). Por linhas, em série ou em paralelo pelo piso medido
    /// [`crate::par::MIN_CELLS_PENTE`] — a mesma resposta (cada célula só lê e escreve a si própria). `passos` = as
    /// células de caminho que o dab representa ([`crate::bristles_relevo::penteia`]).
    #[allow(clippy::too_many_arguments)]
    pub fn pentear_relevo(
        &mut self,
        x: f64,
        y: f64,
        meia: [f64; 2],
        sil: crate::brush::CellFn<'_>,
        alvo: crate::brush::CellFn<'_>,
        passos: f64,
    ) {
        self.pentear_relevo_rows(x, y, meia, sil, alvo, passos, None);
    }

    /// [`Self::pentear_relevo`] com a rota forçada (`None` = o produto: [`Rows::pick`]) — a sonda do piso
    /// (`tests/it/measure_pente_rows.rs`) e o portão da identidade das duas rotas.
    #[allow(clippy::too_many_arguments)]
    pub fn pentear_relevo_rows(
        &mut self,
        x: f64,
        y: f64,
        meia: [f64; 2],
        sil: crate::brush::CellFn<'_>,
        alvo: crate::brush::CellFn<'_>,
        passos: f64,
        mode: Option<Rows>,
    ) {
        let (ax, ay) = (meia[0].ceil() as i32 + 1, meia[1].ceil() as i32 + 1);
        let g = &mut self.layers[self.active_layer].grid;
        let (cx, cy) = (x.round() as i32, y.round() as i32);
        let (x0, x1) = ((cx - ax).max(1), (cx + ax).min(g.w as i32));
        let (y0, y1) = ((cy - ay).max(1), (cy + ay).min(g.h as i32));
        if x1 < x0 || y1 < y0 {
            return;
        }
        let s = g.s;
        let (linhas, largura) = ((y1 - y0 + 1) as usize, (x1 - x0 + 1) as usize);
        let mode = mode.unwrap_or_else(|| Rows::pick(linhas, largura, crate::par::MIN_CELLS_PENTE));
        let campo = crate::grid::altura::garante(g);
        let faixa = &mut campo[y0 as usize * s..(y1 as usize + 1) * s];
        crate::par::walk_rows(mode, faixa, s, |k, linha| {
            let py = y0 + k as i32;
            for px in x0..=x1 {
                let w = sil(px, py);
                if w <= 0.0 {
                    continue;
                }
                let c = &mut linha[px as usize];
                *c = crate::bristles_relevo::penteia(*c, alvo(px, py), w, passos);
            }
        });
        self.merge_dirty(x0, y0, x1, y1);
    }
}
