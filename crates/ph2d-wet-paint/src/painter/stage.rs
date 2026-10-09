//! **As portas do passo RETOMÁVEL** (filho de [`super`] — teto de LOC): drenar um passo em voo e
//! rodar UM estágio.
//!
//! Um passo custa **38,7 ms** na escala do produto (medido) contra 16,6 de um quadro de 60 Hz, então
//! enquanto ele era ATÔMICO o frame que o continha estourava **por construção**. O maior estágio
//! sozinho custa 10,26 ms e cabe na folga ⇒ [`crate::sim::StepCursor`].

use super::*;

impl Engine {
    /// **Termina um passo em voo** — ver [`Sim::drain_step`]. Toda operação de CANVAS (as ações
    /// abaixo, o histórico, o undo) começa por aqui: o grid entre dois estágios é intermediário.
    pub fn drain_step(&mut self) {
        if !self.sim.step_pending() {
            return;
        }
        let idx = self.active_layer;
        let (sim, tuning) = (&mut self.sim, &self.tuning);
        sim.drain_step(&mut self.layers[idx].grid, tuning);
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
        let done =
            crate::sim::sim_step_stage(&mut self.sim, &mut self.layers[idx].grid, &self.tuning);
        // O dirty é marcado a cada estágio: um estágio JÁ mexeu no grid, e o host composita quando o
        // passo completa — mas a bbox pode ter crescido em qualquer um deles.
        if done.is_some_and(|ran| ran) || had_fluid {
            if let Some((x0, y0, x1, y1)) = pre {
                self.merge_dirty(x0, y0, x1, y1);
            }
            let g = &self.layers[idx].grid;
            if g.has_fluid {
                let (x0, y0, x1, y1) = (g.bx0, g.by0, g.bx1, g.by1);
                self.merge_dirty(x0, y0, x1, y1);
            }
        }
        done
    }
}
