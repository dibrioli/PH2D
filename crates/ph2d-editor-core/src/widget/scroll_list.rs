//! ⭐ **A rolagem de uma lista que abre por cima de tudo** (a lista suspensa de um campo do Kanban,
//! D14): a roda, o polegar arrastado (também fora da janela), carregar na calha e trazer uma linha à
//! vista — sem `WidgetStore`, para quem tem os próprios alvos. A barra é uma [`AxisBar`] com setas.

use super::scrollbar::{
    ScrollbarState, delta_for_drag, is_needed, thumb_color, thumb_rect, track_rect,
};
use crate::interaction::drag::{STEPPER_HOLD_INITIAL_DELAY_NS, STEPPER_REPEAT_INTERVAL_NS};
use crate::math::safe_clamp;
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

/// ⭐ **Segurar = repetir** (D17, o idioma do Chrome e do Trello): o carregar já deu um passo; depois
/// do atraso, um passo a cada intervalo, até largar. Os tempos são a lei de segurar do app — os da
/// seta da caixa numérica ([`STEPPER_HOLD_INITIAL_DELAY_NS`], [`STEPPER_REPEAT_INTERVAL_NS`]); o
/// atraso de 250 ms é também o do Chromium nas setas da barra. O relógio é injectado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoldRepeat {
    since: u128,
    fired: u128,
}

impl HoldRepeat {
    /// Armada no carregar, em `now_ns`.
    #[must_use]
    pub const fn new(now_ns: u128) -> Self {
        Self {
            since: now_ns,
            fired: 0,
        }
    }

    /// Os passos vencidos até `now_ns` e ainda não dados. `on = false` (o rato saiu do alvo) deita-os
    /// fora: o relógio continua e a repetição retoma quando o rato volta, como no Chrome.
    pub fn due(&mut self, now_ns: u128, on: bool) -> u32 {
        let t = now_ns.saturating_sub(self.since);
        let total = t
            .checked_sub(STEPPER_HOLD_INITIAL_DELAY_NS)
            .map_or(0, |r| 1 + r / STEPPER_REPEAT_INTERVAL_NS);
        let n = total.saturating_sub(self.fired);
        self.fired = self.fired.max(total);
        if on {
            u32::try_from(n).unwrap_or(u32::MAX)
        } else {
            0
        }
    }
}

/// Uma seta da [`AxisBar`] segura: a direcção, onde o rato está agora e o relógio da repetição.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArrowHold {
    pub dir: f32,
    pub at: (f32, f32),
    pub rep: HoldRepeat,
}

