//! The Painter dock's **Shape** section — the dab's silhouette ("tip"). The Falloff (the procedural
//! silhouette) lives here as the default; once a Shape **image** is assigned (right-click a sprite →
//! "Use as Brush Shape"), the silhouette becomes that image and the Falloff goes **inactive** (it is
//! replaced by the image preview + the image's rotation controls). Mirrors the Grain section's row
//! helpers; all controls are fixed-id, tool-global widgets registered in [`crate::populate`].

use crate::paint_brush::paint_dropdown_row;
use crate::paint_brush_top::{paint_checkbox_row, paint_collapsible_section};
use crate::state;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing};
use ph2d_tool_painter::{
    BrushSettings, Falloff, FootprintDeform, ImageMask, TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX,
    TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind, brush_falloff_weight_at, param_specs,
    render_shape_preview,
};
use ph2d_vector::ImageQuality;

/// Paint the collapsible **Shape** section starting at `y`, returning the next `y`. A **Texture** picker
/// (`PAINTER_SHAPE_KIND`) chooses the silhouette source — `None`, any procedural pattern, or `Image` —
/// the same kinds as the Grain. Above it, the **Falloff** dropdown + its curve preview show for every
/// kind EXCEPT `Image` (then the imported tip IS the silhouette, so the falloff is hidden + inactive);
/// for a procedural kind the falloff stays active and MASKS the pattern. Below the picker: the Image
/// preview + frame controls (Image), or the frame controls (procedural). Both dropdowns' popovers are
/// drained by `paint_brush_popovers`.
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
    // ── Watercolor "Automatic" (doc 13 #1): in watercolor mode the stamp defaults to the built-in
    //    feather silhouette — the section's items don't apply, so they HIDE behind the checked box.
    //    Unchecking opens them AND routes them into the watercolor coverage stamp (the tool also
    //    auto-selects the "Watercolor" falloff preset so the transition is visually continuous). ──
    if brush.watercolor {
        y = paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            core_ids::PAINTER_SHAPE_WATERCOLOR_AUTO,
            "Automatic",
            brush.watercolor_shape_auto,
        );
        if brush.watercolor_shape_auto {
            return y;
        }
    }
    let kind = TextureKind::from_u8(brush.shape_kind);
    let is_image = kind == TextureKind::Image;

    // ── Falloff (dropdown + curve preview) — ABOVE the Texture picker, visible + active for every kind
    //    except Image (then the image IS the silhouette → hidden). For a procedural kind the falloff
    //    masks the pattern. Reuses the existing `PAINTER_BRUSH_FALLOFF` id + popover. (Enio 2026-06-25). ──
    if !is_image {
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
        y = crate::paint_falloff::paint_falloff_section(ctx, theme, x, content_w, y, brush);
    }

    // ── Flatten + rotate gizmo — below the Falloff preview, above the Texture picker (Enio 2026-06-26).
    //    Squishes/rotates the whole dab footprint (falloff + Shape + View-Grain). ──
    y = crate::paint_shape_dab::paint_shape_dab_gizmo(ctx, theme, x, content_w, y, brush);

    // ── Texture source picker (None / procedural kinds / Image) — the same kinds as the Grain. Picking
    //    Image opens a file pick (or use the Hierarchy "Use as Brush Shape"); None reverts to the bare
    //    falloff; a procedural kind is masked by the falloff above. ──
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Texture",
        core_ids::PAINTER_SHAPE_KIND,
        brush.shape_kind,
        kind.name(),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_brush_shape_kind_dd(Some((r, brush.shape_kind)));
    }

    // Use Document Layers (capture the doc's raster layers as a multi-layer Shape) — ONLY when the
    // document actually has > 1 capturable layer (else there's nothing multi-layer to capture). Then
    // Per-Layer Color (the mode toggle + per-layer colour rows), below the Texture dropdown.
    if brush.document_layer_count > 1 {
        y = crate::paint_shape_layers::paint_use_layers_button(ctx, theme, x, content_w, y);
    }
    y = crate::paint_shape_layers::paint_shape_per_layer_color(ctx, theme, x, content_w, y, brush);

    // Live composed-silhouette preview — ALWAYS shown (even `None` = the bare falloff; with no Grain
    // and the Shape colour ramp on, it's the colourised silhouette), re-rendered each frame from the
    // snapshot so it tracks Falloff / Angle / Offset / Size + the ramp live (Enio 2026-06-25).
    y = paint_shape_preview(ctx, theme, x, content_w, y, brush);
    if kind != TextureKind::None {
        y = paint_shape_transform_controls(ctx, theme, x, content_w, y, brush);
        if !is_image {
            // Procedural Shape: the kind's per-pattern params (Contrast / Brightness + its shape knob),
            // exactly like the Grain — short labels pair two-per-line, long labels solo (Enio 2026-06-25).
            let pp: Vec<(&str, ph2d_a11y::NodeId, f32)> = param_specs(kind)
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    (
                        s.label,
                        core_ids::PAINTER_SHAPE_PARAMS[i],
                        brush.shape_params[i],
                    )
                })
                .collect();
            y = crate::number_field::paint_num_params(ctx, theme, x, content_w, y, &pp);
        }
    }
    // None ⇒ nothing more: the Falloff dropdown + its curve preview above ARE the silhouette.
    y
}

