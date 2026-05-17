//! Widget Gallery — floating reference panel that showcases every
//! canonical widget used by the editor.
//!
//! Triggered by clicking the palette icon in the TopBar
//! (`ids::TOPBAR_WIDGET_GALLERY`). Reuses the 10-section showcase
//! painters preserved in [`super::inspector`] as `dead_code` after
//! the live Inspector switched to entity-binding mode. The panel
//! itself is a Procreate-style floating surface — independent drag
//! state, doesn't dock to the four-zone layout, and renders as an
//! overlay on top of the chrome via [`super::HeroScreen`].
//!
//! Design intent: peripheral agents run `./play.command`, click the
//! palette pill, and see every widget rendered the way the editor
//! expects so they can replicate the decoration in their own
//! modules. Single in-app source of UI truth.

use crate::interaction::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore};
use crate::panel_registry::{PaintCtx, PanelManifest};
use crate::screens::hero::HeroScreen;
use crate::zones::Rect;

/// Wave 5 stage C+D — declarative panel manifest. Stage C ships a
/// no-op paint thunk (hero.rs still hard-codes the gallery's per-frame
/// paint block); stage D moves the per-frame logic here and collapses
/// `paint_hero_screen` to a registry iteration.
pub static PANEL_MANIFEST: PanelManifest = PanelManifest {
    id: "widget_gallery",
    panel_node_id: super::ids::GAL_PANEL,
    default_visible: false,
    paint_fn: paint_thunk,
    apply_event_fn: apply_event_thunk,
    populate_fn: populate,
};

#[allow(clippy::needless_pass_by_ref_mut)] // stage D fills the body
fn paint_thunk(_ctx: &mut PaintCtx) {}

fn apply_event_thunk(_hero: &mut HeroScreen, _ev: WidgetEvent) -> bool {
    false
}

/// Register the gallery's drag / resize handles as `BlenderHit`
/// children of `GAL_PANEL` so the existing
/// `BlenderHitKind::DragHandle` / `ResizeHandle` dispatch paths
/// move the panel without bespoke wiring. Called once from
/// `HeroScreen::pre_populate_store`.
pub fn populate(store: &mut WidgetStore) {
    use super::ids;
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
    // Close (X) — plain Button so the HeroScreen apply_event branch
    // (`id == GAL_CLOSE → widget_gallery_visible = false`) fires.
    store.register(
        ids::GAL_CLOSE,
        InteractiveState::Button {
            state: crate::widget::ButtonState::Normal,
        },
    );
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

/// Paint the gallery at `rect`. Delegates the entire showcase body
/// to [`super::inspector::paint_showcase_body`]; this wrapper exists
/// so the call site in `paint_hero_screen` stays self-documenting
/// (a single `widget_gallery::paint(...)` call) and so future
/// gallery-only features (filter / search / themed preview tabs) can
/// land here without churning `inspector.rs`.
pub fn paint(
    rect: Rect,
    scene: &mut ph2d_vector::VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut crate::interaction::HitIndex,
    store: &crate::interaction::WidgetStore,
) {
    super::inspector::paint_showcase_body(rect, scene, text_system, theme, hit_index, store);
}

/// Total height of the gallery body painted on the last frame.
/// Re-export of [`super::inspector::last_gallery_content_h`] so the
/// host (`paint_hero_screen`) can stay decoupled from the inspector
/// module when wiring the scroll clamp.
pub fn last_content_h() -> f32 {
    super::inspector::last_gallery_content_h()
}

/// Visible body region of the gallery on the last frame.
pub fn last_visible_h() -> f32 {
    super::inspector::last_gallery_visible_h()
}
