//! The **Taper** section — Procreate's *Touch Taper*, directly below the Falloff (Enio 2026-08-08).
//!
//! A stroke-shaped preview with a **draggable handle at each end**: the left one sets how far the taper
//! reaches in from the START, the right one from the END. Below it the *Link tip sizes* toggle and the
//! numeric rows the reference app carries — **Tip** (sharp ↔ blunt) and **Opacity** (how much the taper
//! fades as well as narrows).
//!
//! ## The preview is drawn by the engine's own law
//!
//! Every column of the silhouette is [`ph2d_painter_brush::taper::Taper::width`] — the exact function
//! the dab is scaled by. A widget that approximated the curve would be a second answer to *what shape
//! is this stroke*, and the two would drift on the first tweak, in the one place nobody reads a number:
//! a picture.
//!
//! It is drawn as a row of overlapping **discs**, because that is what a stroke is here — the preview is
//! literally the dab list the brush would lay, at the widget's scale.
//!
//! ⚠️ **The silhouette shows WIDTH only, not the Opacity fade.** The two handles author width; the fade
//! is a separate control with its own row, and the reference widget makes the same split. Painting the
//! fade into the preview would make the two ends of a `tip = 1` / `opacity = 1` taper look identical to
//! a `tip = 0` one, which is the one thing the picture exists to distinguish.
//!
//! ## What the handle's position means
//!
//! The widget's full width represents a stroke of [`MAX_TAPER_DIAMETERS`], so a handle sitting a third of
//! the way in is a taper of a third of the maximum — the artist reads the length off the geometry and
//! never has to hold a unit in their head. Both handles at the middle is the *lens*: every dab inside
//! both windows, which the engine resolves by `min` (see [`ph2d_painter_brush::taper`]).

use ph2d_editor_core::ids::{self as core_ids, painter_taper_handle_id};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_circle, paint_text, resolve, stroke_polyline};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};
use ph2d_tool_painter::{BrushSettings, MAX_TAPER_DIAMETERS};

const CANVAS_H: f32 = 56.0; // LITERAL-PX-OK: the taper widget's height
const HALF_PX: f32 = 13.0; // LITERAL-PX-OK: half-thickness of the preview stroke at full width
const HANDLE_R: f32 = 6.0; // LITERAL-PX-OK: draggable handle radius
const GRAB_R: f32 = 10.0; // LITERAL-PX-OK: half-size of a handle's pointer grab box
const AXIS_W: f32 = 1.0; // centre-line stroke (structural)
/// Discs across the preview. Dense enough that neighbours overlap at the tip, where the radius is
/// smallest — a sparser row would read as beads and describe a stroke the brush does not lay.
const DISCS: usize = 96;

/// Paint the Taper section. Returns the next `y`.
pub(crate) fn paint_taper_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        "Taper",
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        content_w,
        resolve(ColorToken::Text2, theme),
    );
    let mut y = y + ROW_H_PX;

    let canvas = Rect::new(x, y, content_w.max(1.0), CANVAS_H);
    let cy = y + CANVAS_H * 0.5;
    // The centre line the two handles ride, the way the reference widget draws it.
    stroke_polyline(
        ctx.scene,
        &[(x, cy), (x + content_w, cy)],
        AXIS_W,
        resolve(ColorToken::Bg3, theme),
    );

    // The silhouette. `width` is asked with a nominal stroke of MAX_TAPER_DIAMETERS diameters, in the
    // widget's own units (diameter = 1), so the handle fractions and the drawn shape share one ruler.
    let taper = brush.taper;
    let ink = resolve(ColorToken::Text1, theme);
    let total = MAX_TAPER_DIAMETERS;
    for i in 0..=DISCS {
        let u = i as f32 / DISCS as f32;
        let s = u * total;
        let w = taper.width(s, total - s, 1.0);
        fill_circle(ctx.scene, x + content_w * u, cy, (HALF_PX * w).max(0.5), ink);
    }

    // The two handles, at the fraction of the widget their length occupies.
    let f_start = (taper.start / MAX_TAPER_DIAMETERS).clamp(0.0, 1.0);
    let f_end = (taper.end / MAX_TAPER_DIAMETERS).clamp(0.0, 1.0);
    let hx = [x + content_w * f_start, x + content_w * (1.0 - f_end)];
    {
        let store = ctx.host.store_mut();
        for ch in 0u8..2 {
            store.register(
                painter_taper_handle_id(ch),
                InteractiveState::CurvePoint {
                    parent: core_ids::PAINTER_TAPER_GIZMO,
                    channel: ch,
                    index: 0,
                    canvas,
                },
            );
        }
    }
    for (ch, px) in hx.iter().enumerate() {
        ctx.host.hit_index_mut().register(
            painter_taper_handle_id(ch as u8),
            Rect::new(px - GRAB_R, cy - GRAB_R, GRAB_R * 2.0, GRAB_R * 2.0),
        );
    }
    for px in hx {
        fill_circle(ctx.scene, px, cy, HANDLE_R, resolve(ColorToken::Accent, theme));
    }
    y += CANVAS_H + Spacing::Sm.px();

    // ── Link tip sizes, then the tip row(s) it governs ──────────────────────────────────────────
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_TAPER_LINK,
        "Link tip sizes",
        taper.link_tips,
    );
    // Linked ⇒ ONE row, because there is one number. A second row showing the same value under a
    // different name is how an artist learns that a control sometimes lies.
    y = crate::number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        if taper.link_tips { "Tip" } else { "Tip Start" },
        core_ids::PAINTER_TAPER_TIP_START,
        taper.tip_start.clamp(0.0, 1.0),
        0.0,
        1.0,
        crate::number_field::FINE_STEP,
        2,
    );
    if !taper.link_tips {
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Tip End",
            core_ids::PAINTER_TAPER_TIP_END,
            taper.tip_end.clamp(0.0, 1.0),
            0.0,
            1.0,
            crate::number_field::FINE_STEP,
            2,
        );
    }
    crate::number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Opacity",
        core_ids::PAINTER_TAPER_OPACITY,
        taper.opacity.clamp(0.0, 1.0),
        0.0,
        1.0,
        crate::number_field::FINE_STEP,
        2,
    )
}
