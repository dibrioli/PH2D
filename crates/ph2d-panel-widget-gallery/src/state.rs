//! Widget Gallery panel state — owned by `ErasedPanel<WidgetGalleryPanel>`
//! after ADR-0029 Phase C.3. Pre-Phase-C the panel's persistent rect lived
//! on `HeroScreen::view.widget_gallery_rect`; visibility flowed through
//! `HeroScreen::view.widget_gallery_visible`. Both moved here / to the
//! canonical visibility map per the Phase C model:
//!
//! - `rect: Option<Rect>` — persisted gallery geometry (drag/resize),
//!   lazy-initialized on first paint. Lives on the typed state.
//! - `visible` — flag migrated to `HeroScreen::panel_visibility` map
//!   keyed by `WidgetGalleryPanel::ID`.

use ph2d_editor_core::zones::Rect;

/// Widget Gallery retained state. Held inside
/// `ErasedPanel<WidgetGalleryPanel>` after Phase C.3.
#[derive(Clone, Debug, Default)]
pub struct WidgetGalleryState {
    /// Rect of the floating gallery panel in viewport pixels. Set on
    /// first toggle to a centered default; persisted across frames so
    /// dragging / resizing keeps the position.
    pub rect: Option<Rect>,
}
