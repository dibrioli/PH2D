//! **A forma v1 do [`WorldSnapshot`], congelada, e a migração para v2** (ADR-0164 F1).
//!
//! # Por que um tipo CONGELADO, e não «ler com o tipo novo»
//!
//! O postcard é **posicional e não auto-descritivo**: não há nomes de campo no wire. Um
//! `EntitySnapshotRow` v2 tem um `StableId` à frente que a v1 não tinha, e o `parent` mudou de
//! `Option<u32>` para `Option<StableId>` (4 → 8 bytes). Ler bytes v1 com o tipo v2 não dá erro
//! — dá **lixo silencioso**, que é a pior das saídas.
//!
//! Por isso a v1 vive aqui como um tipo próprio que **nunca mais muda**. Ele existe para uma
//! coisa só: desserializar bytes antigos e entregá-los à [`migrate_v1_to_v2`].
//!
//! ⚠️ **É a primeira migração da história do repo.** A auditoria de 2026-08-21 registou-a como
//! ambiguidade em aberto (§8 item 7): *"HR-14 exige `migrate_vN_to_vN+1`; o repo tem **zero** e
//! recusa arquivos antigos"*. A recusa não era política escrita — era o que havia.
//!
//! ⛔ **Não «arrume» este arquivo.** Um tipo de migração que segue o tipo vivo deixa de ler o
//! que se propôs a ler, e o modo de falha é um projeto antigo que abre com o conteúdo trocado.

use super::save::{EntitySnapshotRow, WorldSnapshot};
use crate::StableId;
use ph2d_asset::ComponentBlob;
use serde::{Deserialize, Serialize};

/// A linha do snapshot **v1** — chaveada por POSIÇÃO no vetor.
///
/// ⛔ Congelada. Ver o cabeçalho do módulo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySnapshotRowV1 {
    pub components: Vec<ComponentBlob>,
    /// **Índice** da linha do pai neste mesmo vetor.
    pub parent: Option<u32>,
}

/// O snapshot **v1**. ⛔ Congelado.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSnapshotV1 {
    pub version: u32,
    pub entities: Vec<EntitySnapshotRowV1>,
}

impl WorldSnapshotV1 {
    /// A versão que este tipo lê. Um snapshot que diga outra coisa não é v1.
    pub const VERSION: u32 = 1;
}

/// **v1 → v2**: dá a cada linha um [`StableId`] e traduz o `parent` de índice para id.
///
/// # A atribuição é DETERMINÍSTICA, e a razão é o formato antigo
///
/// Os ids saem na **ordem das linhas do ficheiro**, começando em [`StableId::FIRST`]. Isso é
/// determinístico porque a v1 já chegava ao disco em ordem canónica: o `canonicalize` do shell
/// ordenava as linhas por CONTEÚDO antes de cada captura (era a função inteira dele). ⇒ Dois
/// utilizadores que abram o MESMO ficheiro v1 obtêm os MESMOS ids, que é o que faz a migração
/// ser reproduzível e o gate de round-trip poder existir.
///
/// ⚠️ **O contador tem de ser semeado com o que isto devolve** (`entities.len() + 1`), senão a
/// primeira entidade criada depois do load reusa um id vivo. Quem carrega faz a semeadura; ver
/// `StableIdCounter::reconcile_at_least`, que também protege por baixo.
///
/// ⚠️ **Um `parent` fora de alcance vira raiz.** Só é alcançável por ficheiro adulterado — a v1
/// escrevia índices que ela própria produzira —, e a alternativa (recusar o load) perderia a
/// cena inteira por causa de uma aresta.
#[must_use]
pub fn migrate_v1_to_v2(old: &WorldSnapshotV1) -> WorldSnapshot {
    let id_of = |index: u32| -> Option<StableId> {
        (usize::try_from(index).ok()? < old.entities.len())
            .then(|| StableId(u64::from(index) + StableId::FIRST))
    };
    WorldSnapshot {
        version: WorldSnapshot::VERSION,
        entities: old
            .entities
            .iter()
            .enumerate()
            .map(|(i, row)| EntitySnapshotRow {
                id: StableId(i as u64 + StableId::FIRST),
                components: row.components.clone(),
                parent: row.parent.and_then(id_of),
            })
            .collect(),
    }
}

/// O primeiro id livre depois de uma migração — o valor com que semear o contador.
#[must_use]
pub fn next_free_after_migration(old: &WorldSnapshotV1) -> u64 {
    old.entities.len() as u64 + StableId::FIRST
}

#[cfg(test)]
#[path = "save_v1_tests.rs"]
mod tests;
