//! Seam da **BOOLEANA VIVA** — os três widgets estão vivos sob o MOUSE, não só pintados.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — foi essa lacuna que deixou as 36 células da matriz de física e os dez chips de
//! ferramenta do Painter *pintados, hit-registrados e mortos sob o ponteiro*.
//!
//! As duas metades de cada gate são independentes: sair do `populate_ops` mata a primeira (o
//! ponteiro não vira Click), sair do `event_clicks` mata a segunda (o Click não chega ao bus).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// Clica de verdade no widget `id` e devolve se o Click chegou ao barramento.
fn click_reaches_bus(id: ph2d_a11y::NodeId, what: &str) {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "o ponteiro sobre {what} nao virou Click — ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate_ops)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
        )),
        "o Click de {what} nao chegou ao bus — ele acende sob o mouse e nao faz nada (falta a \
         linha na allowlist do event_clicks)"
    );
}

/// **Os dois chips de modo estão vivos sob o ponteiro e chegam ao bus.**
#[test]
fn the_two_live_chips_are_reachable_and_reach_the_bus() {
    state::set_bool_group_selected(false);
    click_reaches_bus(ids::VECTOR_BOOL_LIVE_OFF, "o chip Live=Off");
    click_reaches_bus(ids::VECTOR_BOOL_LIVE_ON, "o chip Live=On");
}

/// **O Apply está vivo sob o ponteiro — quando é oferecido.**
#[test]
fn the_apply_button_is_reachable_when_offered() {
    state::set_bool_group_selected(true);
    click_reaches_bus(ids::VECTOR_BOOL_APPLY, "o botao Apply Boolean");
    state::set_bool_group_selected(false);
}

/// **O Apply só existe com uma booleana viva selecionada** — presença E ausência.
///
/// ⚠️ A metade da AUSÊNCIA é a que impede o botão morto: sem grupo não há o que consolidar, e um
/// *Apply* que não aplica nada é pior que *Apply* nenhum. É a mesma lei do Apply da simetria e
/// dos dois botões do corte.
#[test]
fn the_apply_button_appears_only_with_a_live_boolean_selected() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    state::set_bool_group_selected(false);
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_BOOL_APPLY)
            .is_none(),
        "o Apply foi pintado SEM booleana viva selecionada — um botao que nao aplica nada"
    );

    state::set_bool_group_selected(true);
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_BOOL_APPLY)
            .is_some(),
        "o Apply nao foi pintado COM booleana viva selecionada — o gesto nao tem saida"
    );
    state::set_bool_group_selected(false);
}

/// **As oito operações continuam pintadas e vivas com o modo LIGADO.**
///
/// ⚠️ O modo não é um nono botão: ele decide o que os oito FAZEM. Se ligar o modo escondesse (ou
/// matasse) qualquer um deles, metade das operações ficaria inalcançável no modo vivo — e o gate
/// de presença dos oito, que só corre com o default, nunca veria.
#[test]
fn the_eight_ops_survive_the_live_mode() {
    state::set_bool_live_on(true);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    for (id, what) in [
        (ids::VECTOR_BOOL_UNION, "Union"),
        (ids::VECTOR_BOOL_SUBTRACT, "Subtract"),
        (ids::VECTOR_BOOL_INTERSECT, "Intersect"),
        (ids::VECTOR_BOOL_EXCLUDE, "Exclude"),
        (ids::VECTOR_BOOL_MINUS_BACK, "Minus Back"),
        (ids::VECTOR_BOOL_TRIM, "Trim"),
        (ids::VECTOR_BOOL_CROP, "Crop"),
        (ids::VECTOR_BOOL_MERGE, "Merge"),
    ] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_some(),
            "{what} sumiu com o modo Live ligado"
        );
    }
    state::set_bool_live_on(false);
}

/// **OS QUATRO CHIPS DO VERBO POR FORMA ESTÃO VIVOS SOB O PONTEIRO.**
///
/// ⚠️ Este gate nasceu VERMELHO em 2026-08-23, depois de o Enio reportar *"os botões não
/// funcionam"* pela **segunda** vez. A primeira cura fez a fileira APARECER (o sujeito passou a ser
/// o primário); os chips continuavam **pintados, hit-registrados e mortos** — porque faltavam no
/// `populate_ops`, e sem o registro no store o ponteiro **nunca vira Click**.
///
/// É exactamente a falha que o doc deste arquivo já nomeia (as 36 células da física, os dez chips
/// do Painter), e a razão de os meus gates anteriores não a verem: eles provavam a allowlist e o
/// mapeamento com um `Click` **sintético**, que pula a checagem de focabilidade. *Só o gesto real
/// mede esta costura.*
#[test]
fn the_four_per_shape_verb_chips_are_reachable_and_reach_the_bus() {
    // A fileira só existe quando a shell publica um sujeito — é a mesma regra do Apply.
    state::set_bool_shape_row(Some((0, "Ellipse 2".to_string())));
    click_reaches_bus(ids::VECTOR_BOOL_SHAPE_UNION, "o chip Union desta forma");
    click_reaches_bus(ids::VECTOR_BOOL_SHAPE_SUBTRACT, "o chip Subtract desta forma");
    click_reaches_bus(ids::VECTOR_BOOL_SHAPE_INTERSECT, "o chip Intersect desta forma");
    click_reaches_bus(ids::VECTOR_BOOL_SHAPE_EXCLUDE, "o chip Exclude desta forma");
    state::set_bool_shape_row(None);
}

/// **A fileira só existe quando há sujeito** — presença E ausência, como o Apply.
///
/// ⚠️ A metade da AUSÊNCIA é a que impede quatro controlos mortos numa seleção que não tem forma
/// nenhuma a que eles se apliquem.
#[test]
fn the_per_shape_row_appears_only_with_a_subject() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    state::set_bool_shape_row(None);
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_BOOL_SHAPE_UNION)
            .is_none(),
        "a fileira foi pintada SEM sujeito — quatro controlos que nao mudam nada"
    );

    state::set_bool_shape_row(Some((1, "Rect 1".to_string())));
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_BOOL_SHAPE_UNION)
            .is_some(),
        "a fileira nao foi pintada COM sujeito — era o defeito de 22/08"
    );
    state::set_bool_shape_row(None);
}
