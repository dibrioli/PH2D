//! [`Slider`] — continuous numeric input (0..=1 normalized).
//!
//! Procreate uses sliders for brush size, opacity, flow. Same pattern
//! as [`crate::widget::Button`]: data + state enum + token-resolved
//! colors + AccessKit `Role::Slider` node + `paint_slider` colocated.
//! Supports horizontal and vertical orientation and an optional set
//! of tick positions for snap-to-grid affordance.

use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, Radius, StrokeToken, Theme};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SliderState {
    #[default]
    Normal,
    Hovered,
    Dragging,
    Focused,
    Disabled,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SliderOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub struct Slider {
    pub id: NodeId,
    pub label: String,
    /// Normalized 0..=1; UI maps this to whatever the binding wants.
    pub value: f32,
    pub state: SliderState,
    /// **Quanto do hover está presente**, `0..1`. Neutro = [`crate::motion::SETTLED`] ⇒ toda
    /// construção que não o define pinta **exactamente** o que pintava antes da UI viva.
    pub hover_t: f32,
    pub orientation: SliderOrientation,
    /// True ⇒ filled bar uses `Accent`; false ⇒ `AccentPress` (a dim
    /// default). Distinct from focus highlight.
    pub accent: bool,
    /// Optional snap positions in [0, 1]. Painted as small marks on
    /// the track. Empty means no ticks.
    pub ticks: Vec<f32>,
    /// Espessura da trilha, em px. **`None` é a política de LINHA de um painel** — 25% da
    /// moldura com TETO em [`TRACK_MAX_PX`], que é o que impede uma linha alta de desenhar uma
    /// trilha gorda.
    ///
    /// ⚠️ **O teto é do PAINEL, o piso é do PINTOR.** Uma trilha abaixo de [`TRACK_MIN_PX`] é
    /// invisível seja qual for o chamador, então esse limite é honrado nos dois casos; o teto
    /// não, porque ele descreve uma linha de formulário e não o widget. Irmão exacto do
    /// [`crate::widget::Checkbox::box_px`], com o mesmo consumidor único.
    pub track_px: Option<f32>,
}

impl Slider {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            value: 0.5,
            state: SliderState::Normal,
            hover_t: crate::motion::SETTLED,
            orientation: SliderOrientation::Horizontal,
            accent: false,
            ticks: Vec::new(),
            track_px: None,
        }
    }

    pub fn accent(mut self, yes: bool) -> Self {
        self.accent = yes;
        self
    }

    pub fn state(mut self, state: SliderState) -> Self {
        self.state = state;
        self
    }

    /// **As DUAS metades numa chamada** — o par que a
    /// [`crate::interaction::WidgetStore::slider_visual`] devolve, irmão exacto do
    /// [`super::button::Button::visual`].
    ///
    /// ⚠️ **Existe para ser mais CURTO que a rota errada**, pela razão que o `Button` já registou:
    /// a alternativa é `.state(store.slider(id).map(|(s, _)| s).unwrap_or(Normal))` seguido de um
    /// `.hover_t(..)` que o sítio seguinte esquece — e um slider sem `t` cai no neutro, que
    /// **salta**. Quando a rota certa é a mais curta de escrever, o sítio seguinte nasce certo por
    /// preguiça e não por disciplina.
    #[must_use]
    pub fn visual(self, v: (SliderState, f32)) -> Self {
        self.state(v.0).hover_t(v.1)
    }

    /// Ver [`Self::hover_t`] o campo. Clampa, porque um `t` fora de `0..1` extrapolaria a mistura
    /// para fora dos dois tokens que a nomeiam.
    #[must_use]
    pub fn hover_t(mut self, t: f32) -> Self {
        self.hover_t = t.clamp(0.0, 1.0); // CLAMP-OK: both bounds are literal non-NaN
        self
    }

    pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn ticks(mut self, ticks: impl Into<Vec<f32>>) -> Self {
        self.ticks = ticks.into();
        self
    }

    /// Clamp + assign. Value is held in [0, 1].
    pub fn set_value(&mut self, v: f32) {
        self.value = v.clamp(0.0, 1.0);
    }

    /// Build the AccessKit node. Per ADR-0023 §10: `Role::Slider`
    /// with numeric value + min/max so screen readers can announce
    /// "30 percent" without us spelling it out.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Slider)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != SliderState::Disabled)
            .action(Action::Click)
            .numeric_value(self.value as f64)
            .numeric_value_min(0.0)
            .numeric_value_max(1.0)
            .build()
    }
}

