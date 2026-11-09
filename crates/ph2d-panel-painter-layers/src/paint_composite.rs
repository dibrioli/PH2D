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
    Button, Checkbox, CheckboxValue, ColorSwatch, DEFAULT_LABEL_W, SwatchSize, SwatchState,
    default_chip_w, paint_button, paint_checkbox, paint_color_swatch,
    paint_slider_with_chip_layout,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::BrushSettings;

#[path = "paint_composite_fileiras.rs"]
mod fileiras;
use fileiras::{
    paint_layer_row_b, paint_layer_row_c, paint_layer_row_forca, paint_layer_row_prefixo,
};

// Os rótulos que as caixas de valor deste cartão receberam na última pintura — ver
// [`caixa_de_valor`]. Por THREAD: um átomo global reprovaria na suíte em paralelo.
#[cfg(test)]
thread_local! {
    pub(crate) static ROTULOS_PINTADOS: std::cell::RefCell<Vec<(String, Rect)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// **Quantas fileiras cada camada ocupa** — A (operação + força), B (cor + tamanho) e C (dureza).
///
/// ⚠️ Ela é lida pela ALTURA do cartão e pelo laço que pinta; escrita nos dois sítios, acrescentar
/// uma fileira daria um cartão que corta a última camada ao meio — *um número que decide o tamanho
/// de uma caixa e quantas coisas entram nela tem de ser o mesmo número*.
const FILEIRAS_POR_CAMADA: usize = 3;
/// Fixed-width label column for `N` (the op name moved into its own chip in the same row).
const LABEL_W: f32 = 14.0; // LITERAL-PX-OK: the fixed position number column ("1".."5")
/// Reorder ↑/↓ button column width (mirrors the Layers panel's `REORDER_W`).
pub(crate) const ARROW_W: f32 = 16.0; // LITERAL-PX-OK: reorder button column (matches paint_rows)
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
pub(crate) fn n_camadas(brush: &BrushSettings) -> usize {
    brush.composite_len.min(brush.composite_ops.len())
}

/// The name shown in each layer row for a composite op wire discriminant.
pub(crate) fn op_name(op: u8) -> &'static str {
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
pub(crate) fn largura_do_chip_da_operacao(text_system: &mut ph2d_text::TextSystem) -> f32 {
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
    // ⚠️ **O que cada caixa deste cartão nomeia é OBSERVADO, não lido do fonte.** A ordem do dono
    // de 2026-09-21 é *«todos os sliders devem ter nome inclusive o Strength»*, e a régua tem de
    // ver o rótulo que o PINTOR recebe — um censo textual das chamadas mediria o código, não o
    // produto.
    #[cfg(test)]
    ROTULOS_PINTADOS.with(|c| c.borrow_mut().push((rotulo.to_string(), rect)));
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
        default_chip_w(),
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
}

/// Esta operação deposita a cor do pincel? (o espelho do `CompositeOp::deposita_cor` do motor —
/// ⚠️ o painel não vê o enum, e o gate `a_fileira_da_cor_so_existe_onde_a_cor_chega` ata os dois.)
pub(crate) fn deposita_cor(op: u8) -> bool {
    op == 0
}

/// Esta posição é uma BORRACHA? (o discriminante `3` do [`ph2d_tool_painter`] wire)
pub(crate) fn apaga(op: u8) -> bool {
    op == 3
}

/// O nome do escopo `e` da borracha — a mesma forma do [`op_name`].
pub(crate) fn escopo_name(e: u8) -> &'static str {
    match e {
        1 => tr("panel.painter_layers.composite.erase_stroke"),
        _ => tr("panel.painter_layers.composite.erase_image"),
    }
}

/// A largura do chip do ESCOPO, **medida** dos nomes que ele pode mostrar.
///
/// ⛔ Ela é derivada pela mesma razão que a irmã da operação: um número escolhido à mão com um
/// comentário a dizer que coube foi exactamente o que a foto do dono reprovou em 2026-09-20.
fn largura_do_chip_do_escopo(text_system: &mut ph2d_text::TextSystem) -> f32 {
    let fonte = Button::label_font_px();
    let mais_largo = (0..ph2d_tool_painter::N_COMPOSITE_ERASE_SCOPES)
        .map(|e| text_system.prefix_width(escopo_name(e as u8), fonte))
        .fold(0.0f32, f32::max);
    rect_for_label(mais_largo)
}

