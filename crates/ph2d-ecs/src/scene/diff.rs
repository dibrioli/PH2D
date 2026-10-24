//! **O que mudou entre duas fotografias do mundo** — a diferença por LINHA do [`WorldSnapshot`]
//! (a família «mundo» das macros, `docs/Macros/17_W19_plano.md` §2.1).
//!
//! As linhas são chaveadas pelo [`StableId`]; uma linha partilhada pelos dois lados (o mesmo `Arc`,
//! o caso de quem não mudou na captura incremental) salta-se sem comparar bytes.

use super::registry::ComponentTypeId;
use super::save::{EntitySnapshotRow, WorldSnapshot};
use crate::StableId;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Uma linha que mudou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinhaMudada {
    /// Não existia em `antes`.
    Nasce(Arc<EntitySnapshotRow>),
    /// Não existe em `depois`.
    Morre(StableId),
    /// Existe nos dois e difere.
    Muda {
        id: StableId,
        /// `Some(p)` = o pai passou a `p`.
        pai: Option<Option<StableId>>,
        /// Por `type_id` crescente: `Some(bytes)` = posto ou mudado, `None` = tirado.
        componentes: Vec<(ComponentTypeId, Option<Vec<u8>>)>,
    },
}

impl LinhaMudada {
    #[must_use]
    pub fn id(&self) -> StableId {
        match self {
            Self::Nasce(r) => r.id,
            Self::Morre(id) | Self::Muda { id, .. } => *id,
        }
    }
}

/// **A diferença de `antes` para `depois`**, por `StableId` crescente.
#[must_use]
pub fn linhas_que_mudam(antes: &WorldSnapshot, depois: &WorldSnapshot) -> Vec<LinhaMudada> {
    let a = por_id(antes);
    let d = por_id(depois);
    let mut out = Vec::new();
    for (id, ra) in &a {
        match d.get(id) {
            None => out.push(LinhaMudada::Morre(*id)),
            Some(rd) if Arc::ptr_eq(ra, rd) || ra == rd => {}
            Some(rd) => out.push(muda(ra, rd)),
        }
    }
    for (id, rd) in &d {
        if !a.contains_key(id) {
            out.push(LinhaMudada::Nasce(Arc::clone(rd)));
        }
    }
    out.sort_by_key(LinhaMudada::id);
    out
}

fn por_id(s: &WorldSnapshot) -> BTreeMap<StableId, &Arc<EntitySnapshotRow>> {
    s.entities.iter().map(|r| (r.id, r)).collect()
}

fn muda(a: &EntitySnapshotRow, d: &EntitySnapshotRow) -> LinhaMudada {
    let ca: BTreeMap<_, _> = a.components.iter().map(|b| (b.type_id, &b.data)).collect();
    let cd: BTreeMap<_, _> = d.components.iter().map(|b| (b.type_id, &b.data)).collect();
    let mut componentes = Vec::new();
    for (t, bd) in &cd {
        if ca.get(t) != Some(bd) {
            componentes.push((*t, Some((*bd).clone())));
        }
    }
    for t in ca.keys() {
        if !cd.contains_key(t) {
            componentes.push((*t, None));
        }
    }
    componentes.sort_by_key(|c| c.0);
    LinhaMudada::Muda {
        id: d.id,
        pai: (a.parent != d.parent).then_some(d.parent),
        componentes,
    }
}

#[cfg(test)]
#[path = "diff_tests.rs"]
mod tests;
