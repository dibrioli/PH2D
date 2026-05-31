//! Floating Painter color thumb (W2.T2.3).
//!
//! A small rounded swatch pinned to the **top-right** of the canvas,
//! visible only while the Painter tool is active. It mirrors the
//! Painter's live active color (the shell's `painter_bridge` keeps
//! `store.widget_color(PAINTER_COLOR_THUMB)` in sync); clicking it
//! opens the shared [`crate::ids::INSP_BLENDER_PICKER`] (handled in
//! `interaction::dispatch::pointer`), and the picked color is applied
//! back to the Painter via `PainterUiEdit::SetColorSrgb`.
//!
//! The fill is the user's content (the chosen brush color), so it goes
//! through the canonical [`crate::widget::paint_color_swatch`] which
//! draws a token-colored border + alpha checker and carries the single
//! justified `LITERAL-COLOR-OK` for the user RGBA. No hex literal lives
//! in this file.

use super::HeroLayout;
use super::ids;
use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::{ColorSwatch, SwatchSize, paint_color_swatch};
use crate::zones::Rect;
use ph2d_tokens::{Spacing, Theme};
use ph2d_vector::VectorScene;

/// Edge length of the thumb in DIPs — `SwatchSize::Lg` (48) so the
/// floating swatch reads clearly against the canvas, matching the
/// hero foreground/background swatch scale.
const THUMB_SIZE: f32 = 48.0; // LITERAL-PX-OK: hero-scale color thumb edge (Lg swatch, chrome-specific)

/// Paint the Painter color thumb in the canvas top-right corner and
/// register its hit rect. No-op unless `painter_active` (the caller
/// passes `hero.is_panel_visible("painter_sidebar")`, which the shell
/// bridge drives off the active-tool id).
pub fn paint_painter_color_thumb(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    painter_active: bool,
) {
    if !painter_active {
        return;
    }
    // Top-right of the canvas, inset by a standard margin so the thumb
    // never kisses the canvas edge / hierarchy gutter.
    let margin = Spacing::Lg.px();
    let x = layout.canvas.x + layout.canvas.w - THUMB_SIZE - margin;
    let y = layout.canvas.y + margin;
    let rect = Rect::new(x, y, THUMB_SIZE, THUMB_SIZE);

    let rgba = store
        .widget_color(ids::PAINTER_COLOR_THUMB)
        .unwrap_or([0x88, 0x88, 0x88, 0xFF]);
    let mut sw = ColorSwatch::new(ids::PAINTER_COLOR_THUMB, "Brush color", rgba);
    sw.size = SwatchSize::Lg;
    paint_color_swatch(&sw, rect, scene, theme);
    hit_index.register(ids::PAINTER_COLOR_THUMB, rect);
}
