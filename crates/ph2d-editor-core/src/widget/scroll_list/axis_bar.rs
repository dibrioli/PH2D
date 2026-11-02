//! A [`AxisBar`] (a barra de um eixo, com setas) e a repetição da seta segura — a metade de
//! GEOMETRIA e tinta do [`super`]; a rolagem que ela move é a [`super::ScrollList`].

use super::super::scrollbar::{delta_for_drag, is_needed, thumb_rect};
use crate::interaction::drag::{STEPPER_HOLD_INITIAL_DELAY_NS, STEPPER_REPEAT_INTERVAL_NS};
use crate::math::safe_clamp;
use crate::paint::fill_rounded_rect;
use crate::paint_shapes::fill_polygon;
use crate::zones::Rect;
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

pub(super) fn swap(r: Rect) -> Rect {
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
    pub(super) fn arrow_tris(&self) -> [[(f32, f32); 3]; 2] {
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
