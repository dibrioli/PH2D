//! **A lista de containers, dirigida pelo gesto REAL** (ADR-0133, emendado 2026-07-21).
//!
//! Aqui e não em `tests/` porque a entrada num container é um GESTO de superfície, não o
//! clique de um widget: ele chega por `interact::dispatch_primary`, o mesmo roteador que o
//! painel roda, e essa função é interna à crate. Os gates de widget (o lápis, o
//! "+ Container", as abas) ficam no seam externo, que aperta os botões de verdade.

use super::*;
use ph2d_editor_core::interaction::{GestureMods, GesturePhase};
use ph2d_host::PointerButton;
use ph2d_timeline::{StackHost, StripSource, TimelineDoc, TimelineViewSnapshot};

/// Um documento com dois containers, o primeiro com 2 s de conteúdo dentro.
fn snap() -> TimelineViewSnapshot {
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
    doc.add_container("Jump".into());
    doc.add_lane("L".into()).unwrap();
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    let mut out = TimelineViewSnapshot::default();
    out.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    out
}

fn gesture(index: usize, phase: GesturePhase) -> ph2d_editor_core::interaction::TimelineGesture {
    ph2d_editor_core::interaction::TimelineGesture {
        surface: ids::TIMELINE_CONT_ROW[index],
        kind: ph2d_editor_core::interaction::TimelineHitKind::ContainerRow { index },
        phase,
        x: 0.0,
        y: 0.0,
        button: PointerButton::Primary,
        mods: GestureMods {
            shift: false,
            cmd: false,
            alt: false,
        },
    }
}

/// Roda o roteador de verdade, com o snapshot publicado (o `set_tab` interno lê-o).
fn feed(state: &mut TimelinePanelState, g: ph2d_editor_core::interaction::TimelineGesture) {
    crate::state::set_current_timeline(Some(snap()));
    crate::interact::dispatch_primary(state, 0.0, 120.0, &snap(), g);
    crate::state::set_current_timeline(None);
}

/// Um painel na aba Containers, na raiz (a LISTA).
fn on_the_list() -> TimelinePanelState {
    crate::state::pop_to_depth(0);
    TimelinePanelState {
        tab: crate::tab::Tab::Containers,
        ..TimelinePanelState::default()
    }
}

/// **A raiz da aba Containers é a LISTA dos containers do documento.**
///
/// É a frase inteira do Enio (*"a aba conteiner só serve como uma lista de containers
/// criados"*, 2026-07-21) medida onde ela decide a tela: quantas linhas a faixa tem, e do
/// que elas são feitas.
#[test]
fn the_root_of_the_containers_tab_lists_the_documents_containers() {
    let s = snap();
    let state = on_the_list();
    assert_eq!(
        crate::tab::rows(state.tab, &s),
        crate::tab::Rows::Containers
    );
    assert_eq!(
        geom::stack_bands(&s, state.tab, 0.0, 0.0).count(),
        s.containers.len(),
        "uma linha por container, nem as lanes da cena"
    );
    assert_eq!(s.containers.len(), 2, "a fixture tem de conter o fenômeno");
    // E o Arrange, com o MESMO snapshot, segue mostrando as LANES: a lista é da aba, não do
    // documento (controle positivo — sem ele "Containers" para tudo passaria).
    assert_eq!(
        crate::tab::rows(crate::tab::Tab::Arrange, &s),
        crate::tab::Rows::Lanes
    );
}

/// **O teto de containers É o array de ids desta lista.**
///
/// A metade do documento (ele recusa o 17º) tem gate próprio em `containers_are_assets.rs`;
/// esta é a metade que só o painel enxerga — crescer um sem o outro pinta uma linha que
/// ninguém pode renomear nem entrar, que é precisamente o modo de falha que o cap existe
/// para impedir.
#[test]
fn the_cap_is_the_lists_id_array() {
    assert_eq!(
        ph2d_timeline::MAX_CONTAINERS,
        ids::TIMELINE_CONT_RENAME.len()
    );
    assert_eq!(ph2d_timeline::MAX_CONTAINERS, ids::TIMELINE_CONT_ROW.len());
}

