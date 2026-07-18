//! [`Scrollbar`] — thin vertical drag-affordance for scrollable
//! panels.
//!
//! Single source of truth for the track/thumb geometry: callers
//! ask [`Scrollbar::thumb_rect`] for the hit zone the dispatch
//! should register, and the painter uses [`paint_scrollbar`] to
//! draw the track + thumb at the same coordinates.
//!
//! No state on the widget itself — `panel_scroll`, `panel_content_h`
//! and `panel_visible_h` all live on the [`crate::interaction::WidgetStore`].
//! The dispatcher reads / writes those side-tables; this module
//! only computes geometry + paints.

use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, ROW_H_PX, Theme};
use ph2d_vector::VectorScene;

/// Default scrollbar track width. Wide enough to be a comfortable
/// drag target on iPad/tablet (the user's "Fundamental para
/// ipad/tablet" requirement); narrow enough to not encroach on
/// the panel content.
pub const SCROLLBAR_W: f32 = 10.0; // LITERAL-PX-OK: scrollbar track width (chrome-specific, between Spacing::Md and Lg)
/// Minimum thumb height. When the visible viewport is a large
/// fraction of the content, the proportional thumb height could
/// shrink to 0 px; we clamp so the user can always grab it.
pub const SCROLLBAR_THUMB_MIN_H: f32 = ROW_H_PX;

/// Compute the track rect inside the panel body. The caller is
/// responsible for clipping the track to the panel's body region —
/// the rect returned here is just the geometry.
pub fn track_rect(body_rect: Rect) -> Rect {
    Rect::new(
        body_rect.x + body_rect.w - SCROLLBAR_W - 2.0,
        body_rect.y,
        SCROLLBAR_W,
        body_rect.h,
    )
}

/// Compute the thumb rect inside `track`, given the panel's scroll
/// position and the content / visible heights. Returns the full
/// track when `content_h <= visible_h` (no scroll needed — caller
/// should hide).
pub fn thumb_rect(track: Rect, scroll_y: f32, content_h: f32, visible_h: f32) -> Rect {
    if content_h <= visible_h || track.h <= 0.0 {
        return track;
    }
    let raw_h = visible_h / content_h * track.h;
    let thumb_h = raw_h.max(SCROLLBAR_THUMB_MIN_H).min(track.h);
    let max_scroll = (content_h - visible_h).max(1e-3);
    let t = (scroll_y / max_scroll).clamp(0.0, 1.0);
    let thumb_y = track.y + t * (track.h - thumb_h);
    Rect::new(track.x, thumb_y, track.w, thumb_h)
}

/// Returns true iff the scrollbar should be visible at all (some
/// content overflows the viewport). When false, callers can skip
/// paint AND hit registration entirely.
pub fn is_needed(content_h: f32, visible_h: f32) -> bool {
    content_h > visible_h + 0.5
}

/// Paint the scrollbar track + thumb at `body_rect`. No-op when
/// `!is_needed(content_h, visible_h)`. `is_active` = currently
/// being dragged; thumb fills with `Accent` instead of `Border` so
/// the user gets feedback that their drag is "live".
pub fn paint_scrollbar(
    body_rect: Rect,
    scroll_y: f32,
    content_h: f32,
    visible_h: f32,
    is_active: bool,
    scene: &mut VectorScene,
    theme: Theme,
) {
    if !is_needed(content_h, visible_h) {
        return;
    }
    let track = track_rect(body_rect);
    // Subtle track tint so the rail itself reads against the panel
    // surface; uses `Bg3` (the same token the collapsed-section
    // plate uses, for a coherent "interactive surface" palette).
    fill_rounded_rect(
        scene,
        track,
        SCROLLBAR_W * 0.5,
        resolve(ColorToken::Bg3, theme),
    );
    let thumb = thumb_rect(track, scroll_y, content_h, visible_h);
    let thumb_color = if is_active {
        ColorToken::Accent
    } else {
        ColorToken::BorderEmph
    };
    fill_rounded_rect(
        scene,
        thumb,
        (SCROLLBAR_W * 0.5).min(thumb.h * 0.5),
        resolve(thumb_color, theme),
    );
}

/// Convert a cursor y delta to a scroll-y delta given the track
/// dimensions. Used by the drag dispatcher to translate
/// `cursor_y - cursor_y_at_down` into `scroll_y - scroll_y_at_down`.
pub fn delta_for_drag(cursor_delta_y: f32, track_h: f32, content_h: f32, visible_h: f32) -> f32 {
    let raw_h = visible_h / content_h * track_h;
    let thumb_h = raw_h.max(SCROLLBAR_THUMB_MIN_H).min(track_h);
    let drag_range = (track_h - thumb_h).max(1.0);
    let max_scroll = (content_h - visible_h).max(0.0);
    cursor_delta_y * (max_scroll / drag_range)
}

