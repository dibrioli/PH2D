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

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::paint_text;
use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Checkbox, CheckboxValue, DropdownOption, Slider, SliderState, paint_checkbox, paint_slider,
};
use ph2d_editor_core::zones::Rect;

use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{
    BrushSettings, LineKind, SKETCHY_DENSITY_MAX, SKETCHY_REACH_MAX, THREAD_WIDTH_MAX_PX,
    WIRE_HISTORY_MAX,
};

/// Coluna fixa do rótulo de uma row de parâmetro do tipo (cabe "Line Width" na fonte Base).
const LABEL_W: f32 = 62.0; // LITERAL-PX-OK: coluna do rótulo do card Line
/// Coluna do readout "0.35" à direita do slider.
const READOUT_W: f32 = 34.0; // LITERAL-PX-OK: coluna do readout
/// Piso da pista do slider nu (chão de chrome num painel estreito).
const MIN_SLIDER_W: f32 = 24.0; // LITERAL-PX-OK: piso da pista
/// O nome de cada tipo, para o chip e para as opções do popover.
fn kind_name(k: LineKind) -> &'static str {
    match k {
        LineKind::None => "None",
        LineKind::Speed => "Speed",
        LineKind::Sketchy => "Sketchy",
        LineKind::Wire => "Wire",
    }
}

/// Quantos tipos existem — **derivado da lista abaixo**, nunca um literal, para que um tipo novo
/// apareça no dropdown sem ninguém lembrar de subir um número.
pub(crate) const LINE_KINDS: [LineKind; 4] = [
    LineKind::None,
    LineKind::Speed,
    LineKind::Sketchy,
    LineKind::Wire,
];

