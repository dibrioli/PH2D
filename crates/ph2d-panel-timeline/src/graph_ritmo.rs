//! ⭐ **Override · Retime na vista de velocidade** (plano 18, dono 15/11).
//!
//! **Override** (como sempre): a curva da velocidade no acento, com as alças do EASING — arrastar
//! reescreve o easing. **Retime** (pesquisa 19, opção A): a velocidade do movimento na cor da curva
//! (os quiques do Bounce ficam, só para ler) e, sobre cada trecho seleccionado, a CAIXA DO TEMPO em
//! âmbar: a curva `W` de canto a canto e as duas alças dela; a borda tracejada é o limite, e o
//! arrasto além dela fica na borda ([`ph2d_timeline::retime_with_time_handle`]). A escolha é da
//! sessão, na coluna à esquerda da banda, com «Reset timing» por baixo em Retime.

use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind, WidgetEvent};
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text_centered, resolve, stroke_polyline,
};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx};
use ph2d_editor_core::widget::{TabItem, Tabs, TabsVariant, paint_tabs_with_hover};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{
    KeyView, RetimeIntent, TimelineIntent, TrackView, retime_with_time_handle, time_handle_tip,
};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};

use crate::graph::{Band, TimeView};
use crate::graph_paint::{ANCHOR_R, CURVE_STEP_PX, CURVE_W, HANDLE_HIT_R, HANDLE_R};
use crate::ids;
use crate::state::{self, TimelinePanelState};

/// Onde a alça de um lado de um trecho se pinta: `(tempo, velocidade)`.
pub(crate) type Ponta = fn(&KeyView, &KeyView, u8) -> Option<(f64, f64)>;

/// A altura do «Reset timing».
const RESET_H: f32 = 15.0; // LITERAL-PX-OK: a altura do chip do Buffer, o vizinho da banda

/// A cor da curva da velocidade, pelo modo: em Retime ela só se lê (o âmbar é o tempo).
pub(crate) fn cor_da_curva(state: &TimelinePanelState) -> ColorToken {
    if state.speed_retime {
        ColorToken::TimelineCurve
    } else {
        ColorToken::Accent
    }
}

/// Os trechos que mostram a caixa: os que tocam uma chave seleccionada (os mesmos das alças).
fn trechos(track: &TrackView) -> impl Iterator<Item = (&KeyView, &KeyView)> {
    track
        .keys
        .windows(2)
        .map(|w| (&w[0], &w[1]))
        .filter(|(a, b)| a.selected || b.selected)
}

/// A caixa do tempo de um trecho no ecrã: de chave a chave na horizontal, a altura da banda menos a
/// folga de uma alça em cima e em baixo (`W = 0` no fundo, `W = 1` no topo). Presa ao RECTÂNGULO da
/// banda, nunca à escala da velocidade: o reenquadramento de um arrasto não a move.
pub(crate) fn caixa(band_rect: Rect, view: TimeView, k0: &KeyView, k1: &KeyView) -> Rect {
    let folga = Spacing::Sm.px();
    let (x0, x1) = (view.x(k0.t_seconds), view.x(k1.t_seconds));
    Rect::new(
        x0,
        band_rect.y + folga,
        (x1 - x0).max(0.0),
        (band_rect.h - folga * 2.0).max(0.0),
    )
}

/// O ponto `(u, W)` da caixa no ecrã.
fn no_ecra(c: Rect, (u, w): (f64, f64)) -> (f32, f32) {
    (c.x + u as f32 * c.w, c.y + c.h - w as f32 * c.h)
}

/// A borda da caixa, tracejada: o limite que o gesto não passa.
fn tracejado(ctx: &mut PaintCtx, c: Rect, cor: ph2d_vector::Color) {
    let (traco, vazio) = (Spacing::Xs.px(), Spacing::Xs.px());
    let cantos = [
        (c.x, c.y),
        (c.x + c.w, c.y),
        (c.x + c.w, c.y + c.h),
        (c.x, c.y + c.h),
        (c.x, c.y),
    ];
    for par in cantos.windows(2) {
        let ((ax, ay), (bx, by)) = (par[0], par[1]);
        let comp = (bx - ax).hypot(by - ay);
        let mut d = 0.0;
        while d < comp {
            let e = (d + traco).min(comp);
            let p = |f: f32| (ax + (bx - ax) * f / comp, ay + (by - ay) * f / comp);
            stroke_polyline(ctx.scene, &[p(d), p(e)], StrokeToken::Thin.px(), cor);
            d = e + vazio;
        }
    }
}

