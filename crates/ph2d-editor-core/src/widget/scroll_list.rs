//! ⭐ **A rolagem de uma lista que abre por cima de tudo** (a lista suspensa de um campo do Kanban,
//! D14): a roda, o polegar arrastado (também fora da janela), carregar na calha e trazer uma linha à
//! vista — sem `WidgetStore`, para quem tem os próprios alvos. A barra é a [`super::paint_scrollbar`].

use super::scrollbar::{
    ScrollbarState, delta_for_drag, is_needed, paint_scrollbar, thumb_rect, track_rect,
};
use crate::paint::fill_rounded_rect;
use crate::zones::Rect;
use ph2d_tokens::Theme;
use ph2d_vector::{Color, VectorScene};

/// ⭐ **Uma barra de rolagem num EIXO**, sobre a calha que o chamador dispõe (a de cada lista e a do
/// quadro do Kanban, a da lista suspensa): as contas são as da barra vertical ([`thumb_rect`],
/// [`delta_for_drag`]), com os eixos trocados quando é horizontal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisBar {
    pub track: Rect,
    pub horizontal: bool,
    /// O comprimento do conteúdo e o da parte à vista, no eixo da barra.
    pub content: f32,
    pub visible: f32,
}

fn swap(r: Rect) -> Rect {
    Rect::new(r.y, r.x, r.h, r.w)
}

impl AxisBar {
    /// A calha como se fosse vertical.
    fn v(&self) -> Rect {
        if self.horizontal {
            swap(self.track)
        } else {
            self.track
        }
    }

    fn along(&self, (x, y): (f32, f32)) -> f32 {
        if self.horizontal { x } else { y }
    }

    #[must_use]
    pub fn needed(&self) -> bool {
        is_needed(self.content, self.visible) && self.v().h > 0.0
    }

    #[must_use]
    pub fn max(&self) -> f32 {
        (self.content - self.visible).max(0.0)
    }

    #[must_use]
    pub fn thumb(&self, scroll: f32) -> Rect {
        let t = thumb_rect(self.v(), scroll, self.content, self.visible);
        if self.horizontal { swap(t) } else { t }
    }

    /// Carregar em `p`: no polegar a rolagem fica; na calha o polegar centra-se lá. A rolagem nova.
    #[must_use]
    pub fn press(&self, p: (f32, f32), scroll: f32) -> f32 {
        let (v, a) = (self.v(), self.along(p));
        let thumb = thumb_rect(v, scroll, self.content, self.visible);
        if (thumb.y..thumb.y + thumb.h).contains(&a) {
            return scroll;
        }
        ((a - v.y) / v.h.max(1.0) * self.content - self.visible / 2.0).clamp(0.0, self.max())
    }

    /// O arrasto do polegar agarrado em `from` (com a rolagem `s0` de então) até `to`.
    #[must_use]
    pub fn drag(&self, from: (f32, f32), s0: f32, to: (f32, f32)) -> f32 {
        let d = self.along(to) - self.along(from);
        let d = delta_for_drag(d, self.v().h, self.content, self.visible);
        (s0 + d).clamp(0.0, self.max())
    }

    /// Pinta a calha e o polegar nas cores `(calha, polegar)` (quem a usa sabe sobre que fundo está).
    pub fn paint(&self, scroll: f32, (track, thumb): (Color, Color), scene: &mut VectorScene) {
        if !self.needed() {
            return;
        }
        let r = self.track.w.min(self.track.h) / 2.0;
        fill_rounded_rect(scene, self.track, r, track);
        fill_rounded_rect(scene, self.thumb(scroll), r, thumb);
    }
}

/// A rolagem, o arrasto do polegar (`y` do carregar, rolagem de então) e a geometria do último
/// quadro (a vista, a calha e a altura do conteúdo).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScrollList {
    pub scroll: f32,
    pub grab: Option<(f32, f32)>,
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
        self.scroll = self.scroll.clamp(0.0, self.max());
    }

    /// Rola `dy` px (positivo = ver mais abaixo).
    pub fn scroll_by(&mut self, dy: f32) {
        self.scroll = (self.scroll + dy).clamp(0.0, self.max());
    }

    fn bar(&self, track: Rect) -> AxisBar {
        AxisBar {
            track,
            horizontal: false,
            content: self.content_h,
            visible: self.view.h,
        }
    }

    /// Carregar na barra: no polegar agarra-o; na calha salta o polegar para lá, e agarra.
    pub fn press_bar(&mut self, y: f32) {
        let Some(track) = self.track else {
            return;
        };
        self.scroll = self.bar(track).press((track.x, y), self.scroll);
        self.grab = Some((y, self.scroll));
    }

    /// O arrasto do polegar até `y`.
    pub fn drag_bar(&mut self, y: f32) {
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
        self.scroll = self.scroll.clamp(0.0, self.max());
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
        let vis = (state, 1.0);
        let drawn = paint_scrollbar(
            body,
            self.scroll,
            self.content_h,
            self.view.h,
            vis,
            scene,
            theme,
        );
        self.track = drawn.map(|_| track_rect(body));
        self.track
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A barra horizontal é a vertical com os eixos trocados: o polegar anda em `x`.
    #[test]
    fn a_horizontal_bar_is_the_vertical_one_on_its_side() {
        let b = AxisBar {
            track: Rect::new(10.0, 100.0, 400.0, 6.0),
            horizontal: true,
            content: 2000.0,
            visible: 500.0,
        };
        let t0 = b.thumb(0.0);
        assert_eq!((t0.x, t0.y, t0.h), (10.0, 100.0, 6.0));
        assert_eq!(b.thumb(b.max()).x + b.thumb(b.max()).w, 410.0);
        assert_eq!(
            b.press((t0.x + 1.0, 103.0), 0.0),
            0.0,
            "no polegar a rolagem fica"
        );
        assert_eq!(
            b.press((410.0, 103.0), 0.0),
            b.max(),
            "o fim da calha é o fim"
        );
        let s = b.drag((20.0, 0.0), 0.0, (120.0, 50.0));
        assert!(s > 0.0 && s < b.max(), "só o x conta: {s}");
    }
}
