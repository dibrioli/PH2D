//! Seam do **Resize Box** (plano UI/UX W3b) — o checkbox está vivo sob o MOUSE, chega ao
//! barramento, e a linha **não é pintada** onde não há resposta a mostrar.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — a lacuna que já deixou as 36 células da matriz de física e os dez chips de ferramenta
//! do Painter *pintados, hit-registrados e mortos sob o ponteiro*.

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

/// A seção Transform só existe com uma forma selecionada — o checkbox mora dentro dela.
fn arm(resize_box: Option<bool>) {
    state::set_current_transform(Some([0.0, 0.0, 100.0, 40.0]));
    state::set_resize_box(resize_box);
}

fn clear() {
    state::set_resize_box(None);
    state::set_current_transform(None);
}

fn rect(id: ph2d_a11y::NodeId) -> Option<Rect> {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
}

/// **O checkbox está vivo e o Click chega ao bus** — nos DOIS estados.
///
/// ⚠️ Os dois valores são exercitados de propósito: o `paint_checkbox` ramifica na marcação, e um
/// gate que só clicasse o estado marcado ficaria verde sobre um desmarcado que não pinta o
/// retângulo (e portanto não regista o hit).
#[test]
fn the_resize_box_checkbox_is_reachable_and_reaches_the_bus_in_both_states() {
    for checked in [true, false] {
        arm(Some(checked));
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        let r = host
            .painted_rect::<VectorPanel>(
                &mut panel_state,
                VIEWPORT,
                ids::VECTOR_TRANSFORM_RESIZE_BOX,
            )
            .unwrap_or_else(|| panic!("o checkbox ({checked}) nao foi PINTADO com area clicavel"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        assert!(
            evs.iter().any(
                |e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_TRANSFORM_RESIZE_BOX)
            ),
            "o ponteiro sobre o checkbox ({checked}) nao virou Click — ele esta' desenhado e nao \
             existe para o dispatcher (falta o `register` no populate)"
        );
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
        }
        assert!(
            host.drained_actions().into_iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(c))
                    if c == ids::VECTOR_TRANSFORM_RESIZE_BOX
            )),
            "o Click do checkbox ({checked}) nao chegou ao bus — ele acende sob o mouse e nao faz \
             nada (falta a linha na allowlist do event_clicks)"
        );
    }
    clear();
}

/// **Sem resposta publicada a linha não existe** — a metade da AUSÊNCIA.
///
/// ⚠️ Sem ela o gate acima ficaria verde sobre um checkbox que aparece em TODA seleção, incluindo
/// a múltipla — onde ele descreveria um objeto que não está lá e o clique não teria sujeito.
#[test]
fn the_row_is_not_painted_without_an_answer_to_show() {
    clear();
    state::set_current_transform(Some([0.0, 0.0, 100.0, 40.0]));
    assert!(
        rect(ids::VECTOR_TRANSFORM_RESIZE_BOX).is_none(),
        "o checkbox foi pintado sem resposta publicada (selecao multipla teria um controlo sem \
         sujeito)"
    );
    // E o controlo: com a resposta, ele existe.
    arm(Some(true));
    assert!(rect(ids::VECTOR_TRANSFORM_RESIZE_BOX).is_some());
    clear();
}
