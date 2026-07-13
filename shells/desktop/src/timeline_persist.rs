//! **Identidade** dos objetos animados — como uma track reencontra o objeto dela (W4.T6/B5).
//!
//! O ECS não tem id estável: o snapshot é indexado por posição e o restore **respawna** as
//! entidades com bits NOVOS. Então uma binding não pode ser reconectada por bits — nem através
//! de um Ctrl+Z, nem através de um load. O objeto bound é identificado pelo **nome** (único, por
//! `name_unique`, e estável entre sessões), hasheado no `WireId` da timeline.
//!
//! É o mesmo mecanismo em três situações, e é por isso que ele é UM só:
//!
//! - **delete + undo**: o undo global respawna o objeto (bits novos, mesmo `Name`) e a binding
//!   órfã se recola — as rows voltam ([`upkeep`], por-frame).
//! - **load de projeto**: o `TimelineDoc` viaja DENTRO do arquivo de projeto ([`crate::project`],
//!   campo `timeline`) e volta com as bindings **destacadas** ([`install_from_project`]); o
//!   `upkeep` do frame as recola nos objetos que o load acabou de spawnar.
//! - **save**: [`serialize`] carimba o hash do nome em cada binding antes de gravar.
//!
//! **O sidecar MORREU** (era `ph2d_timeline.postcard` + Ctrl+S/Ctrl+O no contexto do painel).
//! Ele existia porque "não havia save de projeto" — e havia: o bloco GLOBAL de Ctrl+S/Ctrl+O em
//! `keyboard.rs` já retornava antes, então o sidecar era **código morto** que ainda dizia o
//! contrário no comentário. Um 2º formato para o mesmo dado é a coisa que o `project.rs` existe
//! para não ser.

use ph2d_ecs::{Entity, Name, World};
use ph2d_timeline::{
    TimelineState, WireId, refresh_and_heal_bindings, resolve_entities, stamp_wire_ids,
};

/// FNV-1a of a name → a stable non-null `WireId`. Names are unique, so this is a
/// stable per-object id; `NULL` is reserved, so a hash that lands on 0 is nudged.
fn wire_id_for_name(name: &str) -> WireId {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = OFFSET;
    for b in name.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(PRIME);
    }
    WireId(if h == 0 { 1 } else { h })
}

/// The bound entity's name-derived wire id, or `NULL` when it has no `Name`
/// (a transient object that would not survive a reload anyway).
fn wire_of(world: &World, entity_bits: u64) -> WireId {
    match world.get::<Name>(Entity::from_bits(entity_bits)) {
        Some(name) => wire_id_for_name(name.as_str()),
        None => WireId::NULL,
    }
}

/// Per-frame identity upkeep (called by `timeline_bridge::run`, after the apply
/// refreshed the `missing` flags): live bindings keep their name-hash stamped,
/// and missing ones try to reconnect to a live entity with the same name. This
/// is what makes a track survive its object — deleting the object hides its
/// rows; the global undo respawns it with FRESH entity bits but the same
/// `Name`, and the binding heals, rows back. Returns how many healed.
///
/// Steady state (nothing missing) touches no allocation (HR-3: the bridge's
/// paused path is gated zero-alloc): the name map is only built when a missing
/// binding exists to resolve.
pub(crate) fn upkeep(timeline: &mut TimelineState, world: &mut World) -> usize {
    let any_missing = timeline.doc.bindings().iter().any(|b| b.missing);
    let by_wire: std::collections::BTreeMap<u64, u64> = if any_missing {
        let mut q = world.query::<(Entity, &Name)>();
        q.iter(world)
            .map(|(e, name)| (wire_id_for_name(name.as_str()).0, e.to_bits()))
            .collect()
    } else {
        std::collections::BTreeMap::new() // alloc-free
    };
    let world = &*world;
    refresh_and_heal_bindings(
        &mut timeline.doc,
        |bits| wire_of(world, bits),
        |w| by_wire.get(&w.0).copied(),
    )
}

/// Carimba o `wire_id` (hash do nome do objeto) em cada binding e serializa o documento —
/// os bytes que o arquivo de projeto carrega no campo `timeline` ([`crate::project`]).
pub(crate) fn serialize(timeline: &mut TimelineState, world: &World) -> Result<Vec<u8>, String> {
    stamp_wire_ids(&mut timeline.doc, |bits| wire_of(world, bits));
    timeline.doc.to_bytes()
}

