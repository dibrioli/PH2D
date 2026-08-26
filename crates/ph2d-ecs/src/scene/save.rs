//! `WorldSnapshot` — capture-and-restore primitive for save / replay
//! / rollback (ADR-0025 M14.3d).
//!
//! Distinct from `PrefabDoc` / `SceneDoc`:
//!
//! - **Prefab / Scene** are authored content. They reference other
//!   prefabs by `AssetId`, carry Unity-style overrides, and are
//!   designed to be hand-edited via JSON5 then cooked.
//! - **WorldSnapshot** is a full state capture of every registered
//!   component on every entity in a world. It's the "save game"
//!   primitive (HR-14) and the rollback / replay snapshot the
//!   networking layer will consume (HR-5).
//!
//! Both formats use postcard for the wire layout so a future
//! `Saveable` macro can shore up both pipelines simultaneously.

use crate::StableId;
use crate::scene::registry::{ComponentRegistry, RegistryError};
use crate::transform::{TransformPropagationState, WorklistBuf};
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;
use ph2d_asset::ComponentBlob;
use serde::{Deserialize, Serialize};

/// One row in a [`WorldSnapshot`]: a single entity's full state.
///
/// # v2 (ADR-0164 F1): a linha é chaveada por [`StableId`], não por índice
///
/// Na v1 tanto a ORDEM das linhas quanto o `parent` eram **índices** no vetor. Isso
/// era portável entre mundos (que era o requisito de então), mas tem duas
/// consequências que a wave da instância não pode pagar:
///
/// 1. ⚠️ **Um índice desloca-se.** Inserir UMA entidade empurra o `parent` de todas as
///    linhas seguintes ⇒ os bytes delas mudam ⇒ a captura incremental da **F2** veria
///    o mundo inteiro sujo por causa de um objeto novo. Com `StableId`, a linha de um
///    objeto que não mudou é **byte-idêntica** à da captura anterior, que é a
///    propriedade inteira sobre a qual a F2 é construída.
/// 2. **A ordem tinha de ser imposta depois.** O `canonicalize` do shell reordenava as
///    linhas por CONTEÚDO a cada captura — construindo uma chave de ~230 B **dentro do
///    comparador** do sort (medido: 18,7 ms a 10 k entidades). Ordenar por `StableId`
///    custa 0,088 ms e é a mesma resposta.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySnapshotRow {
    /// A identidade durável desta entidade — **a chave da linha, e a ÚNICA fonte dela
    /// no formato**.
    ///
    /// ⚠️ **O `StableId` NÃO é um componente registado, e a ausência é deliberada.**
    /// Se fosse, ele viajaria duas vezes (aqui e num `ComponentBlob`) e as duas cópias
    /// poderiam discordar. Mas a razão dura é outra, e é a F4: a cópia profunda de uma
    /// sub-árvore nasce sobre `extract_component_snapshot` + `insert_from_bytes`, que
    /// copiam blobs **verbatim** — um `StableId` registado seria copiado com o mesmo
    /// valor, e a cópia nasceria com a identidade do original. O ADR-0164 §2.7 exige
    /// *"remapeado em toda cópia de blobs"*; mantê-lo FORA do registo torna esse erro
    /// impossível de cometer em vez de o deixar por lembrar.
    ///
    /// Quem o instala no restore é o [`snapshot_to_world`], explicitamente.
    pub id: StableId,
    /// Registered components on this entity, in `ComponentRegistry`
    /// id-sorted order (HR-5 determinism).
    pub components: Vec<ComponentBlob>,
    /// O `StableId` do pai (`ChildOf::0`), ou `None` para uma raiz.
    ///
    /// ⚠️ **Um id, não um índice** — ver o item 1 acima. É isto que faz a linha de uma
    /// entidade não mudar quando outra entidade nasce ou morre.
    pub parent: Option<StableId>,
}

/// Um `ComponentBlob`, construído aqui para a captura incremental não precisar de importar o
/// `ph2d_asset` só para o tipo (ADR-0164 F2).
pub(crate) fn blob(type_id: super::registry::ComponentTypeId, data: Vec<u8>) -> ComponentBlob {
    ComponentBlob { type_id, data }
}

