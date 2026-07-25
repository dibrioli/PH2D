//! Container-navigation + container-loop gates for `timeline_bridge.rs`, split out
//! (sibling `#[path]` module, `use super::*`) so the main test file stays under the
//! HR-18 shell LOC cap. These share the `nav_scene` fixture, which lives HERE with them.

use super::*;

/// The scene of the container-navigation gates: a 2 s container instanced at `[4, 12)`
/// (stretched 2× → speed 0.5) with a 1 s lead-in and 0.5 s lead-out, and a DOCUMENT loop
/// over `[0, 20)`. Returns the state (path stamped empty) and the entry step.
fn nav_scene() -> (TimelineState, ph2d_timeline::EnterStep) {
    use ph2d_timeline::{StackHost, StripSource};
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    let c = doc.add_container("C".into());
    doc.add_lane_in(StackHost::Container(c), "l".into())
        .unwrap();
    doc.add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 2.0)
        .unwrap();
    let lane = doc.add_lane("doc".into()).unwrap();
    let strip = doc
        .add_strip_to(
            StackHost::Document,
            lane,
            StripSource::Container(u16::try_from(c).unwrap()),
            4.0,
            12.0,
        )
        .unwrap();
    {
        let s = doc.strip_in_mut(StackHost::Document, lane, strip).unwrap();
        s.lead_in = 1.0;
        s.lead_out = 0.5;
    }
    doc.set_active_loop_for(false, Some((0.0, 20.0)));
    (
        st,
        ph2d_timeline::EnterStep {
            container: c,
            lane,
            strip: Some(strip),
        },
    )
}

/// **Entrar/trocar de container arma o RELÓGIO DO CONTAINER com o loop PRÓPRIO dele — e a
/// cena fica intocada** (Enio, 2026-07-22: *"o loop deve ser independente em cada modo"*).
/// O relógio da cena não é mais tocado ao navegar: o loop do Arrange sobrevive a toda visita.
#[test]
fn entering_arms_the_container_clock_with_its_own_loop_and_leaves_the_scene_alone() {
    let (mut st, step) = nav_scene();
    st.doc
        .set_container_loop(step.container, Some((0.5, 1.5)), false);
    let mut container_ph = Playhead::new(1.0 / 60.0);
    container_ph.seek(0.75);
    let mut scene_ph = Playhead::new(1.0 / 60.0);
    scene_ph.set_loop(0.0, 20.0);
    scene_ph.seek(5.0);
    let scene_before = (scene_ph.loop_range(), scene_ph.time());

    on_container_nav_change(&st.doc, Some(step.container), &mut container_ph);
    assert_eq!(
        container_ph.loop_range(),
        Some((0.5, 1.5)),
        "o relógio do container recebe o loop DELE, não o da cena"
    );
    assert!(
        (container_ph.time() - 0.75).abs() < 1e-9,
        "já dentro do loop: sem seek"
    );
    assert_eq!(
        (scene_ph.loop_range(), scene_ph.time()),
        scene_before,
        "a cena não foi tocada — o loop do Arrange sobrevive"
    );
    assert_eq!(
        st.doc.active_loop_for(false),
        Some((0.0, 20.0)),
        "e o documento também não: navegar não é editar"
    );

    // Sair (container None) não faz nada: nada na cena era nosso para restaurar.
    on_container_nav_change(&st.doc, None, &mut container_ph);
    assert_eq!(
        container_ph.loop_range(),
        Some((0.5, 1.5)),
        "sair não mexe no relógio do container (o shell troca de clock, não este)"
    );
}

/// **O relógio do container parado FORA do loop dele é levado ao início** — a mesma cortesia
/// que a versão antiga fazia na cena, agora no relógio próprio.
#[test]
fn entering_from_outside_the_container_loop_seeks_to_its_start() {
    let (mut st, step) = nav_scene();
    st.doc
        .set_container_loop(step.container, Some((0.5, 1.5)), false);
    let mut container_ph = Playhead::new(1.0 / 60.0);
    container_ph.seek(1.9); // além do loop [0.5, 1.5]
    on_container_nav_change(&st.doc, Some(step.container), &mut container_ph);
    assert!(
        (container_ph.time() - 0.5).abs() < 1e-9,
        "fora do loop o relógio pousa no início dele, got {}",
        container_ph.time()
    );
}

