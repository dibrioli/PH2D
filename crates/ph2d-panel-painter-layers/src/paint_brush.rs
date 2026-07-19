//! The Painter dock's **Brush-properties** view (header toggle → "Brush"): the
//! active brush's Size slider, blend-mode chip, and a colour swatch that opens
//! the shared Blender colour picker (`INSP_BLENDER_PICKER`, the same rich picker
//! the Inspector uses — only one is ever open, so they share the slot).
//!
//! The brush is tool-global, so these are FIXED-id widgets (registered in
//! [`crate::populate`]). The panel reads the published [`BrushSettings`] snapshot
//! to position them and forwards edits over the frozen `PanelEvent` channel. The
//! colour round-trip is a per-frame read-back: when the floating picker targets our
//! swatch, its live value (mirrored by the hero loop into `widget_color(target)`) is forwarded.

use crate::paint::register_button;
use crate::state;
use ph2d_editor_core::IconId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{
    self as core_ids, painter_brush_blend_option_id, painter_brush_falloff_option_id,
    painter_brush_preset_option_id,
};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::panel_chrome::PANEL_HEAD_PAD;
use ph2d_editor_core::widget::showcase::paint_section_separator;
use ph2d_editor_core::widget::{DropdownOption, DropdownState};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{BrushBlend, BrushSettings, Falloff, MAX_BRUSH_BLEND_MODES, MAX_FALLOFF};

const LABEL_W: f32 = 60.0; // LITERAL-PX-OK: brush row label column ("Hardness"/"Strength")

// The pre-publish `FALLBACK_BRUSH` snapshot lives in the sibling `brush_fallback` module (file-LOC cap);
// re-exported here so `crate::paint_brush::FALLBACK_BRUSH` (tests + the body) stays stable.
pub(crate) use crate::brush_fallback::FALLBACK_BRUSH;

