//! **A aba Containers, a costura que o artista aperta** (ADR-0133, emendado 2026-07-21).
//!
//! Três abas, três donos: Keys é o clip, **Containers é a LISTA de containers do documento**,
//! **Arrange é sempre a CENA**. O que estes gates protegem é a frase do meio — antes dela,
//! entrar num container reaproveitava o Arrange em silêncio, criar um container despejava uma
//! instância na cena sem que ninguém pedisse, e a única forma de VER os containers era abrir
//! um menu. Era isso que fazia um container ler como *"apenas mais uma lane"* (Enio,
//! 2026-07-21).
//!
//! Todos CLICAM ([[feedback_widget_is_done_when_a_test_clicks_it]]). Os gates da NAVEGAÇÃO
//! (entrar por duplo-clique) moram em `src/container_list_tests.rs`: entrar é um gesto de
//! superfície, e o roteador dele é interno à crate.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_panel_timeline::state::{RenameKind, TimelinePanelState, set_current_timeline};
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

/// Clica `id` com um snapshot publicado, devolvendo os intents levantados.
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

const VIEWPORT: ph2d_editor_core::zones::Rect =
    ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0);

/// Pinta o painel de verdade e devolve o índice de hits que a pintura registrou.
fn paint(
    host: &mut MockPanelHost,
    state: &mut TimelinePanelState,
    snap: TimelineViewSnapshot,
) -> Vec<(ph2d_editor_core::NodeId, ph2d_editor_core::zones::Rect)> {
    set_current_timeline(Some(snap));
    let regs = host.paint::<TimelinePanel>(state, VIEWPORT);
    set_current_timeline(None);
    regs
}

fn has(
    regs: &[(ph2d_editor_core::NodeId, ph2d_editor_core::zones::Rect)],
    id: ph2d_editor_core::NodeId,
) -> bool {
    regs.iter().any(|(w, _)| *w == id)
}

/// **A LISTA existe na tela, com as três coisas que ela faz — e sem nenhuma das que ela não
/// faz.**
///
/// A pintura de verdade, e é onde os dois níveis da aba se separam ou não se separam: no
/// nível da lista há uma barra, um lápis e um lixo por container, "+ Container" e NENHUMA
/// strip; um nível abaixo é o exato oposto. Sem este gate, os dois pintores sobre a MESMA
/// faixa podem decidir que a vez é dos dois
/// ([[feedback_widget_is_done_when_a_test_clicks_it]]).
#[test]
fn the_list_paints_its_verbs_and_no_strip() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState {
        tab: Tab::Containers,
        ..TimelinePanelState::default()
    };
    let strip = ph2d_editor_core::ids::timeline_strip_hit_id(0, 1, 2);

    let regs = paint(&mut host, &mut state, snapshot_with_container());
    assert!(
        has(&regs, ids::TIMELINE_CONT_ROW[0]),
        "a barra do container"
    );
    assert!(has(&regs, ids::TIMELINE_CONT_RENAME[0]), "o lápis dela");
    assert!(has(&regs, ids::TIMELINE_CONT_DELETE[0]), "e o lixo dela");
    assert!(
        has(&regs, ids::TIMELINE_ADD_CONTAINER),
        "o único botão da coluna é '+ Container'"
    );
    assert!(!has(&regs, ids::TIMELINE_ADD_LANE));
    assert!(
        !has(&regs, strip) && !has(&regs, ids::TIMELINE_LANE_ADD_STRIP[0]),
        "a lista não pode desenhar as lanes da CENA sob outro nome"
    );
    // Uma linha só por container que EXISTE — nada de barras órfãs do array de ids.
    assert!(!has(&regs, ids::TIMELINE_CONT_ROW[1]));

    // Um nível abaixo (a shell publica a migalha): as lanes do container, e nenhuma barra.
    let mut inside = snapshot_with_container();
    inside.crumbs = vec![(0, "Walk".into())];
    let regs = paint(&mut host, &mut state, inside);
    assert!(has(&regs, ids::TIMELINE_ADD_LANE), "dentro é '+ Lane'");
    assert!(!has(&regs, ids::TIMELINE_ADD_CONTAINER));
    assert!(
        !has(&regs, ids::TIMELINE_CONT_ROW[0]),
        "e a lista não vaza para dentro do container"
    );
}