/// Stable hit id for the inspector's scrollbar thumb. (Exported so
/// the dispatch can route Down/Move/Up events without a string
/// lookup. Hierarchy gets a separate id.)
pub const INSPECTOR_SCROLLBAR_ID: NodeId = NodeId(820);
pub const HIERARCHY_SCROLLBAR_ID: NodeId = NodeId(821);
/// Widget Gallery floating panel scrollbar — mirrors the Inspector's
/// behavior with an independent thumb id so dispatch can route
/// drag-scroll events without aliasing the Inspector's scroll state.
pub const GALLERY_SCROLLBAR_ID: NodeId = NodeId(822);
/// Grid Settings floating panel scrollbar — same shape as Inspector /
/// Gallery, independent thumb id keeps dispatch from aliasing.
pub const GRID_SETTINGS_SCROLLBAR_ID: NodeId = NodeId(823);
/// Color Equalization panel scrollbar — image-tools docked panel grew
/// past the dock height once Phase 2/3 (sharpen/LUT) landed.
/// Independent thumb id so dispatch can route drag-scroll cleanly.
pub const COLOR_EQUALIZATION_SCROLLBAR_ID: NodeId = NodeId(824);
/// Bg Removal panel scrollbar — same image-tools dock slot as
/// Color Equalization; the Protect-brush sub-section (Size slider +
/// 4-falloff segmented + Show-mask + Clear) pushes the panel past
/// the dock height. Independent thumb id keeps dispatch from
/// aliasing CEQ's scroll state when both tools are toggled.
pub const BG_REMOVAL_SCROLLBAR_ID: NodeId = NodeId(825);
/// Padding panel scrollbar — image-tools dock; Reset/Cancel/Apply
/// drift off the dock once the user shrinks the panel. Enio 2026-05-26:
/// "padrão central do app é painel com scroll. corrija todos".
pub const PADDING_SCROLLBAR_ID: NodeId = NodeId(826);
/// Upscale panel scrollbar — same image-tools dock; same overflow
/// pattern as Padding.
pub const UPSCALE_SCROLLBAR_ID: NodeId = NodeId(827);
/// Equalize Sizes panel scrollbar — image-tools dock; conditional
/// sub-sections (Fixed chips, Grid offset slider + arrange toggle,
/// upscale algorithm row) easily push the body past dock height.
pub const EQUALIZE_SIZES_SCROLLBAR_ID: NodeId = NodeId(828);
/// Painter Layers floating panel scrollbar — the layer stack grows past
/// the panel height once a few layers are added. Independent thumb id so
/// dispatch routes drag-scroll without aliasing the Inspector / other
/// panels. Wheel-scroll already works via the generic `dispatch_wheel`;
/// the panel registers this id on its `scrollbar_thumb_rect` to enable
/// thumb-DRAG (Painter W3 audit item 4).
pub const PAINTER_LAYERS_SCROLLBAR_ID: NodeId = NodeId(829);
/// Brush Studio floating panel scrollbar (W5) — the brush-parameter editor's
/// three sections (Stroke Path / Shape / Rendering) overflow the dock height.
/// Independent thumb id so dispatch routes drag-scroll without aliasing the
/// sidebar / layers panels that share the same dock slot.
pub const PAINTER_BRUSH_STUDIO_SCROLLBAR_ID: NodeId = NodeId(830);
/// Audio Mixer docked-panel scrollbar — the channel strips + the stacked
/// master-effect sections (EQ / Reverb / Delay / Comp / Ducking, each with
/// per-bus rows) overflow the Inspector-dock height. Independent thumb id so
/// dispatch routes drag-scroll without aliasing the other dock-slot panels.
/// (831 is `DROPDOWN_SCROLLBAR_ID` in `widget/dropdown.rs`.)
pub const AUDIO_MIXER_SCROLLBAR_ID: NodeId = NodeId(832);
/// Vector Style docked panel scrollbar (ADR-0108) — the panel body (Stroke/Fill +
/// Cap/Join + Dash/Gap + Draw modes + per-shape sliders + Vertex + Boolean +
/// Arrange) overflows the dock height. Independent thumb id so dispatch routes
/// drag-scroll without aliasing the Inspector that shares the dock slot.
/// NOTE: `831` is `DROPDOWN_SCROLLBAR_ID` (dropdown.rs) — dispatch special-cases
/// that id to the open dropdown, so a collision would make this thumb
/// un-draggable — and `832` is `AUDIO_MIXER_SCROLLBAR_ID` above. Use `833`.
pub const VECTOR_SCROLLBAR_ID: NodeId = NodeId(833);
/// Audio Editor docked-panel scrollbar (docs/Audio/, W3 block 3b) — the transport +
/// edit ops + the effects rack (selector, up to 4 parameter sliders, the chain list
/// and its commit row) overflow the dock height. Independent thumb id so dispatch
/// routes drag-scroll without aliasing the Audio Mixer / Inspector that share the
/// dock slot. Next free id is `835`; re-read the collision note above before taking it.
pub const AUDIO_EDITOR_SCROLLBAR_ID: NodeId = NodeId(834);
/// Flip Style docked-panel scrollbar (ADR-0114 W2) — the panel body (Mode +
/// Brush + Color + Erase + the Layers stack) overflows the dock height once a
/// few layers are added. Independent thumb id so dispatch routes drag-scroll
/// without aliasing the Inspector / Vector panels that share the dock slot.
/// Next free id is `836`; re-read the collision note above before taking it.
pub const FLIP_SCROLLBAR_ID: NodeId = NodeId(835);
/// Physics world docked-panel scrollbar (ADR-0131 D8 / W2b) — the panel body
/// (World + Solver + Damping + Sleep + Debug) overflows the dock height.
/// Independent thumb id so dispatch routes drag-scroll without aliasing the
/// Inspector / Vector / Flip panels that share the dock slot.
/// Next free id is `837`; re-read the collision note above before taking it.
pub const PHYSICS_SCROLLBAR_ID: NodeId = NodeId(836);

