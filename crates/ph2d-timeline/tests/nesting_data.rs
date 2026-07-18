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

/// **v8, e um v7 é recusado — não mal-lido.**
///
/// Este bump SUBSTITUI um campo (`clip: u16` → `source: StripSource`) em vez de apendar um, então
/// os bytes de um v7 significam outra coisa a partir dali. Postcard é posicional: ler assim mesmo
/// não daria erro, daria um documento errado.
#[test]
fn the_schema_is_eight_and_a_v7_blob_is_refused() {
    assert_eq!(DOC_VERSION, 8);
    assert_eq!(TimelineDoc::new().version, 8);

    let mut bytes = TimelineDoc::new().to_bytes().unwrap();
    bytes[0] = 7; // o version é o primeiro varint
    let err = TimelineDoc::from_bytes(&bytes).expect_err("v7 tem de ser recusado");
    assert!(err.contains('7') && err.contains('8'), "diz os dois: {err}");
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
// A inércia, declarada
// ---------------------------------------------------------------------------

/// **Um strip de container ainda não toca nada — e isso é uma decisão, não um esquecimento.**
///
/// O avaliador recursivo é a Fatia 2. Até lá o avaliador PULA um strip de container
/// explicitamente, e o que este gate garante é que pular signifique *nada acontece*, nunca
/// *acontece a coisa errada* (o índice de container lido como índice de clip seria justamente
/// isso, e seria silencioso).
///
/// Quando a Fatia 2 landar, este gate vira vermelho — e é assim que ele avisa que chegou a hora
/// de ser reescrito, em vez de ficar mentindo em verde.
#[test]
fn a_container_strip_is_inert_until_the_recursive_clock_lands() {
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
    // O container tem conteúdo REAL lá dentro: um strip do clip que move o objeto para 7.
    doc.add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 4.0)
        .unwrap();
    // E o documento toca o container.
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
    assert_eq!(
        x, 0.0,
        "Fatia 1: o interior do container NÃO é avaliado ainda. Se isto virou 7.0, a Fatia 2 \
         landou e este gate deve ser reescrito para exigir o valor — não apagado."
    );
}
