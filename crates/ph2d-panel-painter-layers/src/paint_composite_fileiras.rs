//! As fileiras recuadas de cada camada do Composite Brush — a Strength própria, a B (prefixo + tamanho),
//! a do prefixo e a C (dureza). Filho de [`super`] por `#[path]` para ler as medidas privadas dele;
//! cortado do `paint_composite` pelo tecto de LOC.

use super::*;

/// A fileira PRÓPRIA da Strength — só quando ela não cabe na fileira A ([`forca_na_fileira_a`]): recuada e na
/// coluna da B e da C (acaba em [`caixa_direita`]).
pub(super) fn paint_layer_row_forca(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    row_w: f32,
    y: f32,
    pos: usize,
    brush: BrushSettings,
) -> f32 {
    let cx = x + ph2d_tokens::list_indent_px();
    let caixa = Rect::new(
        cx,
        y,
        (caixa_direita(x, row_w) - cx).max(MIN_CAIXA_W),
        ROW_H_PX,
    );
    pinta_forca(ctx, theme, caixa, pos, &brush);
    y + ph2d_tokens::row_pitch_px()
}

/// Row B: a **cor** desta camada (amostra + o botão que a devolve ao pincel) — ou o ESCOPO de uma borracha — e o
/// **tamanho** do carimbo dela. Recuada sob a fileira A. `prefixo_aqui` = o que vem antes do tamanho cabe ao lado
/// dele ([`prefixo_ao_lado_do_tamanho`]); senão vai à fileira própria ([`paint_layer_row_prefixo`]).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_layer_row_b(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    row_w: f32,
    y: f32,
    pos: usize,
    brush: BrushSettings,
    prefixo_aqui: bool,
) -> f32 {
    let mut cx = x + ph2d_tokens::list_indent_px();
    if prefixo_aqui {
        cx = pinta_prefixo(ctx, theme, cx, y, pos, &brush);
    }

    // ── O tamanho do carimbo, em px de diâmetro (o motor guarda o multiplicador do raio) ─────
    //
    // ⚠️ **A caixa acaba onde a da fileira A acaba**, e não na borda do cartão: as duas ficam na
    // MESMA coluna. Sem isso a de baixo passava por baixo das setas de reordenar e as duas
    // fileiras da mesma camada liam-se como pertencendo a grelhas diferentes.
    //
    // ⚠️ **O chip mostra px e a barra guarda a fracção** — a projecção é o `liga_tamanho` de
    // `populate_composite_chips`, re-ligada a cada quadro (a escala segue o Size do pincel).
    let size_id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_SIZE[pos];
    crate::populate_composite_chips::liga_tamanho(ctx.host.store_mut(), &brush, pos);
    let caixa_w = (caixa_direita(x, row_w) - cx).max(MIN_CAIXA_W);
    caixa_de_valor(
        ctx,
        theme,
        Rect::new(cx, y, caixa_w, ROW_H_PX),
        tr("panel.painter_layers.composite.size"),
        crate::populate_composite_chips::pista_do_px(&brush, pos),
        f64::from(crate::populate_composite_chips::tamanho_px(&brush, pos)),
        size_id,
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_SIZE_CHIP[pos],
    );

    y + ph2d_tokens::row_pitch_px()
}

/// O PREFIXO da fileira B em `(cx, y)`, com os hit rects; devolve o `x` seguinte.
///
/// - **A cor, só para quem a deposita** — e o botão de VOLTA só quando há cor autorada para limpar.
/// - **O ESCOPO, só numa borracha** (ordem do dono, 2026-09-20: *«se a borracha atua só no próprio traço do
///   Brush ou se ela apaga também a camada da imagem abaixo»*) — numa borracha a cor não existe, logo o chip
///   ocupa o lugar dela sem custar altura enquanto cabe.
fn pinta_prefixo(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    mut cx: f32,
    y: f32,
    pos: usize,
    brush: &BrushSettings,
) -> f32 {
    let gap = Spacing::Xs.px();
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
    if apaga(brush.composite_ops[pos]) {
        let id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ERASE_SCOPE[pos];
        let w = largura_do_chip_do_escopo(ctx.text_system);
        let rect = Rect::new(cx, y, w, ROW_H_PX);
        paint_button(
            &Button::new(id, escopo_name(brush.composite_erase_scope[pos])),
            rect,
            ctx.scene,
            ctx.text_system,
            theme,
        );
        ctx.host.hit_index_mut().register(id, rect);
        cx += w + gap;
    }
    cx
}

