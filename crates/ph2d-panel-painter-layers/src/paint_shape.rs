//! The Painter dock's **Shape** section — the dab's silhouette ("tip"). The Falloff (the procedural
//! silhouette) lives here as the default; once a Shape **image** is assigned (right-click a sprite →
//! "Use as Brush Shape"), the silhouette becomes that image and the Falloff goes **inactive** (it is
//! replaced by the image preview + the image's rotation controls). Mirrors the Grain section's row
//! helpers; all controls are fixed-id, tool-global widgets registered in [`crate::populate`].

use crate::paint_brush::{ParamRow, paint_dropdown_row, paint_param_row};
use crate::paint_brush_top::{paint_checkbox_row, paint_collapsible_section};
use crate::state;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, TypeToken};
use ph2d_tool_painter::{
    BrushSettings, Falloff, TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX,
    TEX_SIZE_MIN, TextureKind,
};
use ph2d_vector::ImageQuality;

/// Paint the collapsible **Shape** section starting at `y`, returning the next `y`. It always shows the
/// **Falloff** dropdown (the procedural silhouette) and, right below it, a **Shape** source picker
/// (`None` / `Image`) mirroring the Grain Kind picker. With a Shape image assigned the Falloff goes
/// inactive (a caption marks it) and the image preview + rotation controls show; without one the live
/// Falloff curve editor shows. The Falloff dropdown reuses the existing `PAINTER_BRUSH_FALLOFF` id +
/// popover; the Shape picker is `PAINTER_SHAPE_KIND` (both drained by `paint_brush_popovers`).
pub(crate) fn paint_shape_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let (mut y, collapsed) = paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Shape",
        core_ids::PAINTER_SHAPE_SECTION,
        core_ids::PAINTER_SHAPE_SECTION_COLOR,
        core_ids::PAINTER_SHAPE_RESET,
    );
    if collapsed {
        return y;
    }

    // ── Falloff dropdown (the procedural silhouette; the default tip). Always shown — it is the
    //    reference the source picker sits below; greyed by a caption once an image overrides it. ──
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Falloff",
        core_ids::PAINTER_BRUSH_FALLOFF,
        brush.falloff,
        Falloff::from_u8(brush.falloff).name(),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_brush_falloff_dd(Some((r, brush.falloff)));
    }

    // ── Falloff curve preview — directly under the Falloff dropdown and ABOVE the Shape picker, while
    //    the falloff is the active silhouette. Hidden once an image overrides it (then it is inactive,
    //    marked by the caption below). (Enio 2026-06-25). ──
    if !brush.shape_has_image {
        y = crate::paint_falloff::paint_falloff_section(ctx, theme, x, content_w, y, brush);
    }

    // ── Shape source picker (None / Image) — below the Falloff + its preview, mirroring the Grain Kind
    //    picker. Picking Image opens a file pick (or use the Hierarchy "Use as Brush Shape"); None
    //    reverts to the falloff. The chip label tracks the live state (image assigned → "Image"). ──
    let shape_kind = if brush.shape_has_image {
        TextureKind::Image.to_u8()
    } else {
        TextureKind::None.to_u8()
    };
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Shape",
        core_ids::PAINTER_SHAPE_KIND,
        shape_kind,
        TextureKind::from_u8(shape_kind).name(),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_brush_shape_kind_dd(Some((r, shape_kind)));
    }

    if brush.shape_has_image {
        // Image silhouette: the falloff above is inactive (overridden by the image). Mark it, then show
        // the preview (like Grain) + the rotation controls.
        y = caption(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Image silhouette \u{2014} Falloff inactive",
        );
        y = paint_shape_preview(ctx, theme, x, content_w, y);

        let angle_track = f32::from(brush.shape_angle_deg) / f32::from(TEX_ANGLE_MAX_DEG);
        let angle_readout = format!("{}\u{b0}", brush.shape_angle_deg);
        y = paint_param_row(ParamRow {
            ctx,
            theme,
            x,
            content_w,
            y,
            label: "Angle",
            id: core_ids::PAINTER_SHAPE_ANGLE,
            value: angle_track,
            readout: &angle_readout,
        });
        y = paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            core_ids::PAINTER_SHAPE_RAKE,
            "Rake",
            brush.shape_rake,
        );
        y = paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            core_ids::PAINTER_SHAPE_RANDOM,
            "Random",
            brush.shape_random,
        );
        y = shape_xy_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Offset X",
            core_ids::PAINTER_SHAPE_OFFSET_X,
            offset_track(brush.shape_offset[0]),
            brush.shape_offset[0],
        );
        y = shape_xy_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Offset Y",
            core_ids::PAINTER_SHAPE_OFFSET_Y,
            offset_track(brush.shape_offset[1]),
            brush.shape_offset[1],
        );
        y = shape_xy_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Size X",
            core_ids::PAINTER_SHAPE_SIZE_X,
            size_track(brush.shape_size[0]),
            brush.shape_size[0],
        );
        y = shape_xy_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Size Y",
            core_ids::PAINTER_SHAPE_SIZE_Y,
            size_track(brush.shape_size[1]),
            brush.shape_size[1],
        );
    }
    // (No image ⇒ nothing more here: the Falloff dropdown + its curve preview above the Shape picker
    //  are the procedural silhouette.)
    y
}

