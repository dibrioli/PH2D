//! Seam do **LÁPIS** — o chip do 11º modo está vivo sob o MOUSE, não só pintado.
//!
//! O irmão em `seam.rs` dirige um `WidgetEvent::Click` **sintético**, que prova a allowlist do
//! painel e o switch da tool mas **pula a checagem de focabilidade no store** — foi essa lacuna
//! que deixou as 36 células da matriz de física, e os dez chips de ferramenta do Painter,
//! *pintados, hit-registrados e mortos sob o ponteiro*. Aqui o gesto é REAL: acha o retângulo que
//! o painel pintou, manda Down+Up nele, e exige que o Click nasça e chegue ao barramento.
//!
//! As duas metades são independentes: sair do `populate_modes` mata a 1ª asserção (o ponteiro não
//! vira Click), sair do `event_clicks` mata a 2ª (o Click não chega ao bus).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids};
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

/// **O chip Pencil é alcançável por um ponteiro e o clique chega ao bus.**
#[test]
fn the_pencil_chip_is_reachable_by_a_pointer_and_reaches_the_bus() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_MODE_PENCIL)
        .expect("o chip Pencil nao foi PINTADO com area clicavel na fileira TOOL");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_MODE_PENCIL)),
        "o ponteiro sobre o chip Pencil nao virou Click — ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate_modes)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == ids::VECTOR_MODE_PENCIL
        )),
        "o Click do Pencil nao chegou ao bus — o chip acende sob o mouse e nao faz nada (falta a \
         linha na allowlist do event_clicks)"
    );
}
