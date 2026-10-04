//! Brush panel — the reorganized **top basics** (canonical slider-with-chip rows + a checkbox) and
//! the first **collapsible section** ("Randomize Color"), built on the shared Inspector widgets so
//! the Brush panel matches the rest of the app (editable numeric chips, ALL-CAPS section header with
//! a collapse chevron + assignable colour dot). Split from [`crate::paint_brush`] for the LOC cap.

use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Checkbox, CheckboxValue, IconButtonStyle, IconGlyph, SectionFold, paint_checkbox,
    paint_icon_button, paint_section_header, paint_slider_with_chip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};
use ph2d_tool_painter::BrushSettings;

/// The dock header title: `"Layers"` in the layer view, else the ACTIVE TOOL name in the Brush view
/// (Brush / Eraser / Blur / Smear / Clone / Mask), so switching the left-rail tool retitles the panel.
/// The modes are mutually exclusive; Eraser is a flag on Paint; the Composite Brush is still "Brush".
pub(crate) fn header_title(shows_layers: bool, brush: Option<BrushSettings>) -> &'static str {
    if shows_layers {
        return tr("panel.painter_layers.brush.layers");
    }
    match brush {
        Some(b) if b.is_deform => tr("panel.painter_layers.brush.deform"),
        Some(b) if b.is_sculpt => tr("panel.painter_layers.brush.sculpt"),
        Some(b) if b.is_selection => tr("panel.painter_layers.brush.select"),
        Some(b) if b.is_mask => tr("panel.painter_layers.brush.mask"),
        Some(b) if b.is_inpaint => tr("panel.painter_layers.brush.inpaint"),
        Some(b) if b.is_clone => tr("panel.painter_layers.brush.clone"),
        Some(b) if b.is_blur => tr("panel.painter_layers.brush.blur"),
        Some(b) if b.is_smear => tr("panel.painter_layers.brush.smear"),
        Some(b) if b.eraser => tr("panel.painter_layers.brush.eraser"),
        _ => tr("panel.painter_layers.brush.brush"),
    }
}

/// One canonical "label · slider · editable chip" row (the Widget-Gallery / Inspector look). The chip
/// is `link_slider_number`-linked to the slider in `populate`, so typing in it forwards as the
/// slider's `ValueChanged` over the existing brush-slider channel. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_slider_chip_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label: &str,
    slider_id: ph2d_a11y::NodeId,
    chip_id: ph2d_a11y::NodeId,
    value: f32,
) -> f32 {
    let area = Rect::new(x, y, content_w, ROW_H_PX);
    let used = crate::esmaecer::talvez_esmaecido(ctx, theme, area, &[slider_id, chip_id], |ctx| {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_slider_with_chip(
            area,
            label,
            value,
            slider_id,
            chip_id,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        )
    });
    y + used + ph2d_tokens::control_gap_px()
}

/// A canonical checkbox row (box + label) driven by the brush snapshot. The id stays a `Button` in the
/// store (the click forwards over the existing Click channel — the tool toggles the bool); only the
/// VISUAL is a checkbox. Returns the next `y`.
///
/// ⭐⭐⭐ **Ela recebe a CHAVE do rótulo, não o texto** (2026-09-16) — e a razão é a coluna do nome.
/// Uma linha de marcar é uma linha de propriedade (spec §6-quinquies), logo o nome dela vive na
/// coluna da **secção**; e *quem só tem o texto traduzido não sabe a que secção pertence*. A chave
/// sabe ([`crate::seccoes::nome_da_seccao`]), então a escolha deixa de existir nos ~30 sítios de
/// pintura e passa a ser uma derivação num sítio só.
///
/// ⛔ Sem isto a linha caía no default (*«não sei que nomes vou pintar»*), cuja coluna é a metade
/// cega: medido a `273,3` — a largura do dono — um nome saía cortado, `6` a `245` e `11` no mínimo
/// do dock (a tabela está em [`crate::seccoes`]).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_checkbox_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    chave: &str,
    checked: bool,
) -> f32 {
    let value = if checked {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    };
    let seccao = crate::seccoes::seccao_da_chave(ctx.text_system, chave);
    let cb = Checkbox::new(id, tr(chave))
        .visual(ctx.host.store().checkbox_visual(id))
        .value(value)
        .seccao(seccao);
    let rect = Rect::new(x, y, content_w, ROW_H_PX);
    crate::esmaecer::talvez_esmaecido(ctx, theme, rect, &[id], |ctx| {
        paint_checkbox(&cb, rect, ctx.scene, ctx.text_system, theme);
    });
    ctx.host.hit_index_mut().register(id, rect);
    y + ph2d_tokens::row_pitch_px()
}