/// Os tipos como opções de `Dropdown` (valor = o wire `u8`, rótulo = o nome).
pub(crate) fn line_type_options() -> Vec<DropdownOption<u8>> {
    LINE_KINDS
        .iter()
        .map(|k| {
            let w = k.to_wire();
            DropdownOption::new(core_ids::painter_line_type_option_id(w), w, kind_name(*k))
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
    let param_rows = rows_of(kind);
    let card_h = pad + ROW_H_PX + gap + ROW_H_PX + param_rows * (gap + ROW_H_PX) + pad;
    let card = Rect::new(x, y, content_w, card_h);
    let radius = Radius::Md.px();
    fill_rounded_rect(ctx.scene, card, radius, resolve(ColorToken::Bg1, theme));
    stroke_rounded_rect(
        ctx.scene,
        card,
        radius,
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );

    let ix = x + pad;
    let iw = content_w - 2.0 * pad;
    let mut iy = y + pad;

    let cb = Checkbox::new(core_ids::PAINTER_LINE_SOLID, "Solid").value(if brush.style_solid {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    });
    let cb_rect = Rect::new(ix, iy, iw, ROW_H_PX);
    paint_checkbox(&cb, cb_rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_LINE_SOLID, cb_rect);
    iy += ROW_H_PX + gap;

    let (ny, open) = crate::paint_brush_rows::paint_dropdown_row(
        ctx,
        theme,
        ix,
        iw,
        iy,
        "Type",
        core_ids::PAINTER_LINE_TYPE,
        brush.line_kind,
        kind_name(kind),
    );
    if let Some(r) = open {
        crate::state::set_pending_line_type_dd(Some((r, brush.line_kind)));
    }
    iy = ny;

    if param_rows > 0.0 {
        iy = paint_param_rows(ctx, theme, ix, iw, iy + gap, brush, kind);
    }
    let _ = iy;
    y + card_h + Spacing::Sm.px()
}

/// Quantas rows o TIPO acrescenta — **derivado das tabelas**, para que uma row nova mude a altura do
/// card sem ninguém lembrar de subir um número (a mesma lei do [`LINE_KINDS`]).
#[allow(clippy::cast_precision_loss)]
fn rows_of(kind: LineKind) -> f32 {
    sliders_of(kind).len() as f32 + f32::from(u8::from(checkbox_of(kind).is_some()))
}

/// Um slider de parâmetro do tipo: `(id, rótulo, valor na pista 0..1, o que o readout mostra)`.
///
/// ⚠️ **UMA tabela por tipo, DOIS consumidores** — o pintor a percorre e o `populate` registra os
/// mesmos ids; e é ela que faz a altura do card ser CONTADA em vez de escolhida. Uma row a mais
/// aqui nasce pintada, medida e com hit registrado.
type ParamSlider = (
    ph2d_editor_core::NodeId,
    &'static str,
    fn(BrushSettings) -> (f32, f32),
);

/// A `Line Width` e a `Opacity` são as MESMAS duas rows nos dois tipos, na MESMA posição — porque
/// são o mesmo fato (a tinta de um fio; ver `set_thread_width_norm`). Trocar de tipo não embaralha
/// o card, e é isso que torna a troca uma decisão sobre a LEI e não sobre onde os controles foram
/// parar.
const THREAD_INK_ROWS: [ParamSlider; 2] = [
    (core_ids::PAINTER_LINE_SKETCHY_WIDTH, "Line Width", |b| {
        (b.thread_width_px / THREAD_WIDTH_MAX_PX, b.thread_width_px)
    }),
    (core_ids::PAINTER_LINE_SKETCHY_OPACITY, "Opacity", |b| {
        (b.thread_opacity, b.thread_opacity)
    }),
];

const SKETCHY_SLIDERS: [ParamSlider; 4] = [
    (core_ids::PAINTER_LINE_SKETCHY_REACH, "Reach", |b| {
        (b.sketchy_reach / SKETCHY_REACH_MAX, b.sketchy_reach)
    }),
    (core_ids::PAINTER_LINE_SKETCHY_DENSITY, "Density", |b| {
        const PCT: f32 = 100.0; // LITERAL-PX-OK: a Density é lida em PORCENTAGEM na face do artista
        (
            b.sketchy_density / SKETCHY_DENSITY_MAX,
            b.sketchy_density * PCT,
        )
    }),
    THREAD_INK_ROWS[0],
    THREAD_INK_ROWS[1],
];

const WIRE_SLIDERS: [ParamSlider; 3] = [
    (core_ids::PAINTER_LINE_WIRE_HISTORY, "History", |b| {
        (b.wire_history / WIRE_HISTORY_MAX, b.wire_history)
    }),
    THREAD_INK_ROWS[0],
    THREAD_INK_ROWS[1],
];

/// Os sliders deste tipo. `None`/`Speed` não têm **de propósito** — o Alchemy não oferece controle
/// sobre o arremesso (Enio 2026-08-13), e uma row sob eles seria um controle que não faz nada.
fn sliders_of(kind: LineKind) -> &'static [ParamSlider] {
    match kind {
        LineKind::None | LineKind::Speed => &[],
        LineKind::Sketchy => &SKETCHY_SLIDERS,
        LineKind::Wire => &WIRE_SLIDERS,
    }
}

/// O checkbox de um tipo: `(id, rótulo, o valor)`.
type ParamCheckbox = (
    ph2d_editor_core::NodeId,
    &'static str,
    fn(BrushSettings) -> bool,
);

/// O checkbox deste tipo, se houver.
fn checkbox_of(kind: LineKind) -> Option<ParamCheckbox> {
    match kind {
        LineKind::Sketchy => Some((
            core_ids::PAINTER_LINE_SKETCHY_MAGNETIFY,
            "Magnetify",
            |b: BrushSettings| b.sketchy_magnetify,
        )),
        LineKind::Wire => Some((
            core_ids::PAINTER_LINE_WIRE_CONNECTION,
            "Connection Line",
            |b: BrushSettings| b.wire_connection_line,
        )),
        LineKind::None | LineKind::Speed => None,
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
    let gap = Spacing::Xs.px();
    let mut iy = y;
    for (id, label, read) in sliders_of(kind) {
        let (track, shown) = read(brush);
        paint_param_row(ctx, theme, x, row_w, iy, label, *id, track, shown);
        iy += ROW_H_PX + gap;
    }
    if let Some((id, label, read)) = checkbox_of(kind) {
        let cb = Checkbox::new(id, label).value(if read(brush) {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        });
        let cb_rect = Rect::new(x, iy, row_w, ROW_H_PX);
        paint_checkbox(&cb, cb_rect, ctx.scene, ctx.text_system, theme);
        ctx.host.hit_index_mut().register(id, cb_rect);
        iy += ROW_H_PX;
    }
    iy
}

/// Uma row de parâmetro do tipo: rótulo · slider nu · readout. O valor vem do snapshot e o ESTADO do
/// store, como toda row de slider deste painel.
#[allow(clippy::too_many_arguments)]
fn paint_param_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    row_w: f32,
    y: f32,
    label: &str,
    sid: ph2d_editor_core::NodeId,
    track: f32,
    shown: f32,
) {
    let gap = Spacing::Xs.px();
    let font = TypeToken::Base.px();
    let readout_x = x + row_w - READOUT_W;
    let slider_x = x + LABEL_W + gap;
    let slider_w = (readout_x - gap - slider_x).max(MIN_SLIDER_W);

    paint_text(
        ctx.text_system,
        ctx.scene,
        label,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        LABEL_W,
        resolve(ColorToken::Text1, theme),
    );
    let st = ctx
        .host
        .store()
        .slider(sid)
        .map(|(s, _)| s)
        .unwrap_or(SliderState::Normal);
    let mut slider = Slider::new(sid, "").accent(true).state(st);
    slider.value = track.clamp(0.0, 1.0);
    let slider_rect = Rect::new(slider_x, y, slider_w, ROW_H_PX);
    paint_slider(&slider, slider_rect, ctx.scene, theme);
    ctx.host.hit_index_mut().register(sid, slider_rect);
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!("{shown:.2}"),
        readout_x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        READOUT_W,
        resolve(ColorToken::Text2, theme),
    );
}