/// **A barra tem o TAMANHO do container — e um container vazio nasce com 2 segundos**
/// (Enio, 2026-07-21: *"a strip que representa o container Jump não apareceu … seu tamanho
/// inicial (quando o container é vazio) é de 2 segundos"*).
///
/// O rect registrado é o desenhado, então medi-lo aqui mede a tela: `Walk` tem 2 s de
/// conteúdo e `Empty` não tem nada, e as DUAS barras têm de medir 2 s — nem zero (o
/// duplo-clique sem onde pousar), nem a área de tempo inteira (a versão que "não apareceu").
#[test]
fn the_bar_is_sized_by_the_container_and_an_empty_one_is_born_two_seconds() {
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
    doc.add_container("Empty".into());
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState {
        tab: Tab::Containers,
        ..TimelinePanelState::default()
    };
    let regs = paint(&mut host, &mut state, snap);
    let rect_of = |id| {
        regs.iter()
            .find(|(w, _)| *w == id)
            .map(|(_, r)| *r)
            .expect("a barra tem de existir")
    };
    let (walk, empty) = (
        rect_of(ids::TIMELINE_CONT_ROW[0]),
        rect_of(ids::TIMELINE_CONT_ROW[1]),
    );
    #[expect(clippy::cast_possible_truncation, reason = "pixels de teste")]
    let two_seconds = (2.0 * state.px_per_s) as f32;
    assert!(
        (walk.w - two_seconds).abs() < 1.0,
        "Walk tem 2 s de conteúdo: barra de {two_seconds}px, veio {}",
        walk.w
    );
    assert!(
        (empty.w - two_seconds).abs() < 1.0,
        "Empty nasce com os MESMOS 2 s, veio {}",
        empty.w
    );
    assert!(
        (walk.x - empty.x).abs() < f32::EPSILON,
        "toda barra ancora no segundo 0 do PRÓPRIO eixo — {} vs {}",
        walk.x,
        empty.x
    );
}

/// **O lixo de uma linha apaga AQUELE container — e a seleção de fonte segue o asset.**
///
/// `source_container` é um índice na mesma lista que o delete encurta: sem o ajuste, apagar
/// o container 0 faria o `+` da lane passar a colocar o VIZINHO que escorregou para o slot,
/// em silêncio.
#[test]
fn the_trash_on_a_row_deletes_that_container_and_the_selection_follows() {
    // Dois containers: Walk (0) e Jump (1).
    let mut doc = TimelineDoc::new();
    doc.add_container("Walk".into());
    doc.add_container("Jump".into());
    doc.add_lane("L".into()).unwrap();
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);

    // Fonte = Jump (1); apagar Walk (0) → a fonte ainda é Jump, agora no slot 0.
    let mut state = TimelinePanelState {
        tab: Tab::Containers,
        source_container: Some(1),
        ..TimelinePanelState::default()
    };
    let intents = click_with(snap.clone(), &mut state, ids::TIMELINE_CONT_DELETE[0]);
    assert_eq!(intents, vec![TimelineIntent::RemoveContainer { index: 0 }]);
    assert_eq!(
        state.source_container,
        Some(0),
        "a seleção segue o ASSET, não o número do slot"
    );

    // Fonte = o próprio deletado → nenhuma seleção.
    state.source_container = Some(0);
    let _ = click_with(snap.clone(), &mut state, ids::TIMELINE_CONT_DELETE[0]);
    assert_eq!(state.source_container, None);

    // Uma linha que o snapshot não tem expira em silêncio.
    let intents = click_with(snap, &mut state, ids::TIMELINE_CONT_DELETE[7]);
    assert_eq!(intents, vec![]);
}

