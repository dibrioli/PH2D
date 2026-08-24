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

/// Full-world snapshot. Versioned (HR-14); the snapshot pipeline is
/// the canonical save format until a richer `Saveable` derive ships.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub version: u32,
    /// Entities in stable DFS order (roots first, then children).
    pub entities: Vec<EntitySnapshotRow>,
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
        out.entities.push(row);
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
mod tests {
    use super::*;
    use crate::SimWorld;
    use crate::scene::register_ecs_components;
    use crate::{Name, Transform};
    use ph2d_core::Vec2;

    /// ⭐ **A propriedade que o `canonicalize` comprava, provada sem ele** (ADR-0164 F1).
    ///
    /// A lei: *dois estados logicamente iguais dão o MESMO snapshot* — e o caso duro é o
    /// **restore**, que despawna tudo e re-spawna com `Entity` novos. Enquanto a ordem das
    /// linhas vinha do `to_bits()` (id de ALOCAÇÃO), esse respawn mudava os bytes, e o diff
    /// do undo registava um passo espúrio a cada quadro com input — o Ctrl+Z parecia *"não
    /// fazer nada"* (Enio, 2026-07-09). O shell curava-o reordenando por CONTEÚDO a cada
    /// captura (18,7 ms a 10 k entidades).
    ///
    /// Na v2 a ordem é o `StableId`, que **sobrevive ao respawn por construção**. Este gate é
    /// o que prova que a cura não se perdeu com a função que a implementava.
    #[test]
    fn the_snapshot_survives_a_respawn_byte_for_byte() {
        let (mut sim, reg) = populated_world();
        let mut prop = TransformPropagationState::new(sim.world_mut());
        let mut worklist = WorklistBuf::default();

        let mut before = WorldSnapshot::new();
        world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut before)
            .expect("captura");

        // O restore do undo: despawna tudo e re-spawna — `Entity` novos, `to_bits` novos.
        let editable: Vec<Entity> = {
            let mut q = sim.world_mut().query::<Entity>();
            q.iter(sim.world()).collect()
        };
        for e in editable {
            let _ = sim.world_mut().despawn(e);
        }
        snapshot_to_world(sim.world_mut(), &before, &reg).expect("restore");

