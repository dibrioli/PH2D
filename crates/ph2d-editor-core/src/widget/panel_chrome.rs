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
    ColorToken, PANEL_HEAD_PAD_PX, PANEL_RADIUS_PX, PANEL_RESIZE_HANDLE_SIZE_PX, Radius, Spacing,
    StrokeToken, Theme, TypeToken,
};
use ph2d_vector::VectorScene;

/// Title baseline inset from the panel's top edge. Single value so the
/// title, the close/add button row, and the body-top offset all align.
pub const PANEL_TITLE_BASELINE: f32 = 18.0; // LITERAL-PX-OK: canonical panel title baseline

/// Default height of the title-bar drag band when the panel has only a
/// title + subtitle in the header. Covers title baseline + subtitle
/// line + a few px of margin so the cursor can grab the area visually
/// associated with "the title". Panels with an interactive widget in
/// the header BELOW the subtitle (search bar, mode chips) must pass a
/// smaller value to [`panel_drag_handle_rect`] so the drag area
/// doesn't shadow those widgets in the hit-test.
pub const PANEL_HEADER_H_DEFAULT: f32 = 56.0; // LITERAL-PX-OK: title-baseline (18) + Lg.px(~18) + subtitle line (~14) + 6 px margin; sized to stop ABOVE Hierarchy's search bar (62)

/// Width to leave clear on the right of the title bar for a single
/// close-(X) icon button. Passed to [`panel_drag_handle_rect`] so the
/// widened drag area doesn't shadow the close hit.
pub const PANEL_HEADER_CLOSE_RESERVE: f32 = 40.0; // LITERAL-PX-OK: Xl2 close icon size + padding

/// Width to leave clear on the right for a close-(X) + add-(+) pair.
/// Used by Hierarchy (close + Add Entity) and any panel exposing more
/// than one header affordance.
pub const PANEL_HEADER_ADD_RESERVE: f32 = 80.0; // LITERAL-PX-OK: 2 × icon size + gap

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

