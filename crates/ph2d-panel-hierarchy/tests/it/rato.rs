//! O rato pousa numa linha: em repouso a linha não pinta ícones à direita (interruptores no valor
//! normal e ícones de componente, `row_name.rs`), e um clique real neles começa por aqui.

use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_hierarchy::HierarchyPanel;
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_ui_testkit::MockPanelHost;

/// Pinta, pousa o rato no centro da linha `row` e pinta de novo (o quadro que já a vê apontada).
pub(crate) fn aponta(host: &mut MockPanelHost, st: &mut HierarchyState, view: Rect, row: NodeId) {
    let r = host
        .paint::<HierarchyPanel>(st, view)
        .into_iter()
        .rev()
        .find(|(id, r)| *id == row && r.w > 0.0)
        .map(|(_, r)| r)
        .unwrap_or_else(|| panic!("a linha {row:?} não foi pintada"));
    let _ = host.dispatch_pointer_event(PointerEvent {
        kind: PointerKind::Move,
        x: r.x + r.w * 0.5,
        y: r.y + r.h * 0.5,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: 0,
    });
}
