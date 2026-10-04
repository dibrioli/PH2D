//! As seções de APARÊNCIA do painel de Brush (6–11) — Randomize · Shape · Shape Tone ·
//! Paper · Grain · Stroke · Symmetry · Tiling · Watercolor. Arquivo irmão de
//! `paint_brush.rs`, do qual foi extraído pelos caps de painel (200 LOC/fn, 600/arquivo):
//! a linha Painter cresceu a seção de Tiling e estourou até a dispensa de 215 LOC que a
//! função tinha. Mesmo contrato de todo helper de seção: recebe o `y` corrente, devolve o novo.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::section_plan_ctx::PlanoCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_i18n::tr;
use ph2d_tool_painter::{BrushSettings, PaintMedia};

/// Paint the **Paint Mode** chip, stashing the open popover for the deferred pass. Returns the next `y`.
///
/// The current value comes from the tool's own [`BrushSettings::media`] — derived there from the three
/// master flags — so the chip cannot disagree with the section painted under it.
fn paint_media_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let cur = brush.media;
    let (ny, open) = crate::paint_brush_rows::paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "panel.painter_layers.brush.paint_mode",
        ph2d_tool_painter::ids::PAINTER_BRUSH_MEDIA,
        cur,
        tr(PaintMedia::from_u8(cur).name_key()),
    );
    if let Some(r) = open {
        crate::state::set_pending_brush_media_dd(Some((r, cur)));
    }
    ny
}

/// The four media as `Dropdown` options (value = the [`PaintMedia`] wire `u8`, label = its name).
/// Built from the enum itself, so a fifth medium cannot be added without appearing here.
pub(crate) fn media_options() -> Vec<DropdownOption<u8>> {
    (0..PaintMedia::COUNT)
        .map(|i| {
            DropdownOption::new(
                ph2d_tool_painter::ids::painter_brush_media_option_id(i),
                i,
                tr(PaintMedia::from_u8(i).name_key()),
            )
        })
        .collect()
}

