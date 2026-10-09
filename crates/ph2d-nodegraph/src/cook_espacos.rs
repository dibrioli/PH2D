//! ⭐ **O ESPAÇO de cada nó** (doc ObjectsAndComponents 01 §14, item 6 — escolha 4.2 do dono: *«o
//! que vem do mundo é convertido sozinho para o espaço do objecto Motion»*).
//!
//! Um cozido serve vários objectos Motion: os externos são UMA tabela, e o nó não sabe de quem é
//! (ADR-0031, caixa preta). A app diz ao cozido, por quadro, o espaço de cada nó cujo dono não está
//! na origem, e publica as cópias convertidas sob [`crate::external::in_space`];
//! [`super::EvalCtx::external`] faz o resto. Sem dono deslocado o mapa é vazio e todo nó lê o mundo
//! — o caminho de sempre, ao bit.

use std::collections::BTreeMap;

use super::Cook;
use crate::graph::NodeId;

impl Cook {
    /// O espaço de cada nó, por quadro (`NodeId.0` → `StableId` do dono; um nó ausente = mundo).
    pub fn set_spaces(&mut self, spaces: BTreeMap<u32, u64>) {
        self.spaces = spaces;
    }

    /// Esquece as cópias por espaço publicadas (as do quadro anterior podiam ser de um dono que já
    /// voltou à origem ou morreu, e de um nome que já não existe).
    pub fn clear_spaced_externals(&mut self) {
        self.externals
            .retain(|name, _| !crate::external::is_spaced(name));
    }

    pub(super) fn space_of(&self, node: NodeId) -> u64 {
        self.spaces.get(&node.0).copied().unwrap_or(0)
    }
}