/// A fileira PRÓPRIA do prefixo — só quando ele não cabe ao lado do tamanho ([`prefixo_ao_lado_do_tamanho`]):
/// recuada como a B e a C, com o NOME dele («Color» · «Erases», os do cartão novo) medido antes dos controlos.
pub(super) fn paint_layer_row_prefixo(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    y: f32,
    pos: usize,
    brush: BrushSettings,
) -> f32 {
    let gap = Spacing::Xs.px();
    let font = TypeToken::Sm.px();
    let cx = x + ph2d_tokens::list_indent_px();
    let nome = if deposita_cor(brush.composite_ops[pos]) {
        tr("panel.painter_layers.brush.color")
    } else {
        tr("panel.painter_layers.composite.erases")
    };
    let nome_w = ctx.text_system.prefix_width(nome, font);
    paint_text(
        ctx.text_system,
        ctx.scene,
        nome,
        cx,
        y + (ROW_H_PX - font) * 0.5,
        font,
        f32::INFINITY,
        resolve(ColorToken::Text2, theme),
    );
    pinta_prefixo(ctx, theme, cx + nome_w + gap, y, pos, &brush);
    y + ph2d_tokens::row_pitch_px()
}

/// Fileira C: a **dureza** desta camada (ordem do dono, 2026-09-20: *«Hardness para cada um da
/// lista»*), com o botão que a devolve à dureza do pincel. Recuada como a fileira B.
///
/// ⛔⛔ **Ela é uma fileira PRÓPRIA e não um segundo campo na fileira B, e o motivo é MEDIDO:** na
/// largura a que o dono trabalha (`273,3 px` de painel) o que sobra à fileira B depois do recuo, da
/// amostra, do botão de volta e da coluna das setas são `~157 px` — e **duas** caixas ali dariam
/// `~78` cada, quando só a coluna do número de uma caixa mede
/// [`ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX`] (`72`). *O rótulo de cada uma seria elidido
/// até ao nada, e uma caixa sem nome com outra ao lado é pior do que uma linha a mais.*
///
/// ⚠️ O preço é a ALTURA: o cartão passa de duas para três fileiras por camada. O número está
/// medido no handoff, e ele é o que o dono julga.
pub(super) fn paint_layer_row_c(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    row_w: f32,
    y: f32,
    pos: usize,
    brush: BrushSettings,
) -> f32 {
    let gap = Spacing::Xs.px();
    let mut cx = x + ph2d_tokens::list_indent_px();

    // ⛔ O botão de volta só existe quando há dureza AUTORADA para limpar — a mesma lei da cor.
    if brush.composite_hardness_authored[pos] {
        crate::paint_rows::paint_reorder_btn(
            ctx,
            theme,
            ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_HARDNESS_CLEAR[pos],
            Rect::new(cx, y, CLEAR_W, ROW_H_PX),
            true,
            IconId::Close,
        );
        cx += CLEAR_W + gap;
    }

    let dureza = brush.composite_hardness[pos].clamp(0.0, 1.0);
    let caixa_w = (caixa_direita(x, row_w) - cx).max(MIN_CAIXA_W);
    caixa_de_valor(
        ctx,
        theme,
        Rect::new(cx, y, caixa_w, ROW_H_PX),
        tr("panel.painter_layers.composite.hardness"),
        dureza,
        f64::from(dureza),
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_HARDNESS[pos],
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_HARDNESS_CHIP[pos],
    );

    y + ph2d_tokens::row_pitch_px()
}
