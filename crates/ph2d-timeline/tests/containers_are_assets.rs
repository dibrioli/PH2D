//! **A container is an ASSET you make, place and edit — three separate acts** (ADR-0133,
//! amended 2026-07-21).
//!
//! The first cut fused them: one intent created a container AND dropped an instance of it on
//! a new lane, the lane's `+` could only ever place a clip, and a strip could not change lane.
//! The consequence was the report that started this — *"os conteiners funcionam exatamente
//! criando lanes e colocando clipes nas lanes que é o que as lanes já fazem … não vi nenhum
//! diferencial nos containers"* (Enio, 2026-07-21).
//!
//! These gates pin the three acts apart, and the clock an unplaced container gets.

use ph2d_timeline::{
    EnterStep, StackHost, StripId, StripLoop, StripSource, TimelineDoc, TimelineIntent,
    TimelineState, entry_clock,
};

/// A document with one container `Walk` holding a 2 s clip strip inside, and a scene lane.
fn doc_with_container() -> (TimelineDoc, usize, usize) {
    let mut doc = TimelineDoc::new();
    let walk = doc.add_container("Walk".to_string());
    let inner = doc
        .add_lane_in(StackHost::Container(walk), "in".to_string())
        .unwrap();
    doc.add_strip_to(
        StackHost::Container(walk),
        inner,
        StripSource::Clip(0),
        0.0,
        2.0,
    )
    .unwrap();
    let lane = doc.add_lane("L".to_string()).unwrap();
    (doc, walk, lane)
}

fn src(c: usize) -> StripSource {
    StripSource::Container(u16::try_from(c).unwrap())
}

/// **Making a container does not place one** — and the index it gets is the one the panel
/// predicted.
///
/// The panel opens the new container by `containers.len()` read BEFORE the intent, which is
/// only sound while `add_container` appends and refuses nothing. This gate is what makes that
/// a pinned invariant instead of a guess: give containers a cap one day and it turns red at
/// the consumer that would silently open the wrong one.
#[test]
fn making_a_container_appends_the_asset_and_places_nothing() {
    let mut st = TimelineState::new();
    let lanes_before = st.doc.stack().len();
    let next = st.doc.containers().len();

    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    ph2d_timeline::apply_intent(&mut st, &mut ph, TimelineIntent::AddContainer);

    assert_eq!(st.doc.containers().len(), next + 1, "o asset nasce");
    assert_eq!(
        st.doc.add_container("probe".to_string()),
        next + 1,
        "add_container APPENDA e não recusa — é sobre isso que a predição do painel se apoia"
    );
    assert_eq!(
        st.doc.stack().len(),
        lanes_before,
        "criar um container não pode inventar uma lane na CENA — criar e colocar são atos \
         diferentes, e fundi-los é o que fazia um container parecer uma lane"
    );
    assert!(
        st.doc.stack().iter().all(|l| l.strips.is_empty()),
        "nem uma instância"
    );
}

/// **Placing is the lane's `+`, and it accepts a CONTAINER** — the half that did not exist.
#[test]
fn a_container_can_be_placed_on_a_lane_like_any_other_source() {
    let (doc, walk, lane) = doc_with_container();
    let mut st = TimelineState::new();
    st.doc = doc;
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);

    ph2d_timeline::apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::AddStrip {
            lane,
            source: src(walk),
            t_start: 3.0,
            t_end: 5.0,
        },
    );

    let s = &st.doc.stack()[lane].strips[0];
    assert_eq!(
        s.source.container_index().map(usize::from),
        Some(walk),
        "a strip tem de TOCAR o container — placing é o gesto que os dois tipos compartilham"
    );
    assert!((s.t_start - 3.0).abs() < 1e-9 && (s.t_end - 5.0).abs() < 1e-9);
}

