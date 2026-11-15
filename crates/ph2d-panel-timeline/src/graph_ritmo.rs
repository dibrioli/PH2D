//! ⭐ **Override · Retime na vista de velocidade** (plano 18, dono 15/11).
//!
//! **Override** (como sempre): a curva da velocidade no acento, com as alças do EASING — arrastar
//! reescreve o easing. **Retime**: duas curvas — a velocidade do movimento na cor da curva (os quiques
//! do Bounce ficam) e o RITMO em âmbar, com as alças DELE: arrastar muda só quando cada parte do
//! easing acontece ([`ph2d_timeline::retime_with_speed_handle`]). A escolha é da sessão, na coluna à
//! esquerda da banda, com «Reset timing» por baixo em Retime.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_centered, resolve};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx};
use ph2d_editor_core::widget::{TabItem, Tabs, TabsVariant, paint_tabs_with_hover};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{
    KeyView, RetimeIntent, TimelineIntent, TrackView, retime_with_speed_handle, rhythm_handle_tip,
    segment_rhythm, speed_handle_tip,
};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};

use crate::graph::{Band, TimeView};
use crate::ids;
use crate::state::{self, TimelinePanelState};

/// Onde a alça de um lado de um trecho se pinta: `(tempo, velocidade)`.
pub(crate) type Ponta = fn(&KeyView, &KeyView, u8) -> Option<(f64, f64)>;

/// Amostras por trecho para o enquadramento da curva do ritmo.
const AMOSTRAS: usize = 32;
/// A altura do «Reset timing».
const RESET_H: f32 = 15.0; // LITERAL-PX-OK: a altura do chip do Buffer, o vizinho da banda

/// A cor da curva da velocidade e as alças que a vista mostra, pelo modo.
pub(crate) fn curva_e_alcas(state: &TimelinePanelState) -> (ColorToken, Ponta) {
    if state.speed_retime {
        (ColorToken::TimelineCurve, rhythm_handle_tip)
    } else {
        (ColorToken::Accent, speed_handle_tip)
    }
}

/// Os trechos que mostram o ritmo: os que tocam uma chave seleccionada (os mesmos das alças).
fn trechos(track: &TrackView) -> impl Iterator<Item = (&KeyView, &KeyView)> {
    track
        .keys
        .windows(2)
        .map(|w| (&w[0], &w[1]))
        .filter(|(a, b)| a.selected || b.selected)
}

/// A curva do ritmo (`ref · W'`) de cada trecho seleccionado, em âmbar — só em Retime: a curva do
/// movimento já é azul, e duas curvas do mesmo azul não se distinguem (foto de 15/11).
pub(crate) fn paint_rhythm(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    band: &Band,
    view: TimeView,
    track: &TrackView,
) {
    if !state.speed_retime {
        return;
    }
    let cor = resolve(ColorToken::Warn, theme);
    for (k0, k1) in trechos(track) {
        let dentro = |t: f64| (k0.t_seconds..=k1.t_seconds).contains(&t);
        let amostra = |t: f64| {
            dentro(t)
                .then(|| segment_rhythm(k0, k1, t))
                .flatten()
                .map(|v| v as f32)
        };
        crate::graph_paint::paint_speed_curve(ctx, band, view, amostra, cor);
    }
}

/// O enquadramento da banda de velocidade: em Retime, a curva do ritmo e as pontas entram também.
pub(crate) fn extent(
    state: &TimelinePanelState,
    base: Option<(f32, f32)>,
    track: &TrackView,
) -> Option<(f32, f32)> {
    if !state.speed_retime {
        return base;
    }
    let (mut lo, mut hi) = base?;
    let mut inclui = |v: f64| {
        if v.is_finite() {
            lo = lo.min(v as f32);
            hi = hi.max(v as f32);
        }
    };
    for (k0, k1) in trechos(track) {
        let span = k1.t_seconds - k0.t_seconds;
        let curva = (0..=AMOSTRAS).filter_map(|i| {
            segment_rhythm(k0, k1, k0.t_seconds + span * (i as f64 / AMOSTRAS as f64))
        });
        let pontas = [0u8, 1u8]
            .into_iter()
            .filter_map(|which| rhythm_handle_tip(k0, k1, which).map(|(_, v)| v));
        curva.chain(pontas).for_each(&mut inclui);
    }
    Some((lo, hi))
}

/// Um frame do arrasto de uma alça de ritmo, como o comando do documento. `None` num trecho parado.
pub(crate) fn intent_do_arrasto(
    (k0, k1): (&KeyView, &KeyView),
    track: &TrackView,
    which: u8,
    ponto: (f64, f64),
    px: (f64, f64),
) -> Option<TimelineIntent> {
    let retime = retime_with_speed_handle(k0, k1, which, ponto, px)?;
    Some(TimelineIntent::Retime(RetimeIntent::Set {
        target: track.target,
        key: k0.id,
        retime: Some(retime),
    }))
}

