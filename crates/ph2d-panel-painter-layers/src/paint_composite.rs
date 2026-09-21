//! The **Composite Brush** card (below Strength, above Accumulate). A bordered box with a "Composite
//! Brush" checkbox; when on, the single Strength slider hides and the card shows the 5-layer stack —
//! Brush · Smear · Blur · Erase — as reorderable rows modelled on the Layers panel.
//!
//! ## Cada camada são DUAS fileiras, e a segunda é a wave de 2026-09-20
//!
//! - **fileira A** — `N` · o chip da OPERAÇÃO (clica e cicla) · a Strength nua + o mostrador
//!   simples (o padrão da opacidade do painel de Layers, que nunca empilha num painel estreito) ·
//!   as setas ↑/↓ de reordenar, apagadas e inertes nas pontas da lista.
//! - **fileira B** — recuada: a **amostra de cor** (mais o botão que a devolve à cor do pincel,
//!   pintado só quando há cor autorada) e o **tamanho do carimbo**, em multiplicadores do raio.
//!
//! ⚠️ **A fileira B só é pintada para quem a LÊ.** O `Erase` não deposita a cor do pincel
//! (`CompositeOp::deposita_cor`), logo a amostra dele seria um controlo morto — a fileira dele leva
//! só o tamanho. *Um controlo que o barro não sente é a espécie que o §5.0 do `CLAUDE.md` nomeia.*
//!
//! A posição `N` é FIXA de 1 a 5; a ferramenta em cada posição move-se com as setas. Split from
//! `paint_brush` for the LOC cap; the fixed-id widgets are registered in `crate::populate` (so the
//! panel-wiring-parity gate sees them).

use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, Checkbox, CheckboxValue, ColorSwatch, Slider, SwatchSize, SwatchState, paint_button,
    paint_checkbox, paint_color_swatch, paint_slider,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::BrushSettings;

/// Fixed-width label column for `N` (the op name moved into its own chip in the same row).
const LABEL_W: f32 = 14.0; // LITERAL-PX-OK: the fixed position number column ("1".."5")
/// The operation chip column ("Smear" is the widest of the four names at the Base font).
const OP_W: f32 = 44.0; // LITERAL-PX-OK: op chip column (fits "Smear")
/// Plain "0.50" strength readout column, right of the bare slider.
const READOUT_W: f32 = 34.0; // LITERAL-PX-OK: strength readout column
/// Reorder ↑/↓ button column width (mirrors the Layers panel's `REORDER_W`).
const ARROW_W: f32 = 16.0; // LITERAL-PX-OK: reorder button column (matches paint_rows)
/// Minimum bare-slider track width (chrome floor for a very narrow panel).
const MIN_SLIDER_W: f32 = 24.0; // LITERAL-PX-OK: slider track floor
/// The colour swatch column of row B.
const SWATCH_W: f32 = 22.0; // LITERAL-PX-OK: per-layer colour swatch
/// The "back to the brush colour" button of row B.
const CLEAR_W: f32 = 16.0; // LITERAL-PX-OK: the clear-override button
/// Row B's indent — it hangs under its layer's row A.
const INDENT: f32 = 14.0; // LITERAL-PX-OK: row B hangs under row A

/// ⚠️ **A contagem é a do MOTOR** (`BrushSettings::composite_ops`), nunca um literal: a extensão de
/// 2026-09-20 encontrou exactamente um `3` escrito à mão neste ficheiro, e ele teria deixado duas
/// camadas sem fileira com o motor a correr as cinco.
fn n_camadas(brush: &BrushSettings) -> usize {
    brush.composite_ops.len()
}

/// The name shown in each layer row for a composite op wire discriminant.
fn op_name(op: u8) -> &'static str {
    match op {
        1 => tr("panel.painter_layers.composite.smear"),
        2 => tr("panel.painter_layers.composite.blur"),
        3 => tr("panel.painter_layers.composite.erase"),
        _ => tr("panel.painter_layers.composite.brush"),
    }
}

/// Esta operação deposita a cor do pincel? (o espelho do `CompositeOp::deposita_cor` do motor —
/// ⚠️ o painel não vê o enum, e o gate `a_fileira_da_cor_so_existe_onde_a_cor_chega` ata os dois.)
fn deposita_cor(op: u8) -> bool {
    op == 0
}