/// Paint the Brush-properties body rows from `top_y` (already panel-scroll-offset), returning the
/// content-bottom `y`. The caller clips to the body viewport, measures the height for the scrollbar,
/// and drains the dropdown popovers via [`paint_brush_popovers`] AFTER popping the clip.
pub(crate) fn paint_brush_body(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rect: Rect,
    top_y: f32,
) -> f32 {
    let x = rect.x + PANEL_HEAD_PAD;
    let content_w = rect.w - PANEL_HEAD_PAD * 2.0;
    let brush = state::current_brush().unwrap_or(FALLBACK_BRUSH);
    // Deform mode owns the whole panel body (mode-exclusive, like Selection): paint ONLY the Deform section.
    if brush.is_deform {
        return crate::paint_deform::paint_deform_section(ctx, theme, x, content_w, top_y, brush);
    }
    // Selection mode owns the whole panel body (mode-exclusive, ADR-0103): paint ONLY the Selection
    // section — no shared brush control leaks in (the Inpaint precedent).
    if brush.is_selection {
        return crate::paint_selection::paint_selection_section(
            ctx, theme, x, content_w, top_y, brush,
        );
    }
    // Sculpt is the exception to the two above, and deliberately so: its card is ADDED to the brush body
    // rather than replacing it. The sculpt rides the same dab list the colour does, so the brush's own
    // Size / Spacing / Falloff / Shape / Grain / Symmetry / Tiling / stroke method ARE the spatula — take
    // them off screen and you have left the artist the settings for a tool they can no longer aim. What
    // the mode hides instead is the COLOUR half, through `paints_no_color` (see `snapshot.rs`).
    let mut top_y = top_y;
    if brush.is_sculpt {
        top_y = crate::paint_sculpt::paint_sculpt_section(ctx, theme, x, content_w, top_y, brush);
    }
    // If the shared picker is editing our swatch, forward its live colour.
    brush_color_readback(ctx, brush);
    // Keep the store's swatch colour synced to the brush colour while the picker is CLOSED, so the
    // left-rail Eyedropper (the rich colour picker) can seed the picker with it on open — the swatch
    // itself isn't painted in every mode. While the picker owns the swatch, IT drives the value.
    if ctx.host.store().picker_target() != Some(core_ids::PAINTER_COLOR_THUMB) {
        let [r, g, b] = encode_rgb(brush.color);
        ctx.host
            .store_mut()
            .set_widget_color(core_ids::PAINTER_COLOR_THUMB, [r, g, b, 255]);
    }

    let mut y = top_y;
    use crate::paint_brush_top::{paint_checkbox_row, paint_slider_chip_row};

    // **Preset** dropdown at the very TOP — one-click media presets (Digital Basic / Watercolor Basic).
    y = paint_preset_row(ctx, theme, x, content_w, y, brush);

    // "Sync with other tools" — at the very TOP of every tool's panel. Off (default) = this tool keeps its
    // own settings; on = all paint tools share them (the panel where it's checked seeds the others).
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_BRUSH_SYNC,
        "Sync with other tools",
        brush.link_shared,
    );

    // Mask section (Mask tool only): collapsible block at the TOP — sub-brush, canvas ops, overlay colour.
    if brush.is_mask {
        y = crate::paint_mask::paint_mask_section(ctx, theme, x, content_w, y, brush);
        y = paint_section_separator(ctx.scene, theme, x, content_w, y);
    }

    // ── TOP basics (no section), reordered (Enio 2026-06-24):
    //    Blend · Color · Size · Strength · Accumulate · Falloff ──

    // 1. Blend · 2. Color — hidden in Smear/Blur/Clone (process pixels) AND Mask (paints a grayscale
    //    value from the ramp/luma, no colour).
    if !brush.paints_no_color() && !brush.is_mask {
        // #4 (doc 13): the Blend dropdown is INERT in Watercolor mode — the optical wash deposits
        // source-over + its own Beer–Lambert optics, never `BrushBlend` (the layer's own Blend still
        // applies). Hide the dead control there rather than leave it clickable-but-dead (UI honesty;
        // mirrors the Composite card, which also hides in watercolor). KEEP Color — the wash's pigment
        // IS the brush colour. Not painted ⇒ not hit-indexed ⇒ inert; the `populate` register stays.
        if !brush.watercolor {
            let (ny, blend_open) = paint_dropdown_row(
                ctx,
                theme,
                x,
                content_w,
                y,
                "Blend",
                core_ids::PAINTER_BRUSH_BLEND,
                brush.blend,
                BrushBlend::from_u8(brush.blend).name(),
            );
            y = ny;
            if let Some(r) = blend_open {
                state::set_pending_brush_blend_dd(Some((r, brush.blend)));
            }
        }
        y = paint_color_swatch_row(ctx, theme, x, content_w, y, brush);
    }

    // 3. Size + 4. Strength — canonical slider + editable numeric chip (Widget-Gallery look).
    y = paint_slider_chip_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Size",
        core_ids::PAINTER_BRUSH_SIZE_SLIDER,
        core_ids::PAINTER_BRUSH_SIZE_CHIP,
        brush.size_norm,
    );
    // Strength — the single opacity slider. Hidden when the Composite Brush is on: its per-layer
    // Strength sliders (in the card below) replace it. Also hidden in Inpaint (the heal has no opacity).
    // In WATERCOLOR mode the Composite card itself hides (the optical path bypasses it), so a
    // composite_enabled flag left on must NOT hide Strength there.
    if (!brush.composite_enabled || brush.watercolor) && !brush.is_inpaint {
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Strength",
            core_ids::PAINTER_BRUSH_STRENGTH_SLIDER,
            core_ids::PAINTER_BRUSH_STRENGTH_CHIP,
            brush.strength,
        );
    }

    // 4b. Clone card (Set Source + Aligned) — Clone mode only.
    if brush.is_clone {
        y = crate::paint_clone::paint_clone_card(ctx, theme, x, content_w, y, brush);
    }

    // 4b′. Inpaint card (Patch Size / Quality / Search) — Inpaint heal mode only.
    if brush.is_inpaint {
        y = crate::paint_inpaint::paint_inpaint_card(ctx, theme, x, content_w, y, brush);
    }

    // 4c. Composite Brush card (checkbox + the 3-layer Brush/Smear/Blur stack when on) — the plain Brush
    //     tool only (Smear/Blur/Clone/Mask are single-op rail tools; Eraser bypasses composite too — it's
    //     the Erase-Alpha override; `composite_active()` requires the plain Brush + `!eraser`).
    //     HIDDEN in watercolor mode too (Enio 2026-07-07): the optical render-path short-circuits
    //     before the composite routing, so the card would be painted-but-inert (dead UI).
    //     …and HIDDEN in Sculpt for the same reason: `composite_active()` requires `PaintMode::Paint`, so
    //     the card would be painted-but-inert there too. It also has a second-order bite — see
    //     `BrushSettings::paints_no_color`, which the Composite checkbox used to be able to switch OFF,
    //     bringing Blend / Colour / Accumulate / Randomize back into a mode that writes no pigment at all.
    if !brush.is_smear
        && !brush.is_blur
        && !brush.is_clone
        && !brush.eraser
        && !brush.is_mask
        && !brush.is_inpaint
        && !brush.is_sculpt
        && !brush.watercolor
    {
        y = crate::paint_composite::paint_composite_card(ctx, theme, x, content_w, y, brush);
    }

    // 5. Accumulate (checkbox — caps the stroke at Strength when off). Hidden in Smear/Blur (neither uses
    //    the paint-side Strength cap) and, since 2026-07-12 (Enio), under the **optical wash**: the
    //    watercolor path short-circuits before the stamp routing, and `accumulate` is read ONLY there
    //    (`accumulate_cap`), so the checkbox was painted-but-inert — dead UI, exactly like the Composite
    //    card hidden above for the same reason. It is also redundant by construction: the wash's coverage
    //    is MAX-blended (an envelope), which IS "no build-up within a stroke". Note this keys off
    //    `watercolor_active`, not the checkbox: in Eraser/Mask/Inpaint the plain deposit runs again and
    //    Accumulate goes back to meaning something. **Strength stays** — the engine bakes it into
    //    `Dab.coverage` and the wash reads it as the deposit peak (`coverage × (1 − Dilution)`).
    //    Escondido também com **IMPASTO** ligado (Enio 2026-07-18, depois do smoke): ali ele governa
    //    só metade da tinta. O relevo é um ENVELOPE por traço — uma passada de um pincel carregado
    //    deixa uma espessura — então marcar Accumulate acumularia a opacidade e deixaria o CORPO
    //    onde estava: as duas metades da mesma tinta discordando sobre o que uma segunda passada
    //    significa. Estender o acúmulo ao relevo foi construído e **reprovado no smoke**, então a
    //    resposta honesta é não oferecer o controle onde ele só faz metade do que promete.
    if !brush.paints_no_color() && !brush.watercolor_active && !brush.impasto {
        y = paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            core_ids::PAINTER_BRUSH_ACCUMULATE,
            "Accumulate",
            brush.accumulate,
        );
    }

    crate::paint_brush_sections::paint_appearance_sections(ctx, theme, x, content_w, y, brush)
}