/// **An UNPLACED container still has a clock — the identity over its own extent — and it says
/// so.**
///
/// The Containers tab reaches a container by NAME, so it may be instanced nowhere. Refusing a
/// map there would leave its ruler frozen (the bug the entry path already fixed once, one
/// level up); inventing a scene window would label the container's own axis with the scene's
/// name. `placed` is the difference, and it is the one fact the readout branches on.
#[test]
fn an_unplaced_container_gets_the_identity_and_is_marked_unplaced() {
    let (doc, walk, _lane) = doc_with_container();
    let path = [EnterStep {
        container: walk,
        lane: 0,
        strip: None,
    }];

    let c = entry_clock(&doc, &path).expect("um container sem instância ainda é autorável");
    assert!(
        !c.placed,
        "não há instância nenhuma — o readout tem de saber"
    );
    assert!(
        (c.map.t0 - c.map.u0).abs() < 1e-9 && (c.map.t1 - c.map.u1).abs() < 1e-9,
        "sem instância a relação é a IDENTIDADE, veio {:?}",
        c.map
    );
    assert!(
        (c.map.u1 - 2.0).abs() < 1e-9,
        "o eixo é a extensão do PRÓPRIO container (2 s de conteúdo), veio {}",
        c.map.u1
    );
    // E um container VAZIO ainda tem eixo: é onde a primeira strip vai.
    let mut empty = doc;
    let e = empty.add_container("Empty".to_string());
    let c = entry_clock(
        &empty,
        &[EnterStep {
            container: e,
            lane: 0,
            strip: None,
        }],
    )
    .expect("um container vazio é onde se põe a primeira strip");
    assert!(c.map.u1 > c.map.u0, "eixo degenerado não se arrasta");
}

/// **Named by nobody, the walk takes the FIRST instance** — a pure function of the document,
/// stable at every playhead time.
///
/// This is what gives a container opened from the tab a real relation to the scene when it
/// HAS one. It is deliberately not "the instance playing now": that answer flickers away in
/// every gap, which is the defect `entry_map` was written to end.
#[test]
fn without_a_named_strip_the_walk_takes_the_first_instance() {
    let (mut doc, walk, lane) = doc_with_container();
    doc.add_strip_to(StackHost::Document, lane, src(walk), 4.0, 6.0)
        .unwrap();
    doc.add_strip_to(StackHost::Document, lane, src(walk), 9.0, 11.0)
        .unwrap();

    let c = entry_clock(
        &doc,
        &[EnterStep {
            container: walk,
            lane: 0,
            strip: None,
        }],
    )
    .unwrap();
    assert!(c.placed, "há instância — a relação é com a CENA");
    assert!(
        (c.map.t0 - 4.0).abs() < 1e-9,
        "a PRIMEIRA em ordem de documento, veio t0={}",
        c.map.t0
    );

    // ...e continua sendo a primeira em QUALQUER instante: o mapa é puro no documento.
    let c2 = entry_clock(
        &doc,
        &[EnterStep {
            container: walk,
            lane: 0,
            strip: None,
        }],
    )
    .unwrap();
    assert_eq!(c.map, c2.map);
}

/// **Uma trilha OBSOLETA continua sendo uma RECUSA** — e não se confunde com "não colocado".
///
/// São dois fatos diferentes: *o strip que eu nomeei sumiu* (não sei mais de que instância
/// você falava) e *não existe instância nenhuma* (não há o que relacionar). Dobrar o primeiro
/// no segundo faria a régua mapear pelo primeiro strip que sobrou — a instância errada, em
/// silêncio.
#[test]
fn a_stale_named_walk_still_refuses_instead_of_falling_back() {
    let (mut doc, walk, lane) = doc_with_container();
    let a = doc
        .add_strip_to(StackHost::Document, lane, src(walk), 4.0, 6.0)
        .unwrap();
    doc.add_strip_to(StackHost::Document, lane, src(walk), 9.0, 11.0)
        .unwrap();

    let named = |s: StripId| {
        [EnterStep {
            container: walk,
            lane,
            strip: Some(s),
        }]
    };
    assert!(entry_clock(&doc, &named(a)).is_some(), "vivo: tem mapa");
    doc.remove_strip_in(StackHost::Document, lane, a);
    assert!(
        entry_clock(&doc, &named(a)).is_none(),
        "o strip nomeado sumiu — recusar, NUNCA cair no outro que sobrou"
    );
}

