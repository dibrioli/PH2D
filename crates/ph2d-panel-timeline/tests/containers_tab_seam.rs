//! **A aba Containers, a costura que o artista aperta** (ADR-0133, emendado 2026-07-21).
//!
//! Três abas, três donos: Keys é o clip, **Containers é um container**, **Arrange é sempre a
//! CENA**. O que estes gates protegem é a frase do meio — antes dela, entrar num container
//! reaproveitava o Arrange em silêncio, e criar um container despejava uma instância na cena
//! sem que ninguém pedisse. Era isso que fazia um container ler como *"apenas mais uma lane"*
//! (Enio, 2026-07-21).
//!
//! Todos CLICAM ([[feedback_widget_is_done_when_a_test_clicks_it]]).

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_panel_timeline::tab::Tab;
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{StackHost, StripSource, TimelineDoc, TimelineIntent, TimelineViewSnapshot};
use ph2d_ui_testkit::MockPanelHost;

/// Um documento com o container `Walk` (2 s de conteúdo dentro) e uma lane de cena vazia.
fn snapshot_with_container() -> TimelineViewSnapshot {
    let mut doc = TimelineDoc::new();
    let c = doc.add_container("Walk".into());
    let inner = doc
        .add_lane_in(StackHost::Container(c), "in".into())
        .unwrap();
    doc.add_strip_to(
        StackHost::Container(c),
        inner,
        StripSource::Clip(0),
        0.0,
        2.0,
    )
    .unwrap();
    doc.add_lane("L".into()).unwrap();
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    snap
}

/// Clica `id` com um snapshot publicado, devolvendo `(estado do painel, intents)`.
fn click_with(
    snap: TimelineViewSnapshot,
    state: &mut TimelinePanelState,
    id: ph2d_editor_core::NodeId,
) -> Vec<TimelineIntent> {
    let _ = ph2d_panel_timeline::state::drain_intents();
    set_current_timeline(Some(snap));
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let _ = host.apply_panel_event::<TimelinePanel>(state, WidgetEvent::Click(id));
    set_current_timeline(None);
    ph2d_panel_timeline::state::drain_intents()
}

/// **A aba Containers abre FORA de qualquer container** — o nível onde containers são FEITOS
/// (Enio, 2026-07-21).
///
/// O passo publicado aponta UMA ALÉM do fim da lista, fora de alcance de propósito: sem host
/// não há lanes, então ela não desenha a pilha da CENA sob outro nome (a duplicata que a
/// divisão em abas existe para evitar), e o cabeçalho oferece "+ Container" e mais nada. E é
/// o índice que o próximo `AddContainer` vai cunhar: o estado vazio e a primeira criação
/// nomeiam o mesmo lugar, então nada é previsto duas vezes.
#[test]
fn the_containers_tab_opens_outside_any_container() {
    let snap = snapshot_with_container();
    let n = snap.containers.len();
    let mut state = TimelinePanelState::default();
    let _ = click_with(snap, &mut state, ids::TIMELINE_TAB_CONTAINERS);

    assert_eq!(state.tab, Tab::Containers);
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Container(n),
        "uma além do fim: nenhum container é o host, então não há lanes a mostrar"
    );
    assert_eq!(
        ph2d_panel_timeline::state::open_container(),
        Some(n),
        "e é o índice que o próximo '+ Container' vai criar"
    );
}

/// **Arrange é a CENA, por mais fundo que a trilha esteja.**
///
/// A trilha não é apagada (voltar tem de aterrissar onde você estava), ela simplesmente não
/// se aplica ali. Sem esta lei, entrar num container reaproveitava o Arrange e não havia
/// resposta na tela para *"o que estou vendo?"* exceto a migalha.
#[test]
fn arrange_is_the_scene_however_deep_the_trail_is() {
    let mut state = TimelinePanelState::default();
    let _ = click_with(
        snapshot_with_container(),
        &mut state,
        ids::TIMELINE_TAB_CONTAINERS,
    );
    // Entra num container de verdade pelo picker de HOST — o controle da navegação.
    let _ = click_with(
        snapshot_with_container(),
        &mut state,
        ids::TIMELINE_HOST_OPT[0],
    );
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Container(0)
    );

    let _ = click_with(
        snapshot_with_container(),
        &mut state,
        ids::TIMELINE_TAB_ARRANGE,
    );
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Document,
        "no Arrange a edição cai na pilha da CENA"
    );
    assert!(
        ph2d_panel_timeline::state::edit_path().is_empty(),
        "e o caminho publicado é a raiz"
    );

    // ...e voltar aterrissa onde você estava: a trilha sobreviveu.
    let _ = click_with(
        snapshot_with_container(),
        &mut state,
        ids::TIMELINE_TAB_CONTAINERS,
    );
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Container(0),
        "a trilha não foi apagada pelo Arrange — só não se aplicava lá"
    );
}

