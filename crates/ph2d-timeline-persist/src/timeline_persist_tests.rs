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
        .map(|n| {
            sim.world_mut()
                .spawn((ph2d_ecs::Transform::IDENTITY, Name::new(*n)))
                .id()
                .to_bits()
        })
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
    let hero = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("hero")))
        .id();
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

/// ⭐ **DOIS objetos com o mesmo nome já não fazem a animação desaparecer** — a cura de
/// produto do ADR-0164 F1 passo 5b, medida a vermelho antes de existir.
///
/// **O defeito que este gate substitui.** Enquanto a identidade era o hash do `Name`, um
/// homónimo tornava *"de quem é esta track?"* uma pergunta sem resposta, e o `upkeep`
/// recusava as duas saídas: não curava (dirigir a pose do objeto errado) e não purgava
/// (destruir trabalho por um empate). A track ficava **dormente — sumia do painel** —, sem
/// badge e sem erro, até o empate acabar. Medido antes da troca: uma binding carregada de um
/// projeto para um mundo com dois `hero` voltava `missing = true`.
///
/// Hoje a resposta existe: dois objetos nunca partilham um [`ph2d_ecs::StableId`], então o
/// empate **não se forma**. O caso especial não foi resolvido — a representação apagou-o.
///
/// (Mutação: pôr o `wire_of` a devolver o hash do nome outra vez ⇒ `missing` volta — RED.)
#[test]
fn two_homonyms_no_longer_hide_the_animation() {
    let mut sim = SimWorld::new();
    let hero = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("hero")))
        .id();
    let mut timeline = TimelineState::new();
    key(&mut timeline, hero.to_bits());
    upkeep(&mut timeline, sim.world_mut()); // carimba a identidade em vida
    let bytes = serialize(&mut timeline, sim.world_mut()).expect("serializa");

    // A sessão seguinte: o objeto animado volta, e um HOMÓNIMO existe ao lado dele — a
    // forma que a F4 traz (mestre + cópia) e que qualquer duplicação de nome produz.
    let mut sim2 = SimWorld::new();
    let hero2 = sim2
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("hero")))
        .id();
    let twin = sim2
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("hero")))
        .id();
    assert_ne!(hero2, twin);
    // O restore reinsere a identidade da linha do snapshot; aqui ela é encenada à mão.
    let id = ph2d_ecs::stable_id_of(sim.world(), hero).expect("o upkeep atribuiu");
    sim2.world_mut().entity_mut(hero2).insert(id);

    let mut loaded = install_from_project(&bytes).expect("install");
    assert!(
        !upkeep(&mut loaded, sim2.world_mut()),
        "há animação viva: nada reseta"
    );
    let b = &loaded.doc.bindings()[0];
    assert!(
        !b.missing,
        "o homónimo já não esconde a animação — ela recola no objeto certo"
    );
    assert_eq!(b.entity, hero2.to_bits(), "e é o objeto certo, não o gémeo");
}

