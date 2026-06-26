//! The Painter dock's **Texture** sub-section (clean-room port of Blender's brush texture panel,
//! 2D-adapted): a Kind picker ("thumbnail"), then the live preview, Mapping, Rake, Random, Angle,
//! Offset X/Y and Size X/Y — the controls below the picker show only once a texture is assigned
//! (Blender hides dead controls; DIRETIVA §2).
//!
//! All controls are fixed-id, tool-global widgets (registered in [`crate::populate`]); this module
//! only paints them off the published [`BrushSettings`] snapshot and reuses the row/chip helpers
//! from [`crate::paint_brush`]. The slider tracks are `0..1`; the tool maps each onto its real
//! range (the `TEX_*` constants are the single source). Edits forward over the frozen `PanelEvent`
//! channel (drained in [`crate::event`]).

use crate::paint_brush::paint_dropdown_row;
use crate::paint_stroke::section_header;
use crate::state;
use ph2d_editor_core::ids::{
    self as core_ids, painter_brush_texture_kind_option_id, painter_brush_texture_mapping_option_id,
};
use ph2d_editor_core::paint::{resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing};
use ph2d_tool_painter::{
    BrushSettings, ColorRamp, ImageMask, RampAlphaMode, RampColorMode, RampInterp, RampStop,
    TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind,
    TextureLayer, TextureMapping, linear_to_srgb_byte, param_specs, render_texture_preview,
    srgb_to_linear_byte,
};
use ph2d_vector::ImageQuality;

/// Paint the Texture section starting at `y`, returning the next `y`. The Kind picker + Mapping
/// dropdowns stash their open rects for the deferred [`paint_texture_popovers`] pass.
///
/// `compact` (a **Texture layer** editor, not the brush) hides the per-dab-only controls — Mapping,
/// Rake, Random and Angle — since a full-cover layer maps at identity. The Kind picker, the live
/// preview, Size / Offset, per-pattern params and the Color Ramp are always shown.
pub(crate) fn paint_texture_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
    compact: bool,
) -> f32 {
    // Collapsible section (default expanded) on the brush; the inline Texture-LAYER editor keeps the
    // plain divider (it's always-on, nested under the active layer row).
    let (mut y, collapsed) = if compact {
        (section_header(ctx, theme, x, content_w, y, "Grain"), false)
    } else {
        crate::paint_brush_top::paint_collapsible_section(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Grain",
            core_ids::PAINTER_BRUSH_TEXTURE_SECTION,
            core_ids::PAINTER_BRUSH_TEXTURE_SECTION_COLOR,
            core_ids::PAINTER_BRUSH_TEXTURE_RESET,
        )
    };
    if collapsed {
        return y;
    }
    let kind = TextureKind::from_u8(brush.texture_kind);
    let mapping = TextureMapping::from_u8(brush.texture_mapping);
    // Under the Stencil mapping the rect placement (Size / Offset / Rotation) lives in its OWN card
    // (`stencil_*`); the texture's Size / Offset / Angle fields are hidden so the two don't collide.
    let is_stencil = mapping.is_stencil();

    // ── Kind picker ("thumbnail") + New (brush only) ──
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Grain",
        core_ids::PAINTER_BRUSH_TEXTURE_KIND,
        brush.texture_kind,
        kind.name(),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_brush_texture_kind_dd(Some((r, brush.texture_kind)));
    }

    // Everything below modulates an assigned texture — hide it when there is none (no dead controls).
    if kind == TextureKind::None {
        return y;
    }

    // ── Live preview of the current texture pattern, right below the Texture dropdown (Enio 2026-06-24) ──
    y = paint_texture_preview(ctx, theme, x, content_w, y, brush);

    // ── Per-dab mapping controls — brush only (a layer covers the sprite at identity rotation) ──
    if !compact {
        let (ny, open) = paint_dropdown_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Mapping",
            core_ids::PAINTER_BRUSH_TEXTURE_MAPPING,
            brush.texture_mapping,
            TextureMapping::from_u8(brush.texture_mapping).name(),
        );
        y = ny;
        if let Some(r) = open {
            state::set_pending_brush_texture_mapping_dd(Some((r, brush.texture_mapping)));
        }
        // ── Stencil card — the on-canvas gizmo's OWN Size / Offset / Rotation (the rect placement),
        //    independent of the texture's Size / Offset / Angle below (which tile the pattern inside
        //    the rect). Shown right under Mapping when Stencil is active (Enio 2026-06-26). ──
        if is_stencil {
            y = crate::paint_stencil::paint_stencil_card(ctx, theme, x, content_w, y, brush);
        }
        // ── Rake + Random checkboxes — only the per-dab rotation mappings (Stencil has a fixed
        //    frame). Placed under Mapping and above Angle (Enio 2026-06-24). ──
        if mapping.uses_dab_rotation() {
            y = crate::paint_brush_top::paint_checkbox_row(
                ctx,
                theme,
                x,
                content_w,
                y,
                core_ids::PAINTER_BRUSH_TEXTURE_RAKE,
                "Rake",
                brush.texture_rake,
            );
            y = crate::paint_brush_top::paint_checkbox_row(
                ctx,
                theme,
                x,
                content_w,
                y,
                core_ids::PAINTER_BRUSH_TEXTURE_RANDOM,
                "Random Angle",
                brush.texture_random,
            );
        }
        // ── Angle (whole degrees) — the TEXTURE rotation. Under Stencil it rotates the pattern WITHIN
        //    the rect (the rect's own rotation is the Stencil card's Rotation). ──
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Angle",
            core_ids::PAINTER_BRUSH_TEXTURE_ANGLE,
            f32::from(brush.texture_angle_deg),
            0.0,
            f32::from(TEX_ANGLE_MAX_DEG),
            crate::number_field::ANGLE_STEP,
            0,
        );
    }

    // ── Offset X/Y + Size X/Y — the TEXTURE tiling (each pair on ONE line). Always shown; under
    //    Stencil they tile the pattern INSIDE the rect (the rect placement is the Stencil card). ──
    y = crate::number_field::paint_num_xy(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Offset",
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        brush.texture_offset[0],
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y,
        brush.texture_offset[1],
        TEX_OFFSET_MIN,
        TEX_OFFSET_MAX,
        crate::number_field::FINE_STEP,
        2,
    );
    y = crate::number_field::paint_num_xy(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Size",
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        brush.texture_size[0],
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_Y,
        brush.texture_size[1],
        TEX_SIZE_MIN,
        TEX_SIZE_MAX,
        crate::number_field::SIZE_STEP,
        2,
    );

    // ── Depth — how strongly the Grain bites (brush only; a Texture-LAYER is full-cover). ──
    if !compact {
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Depth",
            core_ids::PAINTER_BRUSH_GRAIN_DEPTH,
            brush.grain_depth.clamp(0.0, 1.0),
            0.0,
            1.0,
            crate::number_field::FINE_STEP,
            2,
        );
    }

    // ── Per-pattern parameters — short labels pair two-per-line, long labels go solo. ──
    let pp: Vec<(&str, ph2d_a11y::NodeId, f32)> = param_specs(kind)
        .iter()
        .enumerate()
        .map(|(i, s)| {
            (
                s.label,
                core_ids::PAINTER_BRUSH_TEXTURE_PARAMS[i],
                brush.texture_params[i],
            )
        })
        .collect();
    y = crate::number_field::paint_num_params(ctx, theme, x, content_w, y, &pp);

    // ── Color Ramp sub-editor (maps the texture's scalar to a colour) ──
    crate::paint_texture_ramp::paint_texture_ramp_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        brush,
        "Grain Colors",
    )
}

