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

/// Floating-panel surface common to Inspector and Hierarchy: rounded
/// `BgElev` rect + 1px `Border` outline + a tiny drag-handle pill at
/// the top center.
pub(super) fn paint_panel_surface(rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = PANEL_RADIUS;
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    let handle = Rect::new(rect.x + (rect.w - 36.0) * 0.5, rect.y + 6.0, 36.0, 4.0);
    fill_rounded_rect(scene, handle, 999.0, resolve(ColorToken::BorderEmph, theme));
}

/// Three diagonal pips inside `rect` for a panel's bottom-right
/// resize gripper. Conventional UI affordance.
pub(super) fn paint_resize_gripper(scene: &mut VectorScene, rect: Rect, theme: Theme) {
    let color = resolve(ColorToken::BorderEmph, theme);
    let pip_size = 2.0;
    let pip_radius = 1.0;
    for i in 0..3 {
        let offset = 3.0 + i as f32 * 4.0;
        let r = Rect::new(
            rect.x + rect.w - offset - pip_size,
            rect.y + rect.h - offset - pip_size,
            pip_size,
            pip_size,
        );
        fill_rounded_rect(scene, r, pip_radius, color);
    }
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