/// O NOME desta caixa de valor sai INTEIRO em `caixa`? — a repartição da caixa única
/// ([`ph2d_editor_core::widget::label_column`] com a coluna do valor de [`caixa_de_valor`]) e a medição do pintor
/// (o `layout` na fonte `Sm`, a que o `fit_label` compara).
pub(crate) fn o_nome_cabe(
    text_system: &mut ph2d_text::TextSystem,
    rotulo: &str,
    caixa: Rect,
) -> bool {
    let deco = ph2d_editor_core::widget::FORM_ROWS_SHOW_DECORATOR
        && ph2d_editor_core::paint::ui_is_redesign();
    let coluna = ph2d_editor_core::widget::label_column(caixa, default_chip_w(), deco);
    text_system
        .layout(rotulo, TypeToken::Sm.px(), f32::INFINITY)
        .width()
        <= coluna.w
}

/// A largura do que a fileira B pinta à ESQUERDA do tamanho — a cor e o botão de volta (quem deposita) ou o chip
/// do escopo (a borracha) —, com o vão; `0` sem nada. ⚠️ O botão de volta conta SEMPRE: a decisão de
/// [`prefixo_ao_lado_do_tamanho`] não salta quando o artista autora a cor.
fn prefixo_w(text_system: &mut ph2d_text::TextSystem, op: u8) -> f32 {
    let gap = Spacing::Xs.px();
    if deposita_cor(op) {
        SWATCH_W + gap + CLEAR_W + gap
    } else if apaga(op) {
        largura_do_chip_do_escopo(text_system) + gap
    } else {
        0.0
    }
}

/// ⭐ **O que vem antes do «Size (px)» cabe ao lado dele sem lhe cortar o nome?** Medido, nunca escolhido: o
/// prefixo mede-se ([`prefixo_w`]), a caixa mede o rótulo dela ([`o_nome_cabe`]). Não cabendo, o prefixo desce a
/// uma fileira PRÓPRIA ([`paint_layer_row_prefixo`]) — 08/11: o «(px)» alongou o rótulo e o chip «Image» da
/// borracha (e a cor das Brush, à largura do dono) cortavam-no em `Size…`.
fn prefixo_ao_lado_do_tamanho(
    text_system: &mut ph2d_text::TextSystem,
    x: f32,
    row_w: f32,
    op: u8,
) -> bool {
    let p = prefixo_w(text_system, op);
    if p <= 0.0 {
        return true;
    }
    let cx = x + ph2d_tokens::list_indent_px() + p;
    let caixa = Rect::new(
        cx,
        0.0,
        (caixa_direita(x, row_w) - cx).max(MIN_CAIXA_W),
        ROW_H_PX,
    );
    o_nome_cabe(
        text_system,
        tr("panel.painter_layers.composite.size"),
        caixa,
    )
}

/// Onde acabam as caixas das fileiras B e C: na coluna da caixa da fileira A, antes das setas.
fn caixa_direita(x: f32, row_w: f32) -> f32 {
    let gap = Spacing::Xs.px();
    x + row_w - ARROW_W - gap - ARROW_W - gap
}

/// A caixa da Strength na fileira A: depois do `N` e da coluna do nome da operação, antes das setas e do `x`.
fn caixa_da_forca_na_fileira_a(
    text_system: &mut ph2d_text::TextSystem,
    x: f32,
    row_w: f32,
    y: f32,
) -> Rect {
    let gap = Spacing::Xs.px();
    let up_x = x + row_w - ARROW_W - gap - ARROW_W - gap - ARROW_W;
    let caixa_x = x + LABEL_W + gap + largura_do_chip_da_operacao(text_system) + gap;
    Rect::new(
        caixa_x,
        y,
        (up_x - gap - caixa_x).max(MIN_CAIXA_W),
        ROW_H_PX,
    )
}

