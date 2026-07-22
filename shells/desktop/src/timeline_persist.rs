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

/// A name → the stable `WireId` of the object carrying it.
///
/// The hash itself moved to [`ph2d_ecs::stable_name_id`], beside the `Name` it
/// is derived from, when physics joints (W3) became the second thing in the
/// editor that has to point at an object across a Ctrl+Z and a save. It is the
/// same FNV-1a, byte for byte — pinned there against values computed outside
/// this codebase, because these numbers are already written into every project
/// file on disk. What stays here is the `WireId` wrapper: the timeline's
/// namespace for the id, not a second opinion about how to compute it.
fn wire_id_for_name(name: &str) -> WireId {
    WireId(ph2d_ecs::stable_name_id(name))
}

/// The bound entity's name-derived wire id, or `NULL` when it has no `Name`
/// (a transient object that would not survive a reload anyway).
fn wire_of(world: &World, entity_bits: u64) -> WireId {
    // `try_from_bits`: uma binding DESTACADA carrega `entity = 0`, e `0` não é nulo no bevy —
    // o índice é `NonZero<u32>`, então `from_bits(0)` **entra em pânico**. Salvar um projeto
    // logo depois de abrir outro (bindings ainda pendentes de recolagem) chegaria aqui.
    match Entity::try_from_bits(entity_bits).and_then(|e| world.get::<Name>(e)) {
        Some(name) => wire_id_for_name(name.as_str()),
        None => WireId::NULL,
    }
}

/// Per-frame identity upkeep (called by `timeline_bridge::run`, after the apply
/// refreshed the `missing` flags), in two passes over the SAME name map:
///
/// 1. **Heal** — live bindings keep their name-hash stamped, and missing ones
///    reconnect to a live entity with the same name. This is what recolors a
///    project load (`install_from_project` detaches everything) and what makes
///    a timeline-undo of the purge below recoverable.
/// 2. **Purge** — a binding still missing after the heal, whose object is
///    genuinely GONE (no live entity carries its name), is removed from the
///    document together with its tracks in every clip (Enio, 2026-07-22: *"a
///    timeline precisa ser resetada ao deletar o objeto"*). When the purge
///    removes the LAST animated object, the whole document resets — clips,
///    containers, lanes, loop — because composition authored around an object
///    that no longer exists is exactly the stale state that arrived "totalmente
///    bugada" for the next object created. One timeline-undo step covers it.
///
/// ⚠️ **A ordem heal→purge é load-bearing:** o load de projeto destaca TODA
/// binding (`entity = 0`, `missing`) e conta com o heal DESTE chamado para
/// recolá-las — uma purga que rodasse antes (ou que ignorasse o resultado do
/// heal) apagaria o documento inteiro um frame depois de todo Ctrl+O.
///
/// **Um nome AMBÍGUO não cura NEM purga — recusa.** Se DOIS objetos vivos dividem o
/// nome, "qual deles é o dono desta track?" não tem resposta — curar num deles seria
/// dirigir a pose do objeto errado, e purgar seria destruir trabalho por causa de um
/// empate transitório. A binding fica dormente até o empate acabar.
///
/// Steady state (nothing missing) touches no allocation (HR-3: the bridge's
/// paused path is gated zero-alloc): the name map is only built when a missing
/// binding exists to resolve.
///
/// Returns whether the document was **RESET** this frame — the purge removed
/// the last animated object. The bridge reacts (rewind + pause + drop the
/// panel's container trail); nothing here touches a playhead, because this
/// function does not own one. Healing is not counted back: its oracle is the
/// bindings themselves (`entity` bits + `!missing`).
pub(crate) fn upkeep(timeline: &mut TimelineState, world: &mut World) -> bool {
    let any_missing = timeline.doc.bindings().iter().any(|b| b.missing);
    // `None` no valor = nome AMBÍGUO (dois objetos vivos, o mesmo nome).
    let by_wire: std::collections::BTreeMap<u64, Option<u64>> = if any_missing {
        let mut q = world.query::<(Entity, &Name)>();
        let mut m: std::collections::BTreeMap<u64, Option<u64>> = std::collections::BTreeMap::new();
        for (e, name) in q.iter(world) {
            m.entry(wire_id_for_name(name.as_str()).0)
                .and_modify(|slot| *slot = None) // já havia um: empate
                .or_insert(Some(e.to_bits()));
        }
        m
    } else {
        std::collections::BTreeMap::new() // alloc-free
    };
    {
        let world = &*world;
        refresh_and_heal_bindings(
            &mut timeline.doc,
            |bits| wire_of(world, bits),
            |w| by_wire.get(&w.0).copied().flatten(),
        );
    }
    purge_the_dead(timeline, &by_wire)
}

