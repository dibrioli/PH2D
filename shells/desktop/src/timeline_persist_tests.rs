//! Os gates do [`super`] — a cura das bindings destacadas, a purga do objeto morto,
//! e o reset que devolve a composição do boot.
//!
//! Filho por `#[path]`, e não módulo irmão: `use super::*` tem de alcançar os privados
//! (`purge_the_dead`, `heal_detached`), que é o que este arquivo testa.

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

/// **Um projeto LEGADO (sem duração autorada) abre com 4 s + véu** (Enio, 2026-07-28:
/// *"4 seg e véu visível mesmo sem nenhum clip na timeline"*) — o clip 0 derivado-0 de um
/// save pré-2026-07-23 (o que o Autokey pegava e ficava sem véu) vira a mesma composição de
/// 4 s do boot. (Mutação: tirar o stamp legado do `install_from_project` ⇒ clip 0 volta a
/// `None`, a caixa abre em ∞/sem véu — RED.)
#[test]
fn a_legacy_project_without_authored_durations_opens_at_four_seconds() {
    let mut legacy = ph2d_timeline::TimelineDoc::new();
    assert_eq!(
        legacy.clip_length_override(0),
        None,
        "o doc legado e derivado"
    );
    assert_eq!(legacy.scene_length, None);
    legacy.insert_key(
        1,
        PropKind::TranslationX,
        ph2d_anim::RationalTime::from_seconds(0.0),
        ph2d_anim::AnimValue::Float(0.0),
        ph2d_anim::Interp::Linear,
    );
    let bytes = legacy.to_bytes().expect("serializa");

    let opened = install_from_project(&bytes).expect("carrega");
    assert_eq!(
        opened.doc.clip_length_override(0),
        Some(ph2d_timeline::DEFAULT_DURATION_SECONDS),
        "abrir um projeto legado (tudo derivado) instala 4 s no clip 0, nao Dur 0"
    );
    assert_eq!(
        opened.doc.scene_length,
        Some(ph2d_timeline::DEFAULT_DURATION_SECONDS),
        "e 4 s na cena"
    );
}

/// **Uma composição AUTORADA é preservada no load — incluindo uma deixada INFINITA de
/// propósito.** O stamp legado só dispara quando clip 0 E cena são derivados; um clip 0
/// deixado infinito tem a CENA autorada, então o infinito sobrevive. (Mutação: disparar o
/// stamp sempre ⇒ o clip 0 infinito vira 4 s — RED.)
#[test]
fn an_authored_duration_including_an_infinite_clip_survives_load() {
    let mut doc = ph2d_timeline::TimelineDoc::new();
    doc.set_scene_length(Some(6.0)); // cena AUTORADA -> nao e a assinatura legada
    doc.set_clip_length_override(0, None); // clip 0 deixado INFINITO de proposito
    doc.insert_key(
        1,
        PropKind::TranslationX,
        ph2d_anim::RationalTime::from_seconds(0.0),
        ph2d_anim::AnimValue::Float(0.0),
        ph2d_anim::Interp::Linear,
    );
    let bytes = doc.to_bytes().expect("serializa");

    let opened = install_from_project(&bytes).expect("carrega");
    assert_eq!(
        opened.doc.scene_length,
        Some(6.0),
        "a cena autorada e preservada"
    );
    assert_eq!(
        opened.doc.clip_length_override(0),
        None,
        "um clip 0 deixado INFINITO (com a cena autorada) NAO e clobbered"
    );
}