/// ⭐ **A «Strength» cabe inteira na fileira A?** A mesma lei do «Size (px)» ([`prefixo_ao_lado_do_tamanho`]):
/// medido, nunca escolhido. Não cabendo (≈ `245`–`304` px de painel, 08/11: o número, o nome da operação e as
/// três setas comiam o rótulo até ao vazio), a caixa desce a uma fileira PRÓPRIA recuada, na coluna da B e da C
/// ([`paint_layer_row_forca`]); a fileira A fica com `N` · nome · setas · `x`.
fn forca_na_fileira_a(text_system: &mut ph2d_text::TextSystem, x: f32, row_w: f32) -> bool {
    let caixa = caixa_da_forca_na_fileira_a(text_system, x, row_w, 0.0);
    o_nome_cabe(
        text_system,
        tr("panel.painter_layers.composite.strength"),
        caixa,
    )
}

/// Quantas fileiras a camada `pos` ocupa — as [`FILEIRAS_POR_CAMADA`], mais a da Strength quando ela não cabe na
/// fileira A e a do prefixo quando ele não cabe ao lado do tamanho. ⚠️ A ALTURA do cartão e o laço que pinta
/// leem ESTA função.
fn fileiras_da_camada(
    text_system: &mut ph2d_text::TextSystem,
    x: f32,
    row_w: f32,
    brush: &BrushSettings,
    pos: usize,
) -> usize {
    let propria = !prefixo_ao_lado_do_tamanho(text_system, x, row_w, brush.composite_ops[pos]);
    let forca_propria = !forca_na_fileira_a(text_system, x, row_w);
    FILEIRAS_POR_CAMADA + usize::from(propria) + usize::from(forca_propria)
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
    // ⚠️ **`+ 1` é a fileira do `+`**, e ela é pintada SEMPRE que a secção está ligada — mesmo com
    // a pilha vazia, que desde 2026-09-21 é como ela nasce. *Sem essa fileira uma secção ligada e
    // vazia seria uma caixa sem nada dentro e sem forma de lhe pôr alguma coisa.*
    let ix = x + pad;
    let iw = content_w - 2.0 * pad;
    let fileiras: usize = (0..n)
        .map(|pos| fileiras_da_camada(ctx.text_system, ix, iw, &brush, pos))
        .sum();
    let layers_h = if checked {
        gap + (fileiras + 1) as f32 * ph2d_tokens::row_pitch_px()
    } else {
        0.0
    };

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
    let cb_rect = Rect::new(ix, y + pad, iw, ROW_H_PX);
    // ⛔ A secção QUEBRA o nome que não cabe (`Composite` / `Brush`, 03/11) e a 2.ª linha dele desce um passo:
    // sem a crescer aqui, o «Brush» pintava-se DENTRO da 1.ª camada (o «Bru» à esquerda da Strength da Blur,
    // cortado pela caixa pintada por cima) ou sobre o Accumulate com a secção desligada.
    let linhas_do_nome = cb.linhas_do_nome(cb_rect, ctx.text_system);
    let nome_h = ROW_H_PX + ph2d_tokens::row_pitch_px() * (linhas_do_nome - 1) as f32;
    let card_h = pad + nome_h + layers_h + pad;
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

    paint_checkbox(&cb, cb_rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ENABLE,
        Rect::new(cb_rect.x, cb_rect.y, cb_rect.w, nome_h),
    );
    let mut iy = y + pad + nome_h;

    if checked {
        iy += gap;
        let forca_aqui = forca_na_fileira_a(ctx.text_system, ix, iw);
        for pos in 0..n {
            let propria =
                !prefixo_ao_lado_do_tamanho(ctx.text_system, ix, iw, brush.composite_ops[pos]);
            iy = paint_layer_row(ctx, theme, ix, iw, iy, pos, brush, forca_aqui);
            if !forca_aqui {
                iy = paint_layer_row_forca(ctx, theme, ix, iw, iy, pos, brush);
            }
            iy = paint_layer_row_b(ctx, theme, ix, iw, iy, pos, brush, !propria);
            iy = paint_layer_row_c(ctx, theme, ix, iw, iy, pos, brush);
            if propria {
                iy = paint_layer_row_prefixo(ctx, theme, ix, iy, pos, brush);
            }
        }
        crate::paint_composite_montagem::paint_add_row(ctx, theme, ix, iw, iy, brush);
    }
    y + card_h + ph2d_tokens::control_gap_px()
}

