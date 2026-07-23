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

/// The panel events the bus carries — the same door `seam.rs` reads through.
fn timeline_events(host: &mut MockPanelHost) -> Vec<ph2d_editor_core::tool::PanelEvent> {
    host.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            ph2d_editor_core::action_bus::EditorAction::TimelinePanelEvent(pe) => Some(pe),
            _ => None,
        })
        .collect()
}

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
    assert_eq!(
        state.tab,
        ph2d_panel_timeline::tab::Tab::Containers,
        "entrar é uma troca de ABA: Arrange é sempre a CENA, então sem ela o clique deixaria \
         o artista numa aba que acabou de parar de publicar a trilha — nada visível mudaria"
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

    st.edit_path = Vec::new();
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    assert_eq!(snap.lanes.len(), 1, "fora: as lanes do documento");
    assert!(snap.crumbs.is_empty(), "fora: não há trilha a mostrar");

    // Só o CONTAINER roteia a vista; lane/strip do passo não participam aqui.
    st.edit_path = vec![ph2d_timeline::EnterStep {
        container: c,
        lane: 0,
        strip: Some(ph2d_timeline::StripId(0)),
    }];
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

/// **A régua DENTRO de um container mede o RELÓGIO PRÓPRIO dele** (Enio, 2026-07-22).
///
/// O transporte ali é o relógio do container, um clock autônomo — o shell entrega esse
/// playhead ao `rebuild`, então `time_seconds`/`host_time` SÃO o segundo local do interior,
/// por identidade. `container_open` marca a vista. Este é o modelo que substitui o mapa
/// cena→interior: o playback deixou de ser o do Arrange.
#[test]
fn the_ruler_inside_a_container_measures_the_containers_clock() {
    let mut doc = TimelineDoc::new();
    let c = doc.add_container("Walk".into());
    doc.add_lane_in(StackHost::Container(c), "l".into())
        .unwrap();
    doc.add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 4.0)
        .unwrap();
    let lane = doc.add_lane("doc".into()).unwrap();
    let strip = doc
        .add_strip_to(
            StackHost::Document,
            lane,
            StripSource::Container(c as u16),
            5.0,
            9.0,
        )
        .unwrap();

    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    st.edit_path = vec![ph2d_timeline::EnterStep {
        container: c,
        lane,
        strip: Some(strip),
    }];

    // O playhead entregue é o do CONTAINER — seu tempo é local (0,5 s DELE), não os 5,5 s
    // da timeline em que a instância toca.
    let mut ph = ph2d_core::Playhead::default();
    ph.seek(0.5);
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);

    assert_eq!(
        snap.container_open,
        Some(c),
        "a vista é marcada como a do container — a porta que a régua lê"
    );
    assert_eq!(
        snap.host_time,
        Some(0.5),
        "a régua marca o playhead LOCAL do container, por identidade"
    );
    assert!(
        (snap.time_seconds - 0.5).abs() < 1e-12,
        "e time_seconds É o relógio do container agora"
    );
    // O readout da migalha ainda é a relação com a CENA (onde a instância toca), intacto.
    assert!(
        snap.host_map.is_some_and(|m| (m.t0 - 5.0).abs() < 1e-9),
        "a instância toca a partir de 5 s na cena — o readout sobrevive à troca de relógio"
    );
}

/// **A régua do Arrange lê o relógio do container quando há um aberto** — pela porta
/// `container_open`, marcando o `time_seconds` local por identidade.
///
/// O snapshot marcar `container_open` não adianta se ninguém o ler: este gate pergunta à
/// função que a pintura de fato consulta ([[feedback_painted_is_not_populated_paint_gate]]).
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

    // Dentro (container aberto): o relógio local por identidade, e os markers saem (são
    // stamped em segundos da cena).
    snap.crumbs = vec![(0, "Walk".into())];
    snap.container_open = Some(0);
    snap.time_seconds = 0.5;
    let inside = ruler_clock_for_tests(Tab::Arrange, &snap);
    assert_eq!(
        inside.now,
        Some(0.5),
        "a régua marca o segundo LOCAL do container"
    );
    assert!(
        !inside.markers,
        "markers em segundos da cena sentariam no segundo errado aqui"
    );
}

