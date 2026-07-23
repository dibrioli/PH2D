//! **Fatia 1 do nesting** ([plano](../../../docs/Timeline/04_plano_nesting.md) §3): os dados
//! e o guarda de ciclo, headless — [ADR-0133].
//!
//! O que esta fatia entrega é a REPRESENTAÇÃO: um strip pode nomear um container, o documento
//! guarda containers, e não existe caminho — nem pela API, nem por um arquivo — que produza um
//! ciclo. O relógio recursivo que de fato TOCA o interior é a Fatia 2, e a inércia até lá está
//! pinada aqui de propósito (`a_container_strip_is_inert_until_the_recursive_clock_lands`):
//! um no-op que ninguém escreveu num gate é um no-op que alguém vai confundir com um bug.
//!
//! [ADR-0133]: ../../../docs/architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Transform, World};
use ph2d_timeline::{
    DOC_VERSION, NestRefusal, PropKind, StackHost, StripSource, TimelineDoc, apply_from_doc,
};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// A document with `n` containers, each holding one lane.
fn doc_with_containers(n: usize) -> TimelineDoc {
    let mut doc = TimelineDoc::new();
    for i in 0..n {
        let c = doc.add_container(format!("C{i}"));
        doc.add_lane_in(StackHost::Container(c), "L".into())
            .expect("first lane always fits");
    }
    doc
}

/// Put container `src` inside container `host`, on its first lane.
fn nest(
    doc: &mut TimelineDoc,
    host: usize,
    src: usize,
) -> Result<ph2d_timeline::StripId, NestRefusal> {
    doc.add_strip_to(
        StackHost::Container(host),
        0,
        StripSource::Container(src as u16),
        0.0,
        1.0,
    )
}

// ---------------------------------------------------------------------------
// Camada 1 — a recusa no GESTO
// ---------------------------------------------------------------------------

/// **Um container não entra em si mesmo.** O ciclo trivial, e o único que todo produto
/// pesquisado pega — inclusive os dois que o pegam em silêncio (AE, Animate). Aqui ele TEM nome.
#[test]
fn linking_a_container_into_itself_is_refused_at_the_gesture() {
    let mut doc = doc_with_containers(1);
    assert_eq!(nest(&mut doc, 0, 0), Err(NestRefusal::SelfNest));
    assert!(
        doc.container_stack(0).unwrap()[0].strips.is_empty(),
        "a recusa não pode ter deixado o strip para trás"
    );
}

/// **O laço longo também.** A → B é legal; depois disso B → A fecha o ciclo, e é a recusa que
/// só um DFS pega — nenhuma checagem local de "é ele mesmo?" veria isto.
#[test]
fn a_longer_loop_is_refused_at_the_gesture() {
    let mut doc = doc_with_containers(3);
    nest(&mut doc, 0, 1).expect("A contém B: legal");
    nest(&mut doc, 1, 2).expect("B contém C: legal");
    // Agora C → A fecharia A→B→C→A.
    assert_eq!(nest(&mut doc, 2, 0), Err(NestRefusal::WouldCycle));
    // E o caminho legal continua legal: o guarda recusa o ciclo, não o aninhamento.
    let mut fresh = doc_with_containers(3);
    nest(&mut fresh, 0, 1).expect("A contém B");
    nest(&mut fresh, 0, 2).expect("A também contém C — irmãos, não ciclo");
}

/// Um strip de CLIP nunca é recusado: um clip não tem interior por onde voltar.
#[test]
fn a_clip_strip_is_never_a_cycle() {
    let mut doc = doc_with_containers(1);
    doc.add_strip_to(StackHost::Container(0), 0, StripSource::Clip(0), 0.0, 1.0)
        .expect("um clip dentro de um container é o caso NORMAL");
}

// ---------------------------------------------------------------------------
// Camada 2 — a recusa no LOAD
// ---------------------------------------------------------------------------

/// **Um documento cíclico é REJEITADO no load, não consertado.**
///
/// O Blender conserta (`BKE_collection_cycles_fix`) e o preço é destruir o link do artista em
/// silêncio. Aqui o load diz que não sabe ler o arquivo — que é a verdade.
///
/// ⚠️ O ciclo é construído **por baixo da API** (escrevendo direto no stack do container),
/// porque a API não permite criá-lo — que é exatamente a situação que esta camada existe para
/// cobrir: bytes que não vieram do caminho de "add".
#[test]
fn a_cyclic_document_is_rejected_at_load_not_repaired() {
    let mut doc = doc_with_containers(2);
    nest(&mut doc, 0, 1).expect("A contém B");
    // Fecha B → A à força, contornando o guarda do gesto.
    let strip = doc
        .container_stack(0)
        .unwrap()
        .first()
        .unwrap()
        .strips
        .first()
        .unwrap()
        .clone();
    let mut forged = strip;
    forged.source = StripSource::Container(0);
    doc.container_stack_mut(1).unwrap()[0].strips.push(forged);

    assert!(
        doc.find_nest_cycle().is_some(),
        "o documento forjado É cíclico — se não for, o fixture não contém o fenômeno"
    );

    let bytes = doc.to_bytes().expect("serializa");
    let err = TimelineDoc::from_bytes(&bytes).expect_err("o load tem de RECUSAR");
    assert!(
        err.to_lowercase().contains("cycle"),
        "a recusa tem de dizer o que houve, não só falhar: {err}"
    );
}

