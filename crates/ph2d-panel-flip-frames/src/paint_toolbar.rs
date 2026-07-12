//! A barra da tira: transporte · Ghost Frames · autoria · ops de chave · tween ·
//! ciclo. Um cursor da esquerda para a direita ([`Bar`]) — cada item pinta no
//! cursor, registra o hit e avança.

use crate::ids;
use crate::state::FlipStripSnapshot;
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
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};

/// Largura de um botão de ícone (quadrado, altura da linha).
const ICON_W: f32 = 28.0; // LITERAL-PX-OK: strip toolbar icon slot
/// Largura de uma caixa numérica pequena da barra.
const NUM_W: f32 = 46.0; // LITERAL-PX-OK: strip toolbar number chip
/// Largura de um toggle de texto (Ghost / Auto / Additive).
const TOGGLE_W: f32 = 62.0; // LITERAL-PX-OK: strip toolbar toggle
/// Largura do chip do ciclo.
const CYCLE_W: f32 = 84.0; // LITERAL-PX-OK: strip cycle dropdown chip
/// Largura média de um glifo em fração do tamanho da fonte — só para RESERVAR o
/// espaço de um rótulo curto da barra (não é métrica de design; a medida real do
/// texto sai do shaper, que aqui seria caro por um "FPS").
const GLYPH_W_RATIO: f32 = 0.62; // LITERAL-PX-OK: estimativa de largura de glifo

/// Os 4 modos de ciclo, na ordem do enum (`CycleMode as u8`).
const CYCLE_NAMES: [&str; 4] = ["No Cycle", "Hold", "Loop", "Ping-Pong"];

/// O chip do ciclo, quando o popover está aberto (pintado por último).
pub(crate) struct PendingCycle {
    pub(crate) chip: Rect,
}

/// Um cursor de barra: pinta da esquerda para a direita e vai consumindo `x`.
struct Bar {
    x: f32,
    y: f32,
    h: f32,
    right: f32,
    gap: f32,
}

impl Bar {
    /// Sobra espaço para um item de largura `w`? (A barra nunca transborda: um
    /// item que não cabe simplesmente não é pintado nem registrado — e um widget
    /// não-registrado não pode ser clicado às cegas.)
    fn fits(&self, w: f32) -> bool {
        self.x + w <= self.right
    }

    fn take(&mut self, w: f32) -> Rect {
        let r = Rect::new(self.x, self.y, w, self.h);
        self.x += w + self.gap;
        r
    }

    fn space(&mut self, w: f32) {
        self.x += w;
    }
}