/// Row A: `N` · o NOME da operação · a Strength (com rótulo) · ↑/↓ · o `x` que retira a camada.
///
/// ⛔⛔ **O chip da operação virou um NOME em 2026-09-21.** Ele ciclava as quatro operações, e com
/// as quotas do dono (`3 Brush · 2 Erase · 1 Blur · 1 Smear`) um ciclo livre tornaria a quota uma
/// mentira — dois cliques punham dois Blurs na pilha. A operação escolhe-se na CRIAÇÃO, e trocá-la
/// é retirar a camada e criar outra.
///
/// ⭐ **E a Strength ganhou o rótulo dela** (ordem do dono: *«todos os sliders devem ter nome
/// inclusive o Strength»*). O doc antigo defendia o rótulo vazio com *«o nome desta linha é o chip
/// da operação»* — com o chip a virar nome, essa premissa morreu: o nome da CAMADA e o nome do que
/// a barra CONTROLA são duas coisas.
#[allow(clippy::too_many_arguments)]
fn paint_layer_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    row_w: f32,
    y: f32,
    pos: usize,
    brush: BrushSettings,
    forca_aqui: bool,
) -> f32 {
    let gap = Spacing::Xs.px();
    let font = TypeToken::Base.px();
    let n = n_camadas(&brush);
    let x_x = x + row_w - ARROW_W;
    let down_x = x_x - gap - ARROW_W;
    let up_x = down_x - gap - ARROW_W;
    let op_x = x + LABEL_W + gap;
    let op_w = largura_do_chip_da_operacao(ctx.text_system);

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

    // O NOME da operação desta camada — um rótulo, não um botão: nada aqui é clicável.
    paint_text(
        ctx.text_system,
        ctx.scene,
        op_name(brush.composite_ops[pos]),
        op_x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        op_w,
        resolve(ColorToken::Text1, theme),
    );

    // A Strength na CAIXA ÚNICA do app, COM o nome dela — aqui só quando ele cabe inteiro ([`forca_na_fileira_a`]).
    if forca_aqui {
        let caixa = caixa_da_forca_na_fileira_a(ctx.text_system, x, row_w, y);
        pinta_forca(ctx, theme, caixa, pos, &brush);
    }

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
    // O `x` que retira esta camada — sempre vivo: uma camada criada à mão tira-se à mão.
    crate::paint_rows::paint_reorder_btn(
        ctx,
        theme,
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_REMOVE[pos],
        Rect::new(x_x, y, ARROW_W, ROW_H_PX),
        true,
        IconId::Close,
    );

    y + ph2d_tokens::row_pitch_px()
}

/// A caixa da Strength da camada `pos` em `caixa` — a mesma na fileira A e na fileira própria.
fn pinta_forca(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    caixa: Rect,
    pos: usize,
    brush: &BrushSettings,
) {
    let val = brush.composite_strength[pos].clamp(0.0, 1.0);
    caixa_de_valor(
        ctx,
        theme,
        caixa,
        tr("panel.painter_layers.composite.strength"),
        val,
        f64::from(val),
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[pos],
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_STRENGTH_CHIP[pos],
    );
}

// Os gates deste cartão vivem num irmão `#[path]` para continuarem módulo FILHO (eles leem a
// `largura_do_chip_da_operacao` e o `op_name`, que são privados) enquanto este ficheiro fica sob o
// tecto de LOC — o mesmo corte que o primitivo `button` já pagou.
#[cfg(test)]
#[path = "paint_composite_tests.rs"]
mod tests;