        let mut after = WorldSnapshot::new();
        world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut after)
            .expect("re-captura");

        assert_eq!(
            before.state_hash(),
            after.state_hash(),
            "capturar -> restaurar -> capturar tem de dar o MESMO hash. Se falhar, a ordem \
             das linhas voltou a depender de algo que o respawn muda, e cada quadro com \
             input volta a registar um passo de undo espurio.",
        );
        assert_eq!(before, after, "e byte a byte, nao so o hash");
    }

    /// **O `parent` é um ID, e é isso que faz a linha de um objeto não mudar quando OUTRO
    /// nasce** — a propriedade sobre a qual a captura incremental da F2 é construída.
    ///
    /// Com o `parent` em índice, inserir uma entidade empurrava o índice de todas as linhas
    /// seguintes: os bytes delas mudavam, e um diff por linha veria o mundo inteiro sujo por
    /// causa de um objeto novo.
    #[test]
    fn adding_an_entity_does_not_change_the_other_rows() {
        let (mut sim, reg) = populated_world();
        let mut prop = TransformPropagationState::new(sim.world_mut());
        let mut worklist = WorklistBuf::default();

        let mut before = WorldSnapshot::new();
        world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut before)
            .expect("captura");

        // Uma raiz nova, sem relacao nenhuma com as que ja' existiam.
        sim.world_mut()
            .spawn((Transform::IDENTITY, Name::new("Newcomer")));

        let mut after = WorldSnapshot::new();
        world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut after)
            .expect("re-captura");

        for old in &before.entities {
            let same = after
                .entities
                .iter()
                .find(|r| r.id == old.id)
                .expect("a linha antiga continua la");
            assert_eq!(
                same, old,
                "a linha de {:?} mudou por causa de um objeto NOVO — o `parent` voltou a ser \
                 um indice, e a captura incremental da F2 veria o mundo inteiro sujo.",
                old.id,
            );
        }
    }

    fn populated_world() -> (SimWorld, ComponentRegistry) {
        let mut sim = SimWorld::new();
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        // Build a 3-level hierarchy with names.
        let root = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(10.0, 20.0)),
                Name::new("Root"),
            ))
            .id();
        let mid = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(1.0, 0.0)),
                Name::new("Mid"),
                ChildOf(root),
            ))
            .id();
        sim.world_mut().spawn((
            Transform::from_translation(Vec2::new(0.5, 0.5)),
            Name::new("Leaf"),
            ChildOf(mid),
        ));
        (sim, reg)
    }

    #[test]
    fn snapshot_captures_all_entities() {
        let (mut sim, reg) = populated_world();
        let mut state = TransformPropagationState::new(sim.world_mut());
        let mut worklist = WorklistBuf::new();
        let mut snap = WorldSnapshot::new();
        world_to_snapshot(sim.world_mut(), &mut state, &mut worklist, &reg, &mut snap).unwrap();
        assert_eq!(snap.entities.len(), 3);
    }

    /// ⚠️ **A §5 e a §12 nasceram registadas e SEM prova de que sobrevivem ao disco** (achado da
    /// auditoria de fecho do 9-slice, 2026-08-22).
    ///
    /// Estar no `ComponentRegistry` faz o componente ser **escrito**; nada disso prova que ele
    /// volta igual. E a forma do `SliceNine` mudou três vezes num dia — cada uma dessas mudanças
    /// atravessou este caminho sem um teste a olhar.
    ///
    /// ⚠️ **O fixture usa valores NÃO-DEFAULT em cada campo**, e é isso que o torna uma prova: um
    /// `SliceNine::INERT` gravado e relido daria igual mesmo que o restore devolvesse o default,
    /// e a mesma lei vale para a âncora (nome, pose, bounds e centro todos diferentes de zero).
    #[test]
    fn the_sprite_authoring_components_survive_the_disk() {
        use crate::{
            NamedAnchor, NamedAnchorList, SliceDrawMode, SliceNine, SliceTileMode, TileRegionMode,
        };

        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);

        let slice = SliceNine {
            draw_mode: SliceDrawMode::Sliced,
            borders: [3.0, 5.0, 7.0, 11.0],
            size: [1.5, 2.5],
            tile_modes: [
                TileRegionMode::Stretch,
                TileRegionMode::Repeat,
                TileRegionMode::Blank,
                TileRegionMode::Mirror,
                TileRegionMode::Repeat,
                TileRegionMode::Stretch,
                TileRegionMode::Mirror,
                TileRegionMode::Blank,
            ],
            centre_tile_mode: TileRegionMode::Mirror,
            tile_mode: SliceTileMode::Whole,
            fill_center: false,
        };
        let mut anchors = NamedAnchorList::default();
        let mut a = NamedAnchor::socket("muzzle");
        a.transform.translation = ph2d_core::Vec2::new(0.25, -0.75);
        a.set_bounds(Some([1.0, 2.0, 3.0, 4.0]));
        a.set_center(Some([0.5, 0.5, 0.5, 0.5]));
        anchors.insert(a).expect("cabe");

        let mut sim_a = SimWorld::new();
        sim_a
            .world_mut()
            .spawn((Transform::default(), slice, anchors.clone()));
        let mut state = TransformPropagationState::new(sim_a.world_mut());
        let mut worklist = WorklistBuf::new();
        let mut snap = WorldSnapshot::new();
        world_to_snapshot(
            sim_a.world_mut(),
            &mut state,
            &mut worklist,
            &reg,
            &mut snap,
        )
        .unwrap();

        let mut sim_b = SimWorld::new();
        let back = snapshot_to_world(sim_b.world_mut(), &snap, &reg).unwrap();
        let e = *back.first().expect("uma entidade");
        assert_eq!(
            sim_b.world_mut().get::<SliceNine>(e).copied(),
            Some(slice),
            "o 9-slice nao voltou igual do disco"
        );
        assert_eq!(
            sim_b.world_mut().get::<NamedAnchorList>(e),
            Some(&anchors),
            "a lista de ancoras nao voltou igual do disco"
        );
    }

    /// **A montagem tem de sobreviver ao disco COM o pai** (ADR-0072 §2.6).
    ///
    /// ⚠️ Guardar o componente e perder a hierarquia — ou o contrário — deixa o vínculo
    /// pendurado sem que nada avise: a espada reabre no sítio certo, parada, e só se descobre
    /// quando o braço se mexe. Por isso este gate reabre a árvore e **volta a perguntar o estado
    /// da montagem**, em vez de comparar dois blobs.
    #[test]
    fn a_mount_survives_the_disk_together_with_the_parent_that_gives_it_meaning() {
        use crate::{AnchorMount, MountState, NamedAnchor, NamedAnchorList, mount_state_of};

        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);

        let mut anchors = NamedAnchorList::default();
        let mut a = NamedAnchor::socket("hand_r");
        a.transform.translation = ph2d_core::Vec2::new(0.5, 1.25);
        anchors.insert(a).expect("cabe");

        let mut sim_a = SimWorld::new();
        let host = sim_a
            .world_mut()
            .spawn((Transform::default(), anchors, crate::Name::new("hero")))
            .id();
        sim_a.world_mut().spawn((
            Transform::default(),
            crate::ChildOf(host),
            AnchorMount::new("hand_r"),
            crate::Name::new("sword"),
        ));

        let mut state = TransformPropagationState::new(sim_a.world_mut());
        let mut worklist = WorklistBuf::new();
        let mut snap = WorldSnapshot::new();
        world_to_snapshot(
            sim_a.world_mut(),
            &mut state,
            &mut worklist,
            &reg,
            &mut snap,
        )
        .unwrap();

        let mut sim_b = SimWorld::new();
        let back = snapshot_to_world(sim_b.world_mut(), &snap, &reg).unwrap();
        let sword = back
            .iter()
            .copied()
            .find(|&e| {
                sim_b
                    .world_mut()
                    .get::<crate::Name>(e)
                    .is_some_and(|n| n.as_str() == "sword")
            })
            .expect("a espada voltou");
        assert!(
            matches!(mount_state_of(sim_b.world(), sword), MountState::Mounted(_)),
            "a montagem nao resolveu depois de reabrir — o componente ou o pai perdeu-se"
        );
        assert_eq!(
            sim_b.world_mut().get::<AnchorMount>(sword),
            Some(&AnchorMount::new("hand_r"))
        );
    }

    #[test]
    fn snapshot_restore_round_trip_preserves_names_and_hierarchy() {
        let (mut sim_a, reg) = populated_world();
        let mut state = TransformPropagationState::new(sim_a.world_mut());
        let mut worklist = WorklistBuf::new();
        let mut snap = WorldSnapshot::new();
        world_to_snapshot(
            sim_a.world_mut(),
            &mut state,
            &mut worklist,
            &reg,
            &mut snap,
        )
        .unwrap();

        let mut sim_b = SimWorld::new();
        let entities = snapshot_to_world(sim_b.world_mut(), &snap, &reg).unwrap();
        assert_eq!(entities.len(), 3);

        // Verify names + hierarchy in the restored world.
        let names: Vec<String> = entities
            .iter()
            .map(|e| {
                sim_b
                    .world_mut()
                    .get::<Name>(*e)
                    .unwrap()
                    .as_str()
                    .to_owned()
            })
            .collect();
        assert!(names.contains(&"Root".to_string()));
        assert!(names.contains(&"Mid".to_string()));
        assert!(names.contains(&"Leaf".to_string()));

        // The Leaf entity (last in visit order) should have a parent.
        let parents: Vec<Option<Entity>> = entities
            .iter()
            .map(|e| sim_b.world_mut().get::<ChildOf>(*e).map(|c| c.0))
            .collect();
        // Exactly one root (Root) → exactly one None.
        let root_count = parents.iter().filter(|p| p.is_none()).count();
        assert_eq!(root_count, 1);
    }

    #[test]
    fn state_hash_is_deterministic_across_round_trip() {
        let (mut sim_a, reg) = populated_world();
        let mut state = TransformPropagationState::new(sim_a.world_mut());
        let mut worklist = WorklistBuf::new();
        let mut snap_a = WorldSnapshot::new();
        world_to_snapshot(
            sim_a.world_mut(),
            &mut state,
            &mut worklist,
            &reg,
            &mut snap_a,
        )
        .unwrap();
        let hash_a = snap_a.state_hash();

        // Restore to a fresh world, snapshot again — hashes match.
        let mut sim_b = SimWorld::new();
        snapshot_to_world(sim_b.world_mut(), &snap_a, &reg).unwrap();
        let mut state_b = TransformPropagationState::new(sim_b.world_mut());
        let mut snap_b = WorldSnapshot::new();
        world_to_snapshot(
            sim_b.world_mut(),
            &mut state_b,
            &mut worklist,
            &reg,
            &mut snap_b,
        )
        .unwrap();
        let hash_b = snap_b.state_hash();
        assert_eq!(
            hash_a, hash_b,
            "snapshot round-trip produced a different state hash"
        );
    }

    #[test]
    fn snapshot_postcard_round_trips() {
        let (mut sim, reg) = populated_world();
        let mut state = TransformPropagationState::new(sim.world_mut());
        let mut worklist = WorklistBuf::new();
        let mut snap = WorldSnapshot::new();
        world_to_snapshot(sim.world_mut(), &mut state, &mut worklist, &reg, &mut snap).unwrap();
        let bytes = postcard::to_allocvec(&snap).unwrap();
        let decoded: WorldSnapshot = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, snap);
    }
}
