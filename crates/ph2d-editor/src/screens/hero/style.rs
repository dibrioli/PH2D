//! Shared layout constants + small style helpers for the hero
//! screen sub-modules. Lives here so each region module
//! (topbar/inspector/hierarchy/etc) can import from one place
//! instead of duplicating magic numbers.

use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Density, Radius, SECTION_GAP_PX, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Default mockup viewport (iPad 12.9 landscape). Public so callers
/// like `shells/desktop` and tests can size their windows to match.
pub const HERO_VIEWPORT_W: f32 = 1366.0; // LITERAL-PX-OK: iPad 12.9 landscape viewport (fixture, not design scale)
pub const HERO_VIEWPORT_H: f32 = 1024.0; // LITERAL-PX-OK: iPad 12.9 landscape viewport (fixture)

/// Padding from the screen edge to chrome (TopBar inset, Hierarchy
/// pinned-right inset, etc).
pub(super) const EDGE_PAD: f32 = SECTION_GAP_PX;
pub(super) const TOPBAR_H: f32 = 40.0; // LITERAL-PX-OK: TopBar chrome height (specific dim, not on spacing scale)
pub(super) const TOPBAR_GAP: f32 = Spacing::Xl.px();
/// Mirrors `crate::widget::TOOL_RAIL_WIDTH_PX`. The hero layout uses
/// this for the rail's outer rect; the widget reuses it as a sizing
/// hint. Keep them in lockstep.
pub(super) const RAIL_W: f32 = crate::widget::TOOL_RAIL_WIDTH_PX;
pub(super) const INSPECTOR_W: f32 = 304.0; // LITERAL-PX-OK: Inspector panel fixed width (chrome-specific dim)
pub(super) const HIERARCHY_W: f32 = 308.0; // LITERAL-PX-OK: Hierarchy panel fixed width (chrome-specific dim)
pub(super) const HUD_H: f32 = 34.0; // LITERAL-PX-OK: HUD strip height (chrome-specific dim)
pub(super) const HUD_BOTTOM_PAD: f32 = 18.0; // LITERAL-PX-OK: HUD bottom inset (chrome-specific dim)

/// Inspector + Hierarchy panel layout constants. Field/section
/// metrics were removed alongside the inspector placeholder
/// teardown — they'll be reintroduced when canonical sample
/// widgets land in the inspector body.
pub(crate) const PANEL_RADIUS: f32 = Radius::Xl.px();
pub(super) const PANEL_HEAD_PAD: f32 = 18.0; // LITERAL-PX-OK: panel header inset (chrome-specific dim)
pub(super) const HIER_ROW_H: f32 = Density::Comfortable.row_h_px();

/// Pixel size (square) of every panel's bottom-right resize-gripper
/// hit zone. Centralized so the painters + hit-zone registration use
/// one value.
pub(crate) const PANEL_RESIZE_HANDLE_SIZE: f32 = Spacing::Xl.px();

/// Floating-panel surface — standard BASE chrome for every panel
/// (Inspector, Hierarchy, future panels). Paints the rounded
/// `BgElev` fill + 1px `Border` stroke + a tiny drag-pill at the
/// top center. Run BEFORE the panel body so the body renders on
/// top of the surface.
///
/// The bottom-right resize-gripper dot is painted separately by
/// [`paint_panel_corner_dot`] AFTER the body so body widgets don't
/// cover it (the body's scrollable clip extends into the corner).
pub(crate) fn paint_panel_surface(rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = PANEL_RADIUS;
    // PanelBg = BgElev hue/L with ~0.92 alpha → panel reads as
    // floating glass over canvas while text contrast holds.
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::PanelBg, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    // Drag pill at the top center.
    let handle = Rect::new(
        rect.x + (rect.w - 36.0) * 0.5, // LITERAL-PX-OK: drag pill width 36 (chrome-specific dim)
        rect.y + Spacing::Sm.px(),
        36.0, // LITERAL-PX-OK: drag pill width 36 (chrome-specific dim)
        4.0,  // LITERAL-PX-OK: drag pill height 4 (chrome-specific dim)
    );
    fill_rounded_rect(
        scene,
        handle,
        Radius::Full.px(),
        resolve(ColorToken::BorderEmph, theme),
    );
}

/// Bottom-right resize-gripper corner accent. Painted at the END of
/// each panel painter (after `pop_layer`) so it sits on top of any
/// body widget whose rect drifted into the corner. Soft `Text2`
/// reads as a corner accent rather than a foreign visual element.
pub(crate) fn paint_panel_corner_dot(rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let dot_d = Spacing::Xs.px();
    let inset = 7.0_f32; // LITERAL-PX-OK: corner-dot inset (specific accent geometry)
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
pub(crate) fn panel_resize_handle_rect(panel: Rect) -> Rect {
    Rect::new(
        panel.x + panel.w - PANEL_RESIZE_HANDLE_SIZE,
        panel.y + panel.h - PANEL_RESIZE_HANDLE_SIZE,
        PANEL_RESIZE_HANDLE_SIZE,
        PANEL_RESIZE_HANDLE_SIZE,
    )
}

/// Rect of the top-center drag-pill hit zone for a panel whose outer
/// rect is `panel`. 80×14 — wide enough to grab on touch + mouse.
pub(crate) fn panel_drag_handle_rect(panel: Rect) -> Rect {
    Rect::new(
        panel.x + (panel.w - 80.0) * 0.5, // LITERAL-PX-OK: drag hit-zone width 80 (chrome-specific)
        panel.y + Spacing::Xxs.px(),
        80.0, // LITERAL-PX-OK: drag hit-zone width 80
        SECTION_GAP_PX,
    )
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