/// Instala o documento vindo do arquivo de projeto, com **toda binding DESTACADA**
/// (`entity = 0`, `missing`). Devolve quantas bindings ficaram pendentes de recolagem.
///
/// **Por que destacar, e não resolver aqui.** As bindings chegam com os bits de entidade da
/// sessão em que foram SALVAS — e bits são reciclados: nada impede que os bits gravados de um
/// objeto morto sejam, nesta sessão, os bits de um objeto **diferente e vivo**. Aí a track
/// dirigiria em silêncio a pose do objeto errado, sem nunca ser marcada `missing`. Destacar
/// torna isso impossível: o único caminho de volta é o `wire_id` (o nome).
///
/// Quem recola é o [`upkeep`] do frame — a MESMA função que cura o delete+undo. É de propósito:
/// a resolução por nome existe UMA vez, então o load não pode divergir do undo (e o load não
/// precisa do mundo, o que o torna dirigível sem janela — ver os gates em `project_tests`).
///
/// Selection / history / clipboard não são persistidos: nascem limpos em volta do documento
/// carregado, então um undo depois do load não alcança a sessão anterior.
pub(crate) fn install_from_project(
    timeline: &mut TimelineState,
    bytes: &[u8],
) -> Result<usize, String> {
    let doc = ph2d_timeline::TimelineDoc::from_bytes(bytes)?;
    *timeline = TimelineState::new();
    timeline.doc = doc;
    // `entity_of` sempre `None` ⇒ `entity = 0` + `missing` em TODA binding (contrato do
    // `resolve_entities`). O `wire_id` — que é o que importa — vem do arquivo, intacto.
    resolve_entities(&mut timeline.doc, |_| None);
    Ok(timeline.doc.bindings().len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::SimWorld;
    use ph2d_timeline::{PropKind, TimelineIntent as I, apply_intent};

    /// A world with two named sprites; returns their live entity bits by name.
    fn world_with(names: &[&str]) -> (SimWorld, Vec<u64>) {
        let mut sim = SimWorld::new();
        let bits = names
            .iter()
            .map(|n| sim.world_mut().spawn(Name::new(*n)).id().to_bits())
            .collect();
        (sim, bits)
    }

    fn key(timeline: &mut TimelineState, entity: u64) {
        let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
        apply_intent(
            timeline,
            &mut ph,
            I::AddKey {
                entity,
                prop: PropKind::TranslationX,
                t: ph2d_anim::RationalTime::from_seconds(0.0),
                value: ph2d_anim::AnimValue::Float(1.0),
                interp: ph2d_anim::Interp::Linear,
            },
        );
    }

    #[test]
    fn upkeep_reconnects_a_deleted_objects_track_when_it_comes_back_by_name() {
        // The delete → global-undo cycle: the undo restores the world by
        // RESPAWNING, so "the same object" returns under fresh entity bits with
        // the same Name. The binding must follow it.
        let mut sim = SimWorld::new();
        let old = sim.world_mut().spawn(Name::new("hero")).id();
        let mut timeline = TimelineState::new();
        key(&mut timeline, old.to_bits());

        // Frame upkeep while alive: stamps the name-hash (0 healed).
        assert_eq!(upkeep(&mut timeline, sim.world_mut()), 0);

        // The object is deleted; the apply pass flags the binding missing.
        sim.world_mut().despawn(old);
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut timeline.doc, 0.0);
        assert!(timeline.doc.bindings()[0].missing, "flagged by liveness");
        assert_eq!(
            upkeep(&mut timeline, sim.world_mut()),
            0,
            "no same-name entity yet: stays dormant"
        );

        // The undo respawns it — fresh bits, same name.
        let reborn = sim.world_mut().spawn(Name::new("hero")).id();
        assert_ne!(reborn, old, "a respawn hands out fresh bits");
        assert_eq!(upkeep(&mut timeline, sim.world_mut()), 1, "healed");
        assert_eq!(timeline.doc.bindings()[0].entity, reborn.to_bits());
        assert!(!timeline.doc.bindings()[0].missing);
    }

    /// **A animação atravessa o arquivo de projeto e reencontra os objetos pelo NOME.**
    ///
    /// Sessão 1 salva; a sessão 2 respawna os MESMOS nomes com bits DIFERENTES (é o que o
    /// `apply_project` faz: despawna tudo e re-spawna do snapshot). O `install_from_project`
    /// destaca as bindings e o `upkeep` do frame — a mesma função que cura o delete+undo — as
    /// recola. Nenhuma track fica órfã, e nenhuma cola no objeto errado.
    #[test]
    fn the_animation_crosses_the_project_file_and_finds_its_objects_by_name() {
        // Sessão 1: dois sprites nomeados, uma track cada.
        let (save_world, save_bits) = world_with(&["sprite_001", "sprite_002"]);
        let mut timeline = TimelineState::new();
        key(&mut timeline, save_bits[0]);
        key(&mut timeline, save_bits[1]);
        let bytes = serialize(&mut timeline, save_world.world()).unwrap();

        // Sessão 2: os MESMOS nomes, bits NOVOS (as entidades descartadas deslocam o
        // alocador, para que os bits realmente difiram).
        let mut sim2 = SimWorld::new();
        for _ in 0..3 {
            sim2.world_mut().spawn(());
        }
        let load_bits: Vec<u64> = ["sprite_001", "sprite_002"]
            .iter()
            .map(|n| sim2.world_mut().spawn(Name::new(*n)).id().to_bits())
            .collect();
        assert_ne!(save_bits, load_bits, "um respawn dá bits novos");

        let mut loaded = TimelineState::new();
        let pending = install_from_project(&mut loaded, &bytes).unwrap();
        assert_eq!(
            pending, 2,
            "as duas chegam DESTACADAS (nada resolvido ainda)"
        );
        assert!(
            loaded
                .doc
                .bindings()
                .iter()
                .all(|b| b.missing && b.entity == 0),
            "destacada = `entity` zerada: bits de outra sessão nunca podem colar por acidente"
        );

        // O frame seguinte (o `upkeep` do `timeline_bridge`) recola pelo nome.
        assert_eq!(
            upkeep(&mut loaded, sim2.world_mut()),
            2,
            "as duas recolaram"
        );
        for (b, want) in loaded.doc.bindings().iter().zip(&load_bits) {
            assert_eq!(b.entity, *want, "cada binding no objeto DESTA sessão");
            assert!(!b.missing);
        }
    }

    /// A track cujo objeto não está no projeto carregado **sobrevive como `missing`** — não é
    /// descartada em silêncio (o painel a badgeia; o apply a pula).
    #[test]
    fn a_track_whose_object_is_gone_stays_missing_never_dropped() {
        let (save_world, save_bits) = world_with(&["sprite_001", "sprite_002"]);
        let mut timeline = TimelineState::new();
        key(&mut timeline, save_bits[0]);
        key(&mut timeline, save_bits[1]);
        let bytes = serialize(&mut timeline, save_world.world()).unwrap();

        // Sessão 2: só o primeiro sprite voltou.
        let (mut load_world, _) = world_with(&["sprite_001"]);
        let mut loaded = TimelineState::new();
        assert_eq!(install_from_project(&mut loaded, &bytes).unwrap(), 2);
        assert_eq!(
            upkeep(&mut loaded, load_world.world_mut()),
            1,
            "uma recola; a outra continua pendente"
        );
        assert!(!loaded.doc.bindings()[0].missing);
        assert!(
            loaded.doc.bindings()[1].missing,
            "a track do objeto ausente fica visível como missing, não some"
        );
    }

    /// O install zera selection/history — um Ctrl+Z depois do load não alcança a sessão
    /// anterior. (O undo GLOBAL do editor é zerado pelo `project_load_from`; este é o undo
    /// próprio da timeline.)
    #[test]
    fn install_resets_panel_state_so_undo_cannot_cross_sessions() {
        let (world, bits) = world_with(&["sprite_001"]);
        let mut timeline = TimelineState::new();
        key(&mut timeline, bits[0]);
        let bytes = serialize(&mut timeline, world.world()).unwrap();

        let mut loaded = TimelineState::new();
        key(&mut loaded, bits[0]); // uma sessão suja: um passo no histórico
        assert!(loaded.history.can_undo());
        install_from_project(&mut loaded, &bytes).unwrap();
        assert!(!loaded.history.can_undo(), "histórico zerado no load");
        assert!(loaded.selection.is_empty(), "seleção zerada no load");
    }

    /// Bytes de um `DOC_VERSION` que este binário não lê são **recusados** — não lidos com o
    /// layout novo. (Postcard é posicional: ler seria pior que recusar.)
    #[test]
    fn a_document_from_another_era_is_refused_not_misread() {
        let mut loaded = TimelineState::new();
        assert!(install_from_project(&mut loaded, &[0xff, 0xff, 0xff]).is_err());
        assert!(
            loaded.doc.bindings().is_empty(),
            "e o estado atual não é corrompido pela tentativa"
        );
    }
}
