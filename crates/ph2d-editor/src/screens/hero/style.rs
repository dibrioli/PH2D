//! Shared layout constants + small style helpers for the hero
//! screen sub-modules. Lives here so each region module
//! (topbar/inspector/hierarchy/etc) can import from one place
//! instead of duplicating magic numbers.

use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

/// Default mockup viewport (iPad 12.9 landscape). Public so callers
/// like `shells/desktop` and tests can size their windows to match.
pub const HERO_VIEWPORT_W: f32 = 1366.0;
pub const HERO_VIEWPORT_H: f32 = 1024.0;

/// Padding from the screen edge to chrome (TopBar inset, Hierarchy
/// pinned-right inset, etc).
pub(super) const EDGE_PAD: f32 = 14.0;
pub(super) const TOPBAR_H: f32 = 40.0;
pub(super) const TOPBAR_GAP: f32 = 16.0;
pub(super) const RAIL_W: f32 = 56.0;
pub(super) const INSPECTOR_W: f32 = 304.0;
pub(super) const HIERARCHY_W: f32 = 308.0;
pub(super) const HUD_H: f32 = 34.0;
pub(super) const HUD_BOTTOM_PAD: f32 = 18.0;

/// Inspector + Hierarchy panel layout constants. Field/section
/// metrics were removed alongside the inspector placeholder
/// teardown — they'll be reintroduced when canonical sample
/// widgets land in the inspector body.
pub(super) const PANEL_RADIUS: f32 = 16.0;
pub(super) const PANEL_HEAD_PAD: f32 = 18.0;
pub(super) const HIER_ROW_H: f32 = 32.0;

/// Pixel size (square) of every panel's bottom-right resize-gripper
/// hit zone. Centralized so the painters + hit-zone registration use
/// one value.
pub(super) const PANEL_RESIZE_HANDLE_SIZE: f32 = 16.0;

/// Floating-panel surface — standard chrome for every panel
/// (Inspector, Hierarchy, and any future panel). Paints:
///   1. rounded `BgElev` fill + 1px `Border` stroke
///   2. drag-pill at the top center (`BorderEmph`)
///   3. resize-gripper dot at the bottom-right corner (`Text2`)
///
/// Centralizing the visual here means a single source of truth for
/// panel-chrome style; callers only register hit zones for the
/// drag pill / resize gripper via [`panel_drag_handle_rect`] /
/// [`panel_resize_handle_rect`].
pub(super) fn paint_panel_surface(rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = PANEL_RADIUS;
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    // Drag pill at the top center.
    let handle = Rect::new(rect.x + (rect.w - 36.0) * 0.5, rect.y + 6.0, 36.0, 4.0);
    fill_rounded_rect(scene, handle, 999.0, resolve(ColorToken::BorderEmph, theme));
    // Resize-gripper dot at the bottom-right corner, on the 45°
    // diagonal of the rounded edge. Soft `Text2` so it reads as a
    // corner accent rather than a foreign visual element.
    let dot_d = 4.0_f32;
    let inset = 7.0_f32;
    let dot = Rect::new(
        rect.x + rect.w - inset - dot_d,
        rect.y + rect.h - inset - dot_d,
        dot_d,
        dot_d,
    );
    fill_rounded_rect(scene, dot, dot_d * 0.5, resolve(ColorToken::Text2, theme));
}

/// Rect of the bottom-right resize-gripper hit zone for a panel
/// whose outer rect is `panel`. Callers register this against the
/// panel-specific `*_RESIZE_HANDLE` NodeId.
pub(super) fn panel_resize_handle_rect(panel: Rect) -> Rect {
    Rect::new(
        panel.x + panel.w - PANEL_RESIZE_HANDLE_SIZE,
        panel.y + panel.h - PANEL_RESIZE_HANDLE_SIZE,
        PANEL_RESIZE_HANDLE_SIZE,
        PANEL_RESIZE_HANDLE_SIZE,
    )
}

/// Rect of the top-center drag-pill hit zone for a panel whose outer
/// rect is `panel`. 80×14 — wide enough to grab on touch + mouse.
pub(super) fn panel_drag_handle_rect(panel: Rect) -> Rect {
    Rect::new(panel.x + (panel.w - 80.0) * 0.5, panel.y + 2.0, 80.0, 14.0)
}

/// Pick a chrome icon's foreground tint based on its interactive
/// state. Used by TopBar single-icon clusters and the LeftRail tools.
pub(super) fn icon_button_fg(state: ButtonState) -> ColorToken {
    match state {
        ButtonState::Hovered | ButtonState::Focused => ColorToken::Text1,
        ButtonState::Pressed => ColorToken::Accent,
        ButtonState::Disabled => ColorToken::TextDisabled,
        ButtonState::Normal | ButtonState::Loading => ColorToken::Text2,
    }
}
