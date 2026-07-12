//! As seções de APARÊNCIA do painel de Brush (6–11) — Randomize · Shape · Shape Tone ·
//! Paper · Grain · Stroke · Symmetry · Tiling · Watercolor. Arquivo irmão de
//! `paint_brush.rs`, do qual foi extraído pelos caps de painel (200 LOC/fn, 600/arquivo):
//! a linha Painter cresceu a seção de Tiling e estourou até a dispensa de 215 LOC que a
//! função tinha. Mesmo contrato de todo helper de seção: recebe o `y` corrente, devolve o novo.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::showcase::paint_section_separator;
use ph2d_tool_painter::BrushSettings;

/// Sections 6–11 — the dab-appearance + stroke half of the Brush body (Randomize · Shape · Shape
/// Tone · Paper · Grain · Stroke · Symmetry · Tiling · Watercolor). Split out of
/// [`paint_brush_body`] for the 200-LOC/fn panel cap; same contract as every section helper —
/// takes the running `y` and returns the new one.
pub(crate) fn paint_appearance_sections(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    mut y: f32,
    brush: BrushSettings,
) -> f32 {
    use crate::paint_brush_top::paint_randomize_section;

    // Sections below are separated by the Inspector's discreet divider line (Enio 2026-06-25).
    let sep = paint_section_separator;

    // ── Watercolor — the wet-media look (edge darkening + granulation + pigment). MOVED to the head of the
    //    appearance half (Enio 2026-07-12): it is the switch that REINTERPRETS everything under it — the
    //    Grain slot becomes the granulation map, the Paper section appears, the optics take over the
    //    deposit — so it belongs ABOVE what it governs, not buried below the Stroke sections. Hidden in
    //    Inpaint like the rest of the appearance half. Off by default ⇒ a plain brush is byte-identical.
    //    `docs/Painter/08_…`. ──
    if !brush.is_inpaint {
        y = sep(ctx.scene, theme, x, content_w, y);
        y = crate::paint_watercolor::paint_watercolor_section(ctx, theme, x, content_w, y, brush);
    }

    // ── Section 6: Randomize Color (collapsible; activates on amount > 0). Hidden in Smear/Blur/Clone,
    //    Eraser AND Mask (all colourless — nothing to randomize). ──
    if !brush.paints_no_color() && !brush.eraser && !brush.is_mask {
        y = sep(ctx.scene, theme, x, content_w, y);
        y = paint_randomize_section(ctx, theme, x, content_w, y, brush);
    }

    // ── Sections 7–10: Shape · Shape Tone · Grain · Stroke · Symmetry · Tiling — the dab-appearance +
    //    stroke sections. ALL HIDDEN in Inpaint mode: the heal marks a hard-disc defect mask (it ignores
    //    the Shape silhouette / Falloff / Grain / ramps / stroke dynamics / symmetry / tiling entirely),
    //    so the only relevant controls are Size (above) + the Inpaint card. ──
    if !brush.is_inpaint {
        // ── Section 7: Shape — the dab silhouette. Hosts the Falloff (the procedural default tip) + its
        //    curve preview + a source picker; once a Shape image is assigned the Falloff goes inactive
        //    (replaced by the image + its preview). Sits ABOVE Grain (Enio 2026-06-25). ──
        y = sep(ctx.scene, theme, x, content_w, y);
        y = crate::paint_shape::paint_shape_section(ctx, theme, x, content_w, y, brush);

        // ── Section 7b: Shape Tone — the Shape's B&W value ramp (tonal remap of the silhouette), below
        //    the Shape section. Shown in ALL modes (Smear/Blur/Clone force it to a B&W coverage tone);
        //    HIDDEN only while Per-Layer Color owns the colour per layer (the ramp is nullified). ──
        if !brush.shape_per_layer_color {
            y = sep(ctx.scene, theme, x, content_w, y);
            y = crate::paint_shape_ramp::paint_shape_ramp_section(
                ctx, theme, x, content_w, y, brush,
            );
        }

        // ── Watercolor **Paper** section (the substrate) — ABOVE Grain, watercolor mode only. In
        //    watercolor the Grain slot IS the granulation map (`docs/Painter/10…` §5). ──
        if brush.watercolor {
            y = sep(ctx.scene, theme, x, content_w, y);
            y = crate::paint_watercolor_paper::paint_paper_section(
                ctx, theme, x, content_w, y, brush,
            );
        }

        // ── Section 8: Grain — the texture inside the silhouette (was "Texture", Enio 2026-06-25);
        //    IS the granulation map in watercolor mode ("Same as Paper" + Amount at its top). ──
        y = sep(ctx.scene, theme, x, content_w, y);
        y = crate::paint_texture::paint_texture_section(ctx, theme, x, content_w, y, brush, false);

        // ── Section 9: Stroke · 9b: Symmetry · 10: Tiling (last two collapsed by default) ──
        y = sep(ctx.scene, theme, x, content_w, y);
        y = crate::paint_stroke::paint_stroke_section(ctx, theme, x, content_w, y, brush);
        // Symmetry — hidden in Clone: mirrored dabs would clone from mirrored source positions.
        if !brush.is_clone {
            y = sep(ctx.scene, theme, x, content_w, y);
            y = crate::paint_symmetry::paint_symmetry_section(ctx, theme, x, content_w, y, brush);
        }
        y = sep(ctx.scene, theme, x, content_w, y);
        y = crate::paint_stroke::paint_tiling_section(ctx, theme, x, content_w, y, brush);
    }
    // Eraser is the left-rail Eraser tool (a mode), not a panel checkbox — its former standalone
    // checkbox was removed (Enio). In Eraser mode the panel is the normal Brush panel; only the ramp
    // B&W buttons lock checked (erasing has no colour) — handled in the ramp section builders.
    y
}