/// Publica um snapshot com o container ABERTO e uma única instância começando em `start`,
/// com o playhead dentro dela. É a armação exata do bug: lanes em segundos do interior,
/// playhead em segundos da timeline.
fn open_container_at(start: f64, playhead: f64) -> ph2d_timeline::TimelineViewSnapshot {
    let mut doc = TimelineDoc::new();
    let c = doc.add_container("Walk".into());
    doc.add_lane_in(StackHost::Container(c), "in".into())
        .unwrap();
    doc.add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 8.0)
        .unwrap();
    let lane = doc.add_lane("L".into()).unwrap();
    let strip = doc
        .add_strip_to(
            StackHost::Document,
            lane,
            StripSource::Container(u16::try_from(c).unwrap()),
            start,
            start + 8.0,
        )
        .unwrap();
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    // O passo lembra o STRIP de entrada — é dele que o mapa/marca derivam agora.
    st.edit_path = vec![ph2d_timeline::EnterStep {
        container: c,
        lane,
        strip: Some(strip),
    }];
    let mut ph = ph2d_core::Playhead::default();
    ph.seek(playhead);
    st.doc.prime_stack(playhead);
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);
    snap
}

/// **O arrasto da régua DENTRO de um container busca o segundo LOCAL — identidade** (Enio,
/// 2026-07-22).
///
/// O transporte ali é o relógio próprio do container, então o eixo da régua e o clock que o
/// arrasto busca são o MESMO: o valor cru (`view_start + v·view_span`) já é o tempo. A antiga
/// conversão cena↔interior (via `host_map`) sumiu com o modelo de relógio único. Com span 8 e
/// o slider a 0,5, o arrasto busca o segundo 4 DO CONTAINER.
#[test]
fn scrubbing_inside_a_container_seeks_the_local_second() {
    ph2d_panel_timeline::state::set_current_timeline(Some(open_container_at(4.0, 5.0)));
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState {
        view_start_s: 0.0,
        view_span_s: 8.0, // a régua mostra os 8 s do INTERIOR
        tab: ph2d_panel_timeline::tab::Tab::Arrange, // a régua deste gate É a do Arrange
        ..TimelinePanelState::default()
    };
    host.set_slider_value(ids::TIMELINE_RULER, 0.5);
    let _ = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::TIMELINE_RULER),
    );
    let events = timeline_events(&mut host);
    ph2d_panel_timeline::state::set_current_timeline(None);
    assert_eq!(
        events,
        vec![ph2d_editor_core::tool::PanelEvent::SetValue(
            ids::TIMELINE_RULER,
            4.0
        )],
        "o eixo é o relógio do container: 0,5 sobre 8 s é o segundo 4 DELE, cru"
    );
}

/// **Na CENA é o mesmo — identidade** (o controle positivo do gate acima: os dois modos
/// buscam o valor cru, e nenhum deles converte).
#[test]
fn scrubbing_at_the_scene_root_still_seeks_the_raw_second() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState {
        view_start_s: 0.0,
        view_span_s: 8.0,
        tab: ph2d_panel_timeline::tab::Tab::Arrange, // a régua deste gate É a do Arrange
        ..TimelinePanelState::default()
    };
    host.set_slider_value(ids::TIMELINE_RULER, 0.5);
    let _ = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::TIMELINE_RULER),
    );
    let events = timeline_events(&mut host);
    assert_eq!(
        events,
        vec![ph2d_editor_core::tool::PanelEvent::SetValue(
            ids::TIMELINE_RULER,
            4.0
        )]
    );
}

/// **Dentro de um container o arrasto SEMPRE busca — o relógio é autônomo** (Enio,
/// 2026-07-22).
///
/// A antiga recusa "sem inverso, não busca nada" existia porque o scrub mapeava
/// interior→cena e podia não ter mapa. Com o relógio próprio do container isso deixou de
/// existir: não há mapa a faltar, então o arrasto é sempre o segundo local. Este gate pina a
/// ausência da recusa — o modo de falha que ela protegia (um segundo inventado) não é mais
/// alcançável por construção.
#[test]
fn inside_a_container_the_scrub_always_seeks_the_local_second() {
    let mut snap = open_container_at(4.0, 5.0);
    snap.host_map = None; // sem mapa: no modelo antigo isto RECUSAVA o arrasto
    ph2d_panel_timeline::state::set_current_timeline(Some(snap));
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState {
        view_start_s: 0.0,
        view_span_s: 8.0,
        tab: ph2d_panel_timeline::tab::Tab::Arrange,
        ..TimelinePanelState::default()
    };
    host.set_slider_value(ids::TIMELINE_RULER, 0.5);
    let _ = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::TIMELINE_RULER),
    );
    let events = timeline_events(&mut host);
    ph2d_panel_timeline::state::set_current_timeline(None);
    assert_eq!(
        events,
        vec![ph2d_editor_core::tool::PanelEvent::SetValue(
            ids::TIMELINE_RULER,
            4.0
        )],
        "sem mapa a régua ainda arrasta — o relógio do container não depende de um inverso"
    );
}