/// **Um container SEM loop deixa o relógio onde está** — sem loop, sem seek, nada a armar.
#[test]
fn entering_a_container_without_a_loop_leaves_the_clock_as_it_stands() {
    let (st, step) = nav_scene(); // nav_scene não arma loop de container
    let mut container_ph = Playhead::new(1.0 / 60.0);
    container_ph.set_loop(3.0, 4.0); // um loop velho de uma visita anterior
    container_ph.seek(9.0);
    on_container_nav_change(&st.doc, Some(step.container), &mut container_ph);
    assert_eq!(
        container_ph.loop_range(),
        None,
        "o container não tem loop: limpa o velho"
    );
    assert!(
        (container_ph.time() - 9.0).abs() < 1e-9,
        "sem loop, sem seek"
    );
}

/// **Dentro de um container, Loop/PingPong escrevem o loop PRÓPRIO dele; ir-ao-início/fim
/// falam do relógio LOCAL** (Enio, 2026-07-22). Escrever `SetLoop` dali vazaria o loop da
/// CENA — o bug reportado.
#[test]
fn inside_a_container_the_loop_toggles_write_the_containers_own_loop() {
    let (mut st, step) = nav_scene();
    let c = step.container;
    st.edit_path = vec![step];
    let ph = Playhead::new(1.0 / 60.0);
    let ev = |on| PanelEvent::Toggle(ph2d_editor::ids::TIMELINE_LOOP, on);
    // O container tem 2 s de interior (a strip [0,2) dentro dele).
    let len = 2.0;

    assert_eq!(
        intent_for_transport(&ev(true), &st, &ph),
        Some(TimelineIntent::SetContainerLoop {
            container: c,
            range: Some((0.0, len)), // o interior INTEIRO do container, local
            ping_pong: false,
        }),
    );
    assert_eq!(
        intent_for_transport(&ev(false), &st, &ph),
        Some(TimelineIntent::SetContainerLoop {
            container: c,
            range: None,
            ping_pong: false,
        }),
    );
    assert_eq!(
        intent_for_transport(
            &PanelEvent::Toggle(ph2d_editor::ids::TIMELINE_PINGPONG, true),
            &st,
            &ph
        ),
        Some(TimelineIntent::SetContainerLoop {
            container: c,
            range: Some((0.0, len)),
            ping_pong: true,
        }),
    );
    assert_eq!(
        intent_for_transport(
            &PanelEvent::Click(ph2d_editor::ids::TIMELINE_GO_START),
            &st,
            &ph
        ),
        Some(TimelineIntent::Scrub(0.0)),
        "ir ao início vai ao 0 LOCAL do container"
    );
    assert_eq!(
        intent_for_transport(
            &PanelEvent::Click(ph2d_editor::ids::TIMELINE_GO_END),
            &st,
            &ph
        ),
        Some(TimelineIntent::Scrub(len)),
        "ir ao fim vai ao comprimento do container"
    );

    // Na RAIZ nada muda: o toggle segue escrevendo o loop do DOCUMENTO (a cena).
    st.edit_path.clear();
    assert!(
        matches!(
            intent_for_transport(&ev(true), &st, &ph),
            Some(TimelineIntent::SetLoop {
                range: Some(_),
                ping_pong: false
            })
        ),
        "na cena o Loop é do documento, como sempre"
    );
}