/// Full-world snapshot. Versioned (HR-14); the snapshot pipeline is
/// the canonical save format until a richer `Saveable` derive ships.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub version: u32,
    /// Entities in stable DFS order (roots first, then children).
    ///
    /// ⭐ **`Arc` por linha (ADR-0164 F2).** A captura incremental reaproveita a linha de quem
    /// não mudou, e a pilha de undo PARTILHA-A entre passos — é isso que faz um passo custar o
    /// tamanho da *edição* e não o do *mundo* (medido: clone de 10 k linhas **0,038 ms** contra
    /// 0,776 ms; pilha de 256 passos **~12,5 MB** contra ~614 MB).
    ///
    /// ⚠️ **A partilha NÃO viaja no fio.** A feature `rc` da serde serializa um `Arc<T>` como o
    /// próprio `T`, então os bytes são **idênticos** aos da v2 sem `Arc` — o `PROJECT_SCHEMA`
    /// não se mexe e nenhum ficheiro gravado muda de significado. E o `PartialEq` compara o
    /// CONTEÚDO (o `Arc` delega), não o ponteiro: dois snapshots iguais continuam iguais mesmo
    /// que um tenha sido construído do zero e o outro reaproveitado.
    pub entities: Vec<std::sync::Arc<EntitySnapshotRow>>,
}

impl WorldSnapshot {
    /// Current schema version.
    ///
    /// **v1 → v2** (ADR-0164 F1): as linhas passam a ser chaveadas e ordenadas por
    /// [`StableId`], e o `parent` passa de índice a id. A migração de documentos v1 vive
    /// no `project_load.rs` da shell, junto do degrau do `PROJECT_SCHEMA` — é a primeira
    /// migração da história do repo (HR-14 exigia-a e havia **zero**).
    pub const VERSION: u32 = 2;

    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            entities: Vec::new(),
        }
    }

    /// Stable hash of the snapshot's component bytes. The hash is
    /// blake3 over the postcard encoding — invariant across runs as
    /// long as the encoding is deterministic (which it is per HR-5
    /// + HR-16).
    ///
    /// Used by replay determinism tests (HR-5) and save-load
    /// integrity verification (HR-14 recovery path).
    pub fn state_hash(&self) -> [u8; 32] {
        let bytes = postcard::to_allocvec(self).expect("WorldSnapshot serialize");
        *blake3::hash(&bytes).as_bytes()
    }
}

