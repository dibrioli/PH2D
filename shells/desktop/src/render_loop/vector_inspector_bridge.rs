//! Vector Inspector panel ⟷ shell bridge (W2.T2.4).
//!
//! Three per-frame jobs (mirror of the painter bridge's panel plumbing):
//! 1. **Visibility** — show the vector inspector (right-dock) while a vector
//!    selection tool (`vector_select` / `vector_direct`) is active; hide the
//!    real Inspector (edge-triggered) so they don't both claim the slot.
//! 2. **Picker read-back** — a Down on the fill swatch opened the shared Blender
//!    picker (generic `is_picker_swatch` dispatch). While the picker targets the
//!    swatch, mirror the live picked sRGB8 into the swatch (`set_widget_color`)
//!    and `App.vector_fill_color`.
//! 3. **Publish** — push the current fill color to the panel so it paints the
//!    swatch fill.
//!
//! The **apply** (fill color → the selected regions, logged for undo) is the
//! Vector implementer's `apply-fill` part (T2.4 division). It joins HERE: pass
//! `&mut committed` + `&selection` and call the impl's helper inside the read-
//! back block once delivered.

use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;

/// Per-frame vector inspector plumbing. Safe to call every frame.
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    vector_fill_color: &mut [u8; 4],
) {
    let vector_active = tools
        .active()
        .map(|t| {
            let id = t.id();
            id == ph2d_editor::ToolId::new("vector_select")
                || id == ph2d_editor::ToolId::new("vector_direct")
        })
        .unwrap_or(false);

    // ── Visibility (mirror of the painter sidebar takeover) ───────────────
    hero.panel_visibility
        .insert("vector_inspector", vector_active);
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(vector_active, Ordering::Relaxed);
        if was != vector_active {
            // Edge-triggered: free/restore the shared Inspector slot.
            hero.panel_visibility.insert("inspector", !vector_active);
        }
    }

    // ── Picker read-back: live picked color → swatch + App fill color ─────
    if hero.store.picker_target() == Some(ph2d_editor::ids::VECTOR_INSPECTOR_FILL_SWATCH)
        && let Some((value, _, _, _)) = hero
            .store
            .blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER)
    {
        *vector_fill_color = value.rgba;
        hero.store
            .set_widget_color(ph2d_editor::ids::VECTOR_INSPECTOR_FILL_SWATCH, value.rgba);
        // APPLY JOIN (impl's apply-fill): color → fill of the selected regions
        // (insert_fill + SetRegionFill, logged for undo). Wire the impl's helper
        // here with `&mut committed` + `&selection` when delivered.
    }

    // ── Publish the current fill color so the panel paints the swatch ─────
    #[cfg(feature = "panel-vector-inspector")]
    ph2d_panel_vector_inspector::set_current_fill(*vector_fill_color);
}
