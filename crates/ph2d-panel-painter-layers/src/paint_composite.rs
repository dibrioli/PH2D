//! The **Composite Brush** card (below Strength, above Accumulate). A bordered box with a "Composite
//! Brush" checkbox; when on, the single Strength slider hides and the card shows the 5-layer stack —
//! Brush · Smear · Blur · Erase — as reorderable rows modelled on the Layers panel.
//!
//! ## Cada camada são DUAS fileiras, e a segunda é a wave de 2026-09-20
//!
//! - **fileira A** — `N` · o chip da OPERAÇÃO (clica e cicla) · a Strength na **caixa única** do
//!   app · as setas ↑/↓ de reordenar, apagadas e inertes nas pontas da lista.
//! - **fileira B** — recuada: a **amostra de cor** (mais o botão que a devolve à cor do pincel,
//!   pintado só quando há cor autorada) e o **tamanho do carimbo**, em multiplicadores do raio,
//!   na mesma caixa única e **na mesma coluna** da fileira A.
//!
//! ⛔⛔ **As duas caixas eram BARRAS NUAS com um mostrador de texto ao lado até 2026-09-20**, e o
//! doc deste ficheiro defendia-as com *«o padrão da opacidade do painel de Layers»*. O dono
//! reprovou-as com foto (*«sliders fora do padrão do app. corrija. sliders no padrão»*) — neste
//! painel toda fileira de valor é a caixa única de 2026-09-02, e *um padrão citado de outro painel
//! é uma segunda resposta com proveniência*. Mecanismo: [`caixa_de_valor`].
//!
//! ⚠️ **A fileira B só é pintada para quem a LÊ.** O `Erase` não deposita a cor do pincel
//! (`CompositeOp::deposita_cor`), logo a amostra dele seria um controlo morto — a fileira dele leva
//! só o tamanho. *Um controlo que o barro não sente é a espécie que o §5.0 do `CLAUDE.md` nomeia.*
//!
//! A posição `N` é FIXA de 1 a 5; a ferramenta em cada posição move-se com as setas. Split from
//! `paint_brush` for the LOC cap; the fixed-id widgets are registered in `crate::populate` (so the
//! panel-wiring-parity gate sees them).

use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, rect_for_label, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, Checkbox, CheckboxValue, ColorSwatch, DEFAULT_CHIP_W, DEFAULT_LABEL_W, SwatchSize,
    SwatchState, paint_button, paint_checkbox, paint_color_swatch, paint_slider_with_chip_layout,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::BrushSettings;

/// **Quantas fileiras cada camada ocupa** — A (operação + força), B (cor + tamanho) e C (dureza).
///
/// ⚠️ Ela é lida pela ALTURA do cartão e pelo laço que pinta; escrita nos dois sítios, acrescentar
/// uma fileira daria um cartão que corta a última camada ao meio — *um número que decide o tamanho
/// de uma caixa e quantas coisas entram nela tem de ser o mesmo número*.
const FILEIRAS_POR_CAMADA: usize = 3;
/// Fixed-width label column for `N` (the op name moved into its own chip in the same row).
const LABEL_W: f32 = 14.0; // LITERAL-PX-OK: the fixed position number column ("1".."5")
/// Reorder ↑/↓ button column width (mirrors the Layers panel's `REORDER_W`).
const ARROW_W: f32 = 16.0; // LITERAL-PX-OK: reorder button column (matches paint_rows)
/// Piso da caixa de valor (chão de cromo num painel muito estreito).
const MIN_CAIXA_W: f32 = 24.0; // LITERAL-PX-OK: value-box floor
/// The colour swatch column of row B.
const SWATCH_W: f32 = 22.0; // LITERAL-PX-OK: per-layer colour swatch
/// The "back to the brush colour" button of row B.
const CLEAR_W: f32 = 16.0; // LITERAL-PX-OK: the clear-override button
// ⛔ O recuo da fileira B **não é uma constante deste ficheiro**: ele vem da porta
// [`ph2d_tokens::list_indent_px`], que é o único número de recuo do app (o gate
// `no_surface_declares_an_indent_constant_of_its_own` apanhou-o aqui à primeira — e o marcador
// `LITERAL-PX-OK` não isenta, porque a queixa não é o literal, é a SEGUNDA resposta).

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

/// ⭐⭐⭐ **A largura da coluna do chip da operação, MEDIDA e nunca escolhida** (ordem do dono,
/// 2026-09-20, com foto: *«nomes achatados»*).
///
/// ⛔⛔ Ela era `const OP_W = 44,0`, com o comentário a afirmar *«fits "Smear"»* — e a foto do dono
/// refutou-o: só `Blur` cabia, e `Brush`, `Smear` e `Erase` saíam `Br…`, `S…` e `Er…`. *Uma largura
/// estimada com a afirmação de que mede é pior do que uma sem comentário nenhum: ela convida a não
/// re-medir.*
///
/// ⚠️ **Três coisas aqui são PORTAS, e nenhuma é uma segunda aritmética:** a fonte sai do
/// [`Button::label_font_px`] (*«quem pergunta se um rótulo cabe lê a resposta AQUI»* — medir num
/// corpo e pintar noutro corta curto), o respiro sai do [`rect_for_label`] (a inversa exacta do
/// orçamento que o pintor gasta, com a correcção de ULP que ele já pagou), e a contagem sai do
/// [`ph2d_tool_painter::N_COMPOSITE_OPS`].
///
/// ⚠️ **A coluna é a da SECÇÃO e não a da linha** — ela mede TODOS os nomes, não o desta camada;
/// senão a coluna saltava debaixo do olho do artista a cada clique no chip.
fn largura_do_chip_da_operacao(text_system: &mut ph2d_text::TextSystem) -> f32 {
    let fonte = Button::label_font_px();
    let mais_largo = (0..ph2d_tool_painter::N_COMPOSITE_OPS)
        .map(|op| text_system.prefix_width(op_name(op as u8), fonte))
        .fold(0.0f32, f32::max);
    rect_for_label(mais_largo)
}