/// **Dois cliques na barra ENTRAM; um clique não faz nada.**
///
/// O "não faz nada" é a feature, não uma omissão: um clique simples que agisse faria a
/// primeira metade de todo duplo-clique agir junto. E arrastar não redimensiona porque a
/// barra não é um span — *"não pode ser redimensionada e nem pode sofrer nenhuma outra
/// operação que não seja Renomear e Entrar"* (Enio, 2026-07-21).
#[test]
fn a_double_click_enters_the_container_and_nothing_else_does() {
    for phase in [
        GesturePhase::Begin,
        GesturePhase::Update,
        GesturePhase::End,
        GesturePhase::Click,
    ] {
        let mut state = on_the_list();
        feed(&mut state, gesture(1, phase));
        assert_eq!(
            crate::state::edit_path().len(),
            0,
            "{phase:?} não pode entrar em lugar nenhum"
        );
        assert_eq!(
            crate::state::drain_intents(),
            vec![],
            "{phase:?} não pode editar o documento"
        );
    }
    let mut state = on_the_list();
    feed(&mut state, gesture(1, GesturePhase::DoubleClick));
    assert_eq!(
        crate::state::edit_host(),
        StackHost::Container(1),
        "o duplo-clique entra NO container da barra clicada"
    );
    assert_eq!(state.tab, crate::tab::Tab::Containers);
    crate::state::pop_to_depth(0);
}

/// **Dentro de um container as linhas voltam a ser LANES** — é o outro nível da mesma aba.
#[test]
fn inside_a_container_the_rows_are_its_lanes() {
    let mut state = on_the_list();
    feed(&mut state, gesture(0, GesturePhase::DoubleClick));
    // O snapshot que a shell publicaria com este caminho: uma migalha.
    let mut inside = snap();
    inside.crumbs = vec![(0, "Walk".into())];
    assert_eq!(
        crate::tab::rows(state.tab, &inside),
        crate::tab::Rows::Lanes
    );
    assert_eq!(
        crate::stack_add_header::add_kind(crate::tab::rows(state.tab, &inside)),
        Some(crate::stack_add_header::AddKind::Lane),
        "e o botão da coluna vira '+ Lane'"
    );
    crate::state::pop_to_depth(0);
}

/// **Tocar a aba em que você JÁ está devolve a lista** — a saída de um container.
///
/// Sem ela, entrar num container não teria volta para a lista sem passar pela cena: a
/// migalha "Scene" leva ao Arrange, que é outro lugar. É a convenção da barra de abas do
/// telefone, e o pill do Vector já a usa aqui.
#[test]
fn tapping_the_tab_you_are_on_returns_to_the_list() {
    let mut state = on_the_list();
    feed(&mut state, gesture(0, GesturePhase::DoubleClick));
    assert_eq!(crate::state::edit_host(), StackHost::Container(0));

    crate::state::set_current_timeline(Some(snap()));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    let _ = host.apply_panel_event::<crate::TimelinePanel>(
        &mut state,
        ph2d_editor_core::interaction::WidgetEvent::Click(ids::TIMELINE_TAB_CONTAINERS),
    );
    crate::state::set_current_timeline(None);
    assert_eq!(
        crate::state::edit_host(),
        StackHost::Document,
        "de volta à raiz, que na aba Containers é a LISTA"
    );

    // ⚠️ E vir de OUTRA aba não pode derrubar a trilha: sair para as Keys e voltar aterrissa
    // onde você estava. É por isso que a comparação acontece ANTES da troca.
    feed(&mut state, gesture(1, GesturePhase::DoubleClick));
    for tab in [ids::TIMELINE_TAB_KEYS, ids::TIMELINE_TAB_CONTAINERS] {
        crate::state::set_current_timeline(Some(snap()));
        let _ = host.apply_panel_event::<crate::TimelinePanel>(
            &mut state,
            ph2d_editor_core::interaction::WidgetEvent::Click(tab),
        );
        crate::state::set_current_timeline(None);
    }
    assert_eq!(
        crate::state::edit_host(),
        StackHost::Container(1),
        "Keys -> Containers volta para dentro, não para a lista"
    );
    crate::state::pop_to_depth(0);
}

