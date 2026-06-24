//! The Painter dock's **Texture** sub-section (clean-room port of Blender's brush texture panel,
//! 2D-adapted): a Kind picker ("thumbnail") + New, then Mapping, Angle, Rake, Random, Offset X/Y
//! and Size X/Y — shown only once a texture is assigned (Blender hides dead controls; DIRETIVA §2).
//!
//! All controls are fixed-id, tool-global widgets (registered in [`crate::populate`]); this module
//! only paints them off the published [`BrushSettings`] snapshot and reuses the row/chip helpers
//! from [`crate::paint_brush`]. The slider tracks are `0..1`; the tool maps each onto its real
//! range (the `TEX_*` constants are the single source). Edits forward over the frozen `PanelEvent`
//! channel (drained in [`crate::event`]).

use crate::paint_brush::{ParamRow, paint_dropdown_row, paint_param_row, paint_toggle_row};
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
    BrushSettings, ColorRamp, RampAlphaMode, RampColorMode, RampInterp, RampStop,
    TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind,
    TextureLayer, TextureMapping, linear_to_srgb_byte, param_specs, render_texture_preview,
    srgb_to_linear_byte,
};
use ph2d_vector::ImageQuality;

/// Paint the Texture section starting at `y`, returning the next `y`. The Kind picker + Mapping
/// dropdowns stash their open rects for the deferred [`paint_texture_popovers`] pass.
///
/// `compact` (a **Texture layer** editor, not the brush) hides the per-dab-only controls — the "New"
/// button, Mapping, Angle, Rake and Random — since a full-cover layer maps at identity. The Kind
/// picker, Size / Offset, per-pattern params, the live preview and the Color Ramp are always shown.
pub(crate) fn paint_texture_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
    compact: bool,
) -> f32 {
    let mut y = section_header(ctx, theme, x, content_w, y, "Texture");
    let kind = TextureKind::from_u8(brush.texture_kind);
    let mapping = TextureMapping::from_u8(brush.texture_mapping);

    // ── Kind picker ("thumbnail") + New (brush only) ──
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Texture",
        core_ids::PAINTER_BRUSH_TEXTURE_KIND,
        brush.texture_kind,
        kind.name(),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_brush_texture_kind_dd(Some((r, brush.texture_kind)));
    }
    if !compact {
        // "New" — a momentary button (assigns the default procedural); painted as an un-filled toggle.
        y = paint_toggle_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            core_ids::PAINTER_BRUSH_TEXTURE_NEW,
            "New",
            false,
        );
    }

    // Everything below modulates an assigned texture — hide it when there is none (no dead controls).
    if kind == TextureKind::None {
        return y;
    }

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
        // ── Angle (whole degrees) ──
        let angle_track = f32::from(brush.texture_angle_deg) / f32::from(TEX_ANGLE_MAX_DEG);
        y = paint_param_row(ParamRow {
            ctx,
            theme,
            x,
            content_w,
            y,
            label: "Angle",
            id: core_ids::PAINTER_BRUSH_TEXTURE_ANGLE,
            value: angle_track,
            readout: &format!("{}°", brush.texture_angle_deg),
        });
        // ── Rake + Random toggles — only the per-dab rotation mappings (Stencil has a fixed frame) ──
        if mapping.uses_dab_rotation() {
            y = paint_toggle_row(
                ctx,
                theme,
                x,
                content_w,
                y,
                core_ids::PAINTER_BRUSH_TEXTURE_RAKE,
                "Rake",
                brush.texture_rake,
            );
            y = paint_toggle_row(
                ctx,
                theme,
                x,
                content_w,
                y,
                core_ids::PAINTER_BRUSH_TEXTURE_RANDOM,
                "Random",
                brush.texture_random,
            );
        }
    }

    // ── Offset X / Y (tile fractions) ──
    y = paint_param_row(ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: "Offset X",
        id: core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        value: offset_track(brush.texture_offset[0]),
        readout: &format!("{:.2}", brush.texture_offset[0]),
    });
    y = paint_param_row(ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: "Offset Y",
        id: core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y,
        value: offset_track(brush.texture_offset[1]),
        readout: &format!("{:.2}", brush.texture_offset[1]),
    });

    // ── Size X / Y (scale) ──
    y = paint_param_row(ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: "Size X",
        id: core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        value: size_track(brush.texture_size[0]),
        readout: &format!("{:.2}", brush.texture_size[0]),
    });
    y = paint_param_row(ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: "Size Y",
        id: core_ids::PAINTER_BRUSH_TEXTURE_SIZE_Y,
        value: size_track(brush.texture_size[1]),
        readout: &format!("{:.2}", brush.texture_size[1]),
    });

    // ── Per-pattern parameters — each kind's own knobs (Contrast / Brightness + a shape param) ──
    for (i, spec) in param_specs(kind).iter().enumerate() {
        let value = brush.texture_params[i];
        y = paint_param_row(ParamRow {
            ctx,
            theme,
            x,
            content_w,
            y,
            label: spec.label,
            id: core_ids::PAINTER_BRUSH_TEXTURE_PARAMS[i],
            value,
            readout: &format!("{value:.2}"),
        });
    }

    // ── Live preview of the current texture pattern, right below the settings ──
    y = paint_texture_preview(ctx, theme, x, content_w, y, brush);

    // ── Color Ramp sub-editor (maps the texture's scalar to a colour) ──
    crate::paint_texture_ramp::paint_texture_ramp_section(ctx, theme, x, content_w, y, brush)
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
    // Render at the rect's aspect (bounded cost), then scale-blit to the rect.
    let bw = 140u32;
    let bh = ((ph / content_w * bw as f32).round() as u32).clamp(8, 140);
    let mut buf = vec![0u8; (bw * bh * 4) as usize];
    render_texture_preview(
        TextureKind::from_u8(brush.texture_kind),
        brush.texture_params,
        brush.texture_size,
        brush.texture_offset,
        None, // imported-image pixels aren't in the snapshot → Image kind previews flat
        ramp,
        &mut buf,
        bw,
        bh,
    );
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
fn build_ramp_preview_lut(brush: &BrushSettings, out: &mut [[f32; 4]; 256]) -> bool {
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

/// Map a stored offset (tile fractions, `[TEX_OFFSET_MIN, TEX_OFFSET_MAX]`) onto the slider's
/// `0..1` track. Inverse of the tool's `set_brush_texture_offset_norm`.
fn offset_track(v: f32) -> f32 {
    ((v - TEX_OFFSET_MIN) / (TEX_OFFSET_MAX - TEX_OFFSET_MIN)).clamp(0.0, 1.0)
}

/// Map a stored scale (`[TEX_SIZE_MIN, TEX_SIZE_MAX]`) onto the slider's `0..1` track.
fn size_track(v: f32) -> f32 {
    ((v - TEX_SIZE_MIN) / (TEX_SIZE_MAX - TEX_SIZE_MIN)).clamp(0.0, 1.0)
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