/// The Shape **source** options for the picker popover — only `None` (the procedural Falloff) and
/// `Image` (an assigned silhouette); there is no procedural shape pattern (that role is the Falloff's).
pub(crate) fn shape_kind_options() -> Vec<DropdownOption<u8>> {
    [TextureKind::None, TextureKind::Image]
        .into_iter()
        .map(|k| {
            DropdownOption::new(
                core_ids::painter_shape_kind_option_id(k.to_u8()),
                k.to_u8(),
                k.name(),
            )
        })
        .collect()
}

/// One Shape Offset/Size slider row (`readout` is the absolute value with 2 decimals).
#[allow(clippy::too_many_arguments)]
fn shape_xy_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label_txt: &str,
    id: ph2d_a11y::NodeId,
    track: f32,
    value: f32,
) -> f32 {
    let readout = format!("{value:.2}");
    paint_param_row(ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: label_txt,
        id,
        value: track,
        readout: &readout,
    })
}

/// A muted single-line caption (Text2), advancing `y` by one short row. Laid out over the **full**
/// content width (NOT the narrow `LABEL_W` column the row `label` uses) so a multi-word note stays on
/// one line instead of word-wrapping into the rows below.
fn caption(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    text: &str,
) -> f32 {
    let font = TypeToken::Xs.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y,
        font,
        content_w,
        resolve(ColorToken::Text2, theme),
    );
    y + font + Spacing::Sm.px()
}

/// The Shape image preview: the published luminance drawn as a grayscale strip (the silhouette tip),
/// letterboxed to the image aspect over a dark checker, with a border. `None` (no image) → just the
/// border box. Mirrors the Grain section's `paint_texture_preview`, simplified (the Shape is always an
/// image, never a procedural pattern).
fn paint_shape_preview(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
) -> f32 {
    let ph = (content_w * 0.5).clamp(56.0, 120.0); // LITERAL-PX-OK: one-off preview-strip min/max height
    let rect = Rect::new(x, y, content_w, ph);
    let bw = 140u32;
    let bh = ((ph / content_w * bw as f32).round() as u32).clamp(8, 140);
    let mut buf = vec![0u8; (bw * bh * 4) as usize];
    // Dark checker backdrop so the silhouette bounds read against a translucent / dark tip.
    for py in 0..bh {
        for px in 0..bw {
            let c = if (((px / 6) + (py / 6)) & 1) == 0 {
                0x44
            } else {
                0x2c
            }; // LITERAL-COLOR-OK: letterbox checker
            let i = ((py * bw + px) * 4) as usize;
            buf[i..i + 4].copy_from_slice(&[c, c, c, 255]);
        }
    }
    if let Some((lum, iw, ih)) = state::current_brush_shape_image() {
        let (iw, ih) = (iw.max(1), ih.max(1));
        let (ia, pa) = (iw as f32 / ih as f32, bw as f32 / bh as f32);
        let (sw, sh) = if ia >= pa {
            (bw, ((bw as f32 / ia).round() as u32).clamp(1, bh)) // CLAMP-OK: integer u32; 1 ≤ bh (≥8), no NaN
        } else {
            (((bh as f32 * ia).round() as u32).clamp(1, bw), bh) // CLAMP-OK: integer u32; 1 ≤ bw (≥8), no NaN
        };
        let (ox, oy) = ((bw - sw) / 2, (bh - sh) / 2);
        // Nearest-sample the luminance into the centred sub-rect; expand to gray RGBA.
        for sy in 0..sh {
            for sx in 0..sw {
                let u = ((sx as f32 + 0.5) / sw as f32 * iw as f32) as u32;
                let v = ((sy as f32 + 0.5) / sh as f32 * ih as f32) as u32;
                let g = lum
                    .get((v.min(ih - 1) * iw + u.min(iw - 1)) as usize)
                    .copied()
                    .unwrap_or(0);
                let i = (((oy + sy) * bw + (ox + sx)) * 4) as usize;
                buf[i..i + 4].copy_from_slice(&[g, g, g, 255]);
            }
        }
    }
    ctx.scene.draw_image_rgba(
        &std::sync::Arc::new(buf),
        bw,
        bh,
        (
            rect.x as f64,
            rect.y as f64,
            (rect.x + rect.w) as f64,
            (rect.y + rect.h) as f64,
        ),
        ImageQuality::Medium,
    );
    stroke_rounded_rect(
        ctx.scene,
        rect,
        Radius::Sm.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );
    y + ph + Spacing::Sm.px()
}

/// Map a stored Shape offset (`[TEX_OFFSET_MIN, TEX_OFFSET_MAX]`) onto the slider's `0..1` track.
fn offset_track(v: f32) -> f32 {
    ((v - TEX_OFFSET_MIN) / (TEX_OFFSET_MAX - TEX_OFFSET_MIN)).clamp(0.0, 1.0)
}

/// Map a stored Shape scale (`[TEX_SIZE_MIN, TEX_SIZE_MAX]`) onto the slider's `0..1` track.
fn size_track(v: f32) -> f32 {
    ((v - TEX_SIZE_MIN) / (TEX_SIZE_MAX - TEX_SIZE_MIN)).clamp(0.0, 1.0)
}