/// Paint the **Texture-layer** editor inline under the active Texture-layer row: the same Texture
/// section (Kind / Size / Offset / params / preview / Color Ramp), bound to the layer instead of the
/// brush. Builds a `BrushSettings` *view* by overwriting the published brush snapshot's texture fields
/// from the layer's spec, so the shared fixed-id widgets, preview and ramp all reflect the layer; the
/// tool routes their edits to the active texture layer. Returns the next `y` (no-op before the brush
/// snapshot is published).
pub(crate) fn paint_texture_layer_editor(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    tex: &TextureLayer,
    x: f32,
    content_w: f32,
    y: f32,
) -> f32 {
    let Some(view) = state::current_brush().map(|b| brush_view_from_texture_layer(b, tex)) else {
        return y;
    };
    paint_texture_section(ctx, theme, x, content_w, y, view, true)
}

/// `true` when the inline Texture-LAYER editor (not the brush) is the one showing the Texture section:
/// the Layers dock view is up AND the active layer is a Texture layer. The shared fixed-id widgets +
/// popovers + ramp picker then target the layer.
pub(crate) fn texture_layer_editor_active() -> bool {
    state::current_dock_shows_layers()
        && state::current_layers().is_some_and(|s| s.active().is_some_and(|a| s.is_texture(a)))
}

/// The active Texture layer's spec as a `BrushSettings` *view* (its ramp in display sRGB) — the seed
/// source for the shared Color-Ramp picker when a texture layer is being edited. `None` unless
/// [`texture_layer_editor_active`] (callers fall back to the brush snapshot).
pub(crate) fn active_texture_ramp_view() -> Option<BrushSettings> {
    if !texture_layer_editor_active() {
        return None;
    }
    let stack = state::current_layers()?;
    let base = state::current_brush()?;
    match &stack.get(stack.active()?)?.kind {
        ph2d_tool_painter::LayerKind::Texture(tex) => {
            Some(brush_view_from_texture_layer(base, tex))
        }
        _ => None,
    }
}