/// Drain the Brush-properties dropdown popovers (Blend, Falloff, Stroke Method, Jitter Unit) — one
/// at a time, on TOP of the body. Painted by the caller AFTER the body clip is popped, so an open
/// dropdown is never clipped to the scrolling viewport. The chip rects were stashed during
/// [`paint_brush_body`] (already at their scrolled positions).
pub(crate) fn paint_brush_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip_rect, cur)) = state::take_pending_brush_preset_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_PRESET,
            preset_options(),
            chip_rect,
            cur,
        );
    }
    if let Some((chip_rect, cur)) = state::take_pending_brush_blend_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_BLEND,
            brush_blend_options(),
            chip_rect,
            cur,
        );
    }
    if let Some((chip_rect, cur)) = state::take_pending_brush_falloff_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_FALLOFF,
            falloff_options(),
            chip_rect,
            cur,
        );
    }
    // Shape-section source picker (None / Image).
    if let Some((chip_rect, cur)) = state::take_pending_brush_shape_kind_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_SHAPE_KIND,
            crate::paint_shape::shape_kind_options(),
            chip_rect,
            cur,
        );
    }
    // Per-layer-colour Shape-layer blend ("B" chip) dropdown.
    if let Some((i, chip_rect, cur)) = state::take_pending_shape_blend_dd() {
        crate::paint_shape_layers::paint_shape_blend_popover(ctx, theme, i, chip_rect, cur);
    }
    // Stroke-section dropdowns (Method + Jitter Unit) — drained last so they sit
    // on top of every body row, same as the Blend/Falloff chips above.
    crate::paint_stroke::paint_stroke_popovers(ctx, theme);
    // Texture-section dropdowns (Kind picker + Mapping).
    crate::paint_texture::paint_texture_popovers(ctx, theme);
    // Shape Tone ramp Interpolation dropdown.
    crate::paint_shape_ramp::paint_shape_ramp_popovers(ctx, theme);
    // Watercolor Paper / Granulation kind dropdowns.
    crate::paint_watercolor_paper::paint_watercolor_popovers(ctx, theme);
}

