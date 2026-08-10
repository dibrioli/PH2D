//! The **Taper** section — Procreate's *Touch Taper*, directly below the Falloff (Enio 2026-08-08).
//!
//! A stroke-shaped preview with a **draggable handle at the head**: it sets how far the taper reaches in
//! from the START. Below it the numeric rows the reference app carries — **Tip** (sharp ↔ blunt) and
//! **Opacity** (how much the taper fades as well as narrows).
//!
//! ⛔ It had a **second handle** at the right edge (the END length), a *Link tip sizes* toggle and a
//! second Tip row. They went with the far end (Enio 2026-08-10: *"quanto à cauda do taper vamos
//! desativar para todos os modos de pintura; deixe o ajuste apenas para o início do traço"*).
//! [`ph2d_painter_brush::taper`] carries why, and what the removal cost.
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
//! ⚠️ **The silhouette shows WIDTH only, not the Opacity fade.** The handle authors width; the fade is a
//! separate control with its own row, and the reference widget makes the same split. Painting the fade
//! into the preview would make a `tip = 1` / `opacity = 1` taper look identical to a `tip = 0` one,
//! which is the one thing the picture exists to distinguish.
//!
//! ## What the handle's position means
//!
//! The track represents a stroke of [`MAX_TAPER_DIAMETERS`], so a handle sitting a third of the way in is
//! a taper of a third of the maximum — the artist reads the length off the geometry and never has to
//! hold a unit in their head.

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

/// The track everything is drawn on: the content width **inset by the widest thing this widget draws
/// from its own centre**, returned as `(x, w)`.
///
/// ⚠️ **This is one ruler on purpose, and it used to be two.** The handle rode a track inset by
/// `HANDLE_R` while the silhouette rode the raw content width, which was wrong twice over. The visible
/// half is what Enio reported with the picture (2026-08-10, *"o widget está invadindo a borda direita do
/// painel"*): a disc is drawn from its CENTRE, so the full-width one at the right end hung `HALF_PX`
/// past the panel's edge — the same mistake the handles had already been fixed for, left standing in the
/// bigger of the two shapes. The quiet half is that a handle at fraction `f` and the point where the
/// silhouette reaches full width landed up to `HANDLE_R` apart, so the picture disagreed with the number
/// the drag decoded — while a doc-comment right here claimed they *"share one ruler"*.
///
/// So the inset is the LARGER radius, not the handle's: whatever is widest decides, and neither shape
/// can cross the edge.
fn track(x: f32, content_w: f32) -> (f32, f32) {
    let inset = HALF_PX.max(HANDLE_R);
    (x + inset, (content_w - 2.0 * inset).max(1.0))
}

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

    // The inset is applied to the `canvas` the `CurvePoint` drag normalises against as well — insetting
    // only the DRAWING would put the dot somewhere the decoded value does not agree with, which is the
    // seed-vs-sample split this codebase keeps paying for.
    let (track_x, track_w) = track(x, content_w);
    let canvas = Rect::new(track_x, y, track_w, CANVAS_H);
    let cy = y + CANVAS_H * 0.5;
    // The centre line the handle rides, the way the reference widget draws it. It spans the full content
    // width — a 1 px axis IS the widget's extent, and the shape sits within it.
    stroke_polyline(
        ctx.scene,
        &[(x, cy), (x + content_w, cy)],
        AXIS_W,
        resolve(ColorToken::Bg3, theme),
    );

    // The silhouette. `width` is asked with a nominal stroke of MAX_TAPER_DIAMETERS diameters, in the
    // widget's own units (diameter = 1), so the handle fraction and the drawn shape share one ruler.
    let taper = brush.taper;
    let ink = resolve(ColorToken::Text1, theme);
    let total = MAX_TAPER_DIAMETERS;
    for i in 0..=DISCS {
        let u = i as f32 / DISCS as f32;
        let w = taper.width(u * total, 1.0);
        fill_circle(
            ctx.scene,
            track_x + track_w * u,
            cy,
            (HALF_PX * w).max(0.5),
            ink,
        );
    }

    // The head handle, at the fraction of the track its length occupies.
    let f_start = (taper.start / MAX_TAPER_DIAMETERS).clamp(0.0, 1.0);
    let hx = track_x + track_w * f_start;
    ctx.host.store_mut().register(
        painter_taper_handle_id(0),
        InteractiveState::CurvePoint {
            parent: core_ids::PAINTER_TAPER_GIZMO,
            channel: 0,
            index: 0,
            canvas,
        },
    );
    ctx.host.hit_index_mut().register(
        painter_taper_handle_id(0),
        Rect::new(hx - GRAB_R, cy - GRAB_R, GRAB_R * 2.0, GRAB_R * 2.0),
    );
    fill_circle(
        ctx.scene,
        hx,
        cy,
        HANDLE_R,
        resolve(ColorToken::Accent, theme),
    );
    y += CANVAS_H + Spacing::Sm.px();

    // ── The two numeric rows the head taper has ────────────────────────────────────────────────────
    y = crate::number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Tip",
        core_ids::PAINTER_TAPER_TIP_START,
        taper.tip_start.clamp(0.0, 1.0),
        0.0,
        1.0,
        crate::number_field::FINE_STEP,
        2,
    );
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

#[cfg(test)]
mod tests {
    use super::{HALF_PX, HANDLE_R, track};

    /// **Nothing this widget draws crosses the content's edge** — the defect Enio reported with the
    /// picture, asserted as the PROPERTY rather than as the constant that happens to satisfy it today.
    ///
    /// Both shapes are drawn from a centre ON the track, so containment is `track ± radius` for the
    /// widest of them. A gate that checked `HALF_PX` alone would go quietly wrong the day the handle
    /// grows past the disc.
    #[test]
    fn nothing_the_taper_widget_draws_crosses_the_content_edge() {
        let widest = HALF_PX.max(HANDLE_R);
        for content_w in [80.0_f32, 160.0, 240.0, 317.0] {
            let x = 12.0_f32;
            let (tx, tw) = track(x, content_w);
            assert!(
                tx - widest >= x - f32::EPSILON,
                "content_w {content_w}: a shape at the head reaches {} , left of {x}",
                tx - widest
            );
            assert!(
                tx + tw + widest <= x + content_w + f32::EPSILON,
                "content_w {content_w}: a shape at the tail reaches {}, past {}",
                tx + tw + widest,
                x + content_w
            );
        }
    }

    /// The track is genuinely **inset** — the shape that shipped rode the raw content width, and this is
    /// the number that says it no longer does.
    ///
    /// ⚠️ *"The handle and the silhouette share one ruler"* is NOT gated here, and the honest reason is
    /// that a unit test cannot see it: both would have to be compared against something other than the
    /// function they both call, and asserting `track(..) == track(..)` is a test that cannot fail. What
    /// enforces it is structural — there is ONE [`track`] and the paint has no second spelling of the
    /// ruler. That is what a reviewer checks, and what re-introducing `content_w * u` would break.
    #[test]
    fn the_track_is_inset_from_the_raw_content_width() {
        let (tx, tw) = track(0.0, 200.0);
        assert!(
            tx > 0.0,
            "the head sits inside the content, not on its edge"
        );
        assert!(
            tw < 200.0,
            "the track is narrower than the content it rides"
        );
    }
}