/// **Entrar DOIS níveis mantém o caminho de volta ao do meio.**
///
/// Enquanto a trilha guardava um único container, entrar em B por dentro de A publicava
/// `Scene > B` — uma afirmação ERRADA sobre onde você está — e o único jeito de sair era pular
/// para a cena. O caminho é o que faz a trilha dizer a verdade e o que dá o degrau do meio.
///
/// ⚠️ O gate segue o gesto REAL nos dois níveis (menu de strip → "Enter Container"), porque é a
/// costura que pode morrer; e o oráculo é o HOST, que é o que o shell lê antes de drenar.
#[test]
fn entering_two_deep_keeps_the_way_back_to_the_middle() {
    // Um documento com A dentro do documento, e B dentro de A.
    let mut doc = TimelineDoc::new();
    let a = doc.add_container("A".into());
    let b = doc.add_container("B".into());
    doc.add_lane_in(StackHost::Container(a), "inA".into())
        .unwrap();
    let inner = doc
        .add_strip_to(
            StackHost::Container(a),
            0,
            StripSource::Container(u16::try_from(b).unwrap()),
            0.0,
            2.0,
        )
        .unwrap();
    let lane = doc.add_lane("L".into()).unwrap();
    let outer = doc
        .add_strip_to(
            StackHost::Document,
            lane,
            StripSource::Container(u16::try_from(a).unwrap()),
            0.0,
            2.0,
        )
        .unwrap();
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;

    let publish = |st: &mut ph2d_timeline::TimelineState, path: Vec<ph2d_timeline::EnterStep>| {
        st.edit_path = path;
        let mut snap = TimelineViewSnapshot::default();
        snap.rebuild(st, &ph2d_core::Playhead::default(), false);
        set_current_timeline(Some(snap.clone()));
        snap
    };
    let enter = |host: &mut MockPanelHost, strip: ph2d_timeline::StripId| {
        let mut state = TimelinePanelState::default();
        park_strip_menu(host, strip);
        let _ = host.apply_panel_event::<TimelinePanel>(
            &mut state,
            WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_TL_STRIP_ENTER),
        );
    };

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let _ = publish(&mut st, Vec::new());
    enter(&mut host, outer);
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Container(a)
    );

    let _ = publish(&mut st, ph2d_panel_timeline::state::edit_path());
    enter(&mut host, inner);
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Container(b),
        "dois níveis: o de dentro"
    );
    let snap = publish(&mut st, ph2d_panel_timeline::state::edit_path());
    assert_eq!(
        snap.crumbs.iter().map(|c| c.0).collect::<Vec<_>>(),
        vec![a, b],
        "a trilha tem de publicar o CAMINHO inteiro, não só onde você parou"
    );

    // O degrau do meio: o segmento 1 é A.
    let mut state = TimelinePanelState::default();
    let _ = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::TIMELINE_CRUMB[1]));
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Container(a),
        "clicar o segmento do meio tem de voltar UM nível, não saltar para a cena"
    );
    let _ = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::TIMELINE_CRUMB[0]));
    assert_eq!(ph2d_panel_timeline::state::edit_host(), StackHost::Document);
}

/// **A trilha funda é de fato PINTADA, e cada segmento vira alvo clicável.**
///
/// Os gates acima provam a POLÍTICA (que profundidade cada slot popa); nenhum deles roda o
/// `paint`, e um segmento atrás de um `return` antecipado — ou um índice fora da régua de ids
/// — seria "a trilha não existe" com tudo verde ([[feedback_painted_is_not_populated_paint_gate]]).
/// Este pinta, e lê o que o clique de fato encontraria.
#[test]
fn a_deep_trail_paints_a_clickable_segment_for_every_level() {
    use ph2d_editor_core::zones::Rect;
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 900.0,
    };
    // Uma trilha mais funda que a régua de ids: a pintura não pode estourar o array, e tem de
    // parar exatamente nele.
    let deep = ids::TIMELINE_CRUMB.len() + 3;
    let snap = TimelineViewSnapshot {
        crumbs: (0..deep).map(|i| (i, format!("C{i}"))).collect(),
        ..TimelineViewSnapshot::default()
    };
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    set_current_timeline(Some(snap));
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let painted = ids::TIMELINE_CRUMB
        .iter()
        .filter(|id| regs.iter().any(|(r, _)| r == *id))
        .count();
    assert_eq!(
        painted,
        ids::TIMELINE_CRUMB.len(),
        "a trilha funda tem de encher a régua de ids -- e nunca passar dela"
    );
    // E o controle POSITIVO: na cena a trilha não pinta segmento nenhum.
    set_current_timeline(Some(TimelineViewSnapshot::default()));
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(
        !ids::TIMELINE_CRUMB
            .iter()
            .any(|id| regs.iter().any(|(r, _)| r == id)),
        "na raiz a trilha não paga um pixel"
    );
    set_current_timeline(None);
}

