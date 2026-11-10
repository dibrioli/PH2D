//! ⭐ **O trecho entre dois marcadores, no painel** (doc 17 item 2, maquete B6): o vão entre dois vizinhos
//! na faixa «Markers» (`faixa_marcadores`) é o alvo; o trecho armado é o laço da régua e a caixa da faixa,
//! e a barra mostra a pílula «takeoff → top ✕». Só nos cartões; o motor é `ph2d_timeline::trecho`.
//!
//! O espelho do painel ([`TRECHO`]) obedece à MESMA regra do motor (`Trecho::intervalo`) lida do
//! instantâneo, por isso os dois largam-no no mesmo quadro.

use std::cell::Cell;

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::trecho::vista_com_trecho;
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot, Trecho};
use ph2d_tokens::Theme;
use ph2d_vector::Color;

use crate::ruler::{RULER_H, TICK_MAJOR_H};
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

/// O trecho que vale nesta régua (só nos cartões); confere o espelho a cada quadro. Enquanto vale, ELE
/// é o laço que a régua mostra (maquete B6: a `.c13lp` vai de takeoff a top) e o do clipe some.
pub(crate) fn vale(snap: &TimelineViewSnapshot, clock: RulerClock) -> Option<(f64, f64)> {
    if !ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        return None;
    }
    let vista = vista(snap, clock);
    let v =
        atual().and_then(|t| t.intervalo(t_do_snap(snap), snap.active_clip, snap.keys_mode, vista));
    if v.is_none() {
        TRECHO.with(|c| c.set(None));
    }
    v
}

pub(crate) fn vista(snap: &TimelineViewSnapshot, clock: RulerClock) -> bool {
    clock.markers
        && clock.scrub
        && vista_com_trecho(snap.keys_mode, snap.stacked(), snap.container_open, false)
}

/// `(fundo, contorno)` da caixa do trecho (`.c13sec`: a cor do laço a 20 % com fio de 1 px a 65 %).
pub(crate) fn tintas(theme: Theme) -> (Color, Color) {
    use ph2d_tokens::chrome::{TIMELINE_TRECHO_BORDA_ALPHA, TIMELINE_TRECHO_FUNDO_ALPHA};
    let cor = crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::LacoDaRegua, theme);
    (
        cor.multiply_alpha(TIMELINE_TRECHO_FUNDO_ALPHA),
        cor.multiply_alpha(TIMELINE_TRECHO_BORDA_ALPHA),
    )
}

/// O topo do texto da régua nos cartões: as maiúsculas no meio do vão entre a faixa do laço e o traço
/// maior, para que a faixa nunca corte um número nem um nome (no clássico, o topo da régua).
pub(crate) fn y_do_texto(region: Rect, font: f32) -> f32 {
    use ph2d_tokens::chrome::TIMELINE_LACO_REGUA_PX;
    if !ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        return region.y;
    }
    let vao = RULER_H - TICK_MAJOR_H - TIMELINE_LACO_REGUA_PX;
    region.y + TIMELINE_LACO_REGUA_PX + ph2d_editor_core::paint::folga_do_texto(vao, font)
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

#[cfg(test)]
#[path = "ruler_trecho_tests.rs"]
mod tests;