/// ⭐⭐⭐ **A CAIXA ÚNICA do app** — rótulo dentro à esquerda, valor dentro à direita, preenchimento
/// a dizer a fracção (Enio, 2026-09-02), com o chip numérico REAL na coluna do valor.
///
/// ⛔⛔ **Ela substitui a barra NUA + mostrador de texto que este cartão pintava** (ordem do dono,
/// 2026-09-20: *«sliders fora do padrão do app. corrija. sliders no padrão»*). O doc da 1.ª
/// redacção defendia a barra nua com *«o padrão da opacidade do painel de Layers, que nunca empilha
/// num painel estreito»* — ⚠️ **verdade sobre aquela linha e falsa sobre ESTE painel**, onde toda
/// fileira de valor é a caixa única. *Um padrão citado de outro painel é uma segunda resposta com
/// proveniência, e proveniência lê-se como justificação.*
///
/// ⚠️ **Não-adaptativa de propósito:** a variante que demota o rótulo para uma linha própria é o que
/// «empilha num painel estreito», e num cartão de altura FIXA (`card_frame` dimensiona por
/// `n_rows`) uma linha que cresce escreve por cima da seguinte.
#[allow(clippy::too_many_arguments)]
fn caixa_de_valor(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rect: Rect,
    rotulo: &str,
    fraccao: f32,
    valor_do_chip: f64,
    slider_id: ph2d_a11y::NodeId,
    chip_id: ph2d_a11y::NodeId,
) {
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_slider_with_chip_layout(
        rect,
        rotulo,
        fraccao,
        valor_do_chip,
        None,
        slider_id,
        chip_id,
        DEFAULT_LABEL_W,
        DEFAULT_CHIP_W,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
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
    // ⚠️ **A altura é EXACTA porque nenhuma fileira empilha** — a caixa única entra pela variante
    // NÃO-adaptativa de propósito (ver [`caixa_de_valor`]): padding + a linha da caixa de marcar +
    // (ligada) um vão e [`FILEIRAS_POR_CAMADA`] fileiras por camada.
    let layers_h = if checked {
        gap + FILEIRAS_POR_CAMADA as f32 * n as f32 * ph2d_tokens::row_pitch_px()
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
            iy = paint_layer_row_c(ctx, theme, ix, iw, iy, pos, brush);
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
    let op_x = x + LABEL_W + gap;
    let op_w = largura_do_chip_da_operacao(ctx.text_system);
    let caixa_x = op_x + op_w + gap;
    let caixa_w = (up_x - gap - caixa_x).max(MIN_CAIXA_W);

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
    let op_rect = Rect::new(op_x, y, op_w, ROW_H_PX);
    paint_button(
        &Button::new(op_id, op_name(brush.composite_ops[pos])),
        op_rect,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(op_id, op_rect);

    // A Strength na CAIXA ÚNICA do app — a barra e o chip editável na mesma caixa. ⚠️ O rótulo
    // fica VAZIO porque o nome desta linha é o chip da operação, encostado à esquerda dela: um
    // «Strength» aqui seria o segundo nome da mesma fileira.
    let sid = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[pos];
    let val = brush.composite_strength[pos].clamp(0.0, 1.0);
    caixa_de_valor(
        ctx,
        theme,
        Rect::new(caixa_x, y, caixa_w, ROW_H_PX),
        "",
        val,
        f64::from(val),
        sid,
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_STRENGTH_CHIP[pos],
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
    let bx = x + ph2d_tokens::list_indent_px();
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
    //
    // ⚠️ **A caixa acaba onde a da fileira A acaba**, e não na borda do cartão: as duas ficam na
    // MESMA coluna. Sem isso a de baixo passava por baixo das setas de reordenar e as duas
    // fileiras da mesma camada liam-se como pertencendo a grelhas diferentes.
    //
    // ⚠️ **O chip mostra o MULTIPLICADOR e a barra guarda a fracção** — a projecção é o
    // `link_slider_number_mapped` do `populate`, escrita UMA vez.
    let size_id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_SIZE[pos];
    let mult = brush.composite_size[pos];
    let direita = x + row_w - ARROW_W - gap - ARROW_W - gap;
    let caixa_w = (direita - cx).max(MIN_CAIXA_W);
    caixa_de_valor(
        ctx,
        theme,
        Rect::new(cx, y, caixa_w, ROW_H_PX),
        tr("panel.painter_layers.composite.size"),
        (mult / ph2d_tool_painter::MAX_COMPOSITE_LAYER_SIZE).clamp(0.0, 1.0),
        f64::from(mult),
        size_id,
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_SIZE_CHIP[pos],
    );

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
fn paint_layer_row_c(
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
    let direita = x + row_w - ARROW_W - gap - ARROW_W - gap;
    let caixa_w = (direita - cx).max(MIN_CAIXA_W);
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

// Os gates deste cartão vivem num irmão `#[path]` para continuarem módulo FILHO (eles leem a
// `largura_do_chip_da_operacao` e o `op_name`, que são privados) enquanto este ficheiro fica sob o
// tecto de LOC — o mesmo corte que o primitivo `button` já pagou.
#[cfg(test)]
#[path = "paint_composite_tests.rs"]
mod tests;
