//! End-to-end check that the on-canvas Curve / Free Hand point **handle-kind** menu routes through
//! `HeroScreen::apply_event` with the painter panel actually INSTALLED in the process registry — the
//! real-app condition whenever the Painter tool is active. A sibling panel running its `apply_event`
//! BEFORE the chrome handlers could swallow the `Click(CTX_MENU_CURVE_HANDLE_*)`; this pins that it
//! reaches `chrome::curve_point_handle` and parks the wire u8 in `pending_curve_point_handle`.
//!
//! Mirror of `falloff_handle_menu_e2e.rs` (same registry shape, same hazard).

use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::{ErasedPanel, PanelRegistry, install_panel_registry};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::{NodeId, ids};
use ph2d_panel_painter_layers::PainterLayersPanel;

/// Install a process registry mirroring the REAL shell: the context-menu-aware sibling panels +
/// the painter panel, in registry order (siblings BEFORE painter). Idempotent.
fn ensure_full_registry() {
    let mut reg = PanelRegistry::new_empty();
    reg.push(ErasedPanel::new::<ph2d_panel_hierarchy::HierarchyPanel>());
    reg.push(ErasedPanel::new::<ph2d_panel_inspector::InspectorPanel>());
    reg.push(ErasedPanel::new::<
        ph2d_panel_widget_gallery::WidgetGalleryPanel,
    >());
    reg.push(ErasedPanel::new::<PainterLayersPanel>());
    let _ = install_panel_registry(reg);
}

#[test]
fn curve_handle_click_parks_pending_kind_with_panel_installed() {
    ensure_full_registry();

    // The exact five entries the CurvePointHandle menu renders, with their wire u8.
    for (entry_id, expected_wire) in [
        (ids::CTX_MENU_CURVE_HANDLE_FREE, 0u8),
        (ids::CTX_MENU_CURVE_HANDLE_ALIGNED, 1),
        (ids::CTX_MENU_CURVE_HANDLE_VECTOR, 2),
        (ids::CTX_MENU_CURVE_HANDLE_AUTO, 3),
        (ids::CTX_MENU_CURVE_HANDLE_SYMMETRIC, 4),
    ] {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store.open_context_menu(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: ContextMenuKind::CurvePointHandle,
        });

        let handled = hero.apply_event(WidgetEvent::Click(entry_id));

        assert!(
            handled,
            "menu-item Click {entry_id:?} was not handled — the chrome curve_point_handle path never ran \
             (a sibling panel likely ate it)"
        );
        assert_eq!(
            hero.pending_curve_point_handle, Some(expected_wire),
            "the handle-kind choice never reached chrome::curve_point_handle — pending is {:?}, expected \
             Some({expected_wire})",
            hero.pending_curve_point_handle
        );
        assert!(
            hero.store.context_menu().is_none(),
            "menu must close after the pick"
        );
    }
}
