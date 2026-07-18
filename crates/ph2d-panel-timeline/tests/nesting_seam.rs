//! **Fatia 3b do nesting** ([plano](../../../docs/Timeline/04_plano_nesting.md) §5): a costura
//! que o artista de fato aperta — ADR-0133 §5.
//!
//! Estes gates CLICAM ([[feedback_widget_is_done_when_a_test_clicks_it]]). Um botão pintado e
//! não registrado no `WidgetStore` nunca recebe o Click, e um id registrado que ninguém
//! despacha é um botão morto — as duas metades já morderam este repo, e as duas estão aqui.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{StackHost, StripSource, TimelineDoc, TimelineViewSnapshot};
use ph2d_ui_testkit::MockPanelHost;

/// Drive one Click at `id` through the panel's real event path, and return the intents it
/// raised. Same harness the transport seam uses — a second one would drift.
fn click(id: ph2d_editor_core::NodeId) -> Vec<ph2d_timeline::TimelineIntent> {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let _ = host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(id));
    ph2d_panel_timeline::state::drain_intents()
}

/// Publish a snapshot that has one container instance on lane 0 of the document.
fn snapshot_with_container_strip() -> TimelineViewSnapshot {
    let mut doc = TimelineDoc::new();
    let c = doc.add_container("Walk".into());
    let lane = doc.add_lane("L".into()).unwrap();
    doc.add_strip_to(
        StackHost::Document,
        lane,
        StripSource::Container(c as u16),
        0.0,
        2.0,
    )
    .unwrap();
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    snap
}

/// **"+ Container" existe, está registrado e levanta o intent que nomeia.**
///
/// É o gate anti-botão-morto nos dois sentidos: se ele não estivesse no `WidgetStore`, o
/// `event` nunca veria o Click; se ninguém o despachasse, a lista de intents sairia vazia.
#[test]
fn the_add_container_button_raises_add_container() {
    let intents = click(ids::TIMELINE_ADD_CONTAINER);
    assert_eq!(intents.len(), 1, "um clique, um intent — veio {intents:?}");
    assert!(
        matches!(intents[0], ph2d_timeline::TimelineIntent::AddContainer),
        "o botão tem de levantar AddContainer, veio {:?}",
        intents[0]
    );
}

/// Estaciona o menu do strip como a produção faz: o Secondary Down abriu, o Down seguinte
/// FECHOU (deixando o pedido em `last_context_menu`), e só então chega o Click da linha.
fn park_strip_menu(host: &mut MockPanelHost, strip: ph2d_timeline::StripId) {
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    use ph2d_editor_core::panel::PanelHostInternal;
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::TimelineStrip {
            lane: 0,
            strip: strip.0,
        },
    });
    host.store_mut().close_context_menu();
}

/// **O ciclo COMPLETO: entrar por "Enter Container", sair pela raiz da trilha.**
///
/// ⚠️ A 1ª versão deste gate só clicava a raiz e afirmava `Document` — e `edit_host()` já
/// NASCE `Document`, então ele passaria com os dois botões mortos
/// ([[feedback_a_green_gate_may_be_green_by_accident]]). O oráculo só vale se o estado tiver
/// de MUDAR duas vezes: entra (vira `Container`), sai (volta a `Document`).
///
/// Entrar não levanta intent — move o estado de vista do painel, e é o `edit_host` que o shell
/// lê antes de drenar. Por isso o oráculo é o HOST, não a lista de intents.
#[test]
fn entering_by_the_menu_and_leaving_by_the_crumb_is_a_round_trip() {
    let snap = snapshot_with_container_strip();
    let strip = snap.lanes[0].strips[0].id;
    set_current_timeline(Some(snap));

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Document,
        "a cena é onde se começa"
    );

    park_strip_menu(&mut host, strip);
    let _ = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_TL_STRIP_ENTER),
    );
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Container(0),
        "'Enter Container' tem de ENTRAR — se isto falha, a linha do menu está morta"
    );

    let _ = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::TIMELINE_CRUMB[0]));
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Document,
        "o segmento raiz tem de VOLTAR para a cena"
    );
}

/// **"Enter Container" é inerte num strip de CLIP.** Uma linha que agisse no tipo errado seria
/// pior que uma que simplesmente não faz nada ali.
#[test]
fn entering_does_nothing_on_a_clip_strip() {
    let mut doc = TimelineDoc::new();
    let lane = doc.add_lane("L".into()).unwrap();
    let strip = doc.add_strip(lane, 0, 0.0, 2.0).unwrap();
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    set_current_timeline(Some(snap));

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    park_strip_menu(&mut host, strip);
    let _ = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_TL_STRIP_ENTER),
    );
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Document,
        "não há container para entrar: o host não pode ter se movido"
    );
}

/// **O snapshot mostra as lanes do host ABERTO, não as do documento.**
///
/// É a metade que decide se a breadcrumb significa alguma coisa: entrar sem que a tela troque
/// de conteúdo seria um botão que muda um número invisível.
#[test]
fn entering_a_container_swaps_which_lanes_the_panel_sees() {
    let mut doc = TimelineDoc::new();
    let c = doc.add_container("Walk".into());
    // O documento tem UMA lane; o container tem DUAS.
    doc.add_lane("doc".into()).unwrap();
    doc.add_lane_in(StackHost::Container(c), "a".into())
        .unwrap();
    doc.add_lane_in(StackHost::Container(c), "b".into())
        .unwrap();

    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    let mut snap = TimelineViewSnapshot::default();

    st.edit_host = StackHost::Document;
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    assert_eq!(snap.lanes.len(), 1, "fora: as lanes do documento");
    assert!(snap.crumbs.is_empty(), "fora: não há trilha a mostrar");

    st.edit_host = StackHost::Container(c);
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    assert_eq!(snap.lanes.len(), 2, "dentro: as lanes do container");
    assert_eq!(
        snap.crumbs,
        vec![(c, "Walk".to_string())],
        "dentro: a trilha diz onde você está"
    );
}

