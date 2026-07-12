//! A barra da tira: transporte · Ghost Frames · autoria · ops de chave · tween ·
//! ciclo.
//!
//! A barra **não decide** onde os controles caem — quem decide é
//! [`crate::toolbar_plan`], para que a MEDIDA (de quantas linhas a tira precisa)
//! e a PINTURA nunca divirjam. Aqui só se pinta o que o plano posicionou.

use crate::state::FlipStripSnapshot;
use crate::toolbar_plan::{self, Item};
use crate::{ids, toolbar_plan::CYCLE_W};
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::showcase::read_number_input;
use ph2d_editor_core::widget::{
    ButtonState, Dropdown, DropdownOption, DropdownState, IconButtonStyle, IconGlyph, NumberInput,
    paint_dropdown_chip, paint_dropdown_popover, paint_icon_button, paint_number_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme, TypeToken};

/// Os 4 modos de ciclo, na ordem do enum (`CycleMode as u8`).
const CYCLE_NAMES: [&str; 4] = ["No Cycle", "Hold", "Loop", "Ping-Pong"];

/// O chip do ciclo, quando o popover está aberto (pintado por último).
pub(crate) struct PendingCycle {
    pub(crate) chip: Rect,
}

/// Pinta a barra a partir do plano; devolve o chip do ciclo se o popover estiver
/// aberto. `first_row` é a faixa da PRIMEIRA linha — as demais descem a partir
/// dela (a tira já reservou a altura, via [`toolbar_plan::rows`]).
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    first_row: Rect,
    snap: &FlipStripSnapshot,
) -> Option<PendingCycle> {
    let items = toolbar_plan::items(snap);
    let (rects, _rows) = toolbar_plan::plan(&items, first_row, first_row.h);

    let mut pending = None;
    for (item, r) in items.iter().zip(rects) {
        match item {
            Item::Icon(id, glyph) => icon(ctx, theme, r, *id, *glyph),
            Item::Toggle(id, text, active) => toggle(ctx, theme, r, *id, text, *active),
            Item::Number(id, value) => number(ctx, theme, r, *id, *value),
            Item::Label(text) => label(ctx, theme, r, text),
            Item::Cycle(cur) => pending = cycle_chip(ctx, theme, r, *cur),
            Item::Gap => {}
        }
    }
    pending
}

fn icon(ctx: &mut PaintCtx, theme: Theme, r: Rect, id: NodeId, glyph: IconId) {
    let st = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    paint_icon_button(
        r,
        IconGlyph::Builtin(glyph),
        IconButtonStyle::Plain,
        st,
        ctx.scene,
        theme,
    );
    ctx.host.hit_index_mut().register(id, r);
}

/// Um toggle de texto — o botão segmentado (mesmo idioma do Mode row do painel de
/// estilo): `active` acende.
fn toggle(ctx: &mut PaintCtx, theme: Theme, r: Rect, id: NodeId, text: &str, active: bool) {
    let st = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    paint_segmented_button(r, text, active, st, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(id, r);
}

/// Uma caixa numérica (com drag-scrub e digitação — o range vem do `populate`).
fn number(ctx: &mut PaintCtx, theme: Theme, r: Rect, id: NodeId, doc_value: f64) {
    // Espelha o valor do DOCUMENTO na store enquanto a caixa não está em edição
    // (mesma sincronia das caixas do Inspector) — senão a caixa mostraria o valor
    // velho depois de um undo ou de uma edição vinda de outro caminho.
    let (state, value, buf, caret, anchor) = {
        let (s, v, b, c, a) = read_number_input(ctx.host.store(), id);
        (s, v, b.to_string(), c, a)
    };
    let focused = matches!(state, ph2d_editor_core::widget::TextInputState::Focused);
    let shown = if focused { value } else { doc_value };
    if !focused && (value - doc_value).abs() > f64::EPSILON {
        ctx.host.store_mut().set_number_value(id, doc_value);
    }
    let buf = if focused { buf } else { fmt_num(doc_value) };
    let input = NumberInput::new(id, "", shown).step(1.0).state(state);
    paint_number_input_with_buffer(
        &input,
        Some(&buf),
        caret,
        anchor,
        r,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(id, r);
}

/// Inteiro sem casas (a barra só tem contagens: quadros, FPS, inbetweens).
fn fmt_num(v: f64) -> String {
    format!("{}", v.round() as i64)
}

fn label(ctx: &mut PaintCtx, theme: Theme, r: Rect, text: &str) {
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        r.x,
        r.y + (r.h - font) * 0.5,
        font,
        r.w,
        resolve(ColorToken::Text2, theme),
    );
}

fn cycle_options() -> Vec<DropdownOption<u8>> {
    CYCLE_NAMES
        .iter()
        .enumerate()
        .map(|(i, name)| DropdownOption::new(ids::flip_cycle_option_id(i as u8), i as u8, *name))
        .collect()
}

/// O chip do ciclo (dropdown genérico: o open/close é do dispatch).
fn cycle_chip(ctx: &mut PaintCtx, theme: Theme, r: Rect, cur: u8) -> Option<PendingCycle> {
    debug_assert!((r.w - CYCLE_W).abs() < 1.0, "o plano mediu outro chip");
    let id = ids::FLIP_CYCLE_DD;
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(cur as usize),
        },
    );
    let open = matches!(
        ctx.host.store().get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let dd = Dropdown::new(id, "", cycle_options())
        .selected(cur)
        .open(open)
        .state(DropdownState::Normal);
    paint_dropdown_chip(&dd, r, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(id, r);
    open.then_some(PendingCycle { chip: r })
}

/// O popover aberto do ciclo — pintado por último, fora de qualquer clip.
pub(crate) fn paint_cycle_popover(
    ctx: &mut PaintCtx,
    theme: Theme,
    pending: PendingCycle,
    cur: u8,
) {
    let dd = Dropdown::new(ids::FLIP_CYCLE_DD, "", cycle_options())
        .selected(cur)
        .open(true);
    paint_dropdown_popover(&dd, pending.chip, ctx.scene, ctx.text_system, theme);
    let panel = dd.popover_rect(pending.chip);
    ctx.host
        .store_mut()
        .set_dropdown_popover(ids::FLIP_CYCLE_DD, panel);
    for (i, opt) in dd.options.iter().enumerate() {
        ctx.host.store_mut().register_if_absent(
            opt.id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        let r = dd.option_rect(pending.chip, i);
        ctx.host.hit_index_mut().register(opt.id, r);
    }
}
