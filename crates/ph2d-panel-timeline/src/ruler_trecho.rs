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
use ph2d_tokens::{Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::Color;

use crate::alca_estado::EstadoAlca;
use crate::ids;
use crate::ruler::{MARKER_W, RULER_H, TICK_MAJOR_H};
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

fn vista(snap: &TimelineViewSnapshot, clock: RulerClock) -> bool {
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

/// A caixa do trecho de `x0` a `x1`, a régua inteira: com fundo (armado) ou só o fio (o vão sob o ponteiro).
fn caixa(ctx: &mut PaintCtx, theme: Theme, region: Rect, (x0, x1): (f32, f32), fundo: bool) {
    let (f, c) = tintas(theme);
    let r = Rect::new(x0, region.y, (x1 - x0).max(0.0), RULER_H);
    let fio = StrokeToken::Thin.px();
    if fundo {
        fill_rounded_rect(ctx.scene, r, Radius::Xs.px(), f);
    }
    for lado in [
        Rect::new(r.x, r.y, r.w, fio),
        Rect::new(r.x, r.y + r.h - fio, r.w, fio),
        Rect::new(r.x, r.y, fio, r.h),
        Rect::new(r.x + r.w - fio, r.y, fio, r.h),
    ] {
        fill_rounded_rect(ctx.scene, lado, 0.0, c);
    }
}

/// A caixa do trecho armado, ANTES dos números (eles leem-se por cima, como os nomes na B6).
pub(crate) fn pinta_armado(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    time_to_x: &impl Fn(f64) -> f32,
    vale: Option<(f64, f64)>,
) {
    if let Some((a, b)) = vale {
        caixa(ctx, theme, region, (time_to_x(a), time_to_x(b)), true);
    }
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

/// `(x, y, largura máxima)` do nome do marcador `i` (bandeirola de `MARKER_W` × `alto`): à direita dela;
/// nos cartões à altura dos números e nunca além da bandeirola seguinte (o clássico não muda).
pub(crate) fn lugar_do_nome(
    region: Rect,
    time_to_x: &impl Fn(f64) -> f32,
    snap: &TimelineViewSnapshot,
    i: usize,
    alto: f32,
) -> (f32, f32, f32) {
    let (meia, font, t) = (MARKER_W * 0.5, TypeToken::Xs.px(), snap.markers[i].0);
    let x = time_to_x(t);
    let x0 = x + meia + Spacing::Xs.px();
    let right = region.x + region.w;
    if !ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        let y = region.y + (alto - font) * 0.5;
        return (x0, y, (right - x - meia).max(0.0));
    }
    let fim = snap.markers.iter().filter(|m| m.0 > t).fold(right, |f, m| {
        f.min(time_to_x(m.0) - meia - Spacing::Xs.px())
    });
    (x0, y_do_texto(region, font), (fim - x0).max(0.0))
}

/// Um número da régua de `w` medido a partir de `x0` que cairia sobre um nome de marcador cede-lhe o lugar.
pub(crate) fn cede_ao_nome(
    ctx: &mut PaintCtx,
    cede: &[(f32, f32)],
    x0: f32,
    txt: &str,
    font: f32,
) -> bool {
    !cede.is_empty() && {
        let w = ph2d_editor_core::paint::label_column_width(ctx.text_system, font, [txt]);
        cede.iter().any(|&(a, b)| x0 < b && x0 + w > a)
    }
}

/// ⭐ **Onde os nomes dos marcadores moram na régua** (só nos cartões): `[x0, x1)` de cada bandeirola com
/// o nome. Um número da régua que cairia aí cede o lugar (o nome diz mais; o segundo lê-se ao lado).
pub(crate) fn vaos_dos_nomes(
    ctx: &mut PaintCtx,
    region: Rect,
    time_to_x: &impl Fn(f64) -> f32,
    snap: &TimelineViewSnapshot,
    clock: RulerClock,
    meia: f32,
) -> Vec<(f32, f32)> {
    if !clock.markers || !ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        return Vec::new();
    }
    let font = TypeToken::Xs.px();
    (0..snap.markers.len())
        .map(|i| {
            let (x0, _, max) = lugar_do_nome(region, time_to_x, snap, i, 0.0);
            let nome = [snap.markers[i].1.as_str()];
            let w = ph2d_editor_core::paint::label_column_width(ctx.text_system, font, nome);
            (time_to_x(snap.markers[i].0) - meia, x0 + w.min(max))
        })
        .collect()
}

/// Os alvos dos vãos (só nos cartões), depois do laço (o vão ganha à faixa de mover, na banda das
/// bandeirolas) e antes delas e da alça da duração, que ganham ao vão. O vão sob o ponteiro mostra o
/// fio da caixa do trecho que um clique tocaria.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    time_to_x: &impl Fn(f64) -> f32,
    snap: &TimelineViewSnapshot,
    clock: RulerClock,
    marker: (f32, f32),
) {
    if !ph2d_editor_core::panel::cards::ferramenta::cards_on() || !vista(snap, clock) {
        return;
    }
    let (meia, h) = (marker.0 * 0.5, marker.1);
    for (a, b) in vizinhos(snap) {
        let (ta, tb) = (time_to_x(snap.markers[a].0), time_to_x(snap.markers[b].0));
        let (x0, x1) = (ta + meia, tb - meia);
        if x1 <= x0 {
            continue;
        }
        let id = ids::timeline_marker_gap_hit_id(a);
        if EstadoAlca::nos_cartoes(ctx, id) != EstadoAlca::Repouso {
            caixa(ctx, theme, region, (ta, tb), false);
        }
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

#[cfg(test)]
#[path = "ruler_trecho_tests.rs"]
mod tests;