/// **Arrange é a CENA, por mais fundo que a trilha esteja** (a lei do `scene_root`).
///
/// A trilha não é apagada — voltar tem de aterrissar onde você estava — ela simplesmente
/// não se aplica ali.
#[test]
fn arrange_is_the_scene_however_deep_the_trail_is() {
    let mut state = on_the_list();
    feed(&mut state, gesture(0, GesturePhase::DoubleClick));

    crate::state::set_tab(&mut state, crate::tab::Tab::Arrange);
    assert_eq!(crate::state::edit_host(), StackHost::Document);
    assert!(crate::state::edit_path().is_empty());

    crate::state::set_tab(&mut state, crate::tab::Tab::Containers);
    assert_eq!(
        crate::state::edit_host(),
        StackHost::Container(0),
        "a trilha não foi apagada pelo Arrange — só não se aplicava lá"
    );
    crate::state::pop_to_depth(0);
}

/// **A âncora do rename é a linha do container que ele renomeia**, e some quando a linha
/// sai da faixa visível.
///
/// Um campo de texto flutuando sobre a linha errada renomearia o que o artista está
/// olhando, não o que ele pediu.
#[test]
fn the_rename_field_floats_over_the_row_it_renames() {
    let s = snap();
    let g = geom::resolve(
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 800.0, 400.0),
        40.0,
        200.0,
        geom::MIN_LANE_LABEL_W,
    );
    let mut state = on_the_list();
    assert_eq!(
        rename_anchor(&g, &state, &s),
        None,
        "sem rename aberto não há âncora"
    );
    crate::clip_rename::open_container(&mut state, 1);
    let r = rename_anchor(&g, &state, &s).expect("a linha 1 está na faixa");
    let (_, y, h) = geom::stack_bands(&s, state.tab, g.rows.y, 0.0)
        .find(|(i, _, _)| *i == 1)
        .unwrap();
    assert!(
        (r.y - y).abs() < f32::EPSILON && (r.h - h).abs() < f32::EPSILON,
        "a âncora tem de ser a linha 1, veio {r:?}"
    );
    assert!(
        (r.w - g.label_w).abs() < f32::EPSILON,
        "e ocupa a coluna de rótulo, onde o nome está desenhado"
    );
    // Rolada para fora, não há onde ancorar — e o campo não é pintado sobre a régua.
    state.scroll_y = 10_000.0;
    assert_eq!(rename_anchor(&g, &state, &s), None);
}

/// **O campo commita no intent do TIPO que ele abriu.**
///
/// Um campo, duas listas — e é aqui que "uma porta" pode virar "a porta errada": digitar no
/// nome de um container e renomear um CLIP é a falha que o `RenameKind` existe para impedir,
/// e ela seria invisível (o campo fecha, um nome muda, só não o que estava na tela).
#[test]
fn the_field_commits_the_intent_of_the_kind_it_opened() {
    use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
    use ph2d_editor_core::widget::TextInputState;
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        ids::TIMELINE_CLIP_RENAME_INPUT,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: "Jump".to_string(),
            caret: 4,
            selection_anchor: None,
        },
    );

    let _ = crate::state::drain_intents();
    let mut state = on_the_list();
    crate::clip_rename::open_container(&mut state, 1);
    crate::clip_rename::commit(&mut state, &store);
    assert_eq!(
        crate::state::drain_intents(),
        vec![ph2d_timeline::TimelineIntent::RenameContainer {
            index: 1,
            name: "Jump".to_string(),
        }]
    );

    // Controle positivo: o MESMO campo, aberto como CLIP, commita o intent do clip. Sem ele
    // "sempre RenameContainer" passaria neste gate.
    crate::state::set_current_timeline(Some(snap()));
    crate::clip_rename::open(&mut state, &snap());
    crate::state::set_current_timeline(None);
    crate::clip_rename::commit(&mut state, &store);
    assert!(matches!(
        crate::state::drain_intents().as_slice(),
        [ph2d_timeline::TimelineIntent::RenameClip { .. }]
    ));
}

/// **Um rename de CLIP não ancora numa linha** — ele flutua sobre o chip que o nomeia.
/// Uma porta, duas respostas, e o `RenameKind` é quem as separa.
#[test]
fn a_clip_rename_does_not_take_the_rows_anchor() {
    let s = snap();
    let g = geom::resolve(
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 800.0, 400.0),
        40.0,
        200.0,
        geom::MIN_LANE_LABEL_W,
    );
    let mut state = on_the_list();
    state.clip_rename = Some(crate::state::ClipRename {
        kind: crate::state::RenameKind::Clip,
        index: 0,
        opened: false,
    });
    assert_eq!(rename_anchor(&g, &state, &s), None);
}