/// **A aba Containers abre na LISTA** — fora de qualquer container.
///
/// O caminho publicado é a RAIZ, e é isso que faz a faixa de linhas ser a lista em vez das
/// lanes de alguém. A primeira versão publicava um passo sentinela "uma além do fim" para
/// fingir esse nível; a lista o tornou desnecessário, e um índice que aponta para um
/// container inexistente é exatamente o tipo de estado que envenena a pergunta seguinte.
#[test]
fn the_containers_tab_opens_on_the_list() {
    let mut state = TimelinePanelState::default();
    let _ = click_with(
        snapshot_with_container(),
        &mut state,
        ids::TIMELINE_TAB_CONTAINERS,
    );

    assert_eq!(state.tab, Tab::Containers);
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Document,
        "a raiz da aba é a lista, não o interior de ninguém"
    );
    assert_eq!(ph2d_panel_timeline::state::open_container(), None);
}

/// **"+ Container" CRIA — e não coloca, e não viaja.**
///
/// Os três atos que já estiveram fundidos, agora separados: o intent faz o asset, a lista
/// ganha uma linha, e entrar é o duplo-clique nela. Fundido, o botão fazia a aba pular para
/// o modo de edição num aperto que dizia *"faça um"*.
#[test]
fn add_container_makes_the_asset_and_stays_on_the_list() {
    let mut state = TimelinePanelState {
        tab: Tab::Containers,
        ..TimelinePanelState::default()
    };
    let intents = click_with(
        snapshot_with_container(),
        &mut state,
        ids::TIMELINE_ADD_CONTAINER,
    );

    assert_eq!(
        intents,
        vec![TimelineIntent::AddContainer],
        "UM intent, e nenhum AddStrip: criar não é colocar"
    );
    assert_eq!(state.tab, Tab::Containers);
    assert_eq!(
        ph2d_panel_timeline::state::edit_host(),
        StackHost::Document,
        "e você continua olhando a lista — criar não é entrar"
    );
}

/// **O lápis de uma linha abre o rename DAQUELE container.**
///
/// Um botão e não um duplo-clique, porque o duplo-clique está ocupado: ele entra. E o índice
/// vem do array de ids, então o lápis da linha 0 não pode abrir o nome da linha 1.
#[test]
fn the_pencil_on_a_row_renames_that_container() {
    let mut state = TimelinePanelState {
        tab: Tab::Containers,
        ..TimelinePanelState::default()
    };
    let intents = click_with(
        snapshot_with_container(),
        &mut state,
        ids::TIMELINE_CONT_RENAME[0],
    );
    assert_eq!(intents, vec![], "abrir um campo não é uma edição");
    let cr = state.clip_rename.expect("o campo tem de abrir");
    assert_eq!(cr.kind, RenameKind::Container);
    assert_eq!(cr.index, 0);

    // Uma linha que o snapshot não tem não abre nada: a ação expira com o alvo.
    let mut state = TimelinePanelState::default();
    let _ = click_with(
        snapshot_with_container(),
        &mut state,
        ids::TIMELINE_CONT_RENAME[7],
    );
    assert!(state.clip_rename.is_none());
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

/// **Escolher uma FONTE nunca viaja.**
///
/// Dentro do container A, escolher B na lista de FONTE tem de significar *"coloque B dentro
/// de A"* — o oposto de *"saia de A e vá editar B"*. Um controle que fizesse as duas coisas
/// não conseguiria expressar a primeira, que é justamente como se aninha. É por isso que o
/// chip que navegava foi removido: viajar é o duplo-clique na lista.
#[test]
fn picking_a_source_never_travels() {
    let mut state = TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    };
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
        StackHost::Document,
        "escolher uma fonte NÃO pode mudar de lugar"
    );
    assert_eq!(state.tab, Tab::Arrange, "nem de aba");
}