/// **O strip sabe dizer que é um container** — é disso que a linha "Enter Container" depende
/// para não agir sobre um strip de clip.
#[test]
fn a_container_strip_is_marked_as_one_and_a_clip_strip_is_not() {
    let snap = snapshot_with_container_strip();
    let s = &snap.lanes[0].strips[0];
    assert_eq!(s.container, Some(0), "o strip nomeia o container que toca");
    assert_eq!(s.clip_name, "Walk", "e mostra o NOME dele, não um branco");

    let mut doc = TimelineDoc::new();
    let lane = doc.add_lane("L".into()).unwrap();
    doc.add_strip(lane, 0, 0.0, 2.0).unwrap();
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    let mut plain = TimelineViewSnapshot::default();
    plain.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    assert_eq!(
        plain.lanes[0].strips[0].container, None,
        "um strip de CLIP não é container — sem isto a linha do menu agiria no strip errado"
    );
}

/// **A trilha é zero-largura na raiz.** Um documento que nunca usa containers não paga nem vê
/// nada — e é isso que permite pôr o item no flow do transporte sem custo para todo mundo.
#[test]
fn the_trail_costs_nothing_at_the_scene_root() {
    let snap = TimelineViewSnapshot::default();
    assert!(snap.crumbs.is_empty());
    assert!(
        (ph2d_panel_timeline::breadcrumb_width_for_tests(&snap) - 0.0).abs() < f32::EPSILON,
        "sem trilha, largura zero"
    );
}

/// **A régua DENTRO de um container mede o relógio DELE.**
///
/// Bug que a Fatia 3b introduziu e este gate nasceu vermelho sobre: entrar troca as lanes para
/// os segundos LOCAIS do container, e o playhead continuava marcando o segundo da TIMELINE.
/// Com a instância começando em 5 s, os dois discordavam por exatamente 5 s — a strip debaixo
/// do cursor não era a que o playhead dizia. É a doutrina "uma régua mede um relógio"
/// (`tab.rs`) um nível abaixo de onde ela foi escrita.
#[test]
fn the_ruler_inside_a_container_measures_the_containers_clock() {
    let mut doc = TimelineDoc::new();
    let c = doc.add_container("Walk".into());
    doc.add_lane_in(StackHost::Container(c), "l".into())
        .unwrap();
    doc.add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 4.0)
        .unwrap();
    // A instância começa em 5 s na timeline.
    let lane = doc.add_lane("doc".into()).unwrap();
    doc.add_strip_to(
        StackHost::Document,
        lane,
        StripSource::Container(c as u16),
        5.0,
        9.0,
    )
    .unwrap();

    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    st.edit_host = StackHost::Container(c);

    let mut ph = ph2d_core::Playhead::default();
    ph.seek(5.5); // meio segundo DENTRO da instância
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);

    assert_eq!(
        snap.host_time,
        Some(0.5),
        "dentro do container o playhead está a 0,5 s DELE (a instância começa em 5 s na \
         timeline); publicar 5,5 poria a marca meio traço fora do que as lanes mostram"
    );
    assert!(
        (snap.time_seconds - 5.5).abs() < 1e-12,
        "e o tempo da TIMELINE continua sendo o que é — os dois relógios coexistem, é isso \
         que as duas réguas mostram"
    );
}

/// **A régua do Arrange usa o relógio do container quando há um aberto** — e não desenha
/// playhead nenhum quando o container não toca exatamente uma vez aqui.
///
/// O snapshot publicar `host_time` não adianta se ninguém o ler: este gate pergunta à função
/// que a pintura de fato consulta ([[feedback_painted_is_not_populated_paint_gate]]).
#[test]
fn the_arrange_ruler_takes_its_clock_from_the_open_container() {
    use ph2d_panel_timeline::{ruler_clock_for_tests, tab::Tab};

    let mut snap = TimelineViewSnapshot {
        time_seconds: 5.5,
        ..Default::default()
    };

    // Fora: a régua é a timeline, e ela carrega os markers.
    let out = ruler_clock_for_tests(Tab::Arrange, &snap);
    assert_eq!(out.now, Some(5.5));
    assert!(out.markers, "fora, os markers são desta régua");

    // Dentro: o relógio do container, e os markers saem (são stamped em segundos da timeline).
    snap.crumbs = vec![(0, "Walk".into())];
    snap.host_time = Some(0.5);
    let inside = ruler_clock_for_tests(Tab::Arrange, &snap);
    assert_eq!(
        inside.now,
        Some(0.5),
        "a régua tem de marcar o segundo DO CONTAINER"
    );
    assert!(
        !inside.markers,
        "markers em segundos da timeline sentariam no segundo errado aqui"
    );

    // Dentro, mas o container não toca aqui: nenhum playhead é melhor que um mentiroso.
    snap.host_time = None;
    let nowhere = ruler_clock_for_tests(Tab::Arrange, &snap);
    assert_eq!(nowhere.now, None, "sem um 'agora' único, não se desenha um");
}