/// ⭐ Uma [`AxisBar`] agarrada (a das listas do Kanban e a das folhas dele): onde carregou, a
/// rolagem de então e a seta segura (com ela o arrasto só move o rato; segurar repete, D17).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarGrab {
    pub from: (f32, f32),
    pub s0: f32,
    pub hold: Option<ArrowHold>,
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
    /// ⭐ A margem própria de uma barra no FIM de `area`, de través ao eixo (D17: a da lista à
    /// direita, a do quadro em baixo): `(margem, calha)`, a calha de espessura `bar` centrada na
    /// margem de `gutter`. A horizontal é a vertical deitada — as mesmas contas, eixos trocados.
    #[must_use]
    pub fn gutter(area: Rect, horizontal: bool, gutter: f32, bar: f32) -> (Rect, Rect) {
        let v = if horizontal { swap(area) } else { area };
        let band = Rect::new(v.x + v.w - gutter, v.y, gutter, v.h);
        let track = Rect::new(band.x + (gutter - bar) / 2.0, v.y, bar, v.h);
        if horizontal {
            (swap(band), swap(track))
        } else {
            (band, track)
        }
    }

    /// ⭐ A barra com setas na margem própria de `area` ([`Self::gutter`]) que rola `content` com
    /// `visible` à vista, `step` por seta: `(margem, barra)`. A margem é o alvo inteiro.
    #[must_use]
    pub fn in_gutter(
        area: Rect,
        horizontal: bool,
        (gutter, bar): (f32, f32),
        (content, visible, step): (f32, f32, f32),
    ) -> (Rect, Self) {
        let (band, track) = Self::gutter(area, horizontal, gutter, bar);
        let b = Self {
            track,
            horizontal,
            content,
            visible,
            arrows: true,
            step,
        };
        (band, b)
    }

    /// Carregar em `p` em `now_ns` ([`Self::press_held`]): a rolagem nova e a barra agarrada.
    #[must_use]
    pub fn grab(&self, p: (f32, f32), scroll: f32, now_ns: u128) -> (f32, BarGrab) {
        let (s0, hold) = self.press_held(p, scroll, now_ns);
        (s0, BarGrab { from: p, s0, hold })
    }

    /// Cada quadro com a barra agarrada: a seta segura repete ([`Self::repeat`]). `None` = não há
    /// seta segura (a rolagem fica e o chamador não precisa de pedir quadros).
    #[must_use]
    pub fn tick(&self, g: &mut BarGrab, scroll: f32, now_ns: u128) -> Option<f32> {
        let h = g.hold.as_mut()?;
        Some(self.repeat(h, scroll, now_ns))
    }

    /// O rato em `p` com a barra agarrada: com a seta segura só o sítio dele muda; senão o polegar
    /// segue-o ([`Self::drag`]). A rolagem nova.
    #[must_use]
    pub fn follow(&self, g: &mut BarGrab, p: (f32, f32), scroll: f32) -> f32 {
        if let Some(h) = g.hold.as_mut() {
            h.at = p;
            return scroll;
        }
        self.drag(g.from, g.s0, p)
    }

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
            return safe_clamp(scroll + dir * self.step, 0.0, self.max());
        }
        let (v, a) = (self.v(), self.along(p));
        let thumb = thumb_rect(v, scroll, self.content, self.visible);
        if (thumb.y..thumb.y + thumb.h).contains(&a) {
            return scroll;
        }
        safe_clamp(
            (a - v.y) / v.h.max(1.0) * self.content - self.visible / 2.0,
            0.0,
            self.max(),
        )
    }

    /// [`Self::press`] em `now_ns`, e a seta segura se carregou numa.
    #[must_use]
    pub fn press_held(&self, p: (f32, f32), scroll: f32, now_ns: u128) -> (f32, Option<ArrowHold>) {
        let hold = self.arrow_at(p).map(|dir| ArrowHold {
            dir,
            at: p,
            rep: HoldRepeat::new(now_ns),
        });
        (self.press(p, scroll), hold)
    }

    /// A seta segura em `now_ns`: os passos vencidos, só com o rato sobre ela. A rolagem nova.
    #[must_use]
    pub fn repeat(&self, h: &mut ArrowHold, scroll: f32, now_ns: u128) -> f32 {
        let n = h.rep.due(now_ns, self.arrow_at(h.at) == Some(h.dir));
        let n = f32::from(u16::try_from(n).unwrap_or(u16::MAX));
        safe_clamp(scroll + h.dir * self.step * n, 0.0, self.max())
    }

    /// O arrasto do polegar agarrado em `from` (com a rolagem `s0` de então) até `to`.
    #[must_use]
    pub fn drag(&self, from: (f32, f32), s0: f32, to: (f32, f32)) -> f32 {
        let d = self.along(to) - self.along(from);
        let d = delta_for_drag(d, self.v().h, self.content, self.visible);
        safe_clamp(s0 + d, 0.0, self.max())
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

    /// D17 — a margem horizontal é a vertical deitada: a mesma espessura, as mesmas setas e o
    /// mesmo polegar, transpostos; a margem fica no fim de través (em baixo / à direita).
    #[test]
    fn a_horizontal_gutter_is_the_vertical_one_transposed() {
        let area = Rect::new(12.0, 40.0, 600.0, 300.0);
        let (vb, vt) = AxisBar::gutter(area, false, 23.0, 9.0);
        let (hb, ht) = AxisBar::gutter(swap(area), true, 23.0, 9.0);
        assert_eq!((hb, ht), (swap(vb), swap(vt)));
        assert_eq!(
            (vt.x, vt.w),
            (12.0 + 600.0 - 23.0 + 7.0, 9.0),
            "7 + 9 + 7 à direita"
        );
        let (b, t) = AxisBar::gutter(area, true, 23.0, 9.0);
        assert_eq!(
            (b.y, b.h, b.x, b.w),
            (340.0 - 23.0, 23.0, 12.0, 600.0),
            "em baixo"
        );
        assert_eq!((t.y, t.h), (340.0 - 16.0, 9.0));
        let bar = |track, horizontal| AxisBar {
            track,
            horizontal,
            content: 2000.0,
            visible: 300.0,
            arrows: true,
            step: 40.0,
        };
        let (v, h) = (bar(vt, false), bar(ht, true));
        for s in [0.0, 300.0, v.max()] {
            assert_eq!(h.thumb(s), swap(v.thumb(s)));
        }
        assert_eq!(
            h.arrow_tris(),
            v.arrow_tris().map(|t| t.map(|(x, y)| (y, x)))
        );
        let end = (ht.x + ht.w - 1.0, ht.y + 4.0);
        assert_eq!(h.arrow_at(end), Some(1.0));
        assert_eq!(h.press(end, 0.0), 40.0, "a seta dá um passo");
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

    const D: u128 = STEPPER_HOLD_INITIAL_DELAY_NS;
    const I: u128 = STEPPER_REPEAT_INTERVAL_NS;

    /// D17 — a lei de segurar: nada antes do atraso; no atraso o 1.º passo, depois um a cada
    /// intervalo; um quadro atrasado recebe todos os vencidos; fora do alvo deita-os fora e retoma.
    #[test]
    fn holding_repeats_after_the_delay_at_the_interval_and_pauses_off_target() {
        let mut h = HoldRepeat::new(1_000);
        assert_eq!(h.due(1_000 + D - 1, true), 0, "nada antes do atraso");
        assert_eq!(h.due(1_000 + D, true), 1, "o 1.º passo no atraso");
        assert_eq!(h.due(1_000 + D + I - 1, true), 0);
        assert_eq!(h.due(1_000 + D + 3 * I, true), 3, "os vencidos de uma vez");
        assert_eq!(h.due(1_000 + D + 5 * I, false), 0, "fora da seta, nada");
        assert_eq!(h.due(1_000 + D + 5 * I, true), 0, "e os de fora não voltam");
        assert_eq!(h.due(1_000 + D + 6 * I, true), 1, "de volta à seta, retoma");
    }

    /// A barra agarrada (a das folhas do Kanban): segurar ▼ repete até ao fim e pára lá; largar
    /// (largar a `BarGrab`) acaba; sem seta, `tick` nada pede; o polegar segue o rato como o
    /// arrasto da barra das listas ([`AxisBar::drag`]).
    #[test]
    fn a_grabbed_bar_holds_to_the_end_and_drags_like_the_list_bar() {
        let area = Rect::new(0.0, 0.0, 300.0, 200.0);
        let (band, b) = AxisBar::in_gutter(area, false, (23.0, 9.0), (900.0, 200.0, 40.0));
        assert_eq!((band.x, band.w), (277.0, 23.0), "a margem à direita");
        let down = (b.track.x + 4.0, b.track.y + b.track.h - 1.0);
        let (s, mut g) = b.grab(down, 0.0, 0);
        assert_eq!(s, 40.0, "o carregar é um passo");
        assert_eq!(b.tick(&mut g, s, D - 1), Some(40.0), "nada antes do atraso");
        assert_eq!(b.tick(&mut g, s, D + 2 * I), Some(160.0), "1 + 3 passos");
        assert_eq!(
            b.tick(&mut g, 160.0, D + 1_000 * I),
            Some(b.max()),
            "pára no fim"
        );
        assert_eq!(b.tick(&mut g, b.max(), D + 2_000 * I), Some(b.max()));
        let mid = (down.0, b.track.y + b.track.h / 2.0);
        assert_eq!(
            b.follow(&mut g, mid, 300.0),
            300.0,
            "com a seta, o rato só se move"
        );
        assert_eq!(
            b.tick(&mut g, 300.0, D + 3_000 * I),
            Some(300.0),
            "fora da seta, pausa"
        );
        let t = b.thumb(0.0);
        let at = (t.x + 1.0, t.y + 2.0);
        let (s0, mut g) = b.grab(at, 0.0, 0);
        assert_eq!(
            (s0, b.tick(&mut g, s0, D + I)),
            (0.0, None),
            "o polegar não repete"
        );
        for dy in [10.0, 57.0, 400.0] {
            let to = (at.0 + 30.0, at.1 + dy);
            assert_eq!(b.follow(&mut g, to, s0), b.drag(at, 0.0, to));
        }
    }

    fn list_with_arrows() -> (ScrollList, Rect) {
        let mut l = ScrollList::default();
        let body = Rect::new(0.0, 0.0, 200.0, 4.0 * ROW_H_PX);
        l.set(body, 20.0 * ROW_H_PX);
        l.track = Some(track_rect(body));
        let t = l.track.unwrap();
        (l, t)
    }

    /// D17 — a lista suspensa: segurar ▼ dá um passo no carregar, repete pela lei e pára ao largar;
    /// sair da seta pausa; o polegar não fica agarrado.
    #[test]
    fn holding_the_dropdown_arrow_keeps_scrolling_until_release() {
        let (mut l, t) = list_with_arrows();
        let down = t.y + t.h - 1.0;
        l.press_bar(down, 0);
        assert_eq!(l.scroll, ROW_H_PX, "o carregar é um passo");
        assert!(l.grab.is_none(), "a seta não agarra o polegar");
        assert!(l.tick(D - 1));
        assert_eq!(l.scroll, ROW_H_PX, "nada antes do atraso");
        assert!(l.tick(D + 2 * I));
        assert_eq!(l.scroll, 4.0 * ROW_H_PX, "1 + 3 passos");
        l.drag_bar(t.y + t.h / 2.0);
        assert!(l.tick(D + 4 * I));
        assert_eq!(l.scroll, 4.0 * ROW_H_PX, "fora da seta, pausa");
        l.drag_bar(down);
        assert!(l.tick(D + 5 * I));
        assert_eq!(l.scroll, 5.0 * ROW_H_PX, "de volta, retoma");
        l.release();
        assert!(!l.tick(D + 50 * I), "largou: não pede mais quadros");
        assert_eq!(l.scroll, 5.0 * ROW_H_PX);
        l.press_bar(down, 0);
        assert!(l.tick(D + 1_000 * I));
        assert_eq!(l.scroll, l.max(), "pára no fim");
    }
}