/// Paint a collapsible section header (Inspector pattern: ALL-CAPS label + a **reset** icon button + an
/// assignable colour dot + collapse chevron) and register the header (collapse-toggle on click), the
/// reset button, and the colour dot (opens the picker). Returns `(next_y, collapsed)` — the caller
/// paints no body when collapsed. Shared by every Brush-panel section (Randomize / Texture / Color
/// Ramp / Stroke / Tiling). The reset rect is registered AFTER the full-width header so its click hits
/// the reset button (last-wins), not the collapse toggle — mirror of the Inspector's Transform header.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_collapsible_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label: &str,
    section_id: ph2d_a11y::NodeId,
    reset_id: ph2d_a11y::NodeId,
) -> (f32, Option<SectionFold>) {
    let header_h = TypeToken::Md.px() + Spacing::Md.px();
    // ⭐ O cabeçalho sai do LUGAR da secção (`crate::plano_corpo`): a pega se ela se arrasta, o
    //    livro do quadro se é do corpo — o que torna o título clicável para o menu de tema.
    let header = crate::plano_corpo::cabecalho(ctx, section_id, label).reserve_right(header_h);
    let header_rect = Rect::new(x, y, content_w, header_h);
    let reset_state = ctx.host.store().button_visual(reset_id);
    {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        paint_section_header(&header, header_rect, scene, text_system, theme);
    }
    crate::plano_corpo::regista(ctx, section_id, header_rect);
    // Reset icon button — a square matching the header height, at the right edge (LEFT of the grip
    // when the section drags, as the Inspector's Transform does). ⚠️ The colour dot that sat to its
    // right LEFT on 2026-09-29 (owner's order), and the title reserves the button's width so a long
    // name never runs under it.
    let reset_rect = Rect::new(
        x + content_w - crate::plano_corpo::largura_da_pega(section_id) - header_h,
        y,
        header_h,
        header_h,
    );
    paint_icon_button(
        reset_rect,
        IconGlyph::Builtin(IconId::Reset),
        IconButtonStyle::Plain,
        reset_state,
        ctx.scene,
        theme,
    );
    ctx.host.hit_index_mut().register(reset_id, reset_rect);
    let body_top = y + header_h + Spacing::Xs.px();
    let scene = &mut *ctx.scene;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let fold = SectionFold::begin(store, section_id, x, content_w, body_top, scene, hit_index);
    (body_top, fold)
}

/// Fecha a dobra aberta por [`paint_collapsible_section`] e devolve o `y` de saida.
///
/// Vive aqui, ao lado de quem a abre, porque o `finish` quer `&WidgetStore`, `&mut VectorScene` e
/// `&mut HitIndex` ao mesmo tempo, e num `PaintCtx` os tres saem de campos disjuntos.
pub(crate) fn end_fold(ctx: &mut PaintCtx, fold: SectionFold, y: f32) -> f32 {
    let scene = &mut *ctx.scene;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    fold.finish(store, scene, hit_index, y)
}

/// Paint the collapsible "Randomize Color" section: ALL-CAPS header + collapse chevron + assignable
/// colour dot (Inspector pattern). Collapsed → just the header. Expanded → Hue / Saturation / Value
/// editable slider rows (the effect activates when any amount > 0; there is no enable toggle).
pub(crate) fn paint_randomize_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let (mut y, fold) = paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.brush.randomize_color"),
        ph2d_tool_painter::ids::PAINTER_BRUSH_RANDOMIZE_SECTION,
        ph2d_tool_painter::ids::PAINTER_BRUSH_RANDOMIZE_RESET,
    );
    let Some(fold) = fold else {
        return y;
    };
    for (slot, label) in [
        tr("panel.painter_layers.brush.hue"),
        tr("panel.painter_layers.brush.saturation"),
        tr("panel.painter_layers.brush.value"),
    ]
    .into_iter()
    .enumerate()
    {
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            label,
            ph2d_tool_painter::ids::PAINTER_BRUSH_RANDOMIZE_SLIDERS[slot],
            ph2d_tool_painter::ids::PAINTER_BRUSH_RANDOMIZE_CHIPS[slot],
            brush.color_jitter[slot],
        );
    }
    let out = y;
    end_fold(ctx, fold, out)
}
