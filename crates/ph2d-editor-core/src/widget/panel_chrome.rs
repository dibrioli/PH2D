//! Shared floating-panel chrome — paint helpers + hit-zone rects +
//! clamp math used by every panel (Inspector, Hierarchy, Widget
//! Gallery, Grid Snap, future 3rd-party panels).
//!
//! Wave 8 Phase 2.A hoisted these from
//! `ph2d_editor::screens::hero::style` so panel crates
//! (`ph2d-panel-*`) can paint their surface without depending on
//! `ph2d-editor` (audit S1).
//!
//! ## Stability
//!
//! Wave 8 Phase 3 (audit A1): the module is `#[doc(hidden)]` to
//! signal "panel-author infrastructure, NOT stable user API".
//! Items remain `pub` so panel crates can consume them, but they
//! don't appear in rustdoc and may change shape between 0.x.y
//! releases without a SemVer break. Panel authors are expected to
//! track PH2D releases closely.

#![doc(hidden)]

use crate::interaction::HitIndex;
use crate::paint::{
    fill_rounded_rect, paint_text_centered, paint_text_title, resolve, stroke_rounded_rect,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, PANEL_HEAD_PAD_PX, PANEL_RADIUS_PX, PANEL_RESIZE_HANDLE_SIZE_PX, Radius,
    SECTION_GAP_PX, Spacing, StrokeToken, Theme, TypeToken,
};
use ph2d_vector::VectorScene;

/// Title baseline inset from the panel's top edge. Single value so the
/// title, the close/add button row, and the body-top offset all align.
pub const PANEL_TITLE_BASELINE: f32 = 18.0; // LITERAL-PX-OK: canonical panel title baseline

/// Canonical panel header title — **the single source of truth for the
/// panel name text.** Every `ph2d-panel-*` crate paints its title
/// through this so the size / weight / colour / baseline can never
/// diverge again (they previously ranged `TypeToken::Md`, a raw `15.0`,
/// and `TypeToken::Lg` across panels). Uses [`TypeToken::Lg`] — the
/// larger, canonical size.
///
/// `reserve_right` keeps that many px clear on the right for a close /
/// add affordance. Returns the title font size so callers can position
/// a subtitle / divider directly beneath it.
pub fn paint_panel_title(
    rect: Rect,
    text: &str,
    reserve_right: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let size = TypeToken::Lg.px();
    let x = rect.x + PANEL_HEAD_PAD;
    let y = rect.y + PANEL_TITLE_BASELINE;
    let max_w = (rect.w - PANEL_HEAD_PAD * 2.0 - reserve_right).max(0.0);
    paint_text_title(
        text_system,
        scene,
        text,
        x,
        y,
        size,
        max_w,
        resolve(ColorToken::Text1, theme),
    );
    size
}

/// Canonical segmented / toggle-group button — **the single source of
/// truth for grouped 2–3-way selectors** (Mode pickers, render-strategy
/// switchers, etc.). Always draws a discrete outline so an UNSELECTED
/// segment still reads as a button; the bare `paint_button(Default)`
/// ghost look failed that (inactive halves looked like plain text).
///
/// Selected → `Bg2` fill + `Accent` outline + `Text1`. Unselected →
/// `Bg3` fill + `Border` outline + `Text2`. Caller registers the hit
/// rect against the segment's `NodeId`.
pub fn paint_segmented_button(
    rect: Rect,
    label: &str,
    selected: bool,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    let (bg, fg, border) = if selected {
        (ColorToken::Bg2, ColorToken::Text1, ColorToken::Accent)
    } else {
        (ColorToken::Bg3, ColorToken::Text2, ColorToken::Border)
    };
    fill_rounded_rect(scene, rect, radius, resolve(bg, theme));
    stroke_rounded_rect(
        scene,
        rect,
        radius,
        StrokeToken::Default.px(),
        resolve(border, theme),
    );
    paint_text_centered(
        text_system,
        scene,
        label,
        rect,
        TypeToken::Sm.px(),
        resolve(fg, theme),
    );
}