/// The Shape **frame** controls — Angle / Rake / Random / Offset X·Y / Size X·Y — shared by the Image
/// and procedural branches (they rotate/offset/scale the silhouette frame either way). Returns the next `y`.
fn paint_shape_transform_controls(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    mut y: f32,
    brush: BrushSettings,
) -> f32 {
    // Rake + Random Angle (per-dab rotation), then Angle — Angle sits BELOW the toggles (Enio 2026-06-25).
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
        "Random Angle",
        brush.shape_random,
    );
    y = crate::number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Angle",
        core_ids::PAINTER_SHAPE_ANGLE,
        f32::from(brush.shape_angle_deg),
        0.0,
        f32::from(TEX_ANGLE_MAX_DEG),
        crate::number_field::ANGLE_STEP,
        0,
    );
    y = crate::number_field::paint_num_xy(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Offset",
        core_ids::PAINTER_SHAPE_OFFSET_X,
        brush.shape_offset[0],
        core_ids::PAINTER_SHAPE_OFFSET_Y,
        brush.shape_offset[1],
        TEX_OFFSET_MIN,
        TEX_OFFSET_MAX,
        crate::number_field::FINE_STEP,
        2,
    );
    crate::number_field::paint_num_xy(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Size",
        core_ids::PAINTER_SHAPE_SIZE_X,
        brush.shape_size[0],
        core_ids::PAINTER_SHAPE_SIZE_Y,
        brush.shape_size[1],
        TEX_SIZE_MIN,
        TEX_SIZE_MAX,
        crate::number_field::SIZE_STEP,
        2,
    )
}

/// The Shape **source** options for the picker popover — the full `TextureKind` set (the same kinds as
/// the Grain): `None` (the bare falloff), every procedural pattern (masked by the falloff), and `Image`
/// (an imported tip that replaces the falloff).
pub(crate) fn shape_kind_options() -> Vec<DropdownOption<u8>> {
    (0..TextureKind::COUNT)
        .map(|k| {
            DropdownOption::new(
                core_ids::painter_shape_kind_option_id(k),
                k,
                TextureKind::from_u8(k).name(),
            )
        })
        .collect()
}