/// When the shared Blender picker targets the brush swatch, the hero loop mirrors its live value into
/// `widget_color(PAINTER_COLOR_THUMB)`. Forward that colour to the tool (as `"r,g,b"`) when it differs
/// from the brush's current colour, so the picker drives the brush live.
fn brush_color_readback(ctx: &mut PaintCtx, brush: BrushSettings) {
    if ctx.host.store().picker_target() != Some(core_ids::PAINTER_COLOR_THUMB) {
        return;
    }
    let Some(picked) = ctx.host.store().widget_color(core_ids::PAINTER_COLOR_THUMB) else {
        return;
    };
    let cur = encode_rgb(brush.color);
    if [picked[0], picked[1], picked[2]] == cur {
        return; // already applied — don't spam the bus
    }
    ctx.host
        .bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            core_ids::PAINTER_COLOR_THUMB,
            format!("{},{},{}", picked[0], picked[1], picked[2]),
        )));
}

/// Paint the colour preview swatch (a full-width bar). Registered as a button:
/// clicking it toggles the shared Blender picker (see `event.rs`). The accent
/// border shows when the picker is currently editing it.
fn paint_color_swatch_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let font = TypeToken::Sm.px();
    label(ctx, theme, "Color", x, y, font);
    let sx = x + LABEL_W + Spacing::Sm.px();
    let sw = (content_w - LABEL_W - Spacing::Sm.px()).max(0.0);
    let rect = Rect::new(sx, y, sw, ROW_H_PX);
    register_button(ctx.host.store_mut(), core_ids::PAINTER_COLOR_THUMB);

    let [r, g, b] = encode_rgb(brush.color);
    let col = ph2d_vector::Color::from_rgba8(r, g, b, 255); // LITERAL-COLOR-OK: brush colour (data)
    let radius = Radius::Sm.px();
    fill_rounded_rect(ctx.scene, rect, radius, col);
    let open = ctx.host.store().picker_target() == Some(core_ids::PAINTER_COLOR_THUMB);
    let border = if open {
        ColorToken::Accent
    } else {
        ColorToken::Border
    };
    stroke_rounded_rect(
        ctx.scene,
        rect,
        radius,
        StrokeToken::Default.px(),
        resolve(border, theme),
    );
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_COLOR_THUMB, rect);
    y + ROW_H_PX + Spacing::Sm.px()
}

/// The shared scrollable dropdown-popover renderer (moved to its own module for the LOC cap).
/// Re-exported so the many `crate::paint_brush::paint_dropdown_popover` callers keep resolving.
pub(crate) use crate::dropdown_popover::paint_dropdown_popover;

/// A left-aligned, vertically-centred row label in a `ROW_H_PX` cell.
pub(crate) fn label(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    text: &str,
    x: f32,
    y: f32,
    font: f32,
) {
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        LABEL_W,
        resolve(ColorToken::Text2, theme),
    );
}

/// Paint a "label + dropdown chip" row. Returns `(next_y, Some(chip_rect))` when
/// the chip is open (the caller stashes the rect into the matching pending slot).
/// `pub(crate)` so the Stroke section reuses it for Method + Jitter Unit.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_dropdown_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label_txt: &str,
    id: ph2d_a11y::NodeId,
    cur_value: u8,
    cur_label: &str,
) -> (f32, Option<Rect>) {
    let gap = Spacing::Sm.px();
    label(ctx, theme, label_txt, x, y, TypeToken::Sm.px());
    let chip_w = (content_w - LABEL_W - gap).max(0.0);
    let rect = Rect::new(x + LABEL_W + gap, y, chip_w, ROW_H_PX);
    let open = paint_dropdown_chip(ctx, theme, id, cur_value, cur_label, rect);
    (y + ROW_H_PX + Spacing::Sm.px(), open.then_some(rect))
}