/// **Deletar o último objeto animado reseta a timeline para 4 s + véu — NÃO para
/// derivado-0** (Enio, 2026-07-28: *"deletei os objetos da cena, criei outro, mas a timeline
/// ficou com dur infinita — deveria estar com dur 4 e véu visível"*). O purge esvazia as
/// bindings e reseta o doc; o reset tem de dar a mesma composição de 4 s do boot, senão o
/// próximo objeto criado abre a timeline em ∞ / sem véu. (Mutação: resetar para
/// `TimelineDoc::new()` sem estampar 4 s ⇒ clip 0 volta a `None` — RED.)
#[test]
fn deleting_the_last_animated_object_resets_the_timeline_to_four_seconds() {
    let mut sim = SimWorld::new();
    let hero = sim.world_mut().spawn(Name::new("hero")).id();
    let mut timeline = TimelineState::with_default_duration();
    key(&mut timeline, hero.to_bits()); // anima o objeto -> cria uma binding
    assert_eq!(timeline.doc.bindings().len(), 1);

    // O objeto morre; o apply do frame marca a binding missing.
    sim.world_mut().despawn(hero);
    ph2d_timeline::apply_from_doc(sim.world_mut(), &mut timeline.doc, 0.0);
    assert!(timeline.doc.bindings()[0].missing);

    // O upkeep purga a binding morta; era a última -> a timeline reseta.
    let reset = upkeep(&mut timeline, sim.world_mut());
    assert!(reset, "a ultima binding morta -> a timeline reseta");
    assert!(timeline.doc.bindings().is_empty(), "a binding foi purgada");
    assert_eq!(
        timeline.doc.clip_length_override(0),
        Some(ph2d_timeline::DEFAULT_DURATION_SECONDS),
        "o reset da 4 s no clip 0 (nao derivado-0) -> o proximo objeto abre em 4 s + veu"
    );
    assert_eq!(
        timeline.doc.scene_length,
        Some(ph2d_timeline::DEFAULT_DURATION_SECONDS),
        "e 4 s na cena"
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
    // ⚠️ "Fresco" é o documento que o BOOT instala (4 s + véu), não o
    // `TimelineDoc::new()` DERIVADO. Este oráculo ficou cravado no derivado quando
    // `2695bdcc5` passou o reset a estampar os 4 s: o commit acrescentou um gate novo
    // (`deleting_the_last_animated_object_resets_the_timeline_to_four_seconds`) e
    // deixou este afirmando o OPOSTO sobre o mesmo reset — dois gates em contradição,
    // com a suíte do shell vermelha desde então. O que o gate QUER dizer não mudou:
    // resetada = fresca, não esvaziada aos poucos.
    assert_eq!(
        timeline.doc,
        ph2d_timeline::TimelineState::with_default_duration().doc,
        "resetada = um documento fresco (o do boot: 4 s + véu), \
         não um documento esvaziado aos poucos"
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

/// **Um projeto SEM timeline abre com a composição-padrão de 4 s** (Enio, 2026-07-27:
/// *"a duração padrão do clip deve ser 4seg, mas ao abrir a timeline está 0"* / *"o véu
/// deve ficar visível sempre"*), não com o `new()` DERIVADO-0 — que deixava Dur 0, sem
/// véu, e as expressões PURAS extrapolando (nada as cortava). O MESMO padrão que o boot
/// instala (`main.rs` → `with_default_duration`).
///
/// Mutação: `install_from_project(&[])` voltar a `TimelineState::new()` → `view_authored_end`
/// = None → RED.
#[test]
fn an_empty_project_opens_with_the_default_four_second_composition() {
    let st = install_from_project(&[]).expect("empty is Ok");
    assert_eq!(
        st.doc.view_authored_end(None, false),
        Some(4.0),
        "sem timeline no arquivo, a cena abre com 4 s autorados (o véu é visível desde o 1º frame)"
    );
    assert_eq!(
        st.doc.clip_length_override(0),
        Some(4.0),
        "e o clip 0 (a aba Keys) também abre com 4 s autorados"
    );
}

// ─────────── FASE C.3 — a shell publica QUEM é cada objeto animado ───────────

/// Um snapshot com uma track por entidade dada (só o que o `publish_object_names` lê).
fn view_of(entities: &[u64]) -> ph2d_timeline::TimelineViewSnapshot {
    let tracks = entities
        .iter()
        .map(|bits| ph2d_timeline::TrackView {
            target: ph2d_anim::AnimTarget::new(*bits),
            prop: PropKind::TranslationX,
            entity: *bits,
            missing: false,
            keys: Vec::new(),
            buffer_ghost: None,
            pre: ph2d_anim::Extrap::Hold,
            post: ph2d_anim::Extrap::Hold,
            expr: None,
        })
        .collect();
    ph2d_timeline::TimelineViewSnapshot {
        tracks,
        ..Default::default()
    }
}

/// **A shell publica o nome de quem tem track — e SÓ de quem tem.**
///
/// A metade do escopo é o ponto: a pergunta é sobre as rows que vão ser pintadas, e uma
/// cena de quinhentos objetos com três animados publica três nomes.
///
/// **Mutação que deve sangrar:** varrer o mundo em vez das tracks (o `Extra` entra no mapa).
#[test]
fn the_shell_publishes_a_name_for_every_animated_object_and_no_other() {
    let (sim, bits) = world_with(&["Ball", "Box", "Extra"]);
    let mut view = view_of(&bits[..2]);
    publish_object_names(&mut view, sim.world());

    assert_eq!(view.object_name(bits[0]), Some("Ball"));
    assert_eq!(view.object_name(bits[1]), Some("Box"));
    assert_eq!(
        view.object_names.len(),
        2,
        "o objeto sem track não é publicado: {:?}",
        view.object_names
    );
    assert_eq!(
        view.object_name(bits[2]),
        None,
        "e perguntar por ele devolve None, que é o que faz o rótulo cair no id curto"
    );
}

/// **Um objeto que sai das tracks sai do mapa.**
///
/// ⚠️ Não é higiene: os bits de entidade são RECICLADOS pelo bevy, então um nome que
/// sobrevive à track dele acabaria rotulando outro objeto — a mesma armadilha que faz o
/// load de projeto DESTACAR toda binding em vez de confiar nos bits salvos.
///
/// **Mutação que deve sangrar:** tirar o `retain` do `publish_object_names`.
#[test]
fn a_name_does_not_outlive_the_track_it_was_published_for() {
    let (sim, bits) = world_with(&["Ball", "Box"]);
    let mut view = view_of(&bits);
    publish_object_names(&mut view, sim.world());
    assert_eq!(view.object_names.len(), 2, "premissa: os dois publicados");

    // O Box perde a track (deletado, ou a track foi removida).
    view.tracks.retain(|t| t.entity == bits[0]);
    publish_object_names(&mut view, sim.world());
    assert_eq!(
        view.object_names.len(),
        1,
        "o nome do Box tem de sair com a track dele: {:?}",
        view.object_names
    );
    assert_eq!(view.object_name(bits[0]), Some("Ball"));
}

/// **Renomear o objeto muda o que o painel mostra, no mesmo frame.**
///
/// O rótulo é derivado a cada frame justamente por isto; o mapa tem de acompanhar, senão
/// a derivação lê um nome congelado e a cura vira um cache velho.
///
/// **Mutação que deve sangrar:** o `if slot != name.as_str()` virar `if slot.is_empty()`
/// (a reutilização da `String` passaria a ser um cache que nunca invalida).
#[test]
fn renaming_the_object_renames_its_rows() {
    let (mut sim, bits) = world_with(&["Ball"]);
    let mut view = view_of(&bits);
    publish_object_names(&mut view, sim.world());
    assert_eq!(view.object_name(bits[0]), Some("Ball"));

    let e = Entity::try_from_bits(bits[0]).expect("bits vivos");
    *sim.world_mut().get_mut::<Name>(e).expect("tem Name") = Name::new("Bola");
    publish_object_names(&mut view, sim.world());
    assert_eq!(
        view.object_name(bits[0]),
        Some("Bola"),
        "o nome publicado é o do mundo AGORA, não o do frame em que a track nasceu"
    );
}

/// **Um objeto SEM `Name` não publica nada — e um que PERDE o nome perde a entrada.**
///
/// A segunda metade é a que quase escapou: sem o `remove` no braço `else`, um objeto que
/// perde o `Name` continuaria rotulado com o nome que tinha.
#[test]
fn an_object_without_a_name_publishes_none() {
    let mut sim = SimWorld::new();
    let bits = sim.world_mut().spawn(Name::new("Ghost")).id().to_bits();
    let mut view = view_of(&[bits]);
    publish_object_names(&mut view, sim.world());
    assert_eq!(view.object_name(bits), Some("Ghost"), "premissa");

    let e = Entity::try_from_bits(bits).expect("bits vivos");
    sim.world_mut().entity_mut(e).remove::<Name>();
    publish_object_names(&mut view, sim.world());
    assert_eq!(
        view.object_name(bits),
        None,
        "sem Name, o rótulo cai no id curto em vez de mostrar um nome que já não existe"
    );
}