/// The live Shape **silhouette** preview: re-render the composed tip every frame from the snapshot
/// (`render_shape_preview` — the SAME composition the engine paints with), so it tracks the Angle /
/// Offset / Size edits in real time. `Image` shows the transformed imported tip; a procedural kind
/// shows `falloff × pattern`. The square tip ([`-1, 1`]²) lights up over a dark checker, centred in the
/// strip, with a border.
fn paint_shape_preview(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let ph = (content_w * 0.5).clamp(56.0, 120.0); // LITERAL-PX-OK: one-off preview-strip min/max height
    let rect = Rect::new(x, y, content_w, ph);
    let bw = 140u32;
    let bh = ((ph / content_w * bw as f32).round() as u32).clamp(8, 140);
    let side = bh; // the silhouette is a square tip → render it `bh×bh`, centred horizontally

    // Falloff LUT (radial `t` → weight), baked from the live brush so the preview tracks falloff edits.
    let mut lut = [0.0f32; 64];
    for (i, w) in lut.iter_mut().enumerate() {
        *w = brush_falloff_weight_at(&brush, i as f32 / 63.0).clamp(0.0, 1.0); // LITERAL-PX-OK: LUT last-index normalize (64 entries)
    }
    let kind = TextureKind::from_u8(brush.shape_kind);
    let image = state::current_brush_shape_image();
    let image_mask = image.as_ref().map(|(lum, w, h)| ImageMask {
        lum: lum.as_slice(),
        width: *w,
        height: *h,
    });
    let mut cov = vec![0u8; (side * side) as usize];
    render_shape_preview(
        kind,
        brush.shape_angle_deg,
        brush.shape_offset,
        brush.shape_size,
        brush.shape_params,
        FootprintDeform::new(brush.dab_flatten, brush.dab_angle_deg),
        &lut,
        None,
        image_mask.as_ref(),
        &mut cov,
        side,
        side,
    );

    // Composite over a dark checker. When the Shape Colour Ramp is on, show `ramp[coverage]` (B&W off ⇒
    // colourised; B&W on ⇒ the desaturated ramp, i.e. the silhouette's tone); otherwise lift the
    // checker toward white by the coverage. The ramp applies regardless of the Grain (Enio 2026-06-26).
    let mut clut = [[0.0f32; 4]; 256];
    let colored = shape_ramp_preview_lut(&brush, &mut clut);
    // Per-Layer Color: the tool publishes the multi-layer COLOURED composite (premultiplied RGBA) — show
    // it over the checker instead of the grayscale/ramp silhouette, so the preview reflects the layer
    // colours (the per-layer pixels live only in the tool). `None` ⇒ the grayscale/ramp path below.
    let color_preview = brush
        .shape_per_layer_color
        .then(state::current_brush_shape_color_preview)
        .flatten();
    let ox = (bw - side) / 2;
    let mut buf = vec![0u8; (bw * bh * 4) as usize];
    for py in 0..bh {
        for px in 0..bw {
            let c: u8 = if (((px / 6) + (py / 6)) & 1) == 0 {
                0x44
            } else {
                0x2c
            }; // LITERAL-COLOR-OK: letterbox checker
            let inside = px >= ox && px < ox + side && py < side;
            let cf = if inside {
                f32::from(cov[(py * side + (px - ox)) as usize]) / 255.0 // LITERAL-PX-OK: 8-bit byte normalize
            } else {
                0.0
            };
            let i = ((py * bw + px) * 4) as usize;
            let px4 = if let Some((cp, cw, ch)) = color_preview.as_ref().filter(|_| inside) {
                // The multi-layer coloured composite (premultiplied RGBA, `cw×ch`) sampled at the scaled
                // tip coord, composited OVER the checker: `checker·(1−a) + premul_rgb`.
                let sx = ((px - ox) * cw / side).min(cw - 1);
                let sy = (py * ch / side).min(ch - 1);
                let o = ((sy * cw + sx) * 4) as usize;
                let a = f32::from(cp[o + 3]) / 255.0; // LITERAL-PX-OK: 8-bit alpha normalize
                let over = |prem: u8| (f32::from(c) * (1.0 - a) + f32::from(prem)) as u8; // LITERAL-PX-OK: premul over checker
                [over(cp[o]), over(cp[o + 1]), over(cp[o + 2]), 255]
            } else if colored && inside {
                // `ramp[coverage]` (sRGB straight) composited over the checker at coverage × stop alpha.
                let s = clut[((cf * 255.0) as usize).min(255)]; // LITERAL-PX-OK: 256-entry LUT index
                let a = (cf * s[3]).clamp(0.0, 1.0);
                let mix = |ch: f32| (f32::from(c) * (1.0 - a) + ch * 255.0 * a) as u8; // LITERAL-PX-OK: 8-bit composite
                [mix(s[0]), mix(s[1]), mix(s[2]), 255]
            } else {
                let g = (f32::from(c) * (1.0 - cf) + 255.0 * cf) as u8; // LITERAL-PX-OK: lift checker → white (8-bit)
                [g, g, g, 255]
            };
            buf[i..i + 4].copy_from_slice(&px4);
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

/// The **Shape** ramp preview LUT (256-entry sRGB-straight RGBA, incl. the B&W filter), so the
/// silhouette preview colourises exactly as the tool paints. Thin Shape binding over the shared
/// [`crate::paint_ramp_widget::build_preview_lut`]; `false` (→ grayscale) when off / empty.
fn shape_ramp_preview_lut(brush: &BrushSettings, out: &mut [[f32; 4]; 256]) -> bool {
    let count =
        (brush.shape_color_ramp_stop_count as usize).min(brush.shape_color_ramp_stops.len());
    crate::paint_ramp_widget::build_preview_lut(
        brush.shape_color_ramp_enabled,
        brush.shape_color_ramp_bw,
        brush.shape_color_ramp_mode,
        brush.shape_color_ramp_interp,
        &brush.shape_color_ramp_stops[..count],
        out,
    )
}