/// O token DURO da CALHA por estado — o fundo que sobra atrás do valor.
///
/// ⚠️ `Focused` devolve o mesmo `Bg2` do repouso **de propósito**: o foco já se anuncia pelo anel
/// `BorderEmph` que o [`paint_slider`] traça por cima, e tingir a calha também seria o mesmo facto
/// dito duas vezes — com as duas metades livres para divergir no dia em que uma delas mudar.
fn track_tint(state: SliderState) -> ColorToken {
    match state {
        SliderState::Hovered | SliderState::Dragging => ColorToken::Bg3,
        _ => ColorToken::Bg2,
    }
}

/// O token DURO do PREENCHIMENTO por estado — a parte que mede o valor.
fn fill_tint(state: SliderState) -> ColorToken {
    match state {
        SliderState::Hovered => ColorToken::AccentHover,
        SliderState::Dragging => ColorToken::AccentPress,
        _ => ColorToken::Accent,
    }
}

/// A mistura `repouso → hover` deste canal, **ou `None` quando este estado não é uma quantidade**
/// — a porta do [`crate::motion::hover_axis`], idêntica à do
/// [`super::icon_button`], porque é a mesma pergunta noutro widget.
///
/// ⚠️ **`soft` inclui o `Normal`, e é isso que faz a SAÍDA do hover funcionar.** Se o estado
/// escolhesse sozinho, tirar o rato seria instantâneo: no quadro em que ele sai o `state` já voltou
/// a `Normal` e `track_tint(Normal)` **já É** a cor de repouso, então não sobraria nada entre onde
/// a cor está e onde ela vai.
///
/// ⚠️ **`Dragging` é estado DURO, e aqui isso não é simetria — é o gesto.** Uma trilha agarrada
/// tem de acender no quadro do `Down`; interpolá-la deixaria a superfície a caminho da cor
/// enquanto o dedo já a comanda. `Focused`/`Disabled` são duros pela razão que o botão já
/// registou: não são uma *quantidade* de nada.
fn blend_on_hover_axis(
    state: SliderState,
    t: f32,
    rest: ColorToken,
    hot: ColorToken,
    theme: Theme,
) -> Option<ph2d_vector::Color> {
    crate::motion::hover_axis(
        matches!(state, SliderState::Normal | SliderState::Hovered),
        t,
        Some(rest.resolve(theme)),
        Some(hot.resolve(theme)),
    )
    .map(crate::paint::token_to_vello)
}

/// Canonical slider track: rounded-rect background + a filled portion for the current value.
/// **Single source of truth for the slider look** — both the bare [`paint_slider`] and
/// `widget::slider_with_chip::paint_slider_with_chip` render through this, so every slider in the
/// app shares one rectangular appearance.
///
/// ⚠️ **`visual` é o PAR `(estado, t)` e vive na ASSINATURA, não num campo opcional** — a lei que
/// o [`super::icon_button::paint_icon_button`] instalou. É ela que fecha os cinco chamadores pelo
/// compilador: uma trilha que não quer reagir **declara** o neutro em vez de o herdar por omissão,
/// e a declaração é o sítio onde se lê *porquê*.
///
/// ⚠️ **`(Normal, SETTLED)` é BYTE-IDÊNTICO ao mundo pré-wave** — o `hover_axis` devolve `None`
/// nesse par e a cor sai dos tokens duros `Bg2`/`Accent`, os mesmos literais que aqui estavam.
pub fn paint_slider_track(
    track: Rect,
    value: f32,
    orientation: SliderOrientation,
    visual: (SliderState, f32),
    scene: &mut VectorScene,
    theme: Theme,
) {
    let (state, t) = visual;
    let r = Radius::Xs.px();
    let bg = blend_on_hover_axis(state, t, ColorToken::Bg2, ColorToken::Bg3, theme)
        .unwrap_or_else(|| resolve(track_tint(state), theme));
    fill_rounded_rect(scene, track, r, bg);
    let v = value.clamp(0.0, 1.0);
    if v > 0.0 {
        let filled = match orientation {
            SliderOrientation::Horizontal => Rect::new(track.x, track.y, track.w * v, track.h),
            SliderOrientation::Vertical => {
                let h = track.h * v;
                Rect::new(track.x, track.y + track.h - h, track.w, h)
            }
        };
        let fg = blend_on_hover_axis(state, t, ColorToken::Accent, ColorToken::AccentHover, theme)
            .unwrap_or_else(|| resolve(fill_tint(state), theme));
        fill_rounded_rect(scene, filled, r, fg);
    }
}

