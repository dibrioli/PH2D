//! O card **Line** — o `Style` do Alchemy (*"toggle between drawing with a line or solid fill"*) e o
//! dropdown `Type` dos tipos de traço procedural (plano 38 §1, pedido do Enio).
//!
//! Ele mora **imediatamente acima do Composite Brush**, e desenha o próprio fundo com borda, como o
//! irmão [`crate::paint_composite`].
//!
//! ⚠️ **As rows do TIPO só existem com o tipo escolhido** — `None` e `Speed` não têm parâmetro
//! nenhum, e uma row de `Reach` sob eles seria um controle que não faz nada. É a mesma lei que a
//! `line/anim` aplicou ao menu de fade: *uma tabela por escopo, e um escopo sem parâmetro não pinta
//! parâmetro*.

use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Checkbox, CheckboxValue, DEFAULT_CHIP_W, DEFAULT_LABEL_W, DropdownOption, paint_checkbox,
    paint_slider_with_chip_layout_adaptive, slider_with_chip_height,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;

use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken};
use ph2d_tool_painter::{BrushSettings, LineKind};

/// O nome de cada tipo, para o chip e para as opções do popover.
fn kind_name(k: LineKind) -> &'static str {
    match k {
        LineKind::None => tr("panel.painter_layers.line.none"),
        LineKind::Speed => tr("panel.painter_layers.line.speed"),
        LineKind::Sketchy => tr("panel.painter_layers.line.sketchy"),
        LineKind::Wire => tr("panel.painter_layers.line.wire"),
        LineKind::Ribbon => tr("panel.painter_layers.line.ribbon"),
        LineKind::Rough => tr("panel.painter_layers.line.rough"),
    }
}

/// Quantos tipos existem — **derivado da lista abaixo**, nunca um literal, para que um tipo novo
/// apareça no dropdown sem ninguém lembrar de subir um número.
pub(crate) const LINE_KINDS: [LineKind; 6] = [
    LineKind::None,
    LineKind::Speed,
    LineKind::Sketchy,
    LineKind::Wire,
    LineKind::Ribbon,
    LineKind::Rough,
];

/// Os tipos como opções de `Dropdown` (valor = o wire `u8`, rótulo = o nome).
pub(crate) fn line_type_options() -> Vec<DropdownOption<u8>> {
    LINE_KINDS
        .iter()
        .map(|k| {
            let w = k.to_wire();
            DropdownOption::new(
                ph2d_tool_painter::ids::painter_line_type_option_id(w),
                w,
                kind_name(*k),
            )
        })
        .collect()
}

/// Pinta o card **Line** e devolve o próximo `y`.
///
/// ⚠️ **O checkbox `Solid` é oferecido sempre que a ferramenta pinta um traço**, e não por modo — a
/// §5.1 do plano 38 é do Enio (*"Solid é oferecido para todos que forem possíveis"*), e a resposta
/// medida é que **são todos**, porque todo tipo de linha mantém o caminho-base do gesto. Quem
/// responde a pergunta é uma porta única no motor, não este pintor.
pub(crate) fn paint_line_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let pad = Spacing::Sm.px();
    let gap = Spacing::Xs.px();
    let kind = LineKind::from_wire(brush.line_kind);
    // Rows do TIPO. A altura é EXATA — nenhuma row deste card quebra em duas linhas.
    // ⚠️ O `Speed` não tem row nenhuma **de propósito**: o Alchemy não oferece controle sobre o
    // arremesso (Enio 2026-08-13, *"em alchemy o slider não é necessário"*), então o produto acerta
    // de fábrica em vez de delegar.
    let iw = content_w - 2.0 * pad;
    let linhas_do_tipo = altura_das_linhas(kind, iw);
    // ⚠️ **A altura é a SOMA dos avanços que o pintor dá**, e não uma fórmula ao lado dele: a
    //    anterior contava cada barra como `vão + ROW_H` enquanto o pintor avançava `row_pitch`, e o
    //    chip do tipo como `ROW_H` enquanto ele avançava `row_pitch` — duas aritméticas da mesma
    //    pergunta, que só concordavam enquanto os dois vãos fossem iguais.
    let card_h = pad
        + 2.0 * ph2d_tokens::row_pitch_px()
        + if linhas_do_tipo > 0.0 {
            gap + linhas_do_tipo
        } else {
            0.0
        }
        + pad;
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
    let mut iy = y + pad;

    // ⚠️ A coluna do nome é a da secção *Line* (spec §6-quinquies) — as três caixas deste cartão
    // (`Solid`, `Magnetify`, `Connection line`) caem todas no mesmo `x`.
    let seccao =
        crate::seccoes::seccao_da_chave(ctx.text_system, "panel.painter_layers.line.solid");
    let cb = Checkbox::new(
        ph2d_tool_painter::ids::PAINTER_LINE_SOLID,
        tr("panel.painter_layers.line.solid"),
    )
    .value(if brush.style_solid {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    })
    .seccao(seccao);
    let cb_rect = Rect::new(ix, iy, iw, ROW_H_PX);
    paint_checkbox(&cb, cb_rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(ph2d_tool_painter::ids::PAINTER_LINE_SOLID, cb_rect);
    iy += ph2d_tokens::row_pitch_px();

    let (ny, open) = crate::paint_brush_rows::paint_dropdown_row(
        ctx,
        theme,
        ix,
        iw,
        iy,
        "panel.painter_layers.line.type",
        ph2d_tool_painter::ids::PAINTER_LINE_TYPE,
        brush.line_kind,
        kind_name(kind),
    );
    if let Some(r) = open {
        crate::state::set_pending_line_type_dd(Some((r, brush.line_kind)));
    }
    iy = ny;

    if linhas_do_tipo > 0.0 {
        iy = paint_param_rows(ctx, theme, ix, iw, iy + gap, brush, kind);
    }
    let _ = iy;
    y + card_h + ph2d_tokens::control_gap_px()
}