/// **`SetTransportLoop` arma o RELÓGIO e não toca o documento** — a metade que protege o
/// loop autorado da cena.
/// **O PLAYHEAD É LIVRE — a duração autorada NÃO é mais uma parede** (Enio, 2026-07-25). O
/// transporte também dirige a física dinâmica, e o clamp capava a simulação no fim da timeline.
/// Agora scrub/play atravessam o fim autorado em TODA vista (cena · Keys de clip · interior de
/// container) e o play NÃO pausa — a AVALIAÇÃO (o `cut`) segue clampando o relógio do avaliador,
/// mas o relógio do transporte corre. Mutação: re-introduzir o `pause()+seek(view_authored_end)`
/// no `run()` deixa este gate VERMELHO.
#[test]
fn the_playhead_runs_free_past_the_authored_end() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut st = TimelineState::new();
    let mut intents = Vec::new();
    let mut ak = super::super::autokey_pass::AutokeyState::default();
    let mut ph = Playhead::new(1.0 / 60.0);

    // SEM duracao autorada: um scrub alem do conteudo fica onde caiu (hoje).
    ph.seek(9.0);
    run(
        sim.world_mut(),
        &mut st,
        &mut ph,
        &mut intents,
        None,
        &mut ak,
        false,
        None,
    );
    assert!(
        (ph.time() - 9.0).abs() < 1e-9,
        "sem Dur: aberto como sempre"
    );

    // COM duracao autorada da CENA: o scrub alem do fim fica LIVRE, e o play NAO pausa (a
    // fisica dirigida pelo transporte precisa correr para alem da timeline).
    st.doc.set_scene_length(Some(2.0));
    run(
        sim.world_mut(),
        &mut st,
        &mut ph,
        &mut intents,
        None,
        &mut ak,
        false,
        None,
    );
    assert!(
        (ph.time() - 9.0).abs() < 1e-9,
        "livre alem do fim autorado da cena"
    );
    ph.play();
    ph.seek(5.0); // o play atravessa o fim neste frame — e segue tocando
    run(
        sim.world_mut(),
        &mut st,
        &mut ph,
        &mut intents,
        None,
        &mut ak,
        false,
        None,
    );
    assert!((ph.time() - 5.0).abs() < 1e-9, "o play nao e' preso no fim");
    assert!(
        ph.is_playing(),
        "play que alcanca o fim NAO pausa (o transporte dirige a fisica)"
    );

    // O escopo e' o da VISTA: em Keys quem manda e' o CLIP; a cena nao alcanca.
    let mut clip_ph = Playhead::new(1.0 / 60.0);
    clip_ph.seek(9.0);
    run(
        sim.world_mut(),
        &mut st,
        &mut clip_ph,
        &mut intents,
        None,
        &mut ak,
        true,
        None,
    );
    assert!((clip_ph.time() - 9.0).abs() < 1e-9, "clip sem Dur: livre");
    st.doc.set_clip_length_override(0, Some(1.0));
    run(
        sim.world_mut(),
        &mut st,
        &mut clip_ph,
        &mut intents,
        None,
        &mut ak,
        true,
        None,
    );
    assert!(
        (clip_ph.time() - 9.0).abs() < 1e-9,
        "o clip autorado NAO prende o Keys (livre)"
    );

    // E dentro de um container, o interior tambem e' livre.
    let c = st.doc.add_container("C".into());
    let mut cont_ph = Playhead::new(1.0 / 60.0);
    cont_ph.seek(9.0);
    st.doc.set_container_length_override(c, Some(1.5));
    run(
        sim.world_mut(),
        &mut st,
        &mut cont_ph,
        &mut intents,
        None,
        &mut ak,
        false,
        Some(c),
    );
    assert!(
        (cont_ph.time() - 9.0).abs() < 1e-9,
        "o container autorado NAO prende o interior (livre)"
    );
}

/// **O Dur de CLIP também NÃO prende o playhead sem pilha** (Enio, 2026-07-25): a aba Keys de
/// uma cena não-arranjada (`solo = false`, `container = None`) é onde uma cena de FÍSICA de um
/// clip só vive — e é justamente onde o clamp antigo mordia. Livre aqui é o que deixa a sim
/// correr para além da timeline. A AVALIAÇÃO do clip (o `cut`) segue intacta; só o relógio corre.
#[test]
fn a_clip_duration_no_longer_pins_the_playhead_with_no_stack() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut st = TimelineState::new();
    let mut intents = Vec::new();
    let mut ak = super::super::autokey_pass::AutokeyState::default();

    // Sem pilha, sem scene_length, um override de CLIP = 2 — o estado do screenshot.
    st.doc.set_clip_length_override(0, Some(2.0));

    // `solo = false` (nenhum stack para solar), `container = None` — exatamente o que
    // o shell passa na aba Keys de uma cena não-arranjada.
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.seek(2.333); // além do fim autorado — e fica onde caiu
    run(
        sim.world_mut(),
        &mut st,
        &mut ph,
        &mut intents,
        None,
        &mut ak,
        false, // solo — FALSO sem pilha, exatamente o caso da cena de fisica de 1 clip
        None,
    );
    assert!(
        (ph.time() - 2.333).abs() < 1e-9,
        "sem pilha o Dur do clip NAO prende o playhead — livre em 2.333"
    );

    // Play que atravessa o fim NAO pausa (o transporte dirige a fisica), também sem pilha.
    ph.play();
    ph.seek(5.0);
    run(
        sim.world_mut(),
        &mut st,
        &mut ph,
        &mut intents,
        None,
        &mut ak,
        false,
        None,
    );
    assert!((ph.time() - 5.0).abs() < 1e-9);
    assert!(
        ph.is_playing(),
        "o play que alcança o fim autorado NAO pausa"
    );
}

#[test]
fn set_transport_loop_arms_the_clock_and_leaves_the_document_alone() {
    let (mut st, _step) = nav_scene();
    let mut ph = Playhead::new(1.0 / 60.0);
    let before = st.doc.active_loop_for(false);
    ph2d_timeline::apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::SetTransportLoop {
            range: Some((2.0, 5.0)),
            ping_pong: true,
        },
    );
    assert_eq!(ph.loop_range(), Some((2.0, 5.0)));
    assert!(ph.is_ping_pong());
    assert_eq!(
        st.doc.active_loop_for(false),
        before,
        "o documento fica de fora"
    );
    ph2d_timeline::apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::SetTransportLoop {
            range: None,
            ping_pong: false,
        },
    );
    assert_eq!(ph.loop_range(), None, "off limpa o relógio");
}