/// Overwrite `base`'s texture fields from a Texture layer's spec (kind / params / size / offset + the
/// Color Ramp), leaving the brush's other fields intact. The ramp stops are converted from the layer's
/// linear `ColorRamp` to the panel's display-sRGB form (mirror of the tool's `brush_settings`).
pub(crate) fn brush_view_from_texture_layer(
    mut base: BrushSettings,
    tex: &TextureLayer,
) -> BrushSettings {
    base.texture_kind = tex.kind;
    base.texture_params = tex.params;
    base.texture_size = tex.size;
    base.texture_offset = tex.offset;
    base.texture_ramp_enabled = tex.ramp_enabled;
    base.texture_ramp_mode = tex.ramp.color_mode.to_u8();
    base.texture_ramp_interp = tex.ramp.interp.to_u8();
    base.texture_ramp_alpha_mode = tex.ramp_alpha_mode;
    let srgb = |c: f32| f32::from(linear_to_srgb_byte(c)) / 255.0; // LITERAL-PX-OK: sRGB 8-bit normalize
    let stops = tex.ramp.stops();
    let count = stops.len().min(base.texture_ramp_stops.len());
    base.texture_ramp_stop_count = count as u8;
    for (slot, s) in base.texture_ramp_stops.iter_mut().zip(stops).take(count) {
        *slot = [
            s.pos,
            srgb(s.color[0]),
            srgb(s.color[1]),
            srgb(s.color[2]),
            s.color[3],
            f32::from(s.id),
        ];
    }
    base
}