/// The purge half of [`upkeep`] — see its docs for the policy. Returns whether
/// the document was fully reset.
fn purge_the_dead(
    timeline: &mut TimelineState,
    by_wire: &std::collections::BTreeMap<u64, Option<u64>>,
) -> bool {
    // Still missing after the heal ⇒ dead, EXCEPT the ambiguous tie (a live
    // duplicate exists — `Some(None)` in the map), which stays dormant. A NULL
    // wire id (a transient that never had a name) can never heal, so it purges.
    let dead: Vec<ph2d_anim::AnimTarget> = timeline
        .doc
        .bindings()
        .iter()
        .filter(|b| b.missing && by_wire.get(&b.wire_id.0) != Some(&None))
        .map(|b| b.target)
        .collect();
    if dead.is_empty() {
        return false;
    }
    // ONE timeline-undo step for the whole purge (reset included): the recovery
    // path after an accidental delete is global Ctrl+Z (the object respawns,
    // same name) + timeline Ctrl+Z (the doc returns, bindings dormant) — and
    // the heal above recolors them on the next frame.
    timeline.history.push(timeline.doc.clone());
    for target in dead {
        timeline.doc.purge_binding(target);
    }
    // Purged targets may be selected; a selection into removed tracks is stale.
    timeline.selection.clear();
    if !timeline.doc.bindings().is_empty() {
        return false; // other objects still animated: their work is untouchable
    }
    // Last animated object gone ⇒ the timeline resets. The display fps is a
    // project-ish setting, not the dead object's animation — it survives.
    let fps = timeline.doc.fps_display;
    timeline.doc = ph2d_timeline::TimelineDoc::new();
    timeline.doc.fps_display = fps;
    timeline.edit_path.clear();
    true
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
pub(crate) fn install_from_project(bytes: &[u8]) -> Result<TimelineState, String> {
    let mut timeline = TimelineState::new();
    if bytes.is_empty() {
        return Ok(timeline); // projeto sem animação: a sessão fica com o documento vazio
    }
    timeline.doc = ph2d_timeline::TimelineDoc::from_bytes(bytes)?;
    // `entity_of` sempre `None` ⇒ `entity = 0` + `missing` em TODA binding (contrato do
    // `resolve_entities`). O `wire_id` — que é o que importa — vem do arquivo, intacto.
    //
    // `0` NÃO é um entity válido no bevy (o índice é `NonZero<u32>`), o que é exatamente o que
    // faz dele um sentinel seguro — e exatamente o que obriga TODO leitor de bits a usar
    // `Entity::try_from_bits`: o `from_bits(0)` **entra em pânico**. O apply do frame seguinte
    // faria isso com cada binding recém-carregada, e o Ctrl+O virava um crash.
    resolve_entities(&mut timeline.doc, |_| None);
    Ok(timeline)
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

    /// **Um nome AMBÍGUO não cura, não purga — recusa.**
    ///
    /// A animação reencontra o objeto pelo NOME (`wire_id` = hash do `Name`), e a unicidade é um
    /// invariante mantido em N lugares do shell. Se dois objetos vivos dividem o nome, *"de quem é
    /// esta track?"* não tem resposta — curar num deles seria dirigir a pose do objeto errado, e
    /// PURGAR seria destruir trabalho por causa de um empate transitório. A track fica dormente
    /// (some do painel) até o empate acabar; então cura no que sobrou.
    #[test]
    fn an_ambiguous_name_refuses_to_heal_and_refuses_to_purge() {
        let mut sim = SimWorld::new();
        let hero = sim.world_mut().spawn(Name::new("hero")).id();
        let mut timeline = TimelineState::new();
        key(&mut timeline, hero.to_bits());
        assert!(
            !upkeep(&mut timeline, sim.world_mut()),
            "vivo: nada a fazer"
        );

        // O objeto morre — e ANTES do próximo upkeep dois homônimos entram em cena
        // (um sprite renomeado, uma forma homônima). O empate tem de existir no
        // frame em que a purga olharia, senão a fixture não contém o fenômeno.
        sim.world_mut().despawn(hero);
        let a = sim.world_mut().spawn(Name::new("hero")).id();
        let b = sim.world_mut().spawn(Name::new("hero")).id();
        assert_ne!(a, b);
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut timeline.doc, 0.0);
        assert!(timeline.doc.bindings()[0].missing);

        assert!(
            !upkeep(&mut timeline, sim.world_mut()),
            "empate não é reset"
        );
        assert_eq!(
            timeline.doc.bindings().len(),
            1,
            "empate de nome: a track não escolhe um dos dois NEM é purgada"
        );
        assert!(
            timeline.doc.bindings()[0].missing,
            "ela continua dormente — visível pela ausência, não colada no objeto errado"
        );

        // Desfeito o empate, ela cura no que sobrou.
        sim.world_mut().despawn(b);
        upkeep(&mut timeline, sim.world_mut());
        assert_eq!(timeline.doc.bindings()[0].entity, a.to_bits());
        assert!(!timeline.doc.bindings()[0].missing, "sem empate, cura");
    }

    /// **Deletar o objeto purga a track dele no MESMO upkeep** (Enio, 2026-07-22: *"a timeline
    /// precisa ser resetada ao deletar o objeto"*) — e sendo o único objeto animado, o
    /// documento inteiro RESETA, num passo do undo da timeline.
    ///
    /// Este é o contrato que SUBSTITUI o "delete + Ctrl+Z cura" de 2026-07-11: a dormência que
    /// fazia a cura era a mesma que entregava a timeline velha ("totalmente bugada") ao próximo
    /// objeto criado. A recuperação agora é explícita: Ctrl+Z global (o objeto volta) + Ctrl+Z
    /// da timeline (o documento volta, dormente) — e o heal recola. O gate disso mora em
    /// `timeline_orphan_tests`.
    #[test]
    fn a_deleted_objects_track_is_purged_and_the_last_one_resets_the_document() {
        let mut sim = SimWorld::new();
        let hero = sim.world_mut().spawn(Name::new("hero")).id();
        let mut timeline = TimelineState::new();
        key(&mut timeline, hero.to_bits());
        upkeep(&mut timeline, sim.world_mut()); // carimba o wire_id em vida
        let steps_before = timeline.history.can_undo();

        sim.world_mut().despawn(hero);
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut timeline.doc, 0.0);
        assert!(timeline.doc.bindings()[0].missing, "flagged by liveness");

        assert!(
            upkeep(&mut timeline, sim.world_mut()),
            "último objeto animado deletado => o documento reseta"
        );
        assert!(
            timeline.doc.bindings().is_empty(),
            "a binding foi PURGADA, não deixada dormente"
        );
        assert_eq!(
            timeline.doc,
            ph2d_timeline::TimelineDoc::new(),
            "resetada = um documento fresco, não um documento esvaziado aos poucos"
        );
        assert!(steps_before, "as keys já eram passos");
        assert!(
            timeline.history.can_undo(),
            "a purga é um passo do undo da timeline — trabalho destruível tem caminho de volta"
        );
    }

    /// **A animação atravessa o arquivo de projeto e reencontra os objetos pelo NOME** — e o
    /// heal roda ANTES da purga, no MESMO upkeep.
    ///
    /// A ordem é load-bearing: o `install_from_project` destaca TODA binding (`entity = 0`,
    /// `missing`), então uma purga que rodasse primeiro (ou que ignorasse o resultado do heal)
    /// apagaria o documento inteiro um frame depois de todo Ctrl+O. O oráculo aqui é o
    /// documento INTEIRO intacto depois do primeiro frame — não só as bindings.
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

        let mut loaded = install_from_project(&bytes).unwrap();
        assert_eq!(
            loaded.doc.bindings().len(),
            2,
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

        // O frame seguinte (o `upkeep` do `timeline_bridge`) recola pelo nome — e a purga,
        // que roda no mesmo chamado, não pode tocar num documento que acabou de curar.
        assert!(
            !upkeep(&mut loaded, sim2.world_mut()),
            "um load que cura nunca é um reset"
        );
        assert_eq!(loaded.doc.bindings().len(), 2, "nada foi purgado");
        for (b, want) in loaded.doc.bindings().iter().zip(&load_bits) {
            assert_eq!(b.entity, *want, "cada binding no objeto DESTA sessão");
            assert!(!b.missing);
        }
    }

    /// A track cujo objeto não está no projeto carregado **sai com ele** — purgada no primeiro
    /// upkeep (Enio, 2026-07-22: objeto que não existe não deixa timeline para trás). As
    /// OUTRAS bindings curam normalmente, e curar uma é o que impede o reset total.
    #[test]
    fn a_track_whose_object_is_not_in_the_loaded_project_leaves_with_it() {
        let (save_world, save_bits) = world_with(&["sprite_001", "sprite_002"]);
        let mut timeline = TimelineState::new();
        key(&mut timeline, save_bits[0]);
        key(&mut timeline, save_bits[1]);
        let bytes = serialize(&mut timeline, save_world.world()).unwrap();

        // Sessão 2: só o primeiro sprite voltou.
        let (mut load_world, _) = world_with(&["sprite_001"]);
        let mut loaded = install_from_project(&bytes).unwrap();
        assert_eq!(loaded.doc.bindings().len(), 2);
        assert!(
            !upkeep(&mut loaded, load_world.world_mut()),
            "uma binding curou: o documento não reseta"
        );
        assert_eq!(
            loaded.doc.bindings().len(),
            1,
            "a track do objeto ausente foi purgada com ele"
        );
        assert!(
            !loaded.doc.bindings()[0].missing,
            "e a sobrevivente é a que curou"
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

        let mut dirty = TimelineState::new();
        key(&mut dirty, bits[0]); // uma sessão suja: um passo no histórico
        assert!(dirty.history.can_undo());
        let loaded = install_from_project(&bytes).unwrap();
        assert!(!loaded.history.can_undo(), "histórico zerado no load");
        assert!(loaded.selection.is_empty(), "seleção zerada no load");
    }

    /// Bytes de um `DOC_VERSION` que este binário não lê são **recusados** — não lidos com o
    /// layout novo. (Postcard é posicional: ler seria pior que recusar.)
    #[test]
    fn a_document_from_another_era_is_refused_not_misread() {
        assert!(
            install_from_project(&[0xff, 0xff, 0xff]).is_err(),
            "bytes de outra era são RECUSADOS — o load inteiro é recusado por cima disso"
        );
        assert!(
            install_from_project(&[]).unwrap().doc.bindings().is_empty(),
            "…e um projeto SEM animação abre com o documento vazio, sem erro"
        );
    }
}
