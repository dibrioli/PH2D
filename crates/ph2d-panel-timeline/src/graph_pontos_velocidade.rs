//! Os pontos da curva Custom na vista de VELOCIDADE (Override): os braços de cada ponto e o
//! arrasto deles. Partido de `graph_pontos.rs` (tecto de LOC do painel, 16/11).

use ph2d_anim::HandleKind;
use ph2d_editor_core::interaction::TimelineHitKind;
use ph2d_editor_core::paint::{fill_circle, fill_rounded_rect, resolve, stroke_polyline};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_timeline::{TimelineIntent, TrackView, interp_of_curve, segment_curve};
use ph2d_tokens::{ColorToken, StrokeToken, Theme};

use super::{ANEL, PONTO_R, grava, quadrado, regista, trecho, trechos};
use crate::graph::{Band, TimeView};
use crate::graph_paint::{HANDLE_HIT_R, HANDLE_R};
use crate::ids;
use crate::state::{TimelinePanelState, push_intent};

/// Na vista de VELOCIDADE (Override), cada ponto do meio é uma chave sem chave: a velocidade dele
/// de cada lado, com o braço horizontal da influência e a ponta que se agarra (o mesmo gesto das
/// chaves: vertical = velocidade, horizontal = influência). Corner num quadrado, como no valor.
pub(crate) fn paint_speed_arms(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    band: &Band,
    view: TimeView,
    track: &TrackView,
) {
    let target = track.target.get();
    let braco = resolve(ColorToken::TimelineHandleLine, theme);
    let cor = resolve(ColorToken::TimelineHandle, theme);
    let acento = resolve(ColorToken::TimelineKeyActive, theme);
    let miolo = resolve(ColorToken::BgElev, theme);
    for (k0, k1, c) in trechos(track) {
        let key = k0.id.get();
        for a in ph2d_timeline::point_speed_arms(k0, k1) {
            let (ax, px, py) = (view.x(a.t), view.x(a.tip_t), band.y(a.speed));
            stroke_polyline(
                ctx.scene,
                &[(ax, py), (px, py)],
                StrokeToken::Thin.px(),
                braco,
            );
            let escolhido = state.ponto_escolhido == Some((target, key, a.node));
            let tinta = if escolhido { acento } else { cor };
            if c.kind(a.node) == Some(HandleKind::Corner) {
                fill_rounded_rect(ctx.scene, quadrado((ax, py), PONTO_R), 0.0, tinta);
            } else {
                fill_circle(ctx.scene, ax, py, PONTO_R, tinta);
            }
            fill_circle(ctx.scene, px, py, HANDLE_R, cor);
            fill_circle(ctx.scene, px, py, HANDLE_R * ANEL, miolo);
            if py < band.rect.y || py > band.rect.y + band.rect.h {
                continue;
            }
            let node = a.node as u8;
            let id = ids::timeline_curve_point_hit_id(target, key, node, a.part);
            let kind = TimelineHitKind::CurvePoint {
                target,
                key,
                node,
                part: a.part,
            };
            regista(ctx, band, id, kind, quadrado((px, py), HANDLE_HIT_R));
        }
    }
}

/// Resolve neste quadro o arrasto de um braço de velocidade de um ponto (vista de velocidade).
pub(crate) fn resolve_speed_drag(
    state: &mut TimelinePanelState,
    band: &Band,
    view: TimeView,
    track: &TrackView,
) {
    let target = track.target.get();
    let Some(mut d) = state.ponto_drag.filter(|d| d.target == target) else {
        return;
    };
    let Some((k0, k1)) = trecho(track, d.key) else {
        state.ponto_drag = None;
        push_intent(TimelineIntent::EndEdit);
        return;
    };
    if d.part != 0 {
        let par = (d.node, d.part);
        let corner = d.corner.then(|| {
            let mut c = segment_curve(k0, k1)?;
            c.set_kind(d.node, HandleKind::Corner).then_some(c)
        });
        let base = match corner.flatten() {
            Some(c) => {
                let mut k = k0.clone();
                k.interp = interp_of_curve(c);
                k
            }
            None => k0.clone(),
        };
        d.corner = false;
        let (t, v) = (view.t(d.x), band.value(d.y));
        if let Some(c) = ph2d_timeline::spline_with_point_speed(&base, k1, par, t, v) {
            grava(track, k0, c);
        }
    }
    if d.ending || d.part == 0 {
        state.ponto_drag = None;
        push_intent(TimelineIntent::EndEdit);
    } else {
        state.ponto_drag = Some(d);
    }
}
