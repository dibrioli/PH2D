//! ⭐ **A rolagem de uma lista que abre por cima de tudo** (a lista suspensa de um campo do Kanban,
//! D14): a roda, o polegar arrastado (também fora da janela), carregar na calha e trazer uma linha à
//! vista — sem `WidgetStore`, para quem tem os próprios alvos. A barra é uma [`AxisBar`] com setas.

use super::scrollbar::{
    ScrollbarState, delta_for_drag, is_needed, thumb_color, thumb_rect, track_rect,
};
use crate::paint::{fill_rounded_rect, resolve};
use crate::paint_shapes::fill_polygon;
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Theme};
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
    /// Setas ▲/▼ (◀/▶) nas pontas da calha, cada uma rola `step` (D16, a barra do Trello).
    pub arrows: bool,
    pub step: f32,
}

/// O comprimento de cada ponta com seta, por espessura da barra — medido na captura do Trello do dono
/// (`docs/TrelloClone/smoke/D16`): barra de 9 px, seta de 9 × 5, 7 px até ao polegar ⇒ 12 / 9.
const ARROW_CELL: f32 = 12.0 / 9.0; // LITERAL-PX-OK: proporção medida na captura do Trello (D16)
/// A altura do triângulo da seta, por espessura (5 / 9 na mesma captura).
const ARROW_TIP: f32 = 5.0 / 9.0; // LITERAL-PX-OK: proporção medida na captura do Trello (D16)

fn swap(r: Rect) -> Rect {
    Rect::new(r.y, r.x, r.h, r.w)
}

impl AxisBar {
    /// A calha como se fosse vertical.
    fn vt(&self) -> Rect {
        if self.horizontal {
            swap(self.track)
        } else {
            self.track
        }
    }

    /// O comprimento de cada ponta com seta (`0` sem setas).
    fn cell(&self) -> f32 {
        if self.arrows {
            self.vt().w * ARROW_CELL
        } else {
            0.0
        }
    }

    /// Onde o polegar anda, como se fosse vertical: a calha sem as pontas das setas.
    fn v(&self) -> Rect {
        let (t, a) = (self.vt(), self.cell());
        Rect::new(t.x, t.y + a, t.w, (t.h - 2.0 * a).max(0.0))
    }

    /// A seta em `p`: `-1.0` a do começo (▲/◀), `1.0` a do fim, `None` fora delas.
    #[must_use]
    pub fn arrow_at(&self, p: (f32, f32)) -> Option<f32> {
        let (t, a, along) = (self.vt(), self.cell(), self.along(p));
        if a <= 0.0 || !(t.y..t.y + t.h).contains(&along) {
            return None;
        }
        if along < t.y + a {
            Some(-1.0)
        } else if along >= t.y + t.h - a {
            Some(1.0)
        } else {
            None
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

    /// Carregar em `p`: numa seta rola um passo; no polegar a rolagem fica; na calha o polegar
    /// centra-se lá. A rolagem nova.
    #[must_use]
    pub fn press(&self, p: (f32, f32), scroll: f32) -> f32 {
        if let Some(dir) = self.arrow_at(p) {
            return (scroll + dir * self.step).clamp(0.0, self.max());
        }
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

    /// Pinta a calha (entre as setas), o polegar e as setas nas cores `(calha, polegar)` (quem a usa sabe sobre que
    /// fundo está); as setas levam a cor do polegar. Calha transparente = a do Trello (só o polegar).
    pub fn paint(&self, scroll: f32, (track, thumb): (Color, Color), scene: &mut VectorScene) {
        if !self.needed() {
            return;
        }
        let r = self.track.w.min(self.track.h) / 2.0;
        let lane = if self.horizontal {
            swap(self.v())
        } else {
            self.v()
        };
        fill_rounded_rect(scene, lane, r, track);
        let t = self.thumb(scroll);
        fill_rounded_rect(scene, t, r.min(t.w.min(t.h) / 2.0), thumb);
        if self.arrows {
            for tri in self.arrow_tris() {
                fill_polygon(scene, &tri, thumb);
            }
        }
    }

    /// Os dois triângulos das setas, encostados às pontas da calha (a ponta para fora).
    fn arrow_tris(&self) -> [[(f32, f32); 3]; 2] {
        let t = self.vt();
        let (half, tip) = (t.w / 2.0, t.w * ARROW_TIP);
        let cx = t.x + half;
        let (y0, y1) = (t.y, t.y + t.h);
        let up = [(cx, y0), (cx + half, y0 + tip), (cx - half, y0 + tip)];
        let down = [(cx, y1), (cx - half, y1 - tip), (cx + half, y1 - tip)];
        let map = |tri: [(f32, f32); 3]| {
            if self.horizontal {
                tri.map(|(x, y)| (y, x))
            } else {
                tri
            }
        };
        [map(up), map(down)]
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
            arrows: true,
            step: ROW_H_PX,
        }
    }

    /// Carregar na barra: numa seta rola uma linha; no polegar agarra-o; na calha salta o polegar
    /// para lá, e agarra.
    pub fn press_bar(&mut self, y: f32) {
        let Some(track) = self.track else {
            return;
        };
        let bar = self.bar(track);
        self.scroll = bar.press((track.x, y), self.scroll);
        if bar.arrow_at((track.x, y)).is_none() {
            self.grab = Some((y, self.scroll));
        }
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
        let bar = self.bar(track_rect(body));
        let colors = (
            resolve(ColorToken::Bg3, theme),
            thumb_color((state, 1.0), theme),
        );
        bar.paint(self.scroll, colors, scene);
        self.track = bar.needed().then_some(bar.track);
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
            arrows: false,
            step: 0.0,
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

    /// As setas (D16): cada ponta rola `step` e o polegar anda só entre elas.
    #[test]
    fn an_arrow_scrolls_one_step_and_the_thumb_stays_between_them() {
        let b = AxisBar {
            track: Rect::new(0.0, 0.0, 9.0, 300.0),
            horizontal: false,
            content: 2000.0,
            visible: 300.0,
            arrows: true,
            step: 40.0,
        };
        assert_eq!(b.arrow_at((4.0, 2.0)), Some(-1.0));
        assert_eq!(b.arrow_at((4.0, 298.0)), Some(1.0));
        assert_eq!(b.arrow_at((4.0, 150.0)), None);
        assert_eq!(b.press((4.0, 298.0), 100.0), 140.0);
        assert_eq!(b.press((4.0, 2.0), 20.0), 0.0, "não passa do começo");
        assert_eq!(b.press((4.0, 298.0), b.max()), b.max(), "nem do fim");
        assert_eq!(b.thumb(0.0).y, 12.0, "a ponta de 12 / 9 da espessura");
        let t = b.thumb(b.max());
        assert_eq!(t.y + t.h, 288.0);
    }
}
