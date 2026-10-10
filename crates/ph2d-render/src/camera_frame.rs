//! ⭐ **ENQUADRAR** (F8) — a conta «caixa → câmera» e o VOO até lá.
//!
//! A lei, os oráculos (tldraw 5.5.2 e Excalidraw `next`, corridos sobre cenas nossas) e as recusas
//! vivem em `docs/UI_New_and_Simple/vetor_nova_ui/11_enquadrar_tudo.md`. Aqui fica o que é puro:
//!
//! - [`frame`]: a caixa cabe na ÁREA visível (não na janela: os painéis docados tapam parte dela),
//!   com a margem dada em px de janela, centrada na área; o zoom é cravado pela lei da câmera
//!   ([`Camera2d::clamp_height`] — perguntada, nunca copiada). Os dois oráculos são esta mesma conta
//!   com outros parâmetros ([`frame_within`]; o gate reproduz os números deles).
//! - [`along`]: o percurso entre duas vistas — o zoom em `ln(h)` e o centro pelo PONTO FIXO (o ponto
//!   do mundo que está no mesmo sítio do ecrã nas duas vistas fica parado durante o voo inteiro).
//! - [`ViewFlight`]: o voo por uma mola que o chamador anda (`Role::Viewpoint`), a testemunha das
//!   escritas estrangeiras e a memória da vista anterior (o beliscão rápido AO CONTRÁRIO do Procreate).

use crate::{Camera2d, WorldBbox};
use ph2d_host::WindowSize;

/// Um ponto de vista da câmera 2D — o que a [`Camera2d`] guarda e um enquadramento devolve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub center: [f32; 2],
    pub height_world: f32,
}

/// Um rectângulo em px da JANELA da câmera (y para baixo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Camera2d {
    /// A vista de agora.
    #[must_use]
    pub fn view(&self) -> View {
        View {
            center: self.center,
            height_world: self.height_world,
        }
    }

    /// Põe a vista (o `cull_mask` não é da vista e fica).
    pub fn set_view(&mut self, v: View) {
        self.center = v.center;
        self.height_world = v.height_world;
    }

    /// **O intervalo do zoom** — `height_world` cravado em `[ZOOM_MIN, ZOOM_MAX]`. A roda
    /// ([`Camera2d::zoomed`]) e o enquadramento perguntam aqui.
    #[must_use]
    pub fn clamp_height(h: f32) -> f32 {
        h.clamp(Self::ZOOM_MIN_HEIGHT_WORLD, Self::ZOOM_MAX_HEIGHT_WORLD)
    }
}

/// A caixa que cobre as duas.
#[must_use]
pub fn union(a: Option<WorldBbox>, b: WorldBbox) -> WorldBbox {
    match a {
        None => b,
        Some(a) => WorldBbox {
            min: [a.min[0].min(b.min[0]), a.min[1].min(b.min[1])],
            max: [a.max[0].max(b.max[0]), a.max[1].max(b.max[1])],
        },
    }
}

/// **A LEI do enquadramento**, com o intervalo de `height_world` dado.
///
/// O eixo mais apertado manda (o outro fica centrado); uma caixa sem área (um ponto) vai ao zoom
/// MÁXIMO do intervalo — que é o que impede um ponto sozinho de encher a tela; uma caixa maior do
/// que o zoom mínimo alcança fica centrada e transborda (os dois oráculos fazem o mesmo). Uma área
/// mais estreita do que duas margens enquadra sem margem. `None` só para números não finitos.
#[must_use]
pub fn frame_within(
    b: WorldBbox,
    area: Area,
    window: WindowSize,
    margin_px: f32,
    h_range: (f32, f32),
) -> Option<View> {
    let finite = [b.min, b.max].iter().flatten().all(|v| v.is_finite())
        && [area.x, area.y, area.w, area.h, margin_px]
            .iter()
            .all(|v| v.is_finite());
    if !finite || b.max[0] < b.min[0] || b.max[1] < b.min[1] {
        return None;
    }
    let ww = f64::from(window.width.max(1));
    let wh = f64::from(window.height.max(1));
    let (aw, ah) = (f64::from(area.w.max(1.0)), f64::from(area.h.max(1.0)));
    let m = f64::from(margin_px.max(0.0));
    let m = if aw > 2.0 * m && ah > 2.0 * m { m } else { 0.0 };
    let span = [
        f64::from(b.max[0]) - f64::from(b.min[0]),
        f64::from(b.max[1]) - f64::from(b.min[1]),
    ];
    // px de janela por unidade de mundo que cabem: o eixo mais apertado manda.
    let k_fit = [(aw - 2.0 * m, span[0]), (ah - 2.0 * m, span[1])]
        .iter()
        .filter(|(_, s)| *s > 0.0)
        .map(|(avail, s)| avail / s)
        .fold(f64::INFINITY, f64::min);
    let h = (wh / k_fit).clamp(f64::from(h_range.0), f64::from(h_range.1));
    let k = wh / h;
    let c = [
        (f64::from(b.min[0]) + f64::from(b.max[0])) * 0.5,
        (f64::from(b.min[1]) + f64::from(b.max[1])) * 0.5,
    ];
    let a = [f64::from(area.x) + aw * 0.5, f64::from(area.y) + ah * 0.5];
    // O centro da ÁREA cai no centro da caixa (`screen_to_world(a) == c`; o mundo é y para cima).
    #[allow(clippy::cast_possible_truncation)]
    Some(View {
        center: [
            (c[0] - (a[0] - ww * 0.5) / k) as f32,
            (c[1] + (a[1] - wh * 0.5) / k) as f32,
        ],
        height_world: h as f32,
    })
}