/// **A metade que SOBREVIVE: um documento LEGADO com nome ambíguo continua a recusar as
/// duas saídas.**
///
/// A chave legada (o hash do `Name`) ainda está no mapa do `upkeep` — sem ela a purga
/// apagaria a animação de todo projeto gravado antes do ADR-0164 F1. E enquanto ela for
/// consultada, o empate que ela pode formar tem de manter a lei antiga, **palavra por
/// palavra**: curar num dos dois seria dirigir a pose do objeto errado, e PURGAR seria
/// destruir trabalho por causa de um empate transitório. A track fica dormente até o empate
/// acabar; então cura no que sobrou.
///
/// ⚠️ **Este gate morre com a chave legada, não antes** — ver a nota de deprecação em
/// [`ph2d_ecs::stable_name_id`].
#[test]
fn a_legacy_documents_ambiguous_name_still_refuses_to_heal_and_to_purge() {
    let mut sim = SimWorld::new();
    let hero = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("hero")))
        .id();
    let mut timeline = TimelineState::new();
    key(&mut timeline, hero.to_bits());
    upkeep(&mut timeline, sim.world_mut());

    // ⚠️ A fixtura tem de conter o fenómeno: um documento LEGADO guarda o HASH DO NOME no
    // `wire_id`. Carimbá-lo à mão é o que um `.ph2dproj` pré-ADR-0164 traz de disco.
    timeline.doc.bindings_mut()[0].wire_id =
        ph2d_timeline::WireId(ph2d_ecs::stable_name_id("hero"));

    // O objeto morre — e ANTES do próximo upkeep dois homónimos entram em cena (um sprite
    // renomeado, uma forma homónima). O empate tem de existir no frame em que a purga
    // olharia, senão a fixtura não contém o fenómeno.
    sim.world_mut().despawn(hero);
    let a = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("hero")))
        .id();
    let b = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("hero")))
        .id();
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

    // ⭐ **E o documento legado SOBE DE SUBSTRATO sozinho** — sem degrau de schema e sem
    // migração escrita: o re-carimbo escreve a identidade por cima do hash.
    //
    // ⚠️ **São DOIS frames, e a fronteira é deliberada.** O `refresh_and_heal_bindings` tem
    // uma passagem só, e nela cada binding é OU curada (resolve pela chave guardada) OU
    // re-carimbada (lê o mundo) — nunca as duas. Fundi-las faria a mesma passagem ler e
    // escrever o mesmo campo, e o heal deixaria de ser só uma leitura da chave. A janela é
    // de 1/60 s e não tem consequência: se o Ctrl+S cair dentro dela, o `serialize` carimba
    // a identidade na mesma (ele lê `wire_of` da entidade, que a cura já resolveu).
    let stale = timeline.doc.bindings()[0].wire_id;
    assert_eq!(
        stale,
        ph2d_timeline::WireId(ph2d_ecs::stable_name_id("hero")),
        "no frame da cura a chave ainda é a legada"
    );
    upkeep(&mut timeline, sim.world_mut());
    let id = ph2d_ecs::stable_id_of(sim.world(), a).expect("tem id");
    assert_eq!(
        timeline.doc.bindings()[0].wire_id,
        ph2d_timeline::WireId(id.0),
        "no frame seguinte subiu para a identidade — o hash não fica lá a envelhecer"
    );
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
    let hero = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("hero")))
        .id();
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
    let (mut save_world, save_bits) = world_with(&["sprite_001", "sprite_002"]);
    let mut timeline = TimelineState::new();
    key(&mut timeline, save_bits[0]);
    key(&mut timeline, save_bits[1]);
    let bytes = serialize(&mut timeline, save_world.world_mut()).unwrap();

    // Sessão 2: os MESMOS nomes, bits NOVOS (as entidades descartadas deslocam o
    // alocador, para que os bits realmente difiram).
    let mut sim2 = SimWorld::new();
    for _ in 0..3 {
        sim2.world_mut().spawn(());
    }
    let load_bits: Vec<u64> = ["sprite_001", "sprite_002"]
        .iter()
        .map(|n| {
            sim2.world_mut()
                .spawn((ph2d_ecs::Transform::IDENTITY, Name::new(*n)))
                .id()
                .to_bits()
        })
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
    let (mut save_world, save_bits) = world_with(&["sprite_001", "sprite_002"]);
    let mut timeline = TimelineState::new();
    key(&mut timeline, save_bits[0]);
    key(&mut timeline, save_bits[1]);
    let bytes = serialize(&mut timeline, save_world.world_mut()).unwrap();

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
    let (mut world, bits) = world_with(&["sprite_001"]);
    let mut timeline = TimelineState::new();
    key(&mut timeline, bits[0]);
    let bytes = serialize(&mut timeline, world.world_mut()).unwrap();

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