/// Real-time preview of the active texture pattern: re-rendered each frame from the published snapshot
/// (kind + params + size + offset) into a small buffer, then scale-blitted as one image. Grayscale by
/// default; when the Color Ramp is on it is **ramp-coloured with the stop alpha** (translucent stops
/// composite over a checker). Bounded resolution keeps it cheap; reflects every knob live.
fn paint_texture_preview(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let ph = (content_w * 0.5).clamp(56.0, 120.0); // LITERAL-PX-OK: one-off preview-strip min/max height
    let rect = Rect::new(x, y, content_w, ph);
    // When the Color Ramp is on, bake the EXACT 256-entry sRGB-RGBA LUT the tool paints with (rebuild
    // the real `ColorRamp` + its Mode/Interpolation), so the preview is faithful to every ramp option
    // (Ease/Cardinal/B-Spline/Constant · RGB/HSV/HSL), alpha included. `None` → grayscale scalar.
    let mut lut = [[0.0f32; 4]; 256];
    let ramp = if build_ramp_preview_lut(&brush, &mut lut) {
        Some((
            &lut[..],
            RampAlphaMode::from_u8(brush.texture_ramp_alpha_mode),
        ))
    } else {
        None
    };
    // For the Image kind, render the ACTUAL brush image — the host publishes its luminance (the pixels
    // are too heavy for the `Copy` snapshot); `None` → the Image preview stays black until one is set.
    let image = state::current_brush_texture_image();
    let image_mask = image.as_ref().map(|(lum, w, h)| ImageMask {
        lum: lum.as_slice(),
        width: *w,
        height: *h,
    });
    // Render at the rect's aspect (bounded cost), then scale-blit to the rect.
    let kind = TextureKind::from_u8(brush.texture_kind);
    let bw = 140u32;
    let bh = ((ph / content_w * bw as f32).round() as u32).clamp(8, 140);
    let mut buf = vec![0u8; (bw * bh * 4) as usize];
    if kind == TextureKind::Image
        && let Some((_, iw, ih)) = image.as_ref()
    {
        // Letterbox: fit the image aspect inside the strip, centred, over a checker (so the bounds
        // read against a dark image). Render the image into an aspect-matched sub-buffer (one centred
        // copy), then composite it into the strip.
        let (ia, pa) = (
            (*iw).max(1) as f32 / (*ih).max(1) as f32,
            bw as f32 / bh as f32,
        );
        let (sw, sh) = if ia >= pa {
            (bw, ((bw as f32 / ia).round() as u32).clamp(1, bh)) // CLAMP-OK: integer u32; 1 ≤ bh (≥8), no NaN
        } else {
            (((bh as f32 * ia).round() as u32).clamp(1, bw), bh) // CLAMP-OK: integer u32; 1 ≤ bw (≥8), no NaN
        };
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
        let mut sub = vec![0u8; (sw * sh * 4) as usize];
        render_texture_preview(
            kind,
            brush.texture_params,
            brush.texture_size,
            brush.texture_offset,
            image_mask.as_ref(),
            ramp,
            &mut sub,
            sw,
            sh,
        );
        let (ox, oy) = ((bw - sw) / 2, (bh - sh) / 2);
        for sy in 0..sh {
            let di = (((oy + sy) * bw + ox) * 4) as usize;
            let si = ((sy * sw) * 4) as usize;
            buf[di..di + (sw * 4) as usize].copy_from_slice(&sub[si..si + (sw * 4) as usize]);
        }
    } else {
        render_texture_preview(
            kind,
            brush.texture_params,
            brush.texture_size,
            brush.texture_offset,
            image_mask.as_ref(),
            ramp,
            &mut buf,
            bw,
            bh,
        );
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

/// Rebuild the **exact** `ColorRamp` from the published snapshot — stops are display sRGB, so convert
/// them back to the ramp's linear space; honour the chosen colour **Mode** + **Interpolation** — and
/// bake it into `out` as a 256-entry **sRGB-straight RGBA** LUT (RGB linear→sRGB, alpha straight). This
/// is the same bake the tool paints with (`ensure_ramp_lut`), so the preview is faithful to every ramp
/// option. Returns `false` (→ grayscale) when the ramp is off / has no stops.
pub(crate) fn build_ramp_preview_lut(brush: &BrushSettings, out: &mut [[f32; 4]; 256]) -> bool {
    if !brush.texture_ramp_enabled {
        return false;
    }
    let count = (brush.texture_ramp_stop_count as usize).min(brush.texture_ramp_stops.len());
    if count == 0 {
        return false;
    }
    let s2l = |c: f32| srgb_to_linear_byte((c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8); // LITERAL-PX-OK: sRGB 8-bit normalize
    let stops: Vec<RampStop> = brush.texture_ramp_stops[..count]
        .iter()
        .map(|s| RampStop::new(s[0], [s2l(s[1]), s2l(s[2]), s2l(s[3]), s[4]])) // alpha straight
        .collect();
    let ramp = ColorRamp::new(
        stops,
        RampColorMode::from_u8(brush.texture_ramp_mode),
        RampInterp::from_u8(brush.texture_ramp_interp),
    );
    ramp.bake_into(out); // linear RGBA in the chosen interp/colour space
    for c in out.iter_mut() {
        c[0] = f32::from(linear_to_srgb_byte(c[0])) / 255.0; // LITERAL-PX-OK: sRGB 8-bit normalize
        c[1] = f32::from(linear_to_srgb_byte(c[1])) / 255.0; // LITERAL-PX-OK: sRGB 8-bit normalize
        c[2] = f32::from(linear_to_srgb_byte(c[2])) / 255.0; // LITERAL-PX-OK: sRGB 8-bit normalize
        // alpha stays straight
    }
    true
}

/// Deferred paint of the Texture section's open dropdown popovers (Kind + Mapping), drained at the
/// very end of the Brush body so they sit above every row.
pub(crate) fn paint_texture_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip_rect, cur)) = state::take_pending_brush_texture_kind_dd() {
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_TEXTURE_KIND,
            texture_kind_options(),
            chip_rect,
            cur,
        );
    }
    if let Some((chip_rect, cur)) = state::take_pending_brush_texture_mapping_dd() {
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_TEXTURE_MAPPING,
            texture_mapping_options(),
            chip_rect,
            cur,
        );
    }
    crate::paint_texture_ramp::paint_texture_ramp_popovers(ctx, theme);
}

/// The texture kinds as dropdown options. The brush includes `None` (clear) + `Image` (importable);
/// a **Texture layer** omits both — a layer can never be given image pixels (renders flat) and `None`
/// would make it an opaque slab, so neither is a useful choice there.
fn texture_kind_options() -> Vec<DropdownOption<u8>> {
    let layer = texture_layer_editor_active();
    (0..TextureKind::COUNT)
        .filter(|&k| {
            !(layer
                && matches!(
                    TextureKind::from_u8(k),
                    TextureKind::None | TextureKind::Image
                ))
        })
        .map(|k| {
            DropdownOption::new(
                painter_brush_texture_kind_option_id(k),
                k,
                TextureKind::from_u8(k).name(),
            )
        })
        .collect()
}

/// The texture mappings as dropdown options (View Plane / Tiled / Random).
fn texture_mapping_options() -> Vec<DropdownOption<u8>> {
    (0..TextureMapping::COUNT)
        .map(|m| {
            DropdownOption::new(
                painter_brush_texture_mapping_option_id(m),
                m,
                TextureMapping::from_u8(m).name(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests;