/// Canonical horizontal gap (px) between the segments of a
/// toggle/segmented group. **Single source of truth** — both
/// [`paint_segmented_group`] and the typed `RadioGroup` segmented
/// painter read this so the spacing never diverges (some groups, e.g.
/// the Widget Gallery's Low/Mid/High, previously had zero gap).
pub fn segmented_gap() -> f32 {
    Spacing::Xs.px()
}

/// Canonical segmented / toggle GROUP: lays out `segments` as N
/// equal-width buttons across `rect` with [`segmented_gap`] between
/// them, paints each via [`paint_segmented_button`], and registers each
/// segment's hit rect. **The single source of truth for segmented-group
/// layout** — call sites pass `(label, selected, node_id)` and never
/// compute per-segment rects or gaps inline.
pub fn paint_segmented_group(
    rect: Rect,
    segments: &[(&str, bool, NodeId)],
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    let n = segments.len();
    if n == 0 {
        return;
    }
    let gap = segmented_gap();
    let seg_w = ((rect.w - gap * (n as f32 - 1.0)) / n as f32).max(0.0);
    for (i, (label, selected, id)) in segments.iter().enumerate() {
        let seg = Rect::new(rect.x + (seg_w + gap) * i as f32, rect.y, seg_w, rect.h);
        paint_segmented_button(seg, label, *selected, scene, text_system, theme);
        hit_index.register(*id, seg);
    }
}

/// Outer corner radius of every panel rect (Inspector, Hierarchy,
/// floating panels). Mirrors `tokens.json::chrome.panel-radius`.
pub const PANEL_RADIUS: f32 = PANEL_RADIUS_PX;

/// Header inset from the panel's left/right edges. Title text,
/// close button, scrollbar reserve all derive from this.
pub const PANEL_HEAD_PAD: f32 = PANEL_HEAD_PAD_PX;

/// Pixel size (square) of every panel's bottom-right resize-gripper
/// hit zone. Centralized so the painters + hit-zone registration
/// use one value.
pub const PANEL_RESIZE_HANDLE_SIZE: f32 = PANEL_RESIZE_HANDLE_SIZE_PX;