impl Default for WorldSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum SaveError {
    Registry(RegistryError),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Registry(e) => write!(f, "world_to_snapshot: {e}"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<RegistryError> for SaveError {
    fn from(e: RegistryError) -> Self {
        Self::Registry(e)
    }
}

/// Capture every entity that has at least one registered component.
///
/// **Determinism (HR-5):**
/// - Entities visited via the hierarchy DFS — roots sorted by
///   `Entity::to_bits()`, children in `Children` insertion order.
///   So byte output is invariant given the same spawn sequence.
/// - Components serialized in `ComponentRegistry::iter` order (id-
///   sorted, BTreeMap-backed).
///
/// **Pre-allocated scratch:** caller provides `TransformPropagationState`
/// and `WorklistBuf` so this function makes zero per-call
/// allocations beyond the output `Vec`'s own growth.
pub fn world_to_snapshot(
    world: &mut World,
    state: &mut TransformPropagationState,
    worklist: &mut WorklistBuf,
    registry: &ComponentRegistry,
    out: &mut WorldSnapshot,
) -> Result<(), SaveError> {
    // ⚠️ **A identidade é garantida AQUI, na derivação, e não em cada chamador** (a lei da
    // casa: *invariante na DERIVAÇÃO, não em cada gesto*). Uma entidade sem [`StableId`] não
    // teria chave de linha nem forma de ser referida como pai — e exigir que os 44 sítios de
    // chamada se lembrassem de a semear seria uma pré-condição que apodrece.
    //
    // É idempotente: no app a varredura do passe de quadro já correu, e esta chamada não faz
    // nada. É a rede para os caminhos que constroem um mundo e capturam de imediato.
    crate::assign_missing_stable_ids(world);
    let world: &World = world;

    out.version = WorldSnapshot::VERSION;
    out.entities.clear();
    worklist.clear();

    // Two-pass:
    //   1. Walk the hierarchy via the existing DFS infrastructure
    //      to record (entity, parent_entity) in stable order.
    //   2. Translate Entity → snapshot index, serialize components.

    // Phase 1: traverse — we reuse the WorklistBuf stack but with
    // Transform sentinel so the type stays uniform with
    // propagate_transforms. Parent info is reconstructed via
    // `ChildOf` lookups after the walk.
    for entity in state.roots.iter(world) {
        worklist.stack.push((entity, crate::Transform::IDENTITY));
    }
    worklist.stack.sort_unstable_by_key(|&(e, _)| e.to_bits());

    // Drain in DFS LIFO order, recording the visit sequence.
    let mut visit_order: Vec<Entity> = Vec::with_capacity(worklist.stack.len());
    while let Some((entity, _)) = worklist.stack.pop() {
        // Skip if the entity has no Transform (shouldn't happen
        // given the filter, but defensive).
        if world.get_entity(entity).is_err() {
            continue;
        }
        visit_order.push(entity);
        if let Ok(entity_ref) = world.get_entity(entity)
            && let Some(children) = entity_ref.get::<Children>()
        {
            worklist.children_scratch.clear();
            worklist.children_scratch.extend(children.iter());
            // ⚠️ **A ordem vem do `SiblingOrder`, não da inserção** (ADR-0164 F1) — a mesma
            // porta que a travessia do painel usa (`build_hierarchy_snapshot`). Enquanto a
            // lista `Children` mandava, a ordem era memória de runtime: não entrava no
            // snapshot, logo não era desfazível nem sobrevivia ao load (classe BUGS #15).
            worklist
                .children_scratch
                .sort_by_key(|&c| crate::sibling_key(world, c));
            for c in worklist.children_scratch.iter().rev().copied() {
                worklist.stack.push((c, crate::Transform::IDENTITY));
            }
        }
    }

    // Phase 2: emit rows. A chave de cada uma é o `StableId` da entidade — não é preciso
    // mapa nenhum de índices, porque o `parent` também é um id (v2).
    for entity in &visit_order {
        let mut row = EntitySnapshotRow {
            // O `assign_missing_stable_ids` acima garante que existe. O `NONE` é
            // inalcançável e seria um bug de programa, não estado do utilizador.
            id: crate::stable_id_of(world, *entity).unwrap_or(StableId::NONE),
            components: Vec::new(),
            parent: None,
        };
        for entry in registry.iter() {
            match (entry.serialize)(world, *entity) {
                Ok(Some(bytes)) => {
                    row.components.push(ComponentBlob {
                        type_id: entry.type_id,
                        data: bytes,
                    });
                }
                Ok(None) => {}
                Err(e) => return Err(e.into()),
            }
        }
        // O pai, pelo id dele.
        if let Ok(eref) = world.get_entity(*entity)
            && let Some(co) = eref.get::<ChildOf>()
        {
            row.parent = crate::stable_id_of(world, co.0);
        }
        out.entities.push(std::sync::Arc::new(row));
    }

    // ⭐ **A ordem canónica é o `StableId`, e ela nasce AQUI** — não num passe do shell
    // depois. É isto que apaga o `canonicalize` do `undo.rs`, cuja chave de ordenação era a
    // serialização inteira de cada linha, construída **dentro do comparador** do sort:
    // 18,7 ms a 10 k entidades contra os 0,088 ms desta linha (medido, doc 04 §1.1).
    //
    // ⚠️ **É a MESMA propriedade, obtida mais barato:** o que o `canonicalize` comprava era
    // *"dois estados logicamente iguais dão o mesmo snapshot"*, porque a ordem deixava de
    // depender do `Entity::to_bits()` (id de ALOCAÇÃO, novo a cada respawn do undo). O
    // `StableId` sobrevive ao respawn **por construção**, então ordenar por ele dá a mesma
    // invariância sem ler os bytes.
    out.entities.sort_unstable_by_key(|r| r.id);

    // ⚠️⚠️ **A rede da CLASSE, e ela já apanhou um defeito real.**
    //
    // Uma linha com `StableId::NONE` não é só uma linha sem nome: **todas** as `NONE`
    // colidem, o mapa `id → entidade` do restore colapsa-as numa só, e a hierarquia volta
    // mutilada. Aconteceu: os filhos de uma peça 3D não têm `Transform`, o critério da
    // varredura era `With<Transform>`, e uma peça de 5 nós voltava com 2 — passando em todos
    // os outros gates, porque cada componente individualmente sobrevivia.
    //
    // O `debug_assert` é a forma certa aqui: em release isto é um caminho quente (uma captura
    // por quadro com input), e a condição é um invariante de PROGRAMA — se falhar, o defeito
    // está na varredura, não no documento do utilizador.
    debug_assert!(
        out.entities.iter().all(|r| !r.id.is_none()),
        "world_to_snapshot: {} linha(s) sem StableId. Todas as NONE colidem no mapa do \
         restore e a hierarquia volta mutilada — alargue o criterio de \
         `assign_missing_stable_ids`, nao remende aqui.",
        out.entities.iter().filter(|r| r.id.is_none()).count(),
    );
    Ok(())
}

/// Inverse of [`world_to_snapshot`]: spawn an entity per row, install
/// components, restore parent/child relations. Returns the assigned
/// `Entity` per row (indices match `snapshot.entities`).
///
/// `world` does **not** need to be empty — existing entities are
/// left alone. The caller is responsible for clearing if a full
/// restore is desired.
pub fn snapshot_to_world(
    world: &mut World,
    snapshot: &WorldSnapshot,
    registry: &ComponentRegistry,
) -> Result<Vec<Entity>, SaveError> {
    let mut entities = Vec::with_capacity(snapshot.entities.len());
    // Pass 1: spawn + install components.
    for row in &snapshot.entities {
        let entity = world.spawn_empty().id();
        // ⚠️ **A identidade entra AQUI, do campo da linha** — ela não é um componente
        // registado, então não vem num blob (ver [`EntitySnapshotRow::id`]). É esta linha que
        // faz o objeto voltar do undo **sendo o mesmo objeto**: sem ela, o respawn daria
        // identidade nova a tudo e o binding da timeline e a junta da física perderiam o alvo.
        world.entity_mut(entity).insert(row.id);
        for blob in &row.components {
            let entry = registry.get_by_id(blob.type_id).ok_or(SaveError::Registry(
                RegistryError::UnknownTypeId(blob.type_id),
            ))?;
            (entry.insert_from_bytes)(world, entity, &blob.data).map_err(SaveError::Registry)?;
        }
        entities.push(entity);
    }
    // Pass 2: relations, resolvidas por `StableId` (v2).
    //
    // ⚠️ O mapa é construído das LINHAS e não do mundo: o mundo pode ter entidades
    // pré-existentes (o contrato desta função é *"`world` não precisa de estar vazio"*), e
    // uma delas com o mesmo id resolveria o pai para fora do snapshot restaurado.
    let by_id: std::collections::BTreeMap<StableId, Entity> = snapshot
        .entities
        .iter()
        .zip(entities.iter())
        .map(|(row, &e)| (row.id, e))
        .collect();
    for (i, row) in snapshot.entities.iter().enumerate() {
        if let Some(p) = row.parent {
            // ⚠️ Um pai que não está no snapshot é **ignorado**, e a entidade fica raiz. É o
            // degradado honesto: a alternativa seria pendurá-la em qualquer coisa, e a
            // alternativa oposta (recusar o load inteiro) perderia a cena por causa de uma
            // aresta. O caso só é alcançável por um ficheiro adulterado.
            if let Some(&parent) = by_id.get(&p) {
                world.entity_mut(entities[i]).insert(ChildOf(parent));
            }
        }
    }
    Ok(entities)
}

#[cfg(test)]
#[path = "save_tests.rs"]
mod tests;