/// **Na aba KEYS dentro de um container, a régua scrubba o relógio do CLIP — valor CRU.**
///
/// O braço do scrub perguntava só à trilha ("estou dentro?") e ao mapa; na Keys a trilha
/// está cheia e os readouts do host publicam `None` (são Arrange-only desde o fix do panic)
/// ⇒ o gesto era ENGOLIDO e o playhead congelava (Enio, 2026-07-20: *"ao pular de jump para
/// keys, não consigo mover playhead"*). A pergunta é da ABA: a régua da Keys nem tem mapa a
/// aplicar.
#[test]
fn the_keys_ruler_scrubs_raw_inside_a_container() {
    use ph2d_panel_timeline::tab::Tab;
    let mut doc = TimelineDoc::new();
    let c = doc.add_container("Walk".into());
    doc.add_lane_in(StackHost::Container(c), "in".into())
        .unwrap();
    doc.add_strip_to(StackHost::Container(c), 0, StripSource::Clip(0), 0.0, 8.0)
        .unwrap();
    let lane = doc.add_lane("L".into()).unwrap();
    let strip = doc
        .add_strip_to(
            StackHost::Document,
            lane,
            StripSource::Container(u16::try_from(c).unwrap()),
            4.0,
            12.0,
        )
        .unwrap();
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    st.edit_path = vec![ph2d_timeline::EnterStep {
        container: c,
        lane,
        strip: Some(strip),
    }];
    let mut snap = TimelineViewSnapshot::default();
    // A publicação da ABA KEYS: o relógio passado é o do clip, keys_mode = true.
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), true);
    assert!(
        !snap.crumbs.is_empty() && snap.host_map.is_none(),
        "a armação do bug"
    );
    set_current_timeline(Some(snap));

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState {
        view_start_s: 0.0,
        view_span_s: 8.0,
        tab: Tab::Keys,
        ..TimelinePanelState::default()
    };
    host.set_slider_value(ids::TIMELINE_RULER, 0.5);
    let _ = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::TIMELINE_RULER),
    );
    let events = timeline_events(&mut host);
    set_current_timeline(None);
    assert_eq!(
        events,
        vec![ph2d_editor_core::tool::PanelEvent::SetValue(
            ids::TIMELINE_RULER,
            4.0
        )],
        "na Keys o segundo 4 da régua É o segundo 4 do clip — engolir o gesto congela o playhead"
    );
}

/// **Cada segmento da trilha leva à ABA que mostra AQUELE stack** — container → Containers,
/// raiz → Arrange.
///
/// O pop do segmento final é no-op de propósito; sem a troca de aba o clique não fazia NADA
/// na Keys, e não havia caminho direto de volta ao interior (Enio, 2026-07-20: *"em keys não
/// consigo voltar direto para Jump"*). O que mudou desde então é PARA ONDE ele volta: Arrange
/// é sempre a CENA (`tab::Tab::scene_root`), então o interior de um container mora na aba
/// Containers, e mandar os dois segmentos para a mesma aba faria um deles mentir.
#[test]
fn clicking_the_trail_from_keys_lands_on_the_tab_that_shows_that_stack() {
    use ph2d_panel_timeline::tab::Tab;
    let mut doc = TimelineDoc::new();
    let c = doc.add_container("Jump".into());
    doc.add_lane_in(StackHost::Container(c), "in".into())
        .unwrap();
    let lane = doc.add_lane("L".into()).unwrap();
    let strip = doc
        .add_strip_to(
            StackHost::Document,
            lane,
            StripSource::Container(u16::try_from(c).unwrap()),
            0.0,
            2.0,
        )
        .unwrap();
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;

    // Entra pelo GESTO real (menu do strip), como o gate de dois níveis.
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
        StackHost::Container(c)
    );

    // Na Keys, o clique no segmento FINAL ("Jump") não muda a profundidade — muda a ABA.
    state.tab = Tab::Keys;
    let _ = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::TIMELINE_CRUMB[1]));
    assert_eq!(
        state.tab,
        Tab::Containers,
        "o interior de um container é a aba Containers — Arrange é a CENA"
    );
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Container(c),
        "o segmento final não sai do container"
    );

    // E o "Scene" da Keys leva à CENA do Arrange.
    state.tab = Tab::Keys;
    let _ = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::TIMELINE_CRUMB[0]));
    set_current_timeline(None);
    assert_eq!(state.tab, Tab::Arrange);
    assert_eq!(ph2d_panel_timeline::state::edit_host(), StackHost::Document);
}
