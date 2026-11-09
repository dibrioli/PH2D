//! ⭐ **A barra cujo DONO guarda a rolagem** (fora do `WidgetStore`): a vista do código, os cartões
//! do depurador, a tabela da casa. Pinta-se com o [`paint_scrollbar`](crate::widget::paint_scrollbar)
//! e arrasta-se com a MESMA lei do despachante (`dispatch::scroll::begin_bar_drag`): a trilha, nunca
//! o polegar; carregar fora do polegar salta-o para debaixo do dedo
//! ([`super::scroll_for_track_press`]); o arrasto é proporcional
//! ([`delta_for_drag`](crate::widget::scrollbar_delta_for_drag)) e preso ao fim.

use crate::widget::scrollbar::{delta_for_drag, is_needed, thumb_rect, track_rect};
use crate::zones::Rect;
use ph2d_tokens::Spacing;

/// A geometria de uma barra: o corpo que lhe é dado e as duas alturas (as do `paint_scrollbar`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OwnBar {
    pub body: Rect,
    pub content_h: f32,
    pub visible_h: f32,
}

/// Um arrasto de barra em curso: as medidas fotografadas no Down (o arrasto fica linear mesmo que o
/// pintor republique a meio).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarDrag {
    y0: f32,
    scroll0: f32,
    track_h: f32,
    content_h: f32,
    visible_h: f32,
}

impl OwnBar {
    /// A zona que agarra: a trilha até à borda direita do corpo, nunca mais estreita que
    /// `Spacing::Sm`. `None` = não transborda (não há barra).
    #[must_use]
    pub fn hit(&self) -> Option<Rect> {
        if !is_needed(self.content_h, self.visible_h) {
            return None;
        }
        let t = track_rect(self.body);
        let right = self.body.x + self.body.w;
        let x = t.x.min(right - Spacing::Sm.px());
        Some(Rect::new(x, t.y, right - x, t.h))
    }

    /// Carregar em `(x, y)` com a rolagem `scroll`: fora da zona `None`; dentro, o arrasto e a
    /// rolagem de agora (sobre o polegar a mesma; na trilha, o polegar centrado no dedo).
    #[must_use]
    pub fn press(&self, x: f32, y: f32, scroll: f32) -> Option<(BarDrag, f32)> {
        if !self.hit()?.contains(x, y) {
            return None;
        }
        let track = track_rect(self.body);
        let thumb = thumb_rect(track, scroll, self.content_h, self.visible_h);
        let scroll0 = if y < thumb.y || y > thumb.y + thumb.h {
            super::scroll_for_track_press(track, y, self.content_h, self.visible_h)
        } else {
            scroll.clamp(0.0, (self.content_h - self.visible_h).max(0.0))
        };
        let drag = BarDrag {
            y0: y,
            scroll0,
            track_h: track.h,
            content_h: self.content_h,
            visible_h: self.visible_h,
        };
        Some((drag, scroll0))
    }
}

impl BarDrag {
    /// A rolagem com o ponteiro em `y` (presa entre 0 e o fim).
    #[must_use]
    pub fn scroll_at(&self, y: f32) -> f32 {
        let d = delta_for_drag(y - self.y0, self.track_h, self.content_h, self.visible_h);
        let max = (self.content_h - self.visible_h).max(0.0);
        crate::math::safe_clamp(self.scroll0 + d, 0.0, max)
    }
}
