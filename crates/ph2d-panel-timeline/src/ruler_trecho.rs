//! ⭐ **O trecho entre dois marcadores, no painel** (doc 17 item 2, maquete B6): a zona dos marcadores
//! na régua (a faixa das bandeirolas) entre dois vizinhos é o alvo; o trecho a tocar pinta-se nela, e a
//! barra mostra «takeoff → top» com ✕. Só nos cartões; o motor é `ph2d_timeline::trecho`.
//!
//! O espelho do painel ([`TRECHO`]) obedece à MESMA regra do motor (`Trecho::intervalo`) lida do
//! instantâneo, por isso os dois largam-no no mesmo quadro.

use std::cell::Cell;

use ph2d_editor_core::interaction::{
    GesturePhase, InteractiveState, TimelineGesture, TimelineHitKind,
};
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::trecho::vista_com_trecho;
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot, Trecho};
use ph2d_tokens::{Radius, Theme};

use crate::ids;
use crate::ruler_clock::RulerClock;

thread_local! {
    static TRECHO: Cell<Option<Trecho>> = const { Cell::new(None) };
}

/// O trecho que o painel mostra (o mesmo que o motor toca).
pub(crate) fn atual() -> Option<Trecho> {
    TRECHO.with(Cell::get)
}

fn t_do_snap(snap: &TimelineViewSnapshot) -> impl Fn(usize) -> Option<f64> + '_ {
    |i| snap.markers.get(i).map(|m| m.0)
}

/// Os pares de marcadores vizinhos no tempo, `(a, b)` em índices de armazenamento.
pub(crate) fn vizinhos(snap: &TimelineViewSnapshot) -> Vec<(usize, usize)> {
    let mut ordem: Vec<usize> = (0..snap.markers.len()).collect();
    ordem.sort_by(|&i, &j| snap.markers[i].0.total_cmp(&snap.markers[j].0));
    ordem
        .windows(2)
        .filter(|w| snap.markers[w[1]].0 > snap.markers[w[0]].0)
        .map(|w| (w[0], w[1]))
        .collect()
}

/// A régua: confere o espelho, pinta o trecho e regista um alvo por vão. Depois do laço (o vão ganha à
/// faixa de mover, na banda das bandeirolas) e antes delas e da alça da duração, que ganham ao vão.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    time_to_x: &impl Fn(f64) -> f32,
    snap: &TimelineViewSnapshot,
    clock: RulerClock,
    marker: (f32, f32),
) {
    let vista = clock.markers
        && clock.scrub
        && vista_com_trecho(snap.keys_mode, snap.stacked(), snap.container_open, false);
    let vale =
        atual().and_then(|t| t.intervalo(t_do_snap(snap), snap.active_clip, snap.keys_mode, vista));
    if vale.is_none() {
        TRECHO.with(|c| c.set(None));
    }
    if !vista {
        return;
    }
    let (meia, h) = (marker.0 * 0.5, marker.1);
    if let Some((a, b)) = vale {
        let (_, cor) = crate::ruler::faixa_do_laco(theme);
        let (x0, x1) = (time_to_x(a), time_to_x(b));
        fill_rounded_rect(
            ctx.scene,
            Rect::new(x0, region.y, (x1 - x0).max(0.0), h),
            Radius::Xs.px(),
            cor,
        );
    }
    for (a, b) in vizinhos(snap) {
        let (x0, x1) = (
            time_to_x(snap.markers[a].0) + meia,
            time_to_x(snap.markers[b].0) - meia,
        );
        if x1 <= x0 {
            continue;
        }
        let id = ids::timeline_marker_gap_hit_id(a);
        let hit = Rect::new(x0, region.y, x1 - x0, h);
        ctx.host.store_mut().register(
            id,
            InteractiveState::TimelineSurface {
                parent: ph2d_editor_core::ids::TIMELINE_PANEL,
                kind: TimelineHitKind::MarkerGap { a, b },
                canvas: hit,
            },
        );
        ctx.host.hit_index_mut().register(id, hit);
    }
}

/// Um clique no vão `(a, b)` toca esse trecho em loop.
pub(crate) fn apply(snap: &TimelineViewSnapshot, a: usize, b: usize, g: TimelineGesture) {
    if !matches!(g.phase, GesturePhase::Click) {
        return;
    }
    if let Some(t) = Trecho::novo(t_do_snap(snap), a, b, snap.active_clip, snap.keys_mode) {
        TRECHO.with(|c| c.set(Some(t)));
        crate::state::push_intent(TimelineIntent::PlayStretch(Some((a, b))));
    }
}

/// O ✕ da barra: volta a tocar o clipe todo.
pub(crate) fn larga() {
    TRECHO.with(|c| c.set(None));
    crate::state::push_intent(TimelineIntent::PlayStretch(None));
}

/// O texto do chip, «takeoff → top», se há trecho.
pub(crate) fn rotulo(snap: &TimelineViewSnapshot) -> Option<String> {
    let t = atual()?;
    let nome = |i: usize| snap.markers.get(i).map(|m| m.1.as_str());
    Some(ph2d_i18n::tr_with(
        "panel.timeline.stretch.label",
        &[("from", &nome(t.a)?), ("to", &nome(t.b)?)],
    ))
}
