//! **A barra de DUAS pegas** (o *Sensor Threshold* do popup Dynamics do Aseprite, foto
//! `docs/PixelArt/aseprite/55_dinamica.png`): um intervalo `lo ≤ hi` em fracções `0..=1` da trilha.
//!
//! O widget é sem estado: quem o usa regista a trilha como `Plain`, lê o ponteiro enquanto ela é a
//! activa e pergunta [`range_handle_at`] no premir e [`range_drag`] a cada movimento. As pegas
//! nunca se cruzam: arrastar uma contra a outra encosta-as (juntas = o mais sensível).

use super::{SliderOrientation, SliderState, paint_slider_track};
use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// Qual das duas pegas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeHandle {
    Lo,
    Hi,
}

/// A fracção `0..=1` da trilha sob `x` (fora dela, presa à ponta).
#[must_use]
pub fn range_value_at(track: Rect, x: f32) -> f32 {
    if track.w <= 0.0 || !x.is_finite() {
        return 0.0;
    }
    ((x - track.x) / track.w).clamp(0.0, 1.0)
}

/// A pega que o premir em `x` agarra: a mais perto; empatadas (juntas), a do lado para onde o
/// ponteiro está, e sobre elas a que ainda se pode mexer — assim juntas sempre se separam.
#[must_use]
pub fn range_handle_at(track: Rect, x: f32, lo: f32, hi: f32) -> RangeHandle {
    let v = range_value_at(track, x);
    let (dl, dh) = ((v - lo).abs(), (v - hi).abs());
    if dl != dh {
        return if dl < dh {
            RangeHandle::Lo
        } else {
            RangeHandle::Hi
        };
    }
    if v < lo || (v <= hi && hi >= 1.0) {
        RangeHandle::Lo
    } else {
        RangeHandle::Hi
    }
}

/// O intervalo depois de arrastar `pega` até `x`; a outra fica, e a arrastada encosta nela.
#[must_use]
pub fn range_drag(track: Rect, x: f32, pega: RangeHandle, lo: f32, hi: f32) -> (f32, f32) {
    let v = range_value_at(track, x);
    match pega {
        RangeHandle::Lo => (v.min(hi), hi),
        RangeHandle::Hi => (lo, v.max(lo)),
    }
}

/// A largura de uma pega, em px (o traço de marca do Aseprite, mais largo para o dedo ver).
fn pega_w() -> f32 {
    StrokeToken::Thick.px() * 2.0
}

/// O rect da pega no valor `v` da trilha `track`.
#[must_use]
pub fn range_handle_rect(track: Rect, v: f32) -> Rect {
    let cx = track.x + track.w * v.clamp(0.0, 1.0);
    let w = pega_w();
    Rect::new(cx - w * 0.5, track.y, w, track.h)
}

/// Pinta a trilha com o intervalo `lo..hi` cheio de acento e as duas pegas; `activa` = a agarrada.
pub fn paint_range_slider(
    track: Rect,
    lo: f32,
    hi: f32,
    activa: Option<RangeHandle>,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let (lo, hi) = (lo.clamp(0.0, 1.0), hi.clamp(0.0, 1.0));
    let estado = if activa.is_some() {
        SliderState::Dragging
    } else {
        SliderState::Normal
    };
    paint_slider_track(
        track,
        0.0,
        SliderOrientation::Horizontal,
        (estado, crate::motion::SETTLED),
        scene,
        theme,
    );
    let r = crate::paint::frame_radius(theme, Radius::Xs.px());
    let cheio = Rect::new(
        track.x + track.w * lo,
        track.y,
        track.w * (hi - lo),
        track.h,
    );
    if cheio.w > 0.0 {
        fill_rounded_rect(scene, cheio, r, resolve(ColorToken::Accent, theme));
    }
    for (pega, v) in [(RangeHandle::Lo, lo), (RangeHandle::Hi, hi)] {
        let cor = if activa == Some(pega) {
            ColorToken::AccentPress
        } else {
            ColorToken::Text1
        };
        fill_rounded_rect(scene, range_handle_rect(track, v), r, resolve(cor, theme));
    }
}

#[cfg(test)]
#[path = "range_slider/tests.rs"]
mod tests;