/// Like [`paint_segmented_group`] but **adaptive**: when the labels
/// don't fit in `rect.w` at their natural width, demotes buttons
/// from the END of the list to NEW ROWS below — each demoted button
/// takes the full row width. Returns the total height used (≥ `rect.h`).
///
/// UI canon post-2026-05-24: instead of letting a button's label
/// wrap inside the button (which the canonical button painter does
/// not gracefully support — the `Hand-\npacked` artifact in the
/// 2026-05-24 screenshot), the group reflows by stacking the
/// overflow buttons vertically. Callers advance `cur_y` by the
/// returned height instead of `rect.h`.
///
/// Demotion order: last button first, then next-to-last, etc., so
/// the visual "primary" buttons (left) stay on the top row.
pub fn paint_segmented_group_adaptive(
    rect: Rect,
    segments: &[(&str, bool, NodeId)],
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) -> f32 {
    let n = segments.len();
    if n == 0 {
        return 0.0;
    }
    let gap = segmented_gap();
    let font_size = TypeToken::Sm.px();
    let pad_inside = Spacing::Lg.px() * 2.0; // canonical breathing room

    // Measure each label's natural width (text + breathing room).
    let widths: Vec<f32> = segments
        .iter()
        .map(|(label, _, _)| {
            text_system.layout(label, font_size, f32::INFINITY).width() + pad_inside
        })
        .collect();

    // Find how many buttons fit in the top row at their natural widths
    // (with gaps). Demote from the END until the prefix fits.
    let mut top_n = n;
    while top_n > 1 {
        let total: f32 = widths[..top_n].iter().sum::<f32>() + gap * (top_n as f32 - 1.0);
        if total <= rect.w {
            break;
        }
        top_n -= 1;
    }

    let row_h = rect.h;
    let row_gap = Spacing::Xs.px();

    // Top row: equal widths summing to rect.w (canonical look —
    // when N buttons fit, they share width evenly per
    // `paint_segmented_group`).
    let top_seg_w = ((rect.w - gap * (top_n as f32 - 1.0)) / top_n as f32).max(0.0);
    for (i, (label, selected, id)) in segments[..top_n].iter().enumerate() {
        let seg = Rect::new(
            rect.x + (top_seg_w + gap) * i as f32,
            rect.y,
            top_seg_w,
            row_h,
        );
        paint_segmented_button(seg, label, *selected, scene, text_system, theme);
        hit_index.register(*id, seg);
    }

    // Demoted buttons: each on its own row at full rect.w.
    let mut y = rect.y + row_h;
    for (label, selected, id) in &segments[top_n..] {
        y += row_gap;
        let seg = Rect::new(rect.x, y, rect.w, row_h);
        paint_segmented_button(seg, label, *selected, scene, text_system, theme);
        hit_index.register(*id, seg);
        y += row_h;
    }

    y - rect.y
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
/// rounded `BgElev` fill + 1px `Border` stroke. Run BEFORE the panel
/// body so the body renders on top of the surface.
///
/// Pre-2026-05-24 also painted a tiny 36×4 drag pill at the top
/// center — removed when the drag hit-zone was widened to cover the
/// full title bar ([`panel_drag_handle_rect`]). The title text itself
/// is now the visual cue that "this band is grabbable", consistent
/// with macOS/Windows title bars.
///
/// The corner resize-gripper dots are painted separately by
/// [`paint_panel_corner_dot`] (BR) + [`paint_panel_corner_dot_bl`]
/// (BL) AFTER the body so body widgets don't cover them.
pub fn paint_panel_surface(rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = PANEL_RADIUS;
    // PanelBg = BgElev hue/L with ~0.92 alpha → panel reads as
    // floating glass over canvas while text contrast holds.
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::PanelBg, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
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

/// Hit zone for the panel's title-bar drag band — **full-width**
/// (minus `right_reserve` for header icons) × `header_h` from the top.
///
/// Pre-2026-05-24 this was a fixed 80×14 pill centered at the top,
/// which was too narrow to grab reliably and exposed the body widgets
/// scrolled BEHIND the title to spurious clicks. The wider band fixes
/// both: the user can grab anywhere across the title to drag, AND
/// when the same id is re-registered at the END of the panel paint
/// it sits on top of any body widget that scrolled into the header
/// area (the dispatch's last-registered-wins z-order blocks the
/// click from reaching the scrolled-out-of-view widget).
///
/// **Both params are caller-computed:**
/// - `header_h`: how far down the drag band extends — must stop
///   BEFORE any interactive widget in the header (search bar in
///   Hierarchy, mode chips in tool panels). [`PANEL_HEADER_H_DEFAULT`]
///   is conservative for panels with only title + subtitle.
/// - `right_reserve`: width to leave clear on the right for close /
///   add / settings icons. [`PANEL_HEADER_CLOSE_RESERVE`] for just
///   an X; [`PANEL_HEADER_ADD_RESERVE`] for X + one more icon.
pub fn panel_drag_handle_rect(panel: Rect, header_h: f32, right_reserve: f32) -> Rect {
    Rect::new(
        panel.x,
        panel.y,
        (panel.w - right_reserve).max(0.0),
        header_h,
    )
}

/// Rect of the bottom-LEFT resize-gripper hit zone for a panel whose
/// outer rect is `panel`. Mirror of [`panel_resize_handle_rect`] (the
/// bottom-right gripper) — same size, same offset, mirrored across
/// the panel's vertical axis. Lets the user grab the panel from
/// either bottom corner to resize.
pub fn panel_resize_handle_rect_bl(panel: Rect) -> Rect {
    Rect::new(
        panel.x,
        panel.y + panel.h - PANEL_RESIZE_HANDLE_SIZE,
        PANEL_RESIZE_HANDLE_SIZE,
        PANEL_RESIZE_HANDLE_SIZE,
    )
}

/// Rect of the canonical X close button for a panel — same geometry
/// `paint_panel_close_button` paints into. Exposed so panels with a
/// scrollable body can RE-REGISTER the close hit at end-of-frame
/// (after body widgets register their potentially-scrolled rects)
/// so the click-block z-order doesn't accidentally bury the close
/// hit. Without the re-register, a body widget that scrolled into
/// the title-bar band ends up on top of close in the dispatch's
/// back-to-front `HitIndex` walk.
pub fn panel_close_button_rect(panel: Rect) -> Rect {
    let close_size = Spacing::Xl2.px();
    Rect::new(
        panel.x + panel.w - PANEL_HEAD_PAD - close_size,
        panel.y + PANEL_TITLE_BASELINE - 2.0, // LITERAL-PX-OK: baseline-align with title text
        close_size,
        close_size,
    )
}

/// Paint the canonical X close button at the top-right of a panel
/// and register its hit rect against `close_id`. Returns the rect so
/// callers can position other header affordances relative to it.
///
/// **UI canon (post-2026-05-24):** every floating panel EXCEPT
/// Hierarchy carries this close button. For image-tool panels
/// (BgRemoval, Padding, Color Equalization, Upscale, Equalize Sizes)
/// the `close_id` is the SAME id as the panel's existing "Cancel"
/// button — clicking the X dispatches the same `WidgetEvent::Click`
/// the bottom Cancel button does, which the panel's `apply_event`
/// translates to `EditorAction::CancelActiveTool`. No new handler
/// needed. For visibility-toggle panels (Inspector, Widget Gallery,
/// Grid Snap) the `close_id` is a panel-specific `*_CLOSE` constant
/// that the panel's apply_event toggles via
/// [`PanelHostInternal::set_panel_visible`].
///
/// Visually: 32×32 hit zone with a 16×16 X glyph centered. Color
/// `Text2` (matches subtitle weight — affordance reads as chrome,
/// not a primary action). Sits inside the [`PANEL_HEADER_CLOSE_RESERVE`]
/// slot the [`panel_drag_handle_rect`] leaves clear on the right
/// edge of the title band.
pub fn paint_panel_close_button(
    panel: Rect,
    close_id: ph2d_a11y::NodeId,
    hit_index: &mut crate::interaction::HitIndex,
    scene: &mut VectorScene,
    theme: Theme,
) -> Rect {
    let close_rect = panel_close_button_rect(panel);
    hit_index.register(close_id, close_rect);
    crate::paint::paint_icon(
        scene,
        crate::icons::IconId::Close,
        close_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );
    close_rect
}

/// Bottom-LEFT resize-gripper corner accent. Mirror of
/// [`paint_panel_corner_dot`]. Painted AFTER body widgets so the
/// dot sits on top of anything that drifted into the corner.
pub fn paint_panel_corner_dot_bl(rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let dot_d = Spacing::Xs.px();
    let inset = 7.0_f32; // LITERAL-PX-OK: corner-dot inset (specific accent geometry)
    let dot = Rect::new(
        rect.x + inset,
        rect.y + rect.h - inset - dot_d,
        dot_d,
        dot_d,
    );
    fill_rounded_rect(scene, dot, dot_d * 0.5, resolve(ColorToken::Text2, theme));
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