/// O irmão de PRESENÇA: um documento com aninhamento **legal e profundo** carrega normalmente.
///
/// Sem ele, `find_nest_cycle` poderia devolver `Some` para tudo e a rejeição ficaria verde por
/// recusar o mundo inteiro ([[feedback_absence_gate_needs_a_presence_sibling]]).
#[test]
fn a_deeply_nested_but_acyclic_document_loads_fine() {
    let mut doc = doc_with_containers(4);
    nest(&mut doc, 0, 1).unwrap();
    nest(&mut doc, 1, 2).unwrap();
    nest(&mut doc, 2, 3).unwrap();
    assert!(doc.find_nest_cycle().is_none(), "cadeia, não ciclo");
    let bytes = doc.to_bytes().unwrap();
    let back = TimelineDoc::from_bytes(&bytes).expect("aninhamento legal carrega");
    assert_eq!(back.containers().len(), 4);
    assert_eq!(
        back.container_stack(0).unwrap()[0].strips[0].source,
        StripSource::Container(1),
        "o que o strip nomeia sobrevive à viagem"
    );
}

// ---------------------------------------------------------------------------
// Persistência
// ---------------------------------------------------------------------------

/// **v9, e um blob mais velho é recusado — não mal-lido.**
///
/// O v9 APENDA um campo (`ClipStrip.lead_out`), então um blob mais antigo tem bytes de menos e o
/// postcard não fecha; e o v8 antes dele SUBSTITUIU um campo (`clip` → `source`), onde os bytes
/// significam outra coisa a partir dali. Em qualquer caso, postcard é posicional: ler assim mesmo
/// não daria erro, daria um documento errado. O gate de versão recusa tudo que não é o atual.
#[test]
fn the_schema_is_ten_and_an_older_blob_is_refused() {
    assert_eq!(DOC_VERSION, 10);
    assert_eq!(TimelineDoc::new().version, 10);

    let mut bytes = TimelineDoc::new().to_bytes().unwrap();
    bytes[0] = 8; // o version é o primeiro varint — finge um blob v8
    let err = TimelineDoc::from_bytes(&bytes).expect_err("um blob v8 tem de ser recusado");
    assert!(
        err.contains('8') && err.contains("10"),
        "diz os dois — o do blob e o atual: {err}"
    );
}

/// Um container e seu interior sobrevivem ao round-trip.
#[test]
fn a_container_and_its_interior_survive_the_trip() {
    let mut doc = doc_with_containers(2);
    nest(&mut doc, 0, 1).unwrap();
    doc.add_strip_to(StackHost::Container(1), 0, StripSource::Clip(0), 0.5, 2.5)
        .unwrap();

    let back = TimelineDoc::from_bytes(&doc.to_bytes().unwrap()).unwrap();
    assert_eq!(back.containers()[1].name, "C1");
    let inner = &back.container_stack(1).unwrap()[0].strips[0];
    assert_eq!(inner.source, StripSource::Clip(0));
    assert!((inner.t_start - 0.5).abs() < 1e-12);
    assert!((inner.t_end - 2.5).abs() < 1e-12);
}

// ---------------------------------------------------------------------------
// O que apagar um clip faz — inclusive um nível abaixo
// ---------------------------------------------------------------------------