fn enc(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8 // LITERAL-PX-OK: sRGB 8-bit normalize
}

/// Paint the Composite Brush card and return the next `y`. Draws its own bordered background so it reads
/// as a distinct card. The Strength-slider hide is the caller's job (it owns the row order); this only
/// renders the card.
pub(crate) fn paint_composite_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let pad = Spacing::Sm.px();
    let gap = Spacing::Xs.px();
    let checked = brush.composite_enabled;
    let n = n_camadas(&brush);
    // Single-line rows (bare slider never stacks), so the height is exact: padding + the checkbox row +
    // (when on) a gap and TWO rows per layer (each `ph2d_tokens::row_pitch_px()`).
    let layers_h = if checked {
        gap + 2.0 * n as f32 * ph2d_tokens::row_pitch_px()
    } else {
        0.0
    };
    let card_h = pad + ROW_H_PX + layers_h + pad;
    let card = Rect::new(x, y, content_w, card_h);
    // ⭐ Raio e moldura pela porta do TEMA: o cartão é plano num tema moderno.
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(
        ctx.scene,
        card,
        radius,
        resolve(
            ph2d_editor_core::widget::section_cards::CardDepth::Subsection.token(),
            theme,
        ),
    );
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        card,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );

    let ix = x + pad;
    let iw = content_w - 2.0 * pad;
    let mut iy = y + pad;

    // The enable checkbox (forwards a plain Click → the tool's `toggle_composite`).
    // ⚠️ Ela é uma linha de propriedade como as outras, logo o nome vive na coluna da SECÇÃO
    // (spec §6-quinquies) — aqui construída à mão só por causa do `visual`, não por ser outra lei.
    const CHAVE: &str = "panel.painter_layers.composite.composite_brush";
    let seccao = crate::seccoes::seccao_da_chave(ctx.text_system, CHAVE);
    let cb = Checkbox::new(
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ENABLE,
        tr(CHAVE),
    )
    .visual(
        ctx.host
            .store()
            .checkbox_visual(ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ENABLE),
    )
    .value(if checked {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    })
    .seccao(seccao);
    let cb_rect = Rect::new(ix, iy, iw, ROW_H_PX);
    paint_checkbox(&cb, cb_rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ENABLE,
        cb_rect,
    );
    iy += ROW_H_PX;

    if checked {
        iy += gap;
        for pos in 0..n {
            iy = paint_layer_row(ctx, theme, ix, iw, iy, pos, brush);
            iy = paint_layer_row_b(ctx, theme, ix, iw, iy, pos, brush);
        }
    }
    y + card_h + ph2d_tokens::control_gap_px()
}

/// Row A: `N` · op chip · bare Strength slider · "0.50" readout · ↑/↓ reorder. The number `N` =
/// `pos + 1` is the FIXED position; the tool name follows the reordered stack. The top row's ↑ and the
/// bottom row's ↓ paint dim and are inert (list edges), keeping every row aligned.
fn paint_layer_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    row_w: f32,
    y: f32,
    pos: usize,
    brush: BrushSettings,
) -> f32 {
    let gap = Spacing::Xs.px();
    let font = TypeToken::Base.px();
    let n = n_camadas(&brush);
    let down_x = x + row_w - ARROW_W;
    let up_x = down_x - gap - ARROW_W;
    let readout_x = up_x - gap - READOUT_W;
    let op_x = x + LABEL_W + gap;
    let slider_x = op_x + OP_W + gap;
    let slider_w = (readout_x - gap - slider_x).max(MIN_SLIDER_W);

    // The fixed position number.
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!("{}", pos + 1),
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        LABEL_W,
        resolve(ColorToken::Text1, theme),
    );

    // O chip da OPERAÇÃO — um clique cicla Brush → Smear → Blur → Erase.
    let op_id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_OP[pos];
    let op_rect = Rect::new(op_x, y, OP_W, ROW_H_PX);
    paint_button(
        &Button::new(op_id, op_name(brush.composite_ops[pos])),
        op_rect,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(op_id, op_rect);

    // Bare Strength slider (value from the snapshot; state from the store) + plain readout.
    let sid = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[pos];
    let val = brush.composite_strength[pos].clamp(0.0, 1.0);
    let mut slider = Slider::new(sid, "")
        .accent(true)
        .visual(ctx.host.store().slider_visual(sid));
    slider.value = val;
    let slider_rect = Rect::new(slider_x, y, slider_w, ROW_H_PX);
    paint_slider(&slider, slider_rect, ctx.scene, theme);
    ctx.host.hit_index_mut().register(sid, slider_rect);
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!("{val:.2}"),
        readout_x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        READOUT_W,
        resolve(ColorToken::Text2, theme),
    );

    // Reorder arrows (dim + inert at the list edges) — shared with the Layers rows.
    crate::paint_rows::paint_reorder_btn(
        ctx,
        theme,
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_UP[pos],
        Rect::new(up_x, y, ARROW_W, ROW_H_PX),
        pos > 0,
        IconId::ChevronUp,
    );
    crate::paint_rows::paint_reorder_btn(
        ctx,
        theme,
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_DOWN[pos],
        Rect::new(down_x, y, ARROW_W, ROW_H_PX),
        pos + 1 < n,
        IconId::ChevronDown,
    );

    y + ph2d_tokens::row_pitch_px()
}

