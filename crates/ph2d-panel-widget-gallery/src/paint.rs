//! Widget Gallery panel paint — ADR-0029 Phase C.3 port of the legacy
//! `paint_thunk`.
//!
//! Full per-frame logic:
//! - Visibility gate via [`PanelHostInternal::panel_visible`].
//! - Stale-rect cleanup on hide so a closed gallery doesn't shadow
//!   `GS_PANEL` in `panel_at` lookups (BTreeMap key order, scroll
//!   wheel routing — the comment in the legacy thunk explains the
//!   regression this guards against).
//! - Lazy default rect on first paint (writes back to `state.rect`).
//! - Drag / resize clamp via `widget::panel_chrome::clamp_panel_rect`.
//! - Chrome publish (`set_panel_rect`).
//! - Delegated showcase paint (`widget::showcase::paint_showcase_body`).
//! - `content_h` / `visible_h` publish + scroll clamp.

use crate::WidgetGalleryPanel;
use crate::state::WidgetGalleryState;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::clamp_panel_rect;
use ph2d_editor_core::widget::showcase::{
    last_gallery_content_h, last_gallery_visible_h, paint_showcase_body,
};
use ph2d_editor_core::zones::Rect;

pub(crate) fn paint(state: &mut WidgetGalleryState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(WidgetGalleryPanel::ID) {
        // Stale-rect cleanup: without this, `panel_at` keeps returning
        // GAL_PANEL after the Gallery was closed, and BTreeMap key
        // ordering (950 < 1000) makes that stale rect shadow GS_PANEL
        // for overlapping cursor positions — wheel scroll on the Grid
        // Settings panel then routes to a closed GAL_PANEL and looks
        // like "scroll wheel doesn't work" to the user.
        ctx.host.store_mut().clear_panel_rect(ids::GAL_PANEL);
        return;
    }
    let base_rect = match state.rect {
        Some(r) => r,
        None => {
            let r = default_rect(
                ctx.layout.viewport.w,
                ctx.layout.viewport.h,
                ctx.layout.inspector.w,
            );
            state.rect = Some(r);
            r
        }
    };
    let viewport = ctx.viewport;
    // Read drag/resize deltas and clamp, then write back any clamp
    // corrections + publish the final panel rect to the store. The
    // joint `store_and_hit_index_mut` returns `&WidgetStore`
    // immutable, so we use `store_mut()` for the writes here.
    let store_imm = ctx.host.store();
    let gal_off = store_imm.blender_picker_offset(ids::GAL_PANEL);
    let gal_resize = store_imm.panel_resize_delta(ids::GAL_PANEL);
    let (gallery_rect, gal_clamped_off, gal_clamped_resize) =
        clamp_panel_rect(base_rect, gal_off, gal_resize, viewport);
    {
        let store = ctx.host.store_mut();
        if (gal_clamped_off.0 - gal_off.0).abs() > f32::EPSILON
            || (gal_clamped_off.1 - gal_off.1).abs() > f32::EPSILON
        {
            store.set_blender_picker_offset(ids::GAL_PANEL, gal_clamped_off.0, gal_clamped_off.1);
        }
        if (gal_clamped_resize.0 - gal_resize.0).abs() > f32::EPSILON
            || (gal_clamped_resize.1 - gal_resize.1).abs() > f32::EPSILON
        {
            store.set_panel_resize_delta(
                ids::GAL_PANEL,
                gal_clamped_resize.0,
                gal_clamped_resize.1,
            );
        }
        store.set_panel_rect(ids::GAL_PANEL, gallery_rect);
    }
    let theme = ctx.host.theme();
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_showcase_body(
            gallery_rect,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
        );
    }
    let content_h = last_gallery_content_h();
    let visible_h = last_gallery_visible_h();
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::GAL_PANEL, content_h);
    store.set_panel_visible_h(ids::GAL_PANEL, visible_h);
    let max_scroll = (content_h - visible_h).max(0.0);
    let cur = store.panel_scroll(ids::GAL_PANEL);
    if cur > max_scroll {
        store.set_panel_scroll(ids::GAL_PANEL, max_scroll);
    }
}

/// Default panel geometry when the gallery is first opened. Width
/// matches the live Inspector (`layout.inspector.w`) so the showcase
/// renders at the same dimensions peripheral agents will see when
/// the actual panel reads it. Height clamps to the viewport with
/// 8 px breathing margins.
pub fn default_rect(viewport_w: f32, viewport_h: f32, inspector_w: f32) -> Rect {
    let w = inspector_w.max(280.0).min(viewport_w - 16.0); // LITERAL-PX-OK: gallery min width 280 + viewport margin 16 (chrome-specific)
    let h = 720.0_f32.min(viewport_h - 16.0).max(420.0); // LITERAL-PX-OK: gallery default height 720 + viewport margin + min 420 (chrome-specific)
    let x = ((viewport_w - w) * 0.5).max(8.0); // LITERAL-PX-OK: viewport edge inset
    let y = ((viewport_h - h) * 0.5).max(8.0); // LITERAL-PX-OK: viewport edge inset
    Rect::new(x, y, w, h)
}
