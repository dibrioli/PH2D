//! ⭐ **A rolagem de uma lista que abre por cima de tudo** (a lista suspensa de um campo do Kanban,
//! D14): a roda, o polegar arrastado (também fora da janela), carregar na calha e trazer uma linha à
//! vista — sem `WidgetStore`, para quem tem os próprios alvos. A barra é uma [`AxisBar`] com setas.

mod axis_bar;
#[cfg(test)]
mod tests;

pub use axis_bar::{ArrowHold, AxisBar, BarGrab, HoldRepeat};

use super::scrollbar::ScrollbarState;
use crate::math::safe_clamp;
use crate::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Theme};
use ph2d_vector::VectorScene;

/// A rolagem, o arrasto do polegar (`y` do carregar, rolagem de então) e a geometria do último
/// quadro (a vista, a calha e a altura do conteúdo).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScrollList {
    pub scroll: f32,
    pub grab: Option<(f32, f32)>,
    /// A seta segura (D17).
    pub hold: Option<ArrowHold>,
    pub view: Rect,
    pub track: Option<Rect>,
    pub content_h: f32,
}

impl ScrollList {
    fn max(&self) -> f32 {
        (self.content_h - self.view.h).max(0.0)
    }

    /// A geometria deste quadro (a rolagem fica dentro dela).
    pub fn set(&mut self, view: Rect, content_h: f32) {
        self.view = view;
        self.content_h = content_h;
        self.scroll = safe_clamp(self.scroll, 0.0, self.max());
    }

    /// Rola `dy` px (positivo = ver mais abaixo).
    pub fn scroll_by(&mut self, dy: f32) {
        self.scroll = safe_clamp(self.scroll + dy, 0.0, self.max());
    }

    fn bar(&self, track: Rect) -> AxisBar {
        AxisBar {
            track,
            horizontal: false,
            content: self.content_h,
            visible: self.view.h,
            arrows: true,
            step: ROW_H_PX,
        }
    }

    /// Carregar na barra em `now_ns`: numa seta rola uma linha e segura-a; no polegar agarra-o; na
    /// calha salta o polegar para lá, e agarra.
    pub fn press_bar(&mut self, y: f32, now_ns: u128) {
        let Some(track) = self.track else {
            return;
        };
        let (s, hold) = self
            .bar(track)
            .press_held((track.x, y), self.scroll, now_ns);
        self.scroll = s;
        self.hold = hold;
        if hold.is_none() {
            self.grab = Some((y, self.scroll));
        }
    }

    /// Cada quadro: a seta segura repete ([`AxisBar::repeat`]). `true` = há uma seta segura (o
    /// chamador pede quadros enquanto for).
    pub fn tick(&mut self, now_ns: u128) -> bool {
        let (Some(mut h), Some(track)) = (self.hold, self.track) else {
            return false;
        };
        self.scroll = self.bar(track).repeat(&mut h, self.scroll, now_ns);
        self.hold = Some(h);
        true
    }

    /// Largar a barra: solta o polegar e a seta.
    pub fn release(&mut self) {
        self.grab = None;
        self.hold = None;
    }

    /// O arrasto do polegar até `y` (com uma seta segura, só o sítio do rato).
    pub fn drag_bar(&mut self, y: f32) {
        if let (Some(h), Some(track)) = (self.hold.as_mut(), self.track) {
            h.at = (track.x, y);
        }
        let (Some((y0, s0)), Some(track)) = (self.grab, self.track) else {
            return;
        };
        self.scroll = self.bar(track).drag((track.x, y0), s0, (track.x, y));
    }

    /// Rola o mínimo para a faixa `[top, top + h)` do conteúdo ficar à vista com `pad` de folga.
    pub fn reveal(&mut self, top: f32, h: f32, pad: f32) {
        if top < self.scroll {
            self.scroll = top - pad;
        } else if top + h > self.scroll + self.view.h {
            self.scroll = top + h + pad - self.view.h;
        }
        self.scroll = safe_clamp(self.scroll, 0.0, self.max());
    }

    /// Pinta a barra em `body` (se o conteúdo não cabe) e guarda a calha; devolve-a para o alvo.
    pub fn paint_bar(
        &mut self,
        body: Rect,
        hovered: bool,
        scene: &mut VectorScene,
        theme: Theme,
    ) -> Option<Rect> {
        let state = match (self.grab, hovered) {
            (Some(_), _) => ScrollbarState::Dragging,
            (None, true) => ScrollbarState::Hovered,
            _ => ScrollbarState::Normal,
        };
        let extent = (self.content_h, self.view.h, ROW_H_PX);
        let bar = AxisBar::of_theme(body, false, extent);
        bar.paint_themed(self.scroll, (state, crate::motion::SETTLED), scene, theme);
        self.track = bar.needed().then_some(bar.track);
        self.track
    }
}
