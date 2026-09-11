//! End-to-end reproduction of the "Vector handle does nothing" defect, at the
//! one layer the existing tests SKIP: `HeroScreen::apply_event` with the painter
//! panel actually INSTALLED in the process panel registry — the real-app
//! condition whenever the Painter tool is active.
//!
//! The editor-core unit test `point_type_menu_routes_each_kind_to_pending_index`
//! runs against an EMPTY registry (`ensure_panel_registry()` is a no-op there),
//! so it never exercises the painter panel's `apply_event` running BEFORE the
//! chrome handlers in `HeroScreen::apply_event`. This test installs the real
//! panel and drives the exact `Click(CTX_MENU_FALLOFF_HANDLE_*)` flow.

use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::{ErasedPanel, PanelRegistry, install_panel_registry};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::{NodeId, ids};
use ph2d_panel_painter_layers::PainterLayersPanel;

/// Install a process registry that mirrors the REAL shell: the heavy
/// context-menu-aware sibling panels (inspector, hierarchy, widget-gallery) +
/// the painter panel, in registry order (siblings BEFORE painter — the order
/// `ph2d-panel-registry-init` uses). This is the real-app condition the existing
/// editor-core unit test misses (it runs against an EMPTY registry). If ANY
/// sibling consumes `Click(CTX_MENU_FALLOFF_HANDLE_*)`, the chrome handler never
/// runs — the bug we are hunting. Idempotent (the `OnceLock` swallows re-install).
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
fn vector_handle_click_parks_pending_handle_with_panel_installed() {
    ensure_full_registry();

    // The exact two menu entries the FalloffPointHandle menu renders.
    for (entry_id, expected_wire) in [
        (ids::CTX_MENU_FALLOFF_HANDLE_VECTOR, 1u8),
        (ids::CTX_MENU_FALLOFF_HANDLE_AUTO, 0u8),
    ] {
        let mut hero = HeroScreen::new(NodeId(1));
        // Right-click on a control point opened this menu in the real shell.
        hero.store.open_context_menu(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: ContextMenuKind::FalloffPointHandle,
        });

        // The menu-item Click flows through HeroScreen::apply_event. With the
        // painter panel installed, its `apply_event` runs FIRST. If it consumes
        // (or the chrome walk is shadowed) the choice is silently dropped and
        // `pending_falloff_point_handle` stays None — the reported symptom.
        let handled = hero.apply_event(WidgetEvent::Click(entry_id));

        assert!(
            handled,
            "menu-item Click {entry_id:?} was not handled by anything — the chrome \
             falloff_handle path never ran (painter panel likely ate it)"
        );
        assert_eq!(
            hero.pending_falloff_point_handle,
            Some(expected_wire),
            "the Vector/Auto choice never reached chrome::falloff_handle — \
             pending_falloff_point_handle is {:?}, expected Some({expected_wire})",
            hero.pending_falloff_point_handle
        );
        assert!(
            hero.store.context_menu().is_none(),
            "menu must close after the pick"
        );
    }
}