/// **"+ Container" CRIA e ABRE — e não coloca nada.**
///
/// Os dois atos que estavam fundidos, agora separados: o intent faz o asset, o painel abre a
/// aba nele. O índice não é chute — é `containers.len()`, e o gate irmão em
/// `containers_are_assets.rs` prende `add_container` a apendar.
#[test]
fn add_container_makes_the_asset_and_opens_it() {
    let mut state = TimelinePanelState::default();
    let snap = snapshot_with_container();
    let existing = snap.containers.len();
    let intents = click_with(snap, &mut state, ids::TIMELINE_ADD_CONTAINER);

    assert_eq!(
        intents,
        vec![TimelineIntent::AddContainer],
        "UM intent, e nenhum AddStrip: criar não é colocar"
    );
    assert_eq!(state.tab, Tab::Containers, "e você cai dentro do que criou");
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Container(existing),
        "o container aberto é o que o intent está prestes a criar"
    );
}

/// **O `+` da lane coloca a FONTE selecionada — inclusive um container.**
///
/// Era o buraco: o `+` só sabia clip, então um container não tinha como ser colocado em lugar
/// nenhum, e "+ Container" tinha de criar E colocar numa tecla só.
#[test]
fn the_lane_plus_places_the_selected_container() {
    let mut state = TimelinePanelState {
        tab: Tab::Arrange,
        source_container: Some(0),
        ..TimelinePanelState::default()
    };
    let intents = click_with(
        snapshot_with_container(),
        &mut state,
        ids::TIMELINE_LANE_ADD_STRIP[0],
    );

    let placed = intents.iter().find_map(|i| match i {
        TimelineIntent::AddStrip {
            source,
            t_end,
            t_start,
            ..
        } => Some((*source, *t_end - *t_start)),
        _ => None,
    });
    let (source, span) = placed.expect("o `+` tem de colocar algo: {intents:?}");
    assert_eq!(
        source,
        StripSource::Container(0),
        "o que o dropdown nomeia é o que o `+` coloca"
    );
    assert!(
        (span - 2.0).abs() < 1e-9,
        "dimensionado pelo INTERIOR do container (2 s), não por um mínimo — senão nasceria \
         com uma velocidade que ninguém pediu; veio {span}"
    );
}

/// **Sem container selecionado o `+` continua colocando o CLIP ativo** — a regressão que
/// prova que a fonte é uma escolha, não uma troca de significado.
#[test]
fn the_lane_plus_still_places_the_active_clip_by_default() {
    let mut state = TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    };
    let intents = click_with(
        snapshot_with_container(),
        &mut state,
        ids::TIMELINE_LANE_ADD_STRIP[0],
    );
    assert!(
        intents.iter().any(|i| matches!(
            i,
            TimelineIntent::AddStrip {
                source: StripSource::Clip(0),
                ..
            }
        )),
        "{intents:?}"
    );
}

/// **Escolher uma FONTE nunca viaja; o picker de HOST viaja.** Duas perguntas, dois
/// controles.
///
/// Dentro do container A, escolher B na lista de FONTE tem de significar *"coloque B dentro
/// de A"* — o oposto de *"saia de A e vá editar B"*. Um controle que fizesse as duas coisas
/// não conseguiria expressar a primeira, que é justamente como se aninha.
#[test]
fn picking_a_source_never_travels_but_the_host_picker_does() {
    let snap = snapshot_with_container();
    let n = snap.containers.len();
    let mut state = TimelinePanelState::default();
    let _ = click_with(snap, &mut state, ids::TIMELINE_TAB_CONTAINERS);

    // FONTE: seleciona, e fica onde estava.
    let _ = click_with(
        snapshot_with_container(),
        &mut state,
        ids::TIMELINE_CONT_OPT[0],
    );
    assert_eq!(
        state.source_container,
        Some(0),
        "é a fonte que o `+` colocará"
    );
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Container(n),
        "escolher uma fonte NÃO pode mudar de lugar"
    );

    // HOST: viaja.
    let _ = click_with(
        snapshot_with_container(),
        &mut state,
        ids::TIMELINE_HOST_OPT[0],
    );
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Container(0),
        "o picker de host é quem leva você para dentro"
    );
    assert_eq!(
        state.source_container,
        Some(0),
        "e a fonte escolhida sobrevive à viagem — são estados independentes"
    );
}