/// ⭐⭐⭐ **RENOMEAR um objeto animado NÃO desliga a animação dele** — e nem através do arquivo.
///
/// # ⚠️ Este gate existe porque a AUSÊNCIA dele deixou uma nota mentir durante uma fase inteira
///
/// O plano da F1 carregava *«a outra metade do passo 5: renomear um objeto animado desliga o
/// binding»* como trabalho pendente, e o `CLAUDE.md` §5 dizia o mesmo. **As duas estavam erradas**:
/// o passo 5b trocou o substrato para o [`ph2d_ecs::StableId`] e o doc do módulo até o diz. O que
/// faltava não era o comportamento — era **o gate que o afirma**, e por isso a nota pôde envelhecer
/// sem que nada a contradissesse. *Um comentário não é uma prova; um gate é.*
///
/// ⚠️ **O rename acontece DEPOIS da serialização**, de propósito: é a travessia do arquivo que a
/// nota acusava, e testá-lo só em memória mediria a metade fácil.
///
/// (Mutação: pôr a chave do `StableId` a resolver `None` no `upkeep` ⇒ a binding fica `missing` e
/// este gate fica RED.)
#[test]
fn renaming_an_animated_object_does_not_unbind_it() {
    let mut sim = SimWorld::new();
    let hero = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("hero")))
        .id();
    let mut timeline = TimelineState::new();
    key(&mut timeline, hero.to_bits());
    upkeep(&mut timeline, sim.world_mut());
    let id = ph2d_ecs::stable_id_of(sim.world(), hero).expect("o upkeep atribuiu identidade");
    let bytes = serialize(&mut timeline, sim.world_mut()).expect("serializa");

    // A sessão seguinte: o MESMO objeto (mesma identidade) com **outro nome**.
    let mut sim2 = SimWorld::new();
    let again = sim2
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("o heroi")))
        .id();
    sim2.world_mut().entity_mut(again).insert(id);

    let mut loaded = install_from_project(&bytes).expect("install");
    assert!(
        !upkeep(&mut loaded, sim2.world_mut()),
        "a animacao foi purgada — o objeto esta' vivo, so' mudou de nome"
    );
    let b = &loaded.doc.bindings()[0];
    assert!(
        !b.missing,
        "renomear desligou a binding. O artista renomeia uma camada e a animacao dela para de \
         existir, sem aviso nenhum na tela"
    );
    assert_eq!(
        b.entity,
        again.to_bits(),
        "a binding recolou no objeto errado depois do rename"
    );
}

/// ⚠️ **E o NOME sozinho já não basta** — a metade que prova que o substrato de facto mudou.
///
/// Um objeto com o nome certo e **identidade diferente** é outro objeto. Antes do passo 5b ele
/// teria capturado a animação; hoje a binding fica dormente em vez de dirigir a pose de um
/// estranho. *É a mesma lei dos homónimos, vista pelo outro lado.*
#[test]
fn a_stranger_with_the_old_name_does_not_capture_the_animation() {
    let mut sim = SimWorld::new();
    let hero = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("hero")))
        .id();
    let mut timeline = TimelineState::new();
    key(&mut timeline, hero.to_bits());
    upkeep(&mut timeline, sim.world_mut());
    let bytes = serialize(&mut timeline, sim.world_mut()).expect("serializa");

    let id = ph2d_ecs::stable_id_of(sim.world(), hero).expect("identidade");
    // Outra sessão: alguém chamado "hero", mas com OUTRA identidade.
    //
    // ⚠️⚠️ **A identidade é escrita À MÃO, e a 1.ª versão deste gate não o fazia** — dois mundos
    // novos alocam `StableId` a partir do mesmo contador, então o `assign_missing_stable_ids`
    // dava ao estranho **exactamente o id do herói** e o gate reprovava sobre produto correto.
    // *Uma fixtura cujos dois objetos concordam por acidente não consegue distingui-los.*
    let mut sim2 = SimWorld::new();
    let stranger = sim2
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("hero")))
        .id();
    sim2.world_mut()
        .entity_mut(stranger)
        .insert(ph2d_ecs::StableId(id.0 + 1_000));
    let mut loaded = install_from_project(&bytes).expect("install");
    upkeep(&mut loaded, sim2.world_mut());
    // ⚠️ **A asserção é sobre TODAS as bindings, e não sobre a `[0]`** — e a 1.ª versão indexava
    // uma lista que a purga já tinha esvaziado, então ela rebentava sobre produto **correto**. O
    // desfecho legítimo aqui são dois, e os dois passam: a binding é purgada (o objeto de
    // identidade 1 não existe neste mundo) **ou** fica dormente. O que nenhum deles pode fazer é
    // colar-se ao homónimo. *Um gate que só sabe ler um dos desfechos certos reprova metade deles.*
    assert!(
        loaded
            .doc
            .bindings()
            .iter()
            .all(|b| b.missing || b.entity != stranger.to_bits()),
        "um estranho com o nome antigo capturou a animacao — o substrato ainda e' o NOME, e \
         renomear passa a ser uma forma de roubar a animacao de outro objeto"
    );
}