/// **Apagar um clip reaponta os strips DENTRO dos containers também.**
///
/// Os índices de clip são posicionais, então apagar o clip 1 desliza o 2 para o lugar dele. A
/// função que conserta isso só varria o stack do DOCUMENTO; um container guarda strips que
/// indexam a MESMA lista, e teria ficado tocando o vizinho do clip apagado — o bug exato que
/// aquela função existe para impedir, um nível abaixo de onde alguém olha.
#[test]
fn deleting_a_clip_repoints_the_strips_inside_containers() {
    let mut doc = doc_with_containers(1);
    doc.add_clip("B".into()); // 1
    doc.add_clip("C".into()); // 2
    let host = StackHost::Container(0);
    doc.add_strip_to(host, 0, StripSource::Clip(2), 0.0, 1.0)
        .unwrap();
    doc.add_strip_to(host, 0, StripSource::Clip(1), 2.0, 3.0)
        .unwrap();

    assert!(doc.remove_clip(1), "apaga o clip do meio");

    let strips = &doc.container_stack(0).unwrap()[0].strips;
    assert_eq!(
        strips.len(),
        1,
        "o strip do clip apagado sai; sobra o que apontava para C"
    );
    assert_eq!(
        strips[0].source,
        StripSource::Clip(1),
        "C escorregou de 2 para 1 e o strip DENTRO do container o seguiu"
    );
}

// ---------------------------------------------------------------------------
// O interior TOCA (Fatia 2 — este gate era a inércia, e foi REESCRITO)
// ---------------------------------------------------------------------------

/// **Um strip de container toca o interior dele.**
///
/// Este gate nasceu na Fatia 1 como `a_container_strip_is_inert_until_the_recursive_clock_lands`,
/// afirmando o oposto — que o interior NÃO era avaliado —, e dizendo no próprio texto que
/// deveria ser reescrito, nunca apagado, quando a Fatia 2 landasse. Ele ficou vermelho no
/// commit exato em que o `eval_frame` recursivo entrou, que é como um gate honesto avisa que
/// chegou a hora. Mesmo fixture, asserção invertida.
#[test]
fn a_container_strip_plays_its_interior() {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();

    let mut doc = TimelineDoc::new();
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(7.0),
        Interp::Linear,
    );
    let c = doc.add_container("C".into());
    doc.add_lane_in(StackHost::Container(c), "inner".into())
        .unwrap();
    doc.add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 4.0)
        .unwrap();
    let lane = doc.add_lane("outer".into()).unwrap();
    doc.add_strip_to(
        StackHost::Document,
        lane,
        StripSource::Container(c as u16),
        0.0,
        4.0,
    )
    .unwrap();

    apply_from_doc(&mut world, &mut doc, 1.0);

    let x = world
        .get::<Transform>(bevy_ecs::entity::Entity::try_from_bits(e).unwrap())
        .unwrap()
        .translation
        .x;
    assert!(
        (x - 7.0).abs() < 1e-6,
        "o interior do container tem de dirigir a pose: esperado 7.0, veio {x}"
    );
}

// ---------------------------------------------------------------------------
// Fatia 3a — a edição vai para onde o animador está OLHANDO
// ---------------------------------------------------------------------------

/// **Com um container aberto, um intent de pilha edita o INTERIOR dele.**
///
/// É a metade que decide se o nesting é editável ou só representável: as lanes na tela são as
/// do container, e um edit que caísse na pilha do documento teria mudado algo que o animador
/// não está vendo — em silêncio, porque as duas pilhas são do mesmo tipo e nada reclamaria.
#[test]
fn a_stack_edit_lands_in_the_container_the_animator_opened() {
    use ph2d_core::Playhead;
    use ph2d_timeline::{TimelineIntent, TimelineState, apply_intent};

    let mut st = TimelineState::new();
    let c = st.doc.add_container("C".into());
    let mut ph = Playhead::default();

    // Fechado: a lane vai para o documento.
    st.edit_path = Vec::new();
    apply_intent(&mut st, &mut ph, TimelineIntent::AddLane);
    assert_eq!(st.doc.stack().len(), 1, "fechado, a lane é do documento");
    assert!(st.doc.container_stack(c).unwrap().is_empty());

    // Aberto: a lane vai para o container, e o documento não se move.
    // Só o CONTAINER roteia um edit; lane/strip do passo não participam do edit_host.
    st.edit_path = vec![ph2d_timeline::EnterStep {
        container: c,
        lane: 0,
        strip: Some(ph2d_timeline::StripId(0)),
    }];
    apply_intent(&mut st, &mut ph, TimelineIntent::AddLane);
    assert_eq!(
        st.doc.stack().len(),
        1,
        "a pilha do documento NÃO pode ter crescido — este é o bug silencioso"
    );
    assert_eq!(
        st.doc.container_stack(c).unwrap().len(),
        1,
        "a lane nasceu dentro do container aberto"
    );
}

