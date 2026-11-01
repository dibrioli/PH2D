//! A troca da lei da tinta no motor (E9, `docs/Painter/Arquitetura/E9_portao_mae.md` §5): ao SAIR de
//! `Pigmento` as somas de [`crate::grid::Grid::tinta`] deixam de acompanhar a massa (a lei velha não as move);
//! esvaziam-se, e ao voltar o `garante` migra-as da massa que a lei velha deixou.

use super::Engine;
use crate::sim::LeiDaTinta;

impl Engine {
    /// Muda a lei da tinta; sair de `Pigmento` esvazia `Grid::tinta` em todas as camadas.
    pub fn set_lei_da_tinta(&mut self, lei: LeiDaTinta) {
        if self.sim.tinta == LeiDaTinta::Pigmento && lei != LeiDaTinta::Pigmento {
            for l in &mut self.layers {
                l.grid.tinta = Vec::new();
                l.grid.pigmento = Vec::new();
            }
        }
        self.sim.set_tinta(lei);
    }
}

#[cfg(test)]
#[path = "lei_da_tinta_tests.rs"]
mod tests;
