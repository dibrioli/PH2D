//! **As portas do passo RETOMÁVEL** (filho de [`super`] — teto de LOC): drenar um passo em voo e
//! rodar UM estágio.
//!
//! Um passo custa **38,7 ms** na escala do produto (medido) contra 16,6 de um quadro de 60 Hz, então
//! enquanto ele era ATÔMICO o frame que o continha estourava **por construção**. O maior estágio
//! sozinho custa 10,26 ms e cabe na folga ⇒ [`crate::sim::StepCursor`].

use super::*;
use crate::grid::TILE;

impl Engine {
    pub fn step_simulation(&mut self) -> bool {
        let idx = self.active_layer;
        let had_fluid = self.layers[idx].grid.has_fluid;
        let pre = if had_fluid {
            let g = &self.layers[idx].grid;
            Some((g.bx0, g.by0, g.bx1, g.by1))
        } else {
            None
        };
        self.marca_passo();
        let ran = sim_step(&mut self.sim, &mut self.layers[idx].grid, &self.tuning);
        self.layers[idx].agua_nova = None;
        if ran || had_fluid {
            self.suja_do_passo(pre);
        }
        ran
    }

    /// **Termina um passo em voo** — ver [`Sim::drain_step`]. Toda operação de CANVAS (as ações de tela
    /// inteira, o undo) começa por aqui: o grid entre dois estágios é intermediário. O `capture_history` não:
    /// o registo abre a meio do passo e guarda o cursor, que o undo/redo repõe.
    pub fn drain_step(&mut self) {
        if !self.sim.step_pending() {
            return;
        }
        let idx = self.active_layer;
        self.marca_passo();
        let (sim, tuning) = (&mut self.sim, &self.tuning);
        sim.drain_step(&mut self.layers[idx].grid, tuning);
        self.layers[idx].agua_nova = None;
    }

    /// Roda **UM estágio** do passo corrente; `Some(ran)` quando o passo completou.
    ///
    /// É a porta que o host usa quando o orçamento do frame não cabe um passo inteiro — um passo
    /// custa **38,7 ms** na escala do produto (medido) contra 16,6 de um quadro de 60 Hz, e o maior
    /// estágio sozinho custa 10,26. Ver [`crate::sim::StepCursor`].
    pub fn step_stage(&mut self) -> Option<bool> {
        let idx = self.active_layer;
        let had_fluid = self.layers[idx].grid.has_fluid;
        let pre = if had_fluid {
            let g = &self.layers[idx].grid;
            Some((g.bx0, g.by0, g.bx1, g.by1))
        } else {
            None
        };
        self.marca_passo();
        let done =
            crate::sim::sim_step_stage(&mut self.sim, &mut self.layers[idx].grid, &self.tuning);
        if done.is_some() {
            self.layers[idx].agua_nova = None;
        }
        // O dirty é marcado a cada estágio: um estágio JÁ mexeu no grid, e o host composita quando o
        // passo completa — mas a bbox pode ter crescido em qualquer um deles.
        if done.is_some_and(|ran| ran) || had_fluid {
            self.suja_do_passo(pre);
        }
        done
    }

    /// O sujo de um passo (ou estágio) que correu: a caixa do fluido de antes e a de depois no `dirty` e no sujo da
    /// óptica; no do espelho, B13.1, só os tiles que [`Self::marca_passo`] declarou ([`Self::espelho_tiles`]) — a
    /// caixa de um traço em diagonal é a tela inteira. Sem fluido antes, o `marca_passo` não declarou: a caixa.
    fn suja_do_passo(&mut self, pre: Option<(i32, i32, i32, i32)>) {
        let g = &self.layers[self.active_layer].grid;
        let pos = g.has_fluid.then_some((g.bx0, g.by0, g.bx1, g.by1));
        let declarou = pre.is_some();
        for c in [pre, pos].into_iter().flatten() {
            if declarou {
                self.merge_dirty_do_passo_pela_mascara(c);
            } else {
                self.merge_dirty_do_passo(c.0, c.1, c.2, c.3);
            }
        }
    }

    /// [`Self::merge_dirty_do_passo`] sem a caixa no sujo do espelho: lá vão os tiles declarados.
    fn merge_dirty_do_passo_pela_mascara(&mut self, (x0, y0, x1, y1): (i32, i32, i32, i32)) {
        self.revisao = nova_revisao();
        #[cfg(test)]
        {
            let r = super::espelho::ORLA_DO_ESPELHO;
            self.espelho_passo
                .une((x0 - r, y0 - r, x1 + r, y1 + r), self.width, self.height);
        }
        conta_sujo::regista((i64::from(x1 - x0 + 1).max(0) * i64::from(y1 - y0 + 1).max(0)) as u64);
        self.dirty.une((x0, y0, x1, y1), self.width, self.height);
        self.sujo_da_otica
            .passo
            .une((x0, y0, x1, y1), self.width, self.height);
    }

    /// Junta ao [`Self::espelho_tiles`] os tiles de [`TILE`]² (a grelha do undo) que tocam o retângulo (células finas
    /// com o pad, meio-aberto).
    pub(super) fn declara_tiles_do_passo(&mut self, x0: i64, y0: i64, x1: i64, y1: i64) {
        let g = &self.layers[self.active_layer].grid;
        let cl = |v: i64, n: usize| v.clamp(0, n as i64) as usize;
        let (ax0, ax1, ay0, ay1) = (cl(x0, g.s), cl(x1, g.s), cl(y0, g.rows), cl(y1, g.rows));
        if ax0 >= ax1 || ay0 >= ay1 {
            return;
        }
        let tw = g.s.div_ceil(TILE);
        let n = (tw * g.rows.div_ceil(TILE)).div_ceil(64);
        let m = &mut self.espelho_tiles;
        if m.len() != n {
            m.clear();
            m.resize(n, 0);
        }
        for ty in ay0 / TILE..=(ay1 - 1) / TILE {
            for k in ty * tw + ax0 / TILE..=ty * tw + (ax1 - 1) / TILE {
                m[k / 64] |= 1 << (k % 64);
            }
        }
    }
}