/// As duas células e o «Reset timing»: o clique muda o modo (fecha um arrasto a meio, como o
/// «Show speed») ou repõe o ritmo das chaves seleccionadas.
pub(crate) fn apply_event(state: &mut TimelinePanelState, ev: WidgetEvent) -> Option<EventOutcome> {
    let WidgetEvent::Click(id) = ev else {
        return None;
    };
    if id == ids::TIMELINE_RESET_TIMING {
        state::push_intent(TimelineIntent::Retime(RetimeIntent::ResetSelected));
        return Some(EventOutcome::Consumed);
    }
    let retime = match id {
        i if i == ids::TIMELINE_SPEED_OVERRIDE => false,
        i if i == ids::TIMELINE_SPEED_RETIME => true,
        _ => return None,
    };
    if state.speed_retime != retime && state.handle_drag.take().is_some() {
        state::push_intent(TimelineIntent::EndEdit);
    }
    state.speed_retime = retime;
    Some(EventOutcome::Consumed)
}

const CELULAS: [(ph2d_a11y::NodeId, &str, &str); 2] = [
    (
        ids::TIMELINE_SPEED_OVERRIDE,
        "panel.timeline.speed_mode.override",
        "panel.timeline.speed_mode.override.tip",
    ),
    (
        ids::TIMELINE_SPEED_RETIME,
        "panel.timeline.speed_mode.retime",
        "panel.timeline.speed_mode.retime.tip",
    ),
];

/// A pílula Override · Retime no meio da coluna da esquerda da banda, e o «Reset timing» por baixo
/// em Retime. Uma banda baixa demais para os dois e as duas escalas não pinta nada.
pub(crate) fn paint_switch(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    (row, band_rect): (Rect, Rect),
    label_w: f32,
) {
    let pad = Spacing::Xs.px();
    let w = label_w - pad * 2.0;
    let escalas = TypeToken::Xs.px() * 2.0 + pad * 2.0;
    if w <= 0.0 || band_rect.h < escalas + ROW_H_PX + RESET_H + pad {
        return;
    }
    let y = band_rect.y + (band_rect.h - ROW_H_PX - RESET_H - pad) * 0.5;
    let rect = Rect::new(row.x + pad, y, w, ROW_H_PX);
    let itens = CELULAS
        .iter()
        .map(|(id, chave, _)| TabItem::new(*id, ph2d_i18n::tr(chave)))
        .collect();
    let tabs = Tabs::new(ids::TIMELINE_SPEED_MODE, "", itens)
        .selected(usize::from(state.speed_retime))
        .variant(TabsVariant::Segmented);
    let quente = ctx.host.store().hot_id();
    let hovered = CELULAS.iter().position(|(id, _, _)| quente == Some(*id));
    paint_tabs_with_hover(&tabs, hovered, rect, ctx.scene, ctx.text_system, theme);
    for (i, (id, _, dica)) in CELULAS.iter().enumerate() {
        ctx.host
            .hit_index_mut()
            .register(*id, tabs.tab_rect(rect, i));
        dica_se_falta(ctx, *id, dica);
    }
    if state.speed_retime {
        let reset = Rect::new(rect.x, rect.y + ROW_H_PX + pad, w, RESET_H);
        paint_reset(ctx, theme, reset);
    }
}

fn dica_se_falta(ctx: &mut PaintCtx, id: ph2d_a11y::NodeId, chave: &str) {
    let store = ctx.host.store_mut();
    if store.tooltip_for(id).is_none() {
        store.set_tooltip(id, ph2d_i18n::tr(chave));
    }
}

/// O «Reset timing», no feitio dos chips do Buffer da mesma banda.
fn paint_reset(ctx: &mut PaintCtx, theme: Theme, rect: Rect) {
    let id = ids::TIMELINE_RESET_TIMING;
    let quente = ctx.host.store().hot_id() == Some(id);
    let raio = ph2d_editor_core::paint::frame_radius(theme, Radius::Xs.px());
    fill_rounded_rect(ctx.scene, rect, raio, resolve(ColorToken::BgElev, theme));
    let (feel, borda) = if quente {
        (ph2d_tokens::visuals::Feel::Hovered, ColorToken::Accent)
    } else {
        (ph2d_tokens::visuals::Feel::Rest, ColorToken::Border)
    };
    let borda = resolve(borda, theme);
    let traco = StrokeToken::Thin.px();
    ph2d_editor_core::paint::stroke_frame(ctx.scene, rect, raio, theme, feel, traco, borda);
    let texto = ph2d_i18n::tr("panel.timeline.reset_timing");
    let cor = resolve(ColorToken::Text2, theme);
    let fonte = TypeToken::Xs.px();
    paint_text_centered(ctx.text_system, ctx.scene, texto, rect, fonte, cor);
    ctx.host.hit_index_mut().register(id, rect);
    dica_se_falta(ctx, id, "panel.timeline.reset_timing.tip");
}

#[cfg(test)]
#[path = "graph_ritmo_tests.rs"]
mod tests;