/// Pinta a barra; devolve o chip do ciclo se o popover estiver aberto.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    rect: Rect,
    snap: &FlipStripSnapshot,
) -> Option<PendingCycle> {
    let mut bar = Bar {
        x: rect.x,
        y: rect.y,
        h: rect.h,
        right: rect.x + rect.w,
        gap: Spacing::Xs.px(),
    };

    // ── Transporte: ◀ desenho · play/pause · desenho ▶ ──
    icon(
        ctx,
        theme,
        &mut bar,
        ids::FLIP_PREV_DRAWING,
        IconId::SkipBack,
    );
    let play_icon = if snap.playing {
        IconId::Pause
    } else {
        IconId::Play
    };
    icon(ctx, theme, &mut bar, ids::FLIP_PLAY, play_icon);
    icon(
        ctx,
        theme,
        &mut bar,
        ids::FLIP_NEXT_DRAWING,
        IconId::SkipForward,
    );
    label(ctx, theme, &mut bar, "FPS");
    number(ctx, theme, &mut bar, ids::FLIP_FPS_NUM, f64::from(snap.fps));
    bar.space(Spacing::Md.px());

    // ── Ghost Frames: liga/desliga + quantos antes/depois ──
    toggle(ctx, theme, &mut bar, ids::FLIP_GHOST, "Ghost", snap.ghost);
    number(
        ctx,
        theme,
        &mut bar,
        ids::FLIP_GHOST_BEFORE_NUM,
        f64::from(snap.ghost_before),
    );
    number(
        ctx,
        theme,
        &mut bar,
        ids::FLIP_GHOST_AFTER_NUM,
        f64::from(snap.ghost_after),
    );
    bar.space(Spacing::Md.px());

    // ── Autoria: o que nasce ao desenhar depois do hold ──
    toggle(
        ctx,
        theme,
        &mut bar,
        ids::FLIP_AUTOKEY,
        "Auto",
        snap.autokey,
    );
    toggle(
        ctx,
        theme,
        &mut bar,
        ids::FLIP_ADDITIVE,
        "Add.",
        snap.additive,
    );
    bar.space(Spacing::Md.px());

    // ── Ops de chave: + · duplicar · apagar · exposição · mover ±1 ──
    icon(ctx, theme, &mut bar, ids::FLIP_KEY_ADD, IconId::Plus);
    icon(ctx, theme, &mut bar, ids::FLIP_KEY_DUP, IconId::Copy);
    icon(ctx, theme, &mut bar, ids::FLIP_KEY_DELETE, IconId::Trash);
    label(ctx, theme, &mut bar, "Hold");
    let hold = snap
        .current_key
        .and_then(|k| snap.cells.iter().find(|c| c.key == k))
        .map_or(1, |c| c.exposure);
    number(ctx, theme, &mut bar, ids::FLIP_HOLD_NUM, f64::from(hold));
    icon(
        ctx,
        theme,
        &mut bar,
        ids::FLIP_KEY_LEFT,
        IconId::ChevronLeft,
    );
    icon(
        ctx,
        theme,
        &mut bar,
        ids::FLIP_KEY_RIGHT,
        IconId::ChevronRight,
    );
    bar.space(Spacing::Md.px());

    // ── Tween: quantos inbetweens + gerar ──
    label(ctx, theme, &mut bar, "Tween");
    number(
        ctx,
        theme,
        &mut bar,
        ids::FLIP_TWEEN_NUM,
        f64::from(snap.tween_count),
    );
    toggle(ctx, theme, &mut bar, ids::FLIP_TWEEN_ADD, "Add", false);
    bar.space(Spacing::Md.px());

    // ── Ciclo (post behavior da camada ativa) ──
    cycle_chip(ctx, theme, &mut bar, snap.cycle)
}

fn icon(ctx: &mut PaintCtx, theme: Theme, bar: &mut Bar, id: NodeId, glyph: IconId) {
    if !bar.fits(ICON_W) {
        return;
    }
    let r = bar.take(ICON_W);
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
fn toggle(ctx: &mut PaintCtx, theme: Theme, bar: &mut Bar, id: NodeId, text: &str, active: bool) {
    if !bar.fits(TOGGLE_W) {
        return;
    }
    let r = bar.take(TOGGLE_W);
    let st = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    paint_segmented_button(r, text, active, st, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(id, r);
}

/// Uma caixa numérica (com drag-scrub e digitação — o range vem do `populate`).
fn number(ctx: &mut PaintCtx, theme: Theme, bar: &mut Bar, id: NodeId, doc_value: f64) {
    if !bar.fits(NUM_W) {
        return;
    }
    let r = bar.take(NUM_W);
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

fn label(ctx: &mut PaintCtx, theme: Theme, bar: &mut Bar, text: &str) {
    let font = TypeToken::Sm.px();
    let w = font * GLYPH_W_RATIO * text.len() as f32 + Spacing::Xs.px();
    if !bar.fits(w) {
        return;
    }
    let r = bar.take(w);
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
fn cycle_chip(ctx: &mut PaintCtx, theme: Theme, bar: &mut Bar, cur: u8) -> Option<PendingCycle> {
    if !bar.fits(CYCLE_W) {
        return None;
    }
    let r = bar.take(CYCLE_W);
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