/// Floating-panel surface — standard BASE chrome for every panel
/// (Inspector, Hierarchy, Widget Gallery, Grid Snap, …). Paints the
/// rounded `BgElev` fill + 1px `Border` stroke + a tiny drag-pill
/// at the top center. Run BEFORE the panel body so the body
/// renders on top of the surface.
///
/// The bottom-right resize-gripper dot is painted separately by
/// [`paint_panel_corner_dot`] AFTER the body so body widgets don't
/// cover it (the body's scrollable clip extends into the corner).
pub fn paint_panel_surface(rect: Rect, scene: &mut VectorScene, theme: Theme) {
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

/// Bottom-right resize-gripper corner accent. Painted at the END
/// of each panel painter (after `pop_layer`) so it sits on top of
/// any body widget whose rect drifted into the corner. Soft
/// `Text2` reads as a corner accent rather than a foreign visual
/// element.
pub fn paint_panel_corner_dot(rect: Rect, scene: &mut VectorScene, theme: Theme) {
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
pub fn panel_resize_handle_rect(panel: Rect) -> Rect {
    Rect::new(
        panel.x + panel.w - PANEL_RESIZE_HANDLE_SIZE,
        panel.y + panel.h - PANEL_RESIZE_HANDLE_SIZE,
        PANEL_RESIZE_HANDLE_SIZE,
        PANEL_RESIZE_HANDLE_SIZE,
    )
}

/// Rect of the top-center drag-pill hit zone for a panel whose
/// outer rect is `panel`. 80×14 — wide enough to grab on touch +
/// mouse.
pub fn panel_drag_handle_rect(panel: Rect) -> Rect {
    Rect::new(
        panel.x + (panel.w - 80.0) * 0.5, // LITERAL-PX-OK: drag hit-zone width 80 (chrome-specific)
        panel.y + Spacing::Xxs.px(),
        80.0, // LITERAL-PX-OK: drag hit-zone width 80
        SECTION_GAP_PX,
    )
}

/// Shared floating-panel clamp helper. Used by the hero
/// orchestrator (INSP + HIER clamp before chrome paints) AND by
/// each floating panel's `paint_fn` thunk (Widget Gallery, Grid
/// Snap — they own their base rect lazily). Two clamps:
///
/// 1. Horizontal: the panel stays FULLY inside the viewport — left
///    edge ≥ `viewport.left`, right edge ≤ `viewport.right`. (Wave 10
///    Etapa 5 smoke fix — previously kept only 60 px visible on the
///    far side, which let the rest extend off-screen.)
/// 2. Vertical: the panel's top stays inside the viewport and its
///    bottom never crosses `viewport.bottom - 8`. When the user
///    drags DOWN past where `base.h` fits, the panel auto-shrinks
///    (floor at MIN_H so the header + a row stay visible).
///    Dragging back up restores the natural height.
///
/// Returns `(clamped_rect, clamped_off, clamped_resize)`. The
/// callers write the clamped offset/resize back to the WidgetStore
/// so subsequent drag-begins capture the visible offset rather
/// than an accumulated raw value (no rubber-band on direction
/// reversal).
pub fn clamp_panel_rect(
    base: Rect,
    off: (f32, f32),
    resize: (f32, f32),
    viewport: Rect,
) -> (Rect, (f32, f32), (f32, f32)) {
    const MIN_W: f32 = 220.0; // LITERAL-PX-OK: panel min width (chrome-specific min)
    const MIN_H: f32 = 120.0; // LITERAL-PX-OK: panel min height (chrome-specific min)
    let raw_w = (base.w + resize.0).max(MIN_W);
    let raw_h = (base.h + resize.1).max(MIN_H);
    let max_w = (viewport.w * 0.7).max(MIN_W); // LITERAL-PX-OK: max panel width = 70% viewport (chrome ratio)
    let new_w = raw_w.min(max_w);
    let new_h_user = raw_h.min(viewport.h.max(MIN_H));
    let clamped_dw = new_w - base.w;
    let clamped_dh = new_h_user - base.h;

    // Horizontal clamp: full containment. Panel.left ≥ viewport.left,
    // panel.right ≤ viewport.right. Since `max_w = viewport.w * 0.7`
    // caps the panel below the viewport width, the clamp range
    // `[min_x, max_x]` is always non-degenerate (max_x > min_x).
    let max_x = (viewport.x + viewport.w) - (base.x + new_w);
    let min_x = viewport.x - base.x;
    let max_bottom = viewport.y + viewport.h - Spacing::Md.px();
    let min_y = viewport.y - base.y;
    let max_y = (max_bottom - MIN_H) - base.y;
    let dx = crate::math::safe_clamp(off.0, min_x, max_x);
    let dy = crate::math::safe_clamp(off.1, min_y, max_y);
    let new_y = base.y + dy;
    let natural_bottom = new_y + new_h_user;
    let final_h = if natural_bottom > max_bottom {
        (max_bottom - new_y).max(MIN_H)
    } else {
        new_h_user
    };
    (
        Rect::new(base.x + dx, new_y, new_w, final_h),
        (dx, dy),
        (clamped_dw, clamped_dh),
    )
}

/// Highlighter palette used by user-placed notes + section-outline
/// markings inside panel bodies. Five colors selectable from the
/// right-click context menu ("Section outline" submenu).
///
/// Indices 0..4 map to Yellow / Pink / Green / Blue / Orange.
/// Hoisted to `panel_chrome` so panel crates can paint notes
/// without reaching into `ph2d_editor::screens::hero::context_menu_overlay`.
pub const HIGHLIGHTER_RGBA: [[u8; 4]; 5] = [
    // LITERAL-COLOR-OK: highlighter palette — user-pickable note + outline colors.
    [0xFF, 0xF5, 0x9D, 0xFF], // yellow
    [0xF8, 0xBB, 0xD0, 0xFF], // pink
    [0xC8, 0xE6, 0xC9, 0xFF], // green
    [0xBB, 0xDE, 0xFB, 0xFF], // blue
    [0xFF, 0xE0, 0xB2, 0xFF], // orange
];