/// **Enquadrar** com o intervalo da câmera.
#[must_use]
pub fn frame(b: WorldBbox, area: Area, window: WindowSize, margin_px: f32) -> Option<View> {
    let range = (
        Camera2d::ZOOM_MIN_HEIGHT_WORLD,
        Camera2d::ZOOM_MAX_HEIGHT_WORLD,
    );
    frame_within(b, area, window, margin_px, range)
}

/// **O percurso** de `from` a `to` em `s ∈ [0, 1]`: `ln(h)` linear em `s` (o zoom percebido anda a
/// ritmo igual nos dois sentidos — a lei do `canvas_zoom`) e o centro pelo ponto fixo
/// `c(s) = c₀ + (c₁ − c₀)·(1 − rˢ)/(1 − r)`, `r = h₁/h₀` — que vira a recta quando `r → 1`.
#[must_use]
pub fn along(from: View, to: View, s: f32) -> View {
    if s >= 1.0 {
        return to;
    }
    if s <= 0.0 {
        return from;
    }
    let (h0, h1, s) = (
        f64::from(from.height_world),
        f64::from(to.height_world),
        f64::from(s),
    );
    let lr = (h1 / h0).ln();
    let rs = (lr * s).exp();
    let g = if lr.abs() < 1e-9 {
        s
    } else {
        (1.0 - rs) / (1.0 - lr.exp())
    };
    let c = |i: usize| {
        f64::from(from.center[i]) + (f64::from(to.center[i]) - f64::from(from.center[i])) * g
    };
    #[allow(clippy::cast_possible_truncation)]
    View {
        center: [c(0) as f32, c(1) as f32],
        height_world: (h0 * rs) as f32,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Leg {
    /// `None` até ao 1.º tique: o voo parte da câmera COMO ELA ESTÁ quando ele começa a andar (um
    /// pedido logo a seguir à roda não é desfeito pelo resto da mola dela).
    from: Option<View>,
    to: View,
    /// O valor da mola quando a perna partiu: `s = mola − base`, o destino é `base + 1`.
    base: f32,
    /// A mola não existia ⇒ o 1.º tique planta-a na `base` (a primeira vista de um id CHEGA ao alvo).
    seed: bool,
    /// Ao assentar, lembra onde (a memória do beliscão ao contrário).
    remember: bool,
}

/// ⭐ **O VOO da câmera** até um enquadramento.
///
/// ⚠️ Não conhece a mola: o tique recebe `step(goal) -> valor` (a shell anda a `Role::Viewpoint` do
/// `UiMotion`), e por isso a lei inteira é testável sem relógio. Uma perna nova parte do valor em
/// que a mola está ⇒ a velocidade de um voo interrompido continua no seguinte.
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewFlight {
    leg: Option<Leg>,
    /// O que este voo publicou por último — a TESTEMUNHA: a câmera diferente disto é uma escrita
    /// estrangeira (roda, pan, dedo, load), e ela ganha.
    published: Option<View>,
    /// A vista de antes do último «enquadrar tudo».
    back: Option<View>,
    /// Onde esse enquadramento assentou.
    landed: Option<View>,
}

impl ViewFlight {
    /// Voa para `to`. `track` = o valor actual da mola (`None` = ela não existe).
    pub fn go(&mut self, to: View, track: Option<f32>) {
        self.start(to, track, false);
    }

    /// Voa para um «enquadrar tudo» e LEMBRA a vista de partida.
    pub fn go_remembering(&mut self, to: View, track: Option<f32>) {
        self.back = None;
        self.landed = None;
        self.start(to, track, true);
    }

    /// O beliscão rápido ao contrário: volta à vista de antes do último «enquadrar tudo». `false`
    /// sem memória armada (o gesto fica só a pinça que já foi).
    pub fn go_back(&mut self, track: Option<f32>) -> bool {
        let Some(back) = self.back.take() else {
            return false;
        };
        self.landed = None;
        self.start(back, track, false);
        true
    }

    /// Um gesto de navegação COMEÇA: a memória só vale se a câmera ainda está onde o enquadramento
    /// a deixou (o Procreate: *«the exact view you had before your Quick-Pinch»*).
    pub fn gesture_starts(&mut self, live: View) {
        if self.landed != Some(live) {
            self.back = None;
            self.landed = None;
        }
    }

    fn start(&mut self, to: View, track: Option<f32>, remember: bool) {
        self.leg = Some(Leg {
            from: None,
            to,
            base: track.unwrap_or(0.0),
            seed: track.is_none(),
            remember,
        });
        self.published = None;
    }

    /// Em voo?
    #[must_use]
    pub fn flying(&self) -> bool {
        self.leg.is_some()
    }

    /// **O tique do quadro.** Devolve a vista a publicar, ou `None` quando a câmera não é do voo.
    pub fn tick(&mut self, live: View, mut step: impl FnMut(f32) -> f32) -> Option<View> {
        let mut leg = self.leg?;
        let from = match leg.from {
            None => {
                if leg.remember {
                    self.back = Some(live);
                }
                leg.from = Some(live);
                live
            }
            Some(f) if self.published == Some(live) => f,
            Some(_) => {
                self.leg = None;
                self.published = None;
                return None;
            }
        };
        if leg.seed {
            step(leg.base);
            leg.seed = false;
        }
        let goal = leg.base + 1.0;
        let now = step(goal);
        let v = if now == goal {
            // Assentou: publica o destino VERBATIM (a câmera assentada não carrega resíduo do voo).
            self.leg = None;
            if leg.remember {
                self.landed = Some(leg.to);
            }
            leg.to
        } else {
            self.leg = Some(leg);
            along(from, leg.to, now - leg.base)
        };
        self.published = self.leg.map(|_| v);
        Some(v)
    }
}

#[cfg(test)]
#[path = "camera_frame_tests.rs"]
mod tests;
