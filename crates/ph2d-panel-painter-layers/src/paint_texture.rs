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
    BrushSettings, TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN,
    TextureKind, TextureMapping, param_specs, render_texture_preview,
};
use ph2d_vector::ImageQuality;

/// Paint the Texture section starting at `y`, returning the next `y`. The Kind picker + Mapping
/// dropdowns stash their open rects for the deferred [`paint_texture_popovers`] pass.
pub(crate) fn paint_texture_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let mut y = section_header(ctx, theme, x, content_w, y, "Texture");
    let kind = TextureKind::from_u8(brush.texture_kind);
    let mapping = TextureMapping::from_u8(brush.texture_mapping);

    // ── Kind picker ("thumbnail") + New (always) ──
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

    // Everything below modulates an assigned texture — hide it when there is none (no dead controls).
    if kind == TextureKind::None {
        return y;
    }

    // ── Mapping dropdown ──
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

/// Real-time grayscale preview of the active texture pattern: re-rendered each frame from the
/// published snapshot (kind + params + size + offset) into a small buffer, then scale-blitted as one
/// image. Bounded resolution keeps it cheap; reflects every shape knob live as the user drags.
fn paint_texture_preview(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let ph = (content_w * 0.5).clamp(56.0, 120.0); // a wide preview strip
    let rect = Rect::new(x, y, content_w, ph);
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

/// The texture kinds as dropdown options (includes `None` so the artist can clear the texture).
fn texture_kind_options() -> Vec<DropdownOption<u8>> {
    (0..TextureKind::COUNT)
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