/// Paint a dropdown chip (registered as a `Dropdown` for the generic open/close
/// dispatch). Returns whether it is open. Shared by the Blend + Falloff chips.
pub(crate) fn paint_dropdown_chip(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    cur_value: u8,
    cur_label: &str,
    rect: Rect,
) -> bool {
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(cur_value as usize),
        },
    );
    let open = matches!(
        ctx.host.store().get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );

    let radius = Radius::Sm.px();
    fill_rounded_rect(ctx.scene, rect, radius, resolve(ColorToken::Bg1, theme));
    let border = if open {
        ColorToken::Accent
    } else {
        ColorToken::Border
    };
    stroke_rounded_rect(
        ctx.scene,
        rect,
        radius,
        StrokeToken::Default.px(),
        resolve(border, theme),
    );

    let chevron = Spacing::Md.px();
    let pad = Spacing::Sm.px();
    let chevron_rect = Rect::new(
        rect.x + rect.w - pad - chevron,
        rect.y + (rect.h - chevron) * 0.5,
        chevron,
        chevron,
    );
    let icon = if open {
        IconId::ChevronUp
    } else {
        IconId::ChevronDown
    };
    paint_icon(
        ctx.scene,
        icon,
        chevron_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );

    let font = TypeToken::Sm.px();
    let text_x = rect.x + pad;
    let text_w = (chevron_rect.x - Spacing::Xs.px() - text_x).max(0.0);
    paint_text(
        ctx.text_system,
        ctx.scene,
        cur_label,
        text_x,
        rect.y + (rect.h - font) * 0.5,
        font,
        text_w,
        resolve(ColorToken::Text1, theme),
    );

    ctx.host.hit_index_mut().register(id, rect);
    open
}

/// Paint the top-of-panel **Preset** dropdown row, returning the next `y`. The "current" preset is
/// inferred from the master watercolor flag (there's no stored preset id — a preset just seeds the whole
/// `BrushSpec`); selecting one forwards `SelectOption(PAINTER_BRUSH_PRESET, idx)` → `apply_brush_preset`.
fn paint_preset_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let preset_idx = u8::from(brush.watercolor); // 0 = Digital, 1 = Watercolor
    let (ny, preset_open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Preset",
        core_ids::PAINTER_BRUSH_PRESET,
        preset_idx,
        preset_name(preset_idx),
    );
    if let Some(r) = preset_open {
        state::set_pending_brush_preset_dd(Some((r, preset_idx)));
    }
    ny
}

/// Display name for a brush-preset index (`0` = Digital, `1` = Watercolor). English UI (HR-15).
fn preset_name(idx: u8) -> &'static str {
    match idx {
        1 => "Watercolor Basic",
        _ => "Digital Basic",
    }
}

/// The brush presets as `Dropdown` options (value = preset idx, label = display name).
fn preset_options() -> Vec<DropdownOption<u8>> {
    (0..core_ids::PAINTER_BRUSH_PRESET_COUNT)
        .map(|i| DropdownOption::new(painter_brush_preset_option_id(i), i, preset_name(i)))
        .collect()
}

/// The 24 brush blend modes as `Dropdown` options (value = wire discriminant,
/// label = display name).
fn brush_blend_options() -> Vec<DropdownOption<u8>> {
    (0..MAX_BRUSH_BLEND_MODES)
        .map(|m| {
            DropdownOption::new(
                painter_brush_blend_option_id(m),
                m,
                BrushBlend::from_u8(m).name(),
            )
        })
        .collect()
}

/// The falloff presets as `Dropdown` options (value = wire discriminant, label =
/// Blender's preset name).
fn falloff_options() -> Vec<DropdownOption<u8>> {
    (0..MAX_FALLOFF)
        .map(|p| {
            DropdownOption::new(
                painter_brush_falloff_option_id(p),
                p,
                Falloff::from_u8(p).name(),
            )
        })
        .collect()
}

/// Encode a straight-RGB colour in `[0, 1]` (native space) to 8-bit for display.
fn encode_rgb(c: [f32; 3]) -> [u8; 3] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8, // LITERAL-PX-OK: sRGB 8-bit normalize
        (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8, // LITERAL-PX-OK: sRGB 8-bit normalize
        (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8, // LITERAL-PX-OK: sRGB 8-bit normalize
    ]
}
