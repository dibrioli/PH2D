//! O mapa dos estados dos widgets, com o registo de QUEM mudou (N10 F6d).
//!
//! O `tick_hover` derivava o alvo do hover de TODOS os widgets a cada quadro (~3,9 % do quadro no
//! perfil 4K, `docs/Painter/missao_4k/medicoes/N10_cadencia.md`). O alvo é função só do estado, então
//! basta rever os ids cuja escrita passou por aqui. Toda escrita passa: o mapa é privado e só se lê
//! pelo `Deref`; cada método que devolve `&mut` marca o id (superconjunto — marcar a mais é seguro).

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;

use ph2d_a11y::NodeId;

use super::InteractiveState;

#[derive(Debug, Default)]
pub struct EstadosDosWidgets {
    mapa: BTreeMap<NodeId, InteractiveState>,
    mudados: BTreeSet<NodeId>,
}

impl Deref for EstadosDosWidgets {
    type Target = BTreeMap<NodeId, InteractiveState>;
    fn deref(&self) -> &Self::Target {
        &self.mapa
    }
}

impl EstadosDosWidgets {
    pub fn get_mut(&mut self, id: &NodeId) -> Option<&mut InteractiveState> {
        let st = self.mapa.get_mut(id)?;
        self.mudados.insert(*id);
        Some(st)
    }

    pub fn insert(&mut self, id: NodeId, st: InteractiveState) -> Option<InteractiveState> {
        self.mudados.insert(id);
        self.mapa.insert(id, st)
    }

    pub fn remove(&mut self, id: &NodeId) -> Option<InteractiveState> {
        self.mudados.insert(*id);
        self.mapa.remove(id)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut InteractiveState> {
        self.mudados.extend(self.mapa.keys().copied());
        self.mapa.values_mut()
    }

    pub fn extend(&mut self, pares: impl IntoIterator<Item = (NodeId, InteractiveState)>) {
        for (id, st) in pares {
            self.insert(id, st);
        }
    }

    /// Troca o mapa inteiro (o fecho do ensaio): todo id do novo mapa conta como mudado.
    pub fn substitui(&mut self, mapa: BTreeMap<NodeId, InteractiveState>) {
        self.mudados.extend(mapa.keys().copied());
        self.mapa = mapa;
    }

    /// Os ids escritos desde a última colheita, por ordem; esvazia o registo.
    pub fn colhe_mudados(&mut self) -> BTreeSet<NodeId> {
        std::mem::take(&mut self.mudados)
    }
}