/// **Quanto as linhas do TIPO ocupam** — a soma dos avanços que o [`paint_param_rows`] dá, para
/// que uma barra nova mude a altura do cartão sem ninguém lembrar de subir um número.
///
/// ⚠️ **Pela MESMA régua do pintor**: a caixa única ocupa `slider_with_chip_height` (que conhece o
/// modo empilhado da aparência clássica) mais o vão de controlo, e a caixa de marcar ocupa uma linha.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn altura_das_linhas(kind: LineKind, row_w: f32) -> f32 {
    let barras = crate::line_barras::barras_de(kind).len() as f32;
    let barra = slider_with_chip_height(ROW_H_PX, row_w) + ph2d_tokens::control_gap_px();
    let caixa = if checkbox_of(kind).is_some() {
        ROW_H_PX
    } else {
        0.0
    };
    barras * barra + caixa
}

/// O checkbox de um tipo: `(id, CHAVE do rótulo, o valor)` — a chave, como a das barras
/// ([`crate::line_barras::Barra`]), é traduzida por quem PINTA.
type ParamCheckbox = (
    ph2d_editor_core::NodeId,
    &'static str,
    fn(BrushSettings) -> bool,
);

/// O checkbox deste tipo, se houver.
fn checkbox_of(kind: LineKind) -> Option<ParamCheckbox> {
    match kind {
        LineKind::Sketchy => Some((
            ph2d_tool_painter::ids::PAINTER_LINE_SKETCHY_MAGNETIFY,
            "panel.painter_layers.line.magnetify",
            |b: BrushSettings| b.sketchy_magnetify,
        )),
        LineKind::Wire => Some((
            ph2d_tool_painter::ids::PAINTER_LINE_WIRE_CONNECTION,
            "panel.painter_layers.line.connection_line",
            |b: BrushSettings| b.wire_connection_line,
        )),
        // Nem a fita nem o `Rough` têm checkbox: os knobs deles são contínuos, e um interruptor a
        // mais teria de responder a uma pergunta que nenhum deles já responde. ⚠️ No `Rough` a
        // tentação seria um *"Multi-stroke"* — mas isso é o `Passes = 1`, e o slider já o diz.
        LineKind::None | LineKind::Speed | LineKind::Ribbon | LineKind::Rough => None,
    }
}

/// Pinta as rows do tipo escolhido e devolve o próximo `y`.
fn paint_param_rows(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    row_w: f32,
    y: f32,
    brush: BrushSettings,
    kind: LineKind,
) -> f32 {
    let mut iy = y;
    for barra in crate::line_barras::barras_de(kind) {
        iy = paint_barra(ctx, theme, x, row_w, iy, barra, brush);
    }
    if let Some((id, label, read)) = checkbox_of(kind) {
        let seccao =
            crate::seccoes::seccao_da_chave(ctx.text_system, "panel.painter_layers.line.solid");
        let cb = Checkbox::new(id, tr(label))
            .value(if read(brush) {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            })
            .seccao(seccao);
        let cb_rect = Rect::new(x, iy, row_w, ROW_H_PX);
        paint_checkbox(&cb, cb_rect, ctx.scene, ctx.text_system, theme);
        ctx.host.hit_index_mut().register(id, cb_rect);
        iy += ROW_H_PX;
    }
    iy
}

/// ⭐⭐⭐ **UMA BARRA DO CARTÃO — a CAIXA ÚNICA, com o nome dentro e o número editável.**
///
/// ⛔⛔ Até 2026-09-16 esta linha era `rótulo | trilho nu | readout`: a forma que a spec §2 recusa
/// para um valor com fracção, e que a ordem do dono de 2026-06-26 já tinha tirado de todo o resto
/// do pincel (*«todos usam o slider-with-chip canónico»*). O número só se LIA — para pôr `2.5` no
/// *Reach* era preciso arrastar até lá. E a coluna do nome era o literal `LABEL_W = 62`, com um doc a
/// jurar que *«Line Width»* cabia (mede `66,4`).
///
/// ⚠️ **O número que o chip mostra é `pista × escala` da PRÓPRIA tabela** ([`crate::line_barras`]),
/// e o chip é ligado ao slider pela MESMA escala no `populate` — uma edição volta à ferramenta como o
/// `ValueChanged` do slider, pelo encaminhamento de sempre.
fn paint_barra(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    row_w: f32,
    y: f32,
    barra: &crate::line_barras::Barra,
    brush: BrushSettings,
) -> f32 {
    let pista = (barra.pista)(brush).clamp(0.0, 1.0);
    let numero = barra.numero(brush);
    // ⚠️ Uma CONTAGEM mostra-se sem casas; o resto deixa o chip formatar como toda caixa do app.
    let inteiro = barra.inteiro.then(|| format!("{}", numero.round()));
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let usado = paint_slider_with_chip_layout_adaptive(
        Rect::new(x, y, row_w, ROW_H_PX),
        tr(barra.chave),
        pista,
        f64::from(numero),
        inteiro.as_deref(),
        barra.slider,
        barra.chip,
        DEFAULT_LABEL_W,
        DEFAULT_CHIP_W,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y + usado + ph2d_tokens::control_gap_px()
}