/// Sections 6–11 — the dab-appearance + stroke half of the Brush body (Paint Mode · Mixing ·
/// Randomize · Shape · Shape Tone · Paper · Grain · Stroke · Symmetry · Tiling), declared into the
/// body's [`PlanoCtx`]. Every entry keeps the condition it always had and returns the `y` untouched
/// when it does not apply, which the plan reads as «not on screen».
///
/// ⭐ **A fronteira entre secções é a borda de um CARTÃO** desde 2026-09-06 (ordem do dono:
/// *«vamos eliminar os nossos divisores azuis»*), e desde 2026-09-30 quem a pinta é o plano — o
/// cartão anterior fecha só se ele pintou ([`ph2d_editor_core::panel::section_plan::Corredor`]).
pub(crate) fn declara_aparencia(
    plano: &mut PlanoCtx<'_>,
    x: f32,
    content_w: f32,
    brush: BrushSettings,
) {
    use crate::paint_brush_top::paint_randomize_section;
    use ph2d_tool_painter::ids as pids;

    // ── **Paint Mode** — the paint's MEDIUM, and then that medium's own section and no other.
    //
    //    Until 2026-07-22 this was three independent **Enable** checkboxes, one at the head of each of
    //    the three sections (Enio: *"temos na seção de modo da pintura 3 checkbox … no lugar dos
    //    checkbox coloque um dropdown para o modo de pintura com os 4 modos. O padrão é o Digital
    //    normal"*). Three booleans express eight states of which four mean anything, and the fourth
    //    medium — the plain Blender-style brush — had no name at all: it was "none of the boxes".
    //
    //    The media reinterpret everything below them (the Grain slot becomes the granulation map, the
    //    Paper section appears, the fluid engine takes the deposit outright), so the chip sits ABOVE
    //    what it governs — and the medium's section is FIXED in the plan (`crate::plano_corpo`):
    //    dragging it under what it governs would put the governor below the governed. Hidden in
    //    Inpaint like the rest of the appearance half.
    //
    //    ⚠️ Exactly ONE of the four entries paints, and the exclusivity is the tool's
    //    (`set_paint_media`), not a re-derivation here. The chip rides INSIDE its medium's entry so
    //    the two share one card, as they always did. Impasto keeps its own `impasto_section_applies`
    //    gate — a medium can be selected in a mode it does not act in (the rail's Blur, say), and
    //    that returns `y` untouched. ──
    let meio = move |m: PaintMedia| !brush.is_inpaint && PaintMedia::from_u8(brush.media) == m;
    plano.bloco(move |ctx, theme, y| {
        if meio(PaintMedia::Digital) {
            paint_media_row(ctx, theme, x, content_w, y, brush)
        } else {
            y
        }
    });
    plano.fixa(pids::PAINTER_WATERCOLOR_SECTION, move |ctx, theme, y| {
        if !meio(PaintMedia::Watercolor) {
            return y;
        }
        let y = paint_media_row(ctx, theme, x, content_w, y, brush);
        crate::paint_watercolor::paint_watercolor_section(ctx, theme, x, content_w, y, brush)
    });
    plano.fixa(pids::PAINTER_IMPASTO_SECTION, move |ctx, theme, y| {
        if !meio(PaintMedia::Impasto) {
            return y;
        }
        let y = paint_media_row(ctx, theme, x, content_w, y, brush);
        crate::paint_impasto::paint_impasto_section(ctx, theme, x, content_w, y, brush)
    });
    plano.fixa(pids::PAINTER_WETPAINT_SECTION, move |ctx, theme, y| {
        if !meio(PaintMedia::WetPaint) {
            return y;
        }
        let y = paint_media_row(ctx, theme, x, content_w, y, brush);
        crate::paint_wetpaint::paint_wetpaint_section(ctx, theme, x, content_w, y, brush)
    });

    // ── O cartão **Mixing** (a fileira `Pigment`) — o ÚNICO hospedeiro dela, nos três meios.
    //
    //    ⭐⭐ Ele nasceu da ordem do dono de 2026-09-20 (*«ligue o digital»*), que tirou a cerca
    //    `watercolor &&` do `effective_pigment_mix`. Desde aquela ordem o `Pigment` vale para três
    //    meios, logo deixou de pertencer à secção de um — e fica no LUGAR, junto do meio.
    //
    //    ⛔ **A condição é UMA** (`pigment_offered`, derivada pela ferramenta do MEIO ∧ do GESTO), e é
    //    ela que responde ao 2.º report do dono — *«confira se funciona para Blur e Smear»*: não
    //    funciona, e antes desta linha a fileira aparecia lá na mesma. ──
    plano.bloco(move |ctx, theme, y| {
        if !brush.is_inpaint && brush.pigment_offered {
            crate::paint_pigment::paint_mixing_section(ctx, theme, x, content_w, y, &brush)
        } else {
            y
        }
    });

    // ── Section 6: Randomize Color (collapsible; activates on amount > 0). Hidden in Smear/Blur/Clone,
    //    Eraser AND Mask (all colourless — nothing to randomize). ──
    plano.seccao(
        pids::PAINTER_BRUSH_RANDOMIZE_SECTION,
        move |ctx, theme, y| {
            if !brush.paints_no_color() && !brush.eraser && !brush.is_mask {
                paint_randomize_section(ctx, theme, x, content_w, y, brush)
            } else {
                y
            }
        },
    );

    // ── Sections 7–10: Shape · Shape Tone · Paper · Grain · Stroke · Symmetry · Tiling — the
    //    dab-appearance + stroke sections. ALL HIDDEN in Inpaint mode: the heal marks a hard-disc
    //    defect mask (it ignores the Shape silhouette / Falloff / Grain / ramps / stroke dynamics /
    //    symmetry / tiling entirely), so the only relevant controls are Size (above) + the Inpaint card.
    let fora = !brush.is_inpaint;

    // ── Section 7: Shape — the dab silhouette. Hosts the Falloff (the procedural default tip) + its
    //    curve preview + a source picker; once a Shape image is assigned the Falloff goes inactive
    //    (replaced by the image + its preview). Sits ABOVE Grain (Enio 2026-06-25). ──
    plano.seccao(pids::PAINTER_SHAPE_SECTION, move |ctx, theme, y| {
        if fora {
            crate::paint_shape::paint_shape_section(ctx, theme, x, content_w, y, brush)
        } else {
            y
        }
    });

    // ── Section 7b: Shape Tone — the Shape's B&W value ramp (tonal remap of the silhouette), below
    //    the Shape section. Shown in ALL modes (Smear/Blur/Clone force it to a B&W coverage tone);
    //    HIDDEN while Per-Layer Color owns the colour per layer (the ramp is nullified) and where the
    //    medium does not offer it (`shape_ramp_offered`: not in Wet Paint). ──
    plano.seccao(pids::PAINTER_SHAPE_RAMP_SECTION, move |ctx, theme, y| {
        if fora && !brush.shape_per_layer_color && brush.shape_ramp_offered() {
            crate::paint_shape_ramp::paint_shape_ramp_section(ctx, theme, x, content_w, y, brush)
        } else {
            y
        }
    });

    // ── The **Paper** section (the substrate) — ABOVE Grain, in EVERY medium.
    //
    //    ⚠️ Era `watercolor || wetpaint`, sob a nota *"escondida para o brush comum / Impasto, que
    //    não leem substrato nenhum (Enio 2026-07-21: 'deve ser assim mesmo')"*. A premissa dela
    //    caiu quando o dente do papel virou SUPERFÍCIE ILUMINADA (`substrate_relief.rs`): hoje o
    //    Digital lê o substrato, e é justamente o meio para o qual o relevo foi pedido. As rows que
    //    continuam sendo só da aguada são gateadas DENTRO da seção, por quem as lê. ──
    plano.seccao(
        pids::PAINTER_WATERCOLOR_PAPER_SECTION,
        move |ctx, theme, y| {
            if fora {
                crate::paint_watercolor_paper::paint_paper_section(
                    ctx, theme, x, content_w, y, brush,
                )
            } else {
                y
            }
        },
    );

    // ── Section 8: Grain — the texture inside the silhouette (was "Texture", Enio 2026-06-25);
    //    IS the granulation map in watercolor mode ("Same as Paper" + Amount at its top). ──
    plano.seccao(pids::PAINTER_BRUSH_TEXTURE_SECTION, move |ctx, theme, y| {
        if fora {
            crate::paint_texture::paint_texture_section(ctx, theme, x, content_w, y, brush, false)
        } else {
            y
        }
    });

    // ── Section 9: Stroke · 9b: Symmetry · 10: Tiling (last two collapsed by default) ──
    plano.seccao(pids::PAINTER_BRUSH_STROKE_SECTION, move |ctx, theme, y| {
        if fora {
            crate::paint_stroke::paint_stroke_section(ctx, theme, x, content_w, y, brush)
        } else {
            y
        }
    });
    // Symmetry — hidden in Clone: mirrored dabs would clone from mirrored source positions.
    plano.seccao(
        pids::PAINTER_BRUSH_SYMMETRY_SECTION,
        move |ctx, theme, y| {
            if fora && !brush.is_clone {
                crate::paint_symmetry::paint_symmetry_section(ctx, theme, x, content_w, y, brush)
            } else {
                y
            }
        },
    );
    plano.seccao(pids::PAINTER_BRUSH_TILING_SECTION, move |ctx, theme, y| {
        if fora {
            crate::paint_stroke::paint_tiling_section(ctx, theme, x, content_w, y, brush)
        } else {
            y
        }
    });
    // Eraser is the left-rail Eraser tool (a mode), not a panel checkbox — its former standalone
    // checkbox was removed (Enio). In Eraser mode the panel is the normal Brush panel; only the ramp
    // B&W buttons lock checked (erasing has no colour) — handled in the ramp section builders.
}