/// Row B: a **cor** desta camada (amostra + o botão que a devolve ao pincel) e o **tamanho** do
/// carimbo dela. Recuada sob a fileira A.
fn paint_layer_row_b(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    row_w: f32,
    y: f32,
    pos: usize,
    brush: BrushSettings,
) -> f32 {
    let gap = Spacing::Xs.px();
    let font = TypeToken::Sm.px();
    let bx = x + INDENT;
    let mut cx = bx;

    // ── A cor, só para quem a deposita ───────────────────────────────────────────────────────
    if deposita_cor(brush.composite_ops[pos]) {
        let sw_id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_COLOR[pos];
        let c = brush.composite_color[pos];
        let aberto = ctx.host.store().picker_target() == Some(sw_id);
        let sw_rect = Rect::new(cx, y, SWATCH_W, ROW_H_PX);
        paint_color_swatch(
            &ColorSwatch {
                id: sw_id,
                label: String::new(),
                rgba: [enc(c[0]), enc(c[1]), enc(c[2]), 255],
                state: if aberto {
                    SwatchState::Focused
                } else {
                    SwatchState::Normal
                },
                size: SwatchSize::Sm,
            },
            sw_rect,
            ctx.scene,
            theme,
        );
        ctx.host.hit_index_mut().register(sw_id, sw_rect);
        crate::composite_picker::readback(ctx, sw_id, c);
        cx += SWATCH_W + gap;

        // ⛔ O botão de VOLTA só existe quando há uma cor autorada para limpar.
        if brush.composite_color_authored[pos] {
            crate::paint_rows::paint_reorder_btn(
                ctx,
                theme,
                ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_COLOR_CLEAR[pos],
                Rect::new(cx, y, CLEAR_W, ROW_H_PX),
                true,
                IconId::Close,
            );
            cx += CLEAR_W + gap;
        }
    }

    // ── O tamanho do carimbo, em multiplicadores do raio do pincel ───────────────────────────
    let size_id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_SIZE[pos];
    let mult = brush.composite_size[pos];
    let readout_x = x + row_w - READOUT_W;
    let slider_w = (readout_x - gap - cx).max(MIN_SLIDER_W);
    let mut slider = Slider::new(size_id, "").visual(ctx.host.store().slider_visual(size_id));
    slider.value = (mult / ph2d_tool_painter::MAX_COMPOSITE_LAYER_SIZE).clamp(0.0, 1.0);
    let slider_rect = Rect::new(cx, y, slider_w, ROW_H_PX);
    paint_slider(&slider, slider_rect, ctx.scene, theme);
    ctx.host.hit_index_mut().register(size_id, slider_rect);
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!("{mult:.2}x"),
        readout_x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        READOUT_W,
        resolve(ColorToken::Text2, theme),
    );

    y + ph2d_tokens::row_pitch_px()
}