/// **Um strip arrastado para outra lane chega INTEIRO** — mesma identidade, mesmos fades,
/// mesma velocidade, mesmo modo de loop.
///
/// "Remove e adiciona" perderia todos eles e cunharia um `StripId` novo, quebrando o próprio
/// arrasto que ainda segura o antigo. E a re-inserção reordena por `t_start`, que é o
/// invariante que `ClipLane::blend_in` lê para achar o vizinho de um strip.
#[test]
fn moving_a_strip_to_another_lane_carries_the_whole_strip() {
    let (mut doc, _walk, lane) = doc_with_container();
    let other = doc.add_lane("R".to_string()).unwrap();
    let id = doc
        .add_strip_to(StackHost::Document, lane, StripSource::Clip(0), 1.0, 3.0)
        .unwrap();
    {
        let s = doc.strip_in_mut(StackHost::Document, lane, id).unwrap();
        s.ease_in = 0.25;
        s.lead_out = 0.5;
        s.speed = 0.5;
        s.loop_mode = StripLoop::PingPong;
    }

    assert!(doc.move_strip_in(StackHost::Document, lane, other, id, 7.0));

    assert!(
        doc.stack()[lane].strips.is_empty(),
        "saiu da lane de origem"
    );
    let s = doc.strip_in(StackHost::Document, other, id).unwrap();
    assert_eq!(
        s.id, id,
        "a IDENTIDADE atravessa — o arrasto ainda a segura"
    );
    assert!(
        (s.t_start - 7.0).abs() < 1e-9 && (s.t_end - 9.0).abs() < 1e-9,
        "rígido"
    );
    assert!(
        (s.ease_in - 0.25).abs() < 1e-9,
        "o fade de dentro veio junto"
    );
    assert!((s.lead_out - 0.5).abs() < 1e-9, "e o de fora");
    assert!((s.speed - 0.5).abs() < 1e-9, "e a velocidade");
    assert_eq!(s.loop_mode, StripLoop::PingPong, "e o modo de loop");

    // Uma lane que não existe é RECUSA, nunca um strip que evapora.
    assert!(
        !doc.move_strip_in(StackHost::Document, other, 99, id, 0.0),
        "lane fora de alcance tem de recusar"
    );
    assert!(
        doc.strip_in(StackHost::Document, other, id).is_some(),
        "e o strip fica onde estava"
    );
}

/// **Deslizar por cima do vizinho reordena a lane** — o invariante que a lane documenta
/// ("strips em ordem de início") e que o crossfade lê para saber quem é o vizinho.
#[test]
fn sliding_past_a_neighbour_reorders_the_lane() {
    let (mut doc, _walk, lane) = doc_with_container();
    let a = doc
        .add_strip_to(StackHost::Document, lane, StripSource::Clip(0), 0.0, 2.0)
        .unwrap();
    let b = doc
        .add_strip_to(StackHost::Document, lane, StripSource::Clip(0), 4.0, 6.0)
        .unwrap();
    assert_eq!(doc.stack()[lane].strips[0].id, a);

    doc.move_strip_in(StackHost::Document, lane, lane, a, 8.0);
    let order: Vec<_> = doc.stack()[lane].strips.iter().map(|s| s.id).collect();
    assert_eq!(
        order,
        vec![b, a],
        "quem começa antes vem antes — senão o crossfade mistura com o vizinho errado"
    );
}

/// **Uma instância de container nasce em VELOCIDADE 1** — porque "quanto dura este container"
/// tem UMA porta.
///
/// O painel dimensiona o span da strip nova pela extensão que o snapshot publica
/// (`host_end_seconds`, que conta o `lead_end`) e o documento dimensiona o SLICE pela sua
/// própria conta. Enquanto as duas discordavam — e discordavam por exatamente o lead-out da
/// última strip de dentro — `slice != span` no nascimento, que é `speed = slice / span`: uma
/// instância retimada por um fade que alguém autorou lá dentro semanas antes, sem nada na
/// tela dizendo por quê.
#[test]
fn a_container_instance_is_born_at_speed_one() {
    let (mut doc, walk, lane) = doc_with_container();
    // O fade PARA FORA da última strip de dentro: conteúdo, e é onde as duas contas divergiam.
    let inner = doc.container_stack(walk).unwrap()[0].strips[0].id;
    doc.strip_in_mut(StackHost::Container(walk), 0, inner)
        .unwrap()
        .lead_out = 0.5;

    let len = doc
        .host_end_seconds(StackHost::Container(walk))
        .expect("o container tem conteúdo");
    assert!(
        (len - 2.5).abs() < 1e-9,
        "a extensão conta o lead-out (2.0 + 0.5), veio {len}"
    );

    // O painel coloca com ESTE span — é o número que ele lê do snapshot.
    let id = doc
        .add_strip_to(StackHost::Document, lane, src(walk), 0.0, len)
        .unwrap();
    let s = doc.strip_in(StackHost::Document, lane, id).unwrap();
    assert!(
        (s.speed - 1.0).abs() < 1e-9,
        "duas portas para o mesmo fato ⇒ a instância nasce retimada: speed={}",
        s.speed
    );
    assert!(
        (s.slice() - s.span()).abs() < 1e-9,
        "slice == span é o invariante da lane, e tem de ser VERDADE no nascimento"
    );
}
