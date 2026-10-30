//! ⭐ **A rolagem de uma lista que abre por cima de tudo** (a lista suspensa de um campo do Kanban,
//! D14): a roda, o polegar arrastado (também fora da janela), carregar na calha e trazer uma linha à
//! vista — sem `WidgetStore`, para quem tem os próprios alvos. A barra é a [`super::paint_scrollbar`].

use super::scrollbar::{ScrollbarState, delta_for_drag, paint_scrollbar, thumb_rect, track_rect};
use crate::zones::Rect;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

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

    /// Carregar na barra: no polegar agarra-o; na calha salta o polegar para lá, e agarra.
    pub fn press_bar(&mut self, y: f32) {
        let Some(track) = self.track else {
            return;
        };
        let thumb = thumb_rect(track, self.scroll, self.content_h, self.view.h);
        if !(thumb.y..thumb.y + thumb.h).contains(&y) {
            let at = (y - track.y) / track.h.max(1.0) * self.content_h - self.view.h / 2.0;
            self.scroll = at.clamp(0.0, self.max());
        }
        self.grab = Some((y, self.scroll));
    }

    /// O arrasto do polegar até `y`.
    pub fn drag_bar(&mut self, y: f32) {
        let (Some((y0, s0)), Some(track)) = (self.grab, self.track) else {
            return;
        };
        let d = delta_for_drag(y - y0, track.h, self.content_h, self.view.h);
        self.scroll = (s0 + d).clamp(0.0, self.max());
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