/// Rectangular track + accent fill (no circular thumb — the filled
/// portion is the value readout, matching `paint_slider_with_chip`).
/// `Focused` adds a `BorderEmph` outline on the track; `Disabled`
/// draws a flat `Border` track.
pub fn paint_slider(slider: &Slider, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let track = track_rect(slider, rect);
    let r = Radius::Xs.px();
    if slider.state == SliderState::Disabled {
        fill_rounded_rect(scene, track, r, resolve(ColorToken::Border, theme));
        return;
    }

    paint_slider_track(
        track,
        slider.value,
        slider.orientation,
        (slider.state, slider.hover_t),
        scene,
        theme,
    );

    for tick in &slider.ticks {
        let pos = tick.clamp(0.0, 1.0);
        let mark = tick_mark_rect(slider.orientation, rect, pos);
        fill_rounded_rect(scene, mark, r, resolve(ColorToken::Text3, theme));
    }

    if slider.state == SliderState::Focused {
        stroke_rounded_rect(
            scene,
            track,
            r,
            StrokeToken::Default.px(),
            resolve(ColorToken::BorderEmph, theme),
        );
    }
}

/// Trilha mais fina que isto é invisível — piso de LEGIBILIDADE, honrado por todo chamador.
pub const TRACK_MIN_PX: f32 = 2.0; // LITERAL-PX-OK: slider track legibility floor

/// O teto da política de LINHA: uma linha alta de painel não desenha uma trilha gorda.
/// ⚠️ Não vale para quem informa a espessura (`Slider::track_px`) — ver o campo.
pub const TRACK_MAX_PX: f32 = 8.0; // LITERAL-PX-OK: slider track ceiling inside a panel row

/// A espessura ao longo do eixo CURTO da trilha. `across` é a medida da moldura nesse eixo.
fn track_thickness(track_px: Option<f32>, across: f32) -> f32 {
    match track_px {
        // A política de linha: 25% da moldura, com piso E teto.
        None => (across * 0.25).clamp(TRACK_MIN_PX, TRACK_MAX_PX), // LITERAL-PX-OK: 25% ratio; CLAMP-OK: both bounds are literal non-NaN consts
        // Quem informa a espessura já a mediu contra a própria moldura; só o piso sobrevive.
        Some(px) => px.max(TRACK_MIN_PX),
    }
}

fn track_rect(slider: &Slider, rect: Rect) -> Rect {
    match slider.orientation {
        SliderOrientation::Horizontal => {
            let h = track_thickness(slider.track_px, rect.h);
            let y = rect.y + (rect.h - h) / 2.0;
            Rect::new(rect.x, y, rect.w, h)
        }
        SliderOrientation::Vertical => {
            let w = track_thickness(slider.track_px, rect.w);
            let x = rect.x + (rect.w - w) / 2.0;
            Rect::new(x, rect.y, w, rect.h)
        }
    }
}

fn tick_mark_rect(orientation: SliderOrientation, rect: Rect, value: f32) -> Rect {
    match orientation {
        SliderOrientation::Horizontal => {
            let x = rect.x + rect.w * value - 1.0;
            let h = (rect.h * 0.5).max(4.0); // LITERAL-PX-OK: tick mark minimum height
            let y = rect.y + (rect.h - h) / 2.0;
            Rect::new(x, y, 2.0, h)
        }
        SliderOrientation::Vertical => {
            let y = rect.y + rect.h - rect.h * value - 1.0;
            let w = (rect.w * 0.5).max(4.0); // LITERAL-PX-OK: tick mark minimum width
            let x = rect.x + (rect.w - w) / 2.0;
            Rect::new(x, y, w, 2.0)
        }
    }
}

#[cfg(test)]
mod tests;