/// Em Retime, sobre cada trecho seleccionado com ritmo: a caixa do tempo, a curva `W` em âmbar e as
/// duas alças (braço do canto à ponta), registadas no MESMO `CurveHandle` das alças de sempre.
pub(crate) fn paint_caixas(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    band: &Band,
    view: TimeView,
    track: &TrackView,
) {
    let ambar = resolve(ColorToken::Warn, theme);
    let target = track.target.get();
    for (k0, k1) in trechos(track) {
        let c = caixa(band.rect, view, k0, k1);
        let Some(pontas) = [0u8, 1u8]
            .into_iter()
            .map(|which| time_handle_tip(k0, k1, which))
            .collect::<Option<Vec<_>>>()
        else {
            continue; // trecho parado: o ritmo não muda o que toca
        };
        if c.w < HANDLE_HIT_R * 2.0 || c.h <= 0.0 {
            continue;
        }
        tracejado(ctx, c, resolve(ColorToken::Text3, theme));
        let n = (c.w / CURVE_STEP_PX).ceil().max(1.0) as usize;
        let curva: Vec<_> = (0..=n)
            .map(|i| {
                let u = i as f64 / n as f64;
                no_ecra(c, (u, ph2d_anim::retime::warp(k0.retime, u)))
            })
            .collect();
        stroke_polyline(ctx.scene, &curva, CURVE_W, ambar);
        for (which, ponta) in pontas.into_iter().enumerate() {
            let which = which as u8;
            let canto = no_ecra(c, if which == 0 { (0.0, 0.0) } else { (1.0, 1.0) });
            let (px, py) = no_ecra(c, ponta);
            let linha = resolve(ColorToken::TimelineHandleLine, theme);
            stroke_polyline(ctx.scene, &[canto, (px, py)], StrokeToken::Thin.px(), linha);
            let chave = resolve(ColorToken::TimelineKey, theme);
            fill_circle(ctx.scene, canto.0, canto.1, ANCHOR_R, chave);
            let arrastada = state
                .handle_drag
                .is_some_and(|d| d.target == target && d.key == k0.id.get() && d.which == which);
            let tok = if arrastada {
                ColorToken::TimelineKeyActive
            } else {
                ColorToken::Warn
            };
            fill_circle(ctx.scene, px, py, HANDLE_R, resolve(tok, theme));
            let id = ids::timeline_handle_hit_id(target, k0.id.get(), which);
            let hit = Rect::new(
                px - HANDLE_HIT_R,
                py - HANDLE_HIT_R,
                HANDLE_HIT_R * 2.0,
                HANDLE_HIT_R * 2.0,
            );
            ctx.host.store_mut().register(
                id,
                InteractiveState::TimelineSurface {
                    parent: ph2d_editor_core::ids::TIMELINE_PANEL,
                    kind: TimelineHitKind::CurveHandle {
                        target,
                        key: k0.id.get(),
                        which,
                    },
                    canvas: band.rect,
                },
            );
            ctx.host.hit_index_mut().register(id, hit);
        }
    }
}

/// Um frame do arrasto de uma alça da caixa, como o comando do documento: o ponteiro no ecrã vira
/// `(u, W)` da caixa do trecho. `None` num trecho parado ou numa caixa sem área.
pub(crate) fn intent_do_arrasto(
    (k0, k1): (&KeyView, &KeyView),
    track: &TrackView,
    which: u8,
    (x, y): (f32, f32),
    (band_rect, view): (Rect, TimeView),
) -> Option<TimelineIntent> {
    let c = caixa(band_rect, view, k0, k1);
    if c.w <= 0.0 || c.h <= 0.0 {
        return None;
    }
    let u = f64::from((x - c.x) / c.w);
    let w = f64::from((c.y + c.h - y) / c.h);
    let retime = retime_with_time_handle(k0, k1, which, (u, w))?;
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
