//! Widget Gallery panel populate — ADR-0029 Phase C.3 port of the
//! legacy `ph2d_editor_core::screens::hero::widget_gallery::populate`.
//!
//! Registers the gallery's drag / resize handles as `BlenderHit`
//! children of `GAL_PANEL` so the existing dispatch paths move the
//! panel without bespoke wiring. Also registers the close (X) button
//! as a plain `Button` so the typed `apply_event` branch can toggle
//! visibility off.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

pub(crate) fn populate(store: &mut WidgetStore) {
    store.register(
        ids::GAL_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::GAL_PANEL,
            kind: BlenderHitKind::DragHandle,
        },
    );
    store.register(
        ids::GAL_RESIZE_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::GAL_PANEL,
            kind: BlenderHitKind::ResizeHandle,
        },
    );
    store.register(
        ids::GAL_RESIZE_HANDLE_BL,
        InteractiveState::BlenderHit {
            parent: ids::GAL_PANEL,
            kind: BlenderHitKind::ResizeHandleBl,
        },
    );
    // Close (X) — plain Button so the panel's apply_event branch
    // (`id == GAL_CLOSE → panel_visible(widget_gallery) = false`) fires.
    store.register(
        ids::GAL_CLOSE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}