#[cfg(test)]
mod tests {
    use super::*;

    /// The scrollbar thumb ids + `DROPDOWN_SCROLLBAR_ID` are hand-assigned raw
    /// `NodeId`s (820..) — NOT hashed, so `node_id_collisions` (which scans the
    /// chrome hash-ids) does NOT cover them. A collision silently breaks drag
    /// routing: dispatch special-cases `DROPDOWN_SCROLLBAR_ID`, so any thumb
    /// aliased onto it becomes un-draggable (the Vector panel bug, 2026-07-07).
    /// Assert pairwise uniqueness here so a new panel's id can't re-collide.
    #[test]
    fn scrollbar_and_dropdown_thumb_ids_are_unique() {
        let ids = [
            ("INSPECTOR", INSPECTOR_SCROLLBAR_ID),
            ("HIERARCHY", HIERARCHY_SCROLLBAR_ID),
            ("GALLERY", GALLERY_SCROLLBAR_ID),
            ("GRID_SETTINGS", GRID_SETTINGS_SCROLLBAR_ID),
            ("COLOR_EQUALIZATION", COLOR_EQUALIZATION_SCROLLBAR_ID),
            ("BG_REMOVAL", BG_REMOVAL_SCROLLBAR_ID),
            ("PADDING", PADDING_SCROLLBAR_ID),
            ("UPSCALE", UPSCALE_SCROLLBAR_ID),
            ("EQUALIZE_SIZES", EQUALIZE_SIZES_SCROLLBAR_ID),
            ("PAINTER_LAYERS", PAINTER_LAYERS_SCROLLBAR_ID),
            ("PAINTER_BRUSH_STUDIO", PAINTER_BRUSH_STUDIO_SCROLLBAR_ID),
            ("AUDIO_MIXER", AUDIO_MIXER_SCROLLBAR_ID),
            ("VECTOR", VECTOR_SCROLLBAR_ID),
            ("AUDIO_EDITOR", AUDIO_EDITOR_SCROLLBAR_ID),
            ("FLIP", FLIP_SCROLLBAR_ID),
            ("PHYSICS", PHYSICS_SCROLLBAR_ID),
            ("DROPDOWN", crate::widget::DROPDOWN_SCROLLBAR_ID),
        ];
        for (i, (na, a)) in ids.iter().enumerate() {
            for (nb, b) in &ids[i + 1..] {
                assert_ne!(a, b, "scrollbar id collision: {na} == {nb} ({a:?})");
            }
        }
    }

    #[test]
    fn no_scrollbar_when_content_fits() {
        assert!(!is_needed(100.0, 200.0));
    }

    #[test]
    fn thumb_at_top_when_scroll_zero() {
        let body = Rect::new(0.0, 0.0, 100.0, 200.0);
        let track = track_rect(body);
        let thumb = thumb_rect(track, 0.0, 400.0, 200.0);
        assert!((thumb.y - track.y).abs() < 0.001);
    }

    #[test]
    fn thumb_at_bottom_when_scroll_max() {
        let body = Rect::new(0.0, 0.0, 100.0, 200.0);
        let track = track_rect(body);
        let max = 400.0 - 200.0;
        let thumb = thumb_rect(track, max, 400.0, 200.0);
        let expected_bottom = track.y + track.h;
        assert!((thumb.y + thumb.h - expected_bottom).abs() < 0.001);
    }

    #[test]
    fn delta_for_drag_proportional() {
        // track 100 px, content 400, visible 200 → thumb 50 px,
        // drag_range 50 px, max_scroll 200 → delta×4.
        let d = delta_for_drag(10.0, 100.0, 400.0, 200.0);
        // raw_h = 50, drag_range = 50, ratio = 200/50 = 4 → 10*4 = 40.
        assert!((d - 40.0).abs() < 0.001);
    }
}
