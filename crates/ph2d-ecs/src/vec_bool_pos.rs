//! **ONDE cada círculo do diagrama booleano fica** — a disposição que o artista arruma à mão.
//!
//! Irmão do [`crate::VecBoolEdges`], e o corte é por NATUREZA: as ligações são **semântica** (elas
//! mudam o desenho), a disposição é **cosmética** (ela não muda um pixel da arte). Guardá-las no
//! mesmo componente faria mover um círculo parecer, aos bytes, o mesmo tipo de edição que trocar
//! uma operação — e um documento que nunca abriu o diagrama teria de carregar posições que ninguém
//! escolheu.
//!
//! Ausência = **o diagrama arruma sozinho** (o anel default). É isso que faz abrir a janela pela
//! primeira vez mostrar algo legível sem obrigar ninguém a arrastar nada.
//!
//! # As coordenadas são LOCAIS ao card
//!
//! ⚠️ `(0, 0)` é o canto do conteúdo do card, não da tela. Guardar coordenadas de TELA amarraria a
//! disposição ao sítio onde o card estava aberto — mover a janela reorganizaria o diagrama, e
//! reabri-lo noutra resolução espalharia os círculos para fora.
//!
//! # A ordem é CANÓNICA, como a das ligações
//!
//! ⚠️ Mesma razão do irmão: o undo regista um passo por **DIFF de bytes**, então duas listas com o
//! mesmo significado e ordens diferentes seriam dois estados, e reordená-la viraria um passo de
//! undo que não mudou nada na tela.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Onde o círculo de uma forma foi posto, em coordenadas **locais ao conteúdo do card**.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct VecBoolNodePos {
    /// O `VecPathId` cru — a mesma identidade durável que as ligações usam.
    pub id: u64,
    /// O centro do círculo, em px locais ao card.
    pub at: [f32; 2],
}

/// **A disposição dos círculos** de um grupo booleano. Ausente = o diagrama arruma sozinho.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VecBoolGraphPos {
    /// As posições, ordenadas por id. ⚠️ Escreva por [`VecBoolGraphPos::set`], nunca por `push`.
    pub nodes: Vec<VecBoolNodePos>,
}

impl VecBoolGraphPos {
    /// Onde este círculo foi posto, se alguém o pôs.
    #[must_use]
    pub fn get(&self, id: u64) -> Option<[f32; 2]> {
        self.nodes.iter().find(|n| n.id == id).map(|n| n.at)
    }

    /// Põe (ou move) o círculo desta forma, mantendo a forma canónica.
    pub fn set(&mut self, id: u64, at: [f32; 2]) {
        match self.nodes.iter_mut().find(|n| n.id == id) {
            Some(slot) => slot.at = at,
            None => {
                let at = VecBoolNodePos { id, at };
                let k = self.nodes.partition_point(|n| n.id < id);
                self.nodes.insert(k, at);
            }
        }
    }

    /// Esquece a posição de uma forma apagada.
    pub fn forget(&mut self, id: u64) {
        self.nodes.retain(|n| n.id != id);
    }
}

impl SimComponent for VecBoolGraphPos {}

#[cfg(test)]
#[path = "vec_bool_pos_tests.rs"]
mod tests;