/// **O report inteiro, end-to-end: ligar o loop DENTRO de um container não toca o loop do
/// Arrange** (Enio, 2026-07-22: *"o loop configurado no container está afetando o loop do
/// arrange"*). Dirige o caminho real — evento do toggle -> intent -> apply — nos dois modos.
#[test]
fn a_loop_toggled_inside_a_container_does_not_touch_the_arrange_loop() {
    let (mut st, step) = nav_scene();
    let c = step.container;
    // A cena tem o loop [0,20) (nav_scene o arma). Guarda-o.
    let arrange_before = st.doc.active_loop_for(false);
    assert_eq!(
        arrange_before,
        Some((0.0, 20.0)),
        "fixture: a cena tem loop"
    );

    // O animador entra no container e liga o Loop.
    st.edit_path = vec![step];
    let mut container_ph = Playhead::new(1.0 / 60.0);
    let intent = intent_for_transport(
        &PanelEvent::Toggle(ph2d_editor::ids::TIMELINE_LOOP, true),
        &st,
        &container_ph,
    )
    .expect("o toggle dentro do container vira um intent");
    ph2d_timeline::apply_intent(&mut st, &mut container_ph, intent);

    // O container ganhou um loop; a CENA não mudou.
    assert!(
        st.doc.container_loop(c).0.is_some(),
        "o container recebeu o loop dele"
    );
    assert_eq!(
        st.doc.active_loop_for(false),
        arrange_before,
        "o loop do ARRANGE ficou EXATAMENTE como estava — sem vazamento (o report)"
    );
    // E o relógio que rodou é o do container, não o da cena (que nem passou por aqui).
    assert!(
        container_ph.loop_range().is_some(),
        "o relógio do container ficou armado"
    );
}

/// **`run` dentro de um container reproduz o INTERIOR dele no relógio LOCAL** — o dispatch
/// de 3 vias do bridge, dirigido de verdade (Enio, 2026-07-22). O oráculo é a POSE: no
/// segundo 1 LOCAL do container a rampa está na metade (x=5); rodar a CENA (o bug) deixaria o
/// objeto em repouso, porque a instância só começa em 5 s na timeline.
#[test]
fn run_inside_a_container_plays_the_interior_at_its_own_clock() {
    use ph2d_ecs::{Entity, Name, Transform};
    use ph2d_timeline::{PropKind, StackHost, StripSource};

    let mut sim = ph2d_ecs::SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Mover")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    {
        let doc = &mut st.doc;
        doc.rename_clip(0, "Step".into());
        for (t, v) in [(0.0, 0.0), (2.0, 10.0)] {
            doc.upsert_key(
                bits,
                PropKind::TranslationX,
                ph2d_anim::RationalTime::from_seconds(t),
                ph2d_anim::AnimValue::Float(v),
                ph2d_anim::Interp::Linear,
            );
        }
        let walk = doc.add_container("Walk".into());
        doc.add_lane_in(StackHost::Container(walk), "in".into())
            .unwrap();
        doc.add_strip_to(
            StackHost::Container(walk),
            0,
            StripSource::Clip(0),
            0.0,
            2.0,
        )
        .unwrap();
        let lane = doc.add_lane("scene".into()).unwrap();
        doc.add_strip_to(
            StackHost::Document,
            lane,
            StripSource::Container(u16::try_from(walk).unwrap()),
            5.0,
            7.0,
        )
        .unwrap();
    }
    let mut intents = Vec::new();
    let mut ak = super::super::autokey_pass::AutokeyState::default();
    let mut container_ph = Playhead::new(1.0 / 60.0);
    container_ph.pause();
    container_ph.seek(1.0); // segundo 1 LOCAL do container

    run(
        sim.world_mut(),
        &mut st,
        &mut container_ph,
        &mut intents,
        None,
        &mut ak,
        false,   // não é Keys
        Some(0), // é o container 0
    );
    let x = f64::from(
        sim.world()
            .get::<Transform>(Entity::from_bits(bits))
            .unwrap()
            .translation
            .x,
    );
    assert!(
        (x - 5.0).abs() < 1e-4,
        "o bridge rodou o INTERIOR do container (x=5 na metade da rampa), não a cena; got {x}"
    );
}