/// **A ordenação por tempo é re-derivada DENTRO dos containers também.**
///
/// "Strips ordenados por início" é a invariante em que o vizinho significa alguma coisa —
/// `hold_at`, o crossfade e `gap_before` leem a lane em ordem. Um strip arrastado para trás
/// dentro de um container deixaria a lane fora de ordem, e o estrago apareceria como um
/// crossfade contra o vizinho errado, um nível abaixo de onde alguém olha.
#[test]
fn moving_a_strip_inside_a_container_keeps_its_lane_sorted() {
    use ph2d_core::Playhead;
    use ph2d_timeline::{TimelineIntent, TimelineState, apply_intent};

    let mut st = TimelineState::new();
    let c = st.doc.add_container("C".into());
    st.doc
        .add_lane_in(StackHost::Container(c), "l".into())
        .unwrap();
    let host = StackHost::Container(c);
    let a = st
        .doc
        .add_strip_to(host, 0, StripSource::Clip(0), 0.0, 2.0)
        .unwrap();
    st.doc
        .add_strip_to(host, 0, StripSource::Clip(0), 4.0, 6.0)
        .unwrap();

    st.edit_path = match host {
        StackHost::Document => Vec::new(),
        StackHost::Container(c) => vec![ph2d_timeline::EnterStep {
            container: c,
            lane: 0,
            strip: Some(ph2d_timeline::StripId(0)),
        }],
    };
    let mut ph = Playhead::default();
    // Empurra o PRIMEIRO para depois do segundo.
    apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::MoveStrip {
            lane: 0,
            to_lane: 0,
            id: a,
            t_start: 8.0,
        },
    );

    let strips = &st.doc.container_stack(c).unwrap()[0].strips;
    assert_eq!(strips.len(), 2);
    assert!(
        strips[0].t_start <= strips[1].t_start,
        "a lane do container ficou fora de ordem: {:?}",
        strips.iter().map(|s| s.t_start).collect::<Vec<_>>()
    );
    assert!(
        (strips[1].t_start - 8.0).abs() < 1e-12,
        "o strip movido é o de trás agora"
    );
}

/// **Lanes are named against the STACK they join, not always the document's** (Enio,
/// 2026-07-23). The old `fresh_lane_name` counted the document's stack, so every lane added
/// inside a container came out "Lane 1" — the container's own lanes were invisible to the
/// counter. Each host numbers its own.
#[test]
fn fresh_lane_names_count_the_host_stack_not_the_document() {
    let mut doc = TimelineDoc::new();
    let c = doc.add_container("C".into());
    let host = StackHost::Container(c);

    // The DOCUMENT stack has two lanes — a decoy: the container must not count them.
    doc.add_lane_in(
        StackHost::Document,
        doc.fresh_lane_name_in(StackHost::Document),
    )
    .unwrap();
    doc.add_lane_in(
        StackHost::Document,
        doc.fresh_lane_name_in(StackHost::Document),
    )
    .unwrap();

    // Three lanes INSIDE the container, each named against the container.
    for _ in 0..3 {
        let name = doc.fresh_lane_name_in(host);
        doc.add_lane_in(host, name).unwrap();
    }
    let names: Vec<&str> = doc
        .container_stack(c)
        .unwrap()
        .iter()
        .map(|l| l.name.as_str())
        .collect();
    assert_eq!(
        names,
        ["Lane 1", "Lane 2", "Lane 3"],
        "container lanes number themselves — not all 'Lane 1'"
    );
    // And the document's own numbering is unaffected by the container's lanes.
    assert_eq!(
        doc.fresh_lane_name_in(StackHost::Document),
        "Lane 3",
        "the document counts ITS two lanes, blind to the container's three"
    );
}

/// **Renaming a lane lands in the host's stack** — the Arrange/Container rename, and a
/// stale index is a refused no-op.
#[test]
fn rename_lane_writes_the_host_stack() {
    let mut doc = TimelineDoc::new();
    let c = doc.add_container("C".into());
    let host = StackHost::Container(c);
    doc.add_lane_in(host, "Lane 1".into()).unwrap();
    doc.add_lane_in(host, "Lane 2".into()).unwrap();

    assert!(doc.rename_lane_in(host, 1, "Legs".into()));
    assert_eq!(doc.container_stack(c).unwrap()[1].name, "Legs");
    assert_eq!(
        doc.container_stack(c).unwrap()[0].name,
        "Lane 1",
        "só a lane pedida muda"
    );
    assert!(
        !doc.rename_lane_in(host, 9, "x".into()),
        "índice obsoleto: no-op recusado, não pânico"
    );
    // The DOCUMENT stack is a different host — a lane rename in the container never touches it.
    doc.add_lane_in(StackHost::Document, "Doc Lane".into())
        .unwrap();
    assert!(doc.rename_lane_in(StackHost::Document, 0, "Scene".into()));
    assert_eq!(doc.stack()[0].name, "Scene");
    assert_eq!(
        doc.container_stack(c).unwrap()[1].name,
        "Legs",
        "o container fica firme"
    );
}
