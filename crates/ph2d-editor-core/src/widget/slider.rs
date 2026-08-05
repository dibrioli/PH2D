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

/// Canonical slider track: rounded-rect background (`Bg2`) + an
/// `Accent`-filled portion for the current value. **Single source of
/// truth for the slider look** — both the bare [`paint_slider`] and
/// `widget::slider_with_chip::paint_slider_with_chip` render through
/// this, so every slider in the app shares one rectangular appearance.
pub fn paint_slider_track(
    track: Rect,
    value: f32,
    orientation: SliderOrientation,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let r = Radius::Xs.px();
    fill_rounded_rect(scene, track, r, resolve(ColorToken::Bg2, theme));
    let v = value.clamp(0.0, 1.0);
    if v > 0.0 {
        let filled = match orientation {
            SliderOrientation::Horizontal => Rect::new(track.x, track.y, track.w * v, track.h),
            SliderOrientation::Vertical => {
                let h = track.h * v;
                Rect::new(track.x, track.y + track.h - h, track.w, h)
            }
        };
        fill_rounded_rect(scene, filled, r, resolve(ColorToken::Accent, theme));
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

    paint_slider_track(track, slider.value, slider.orientation, scene, theme);

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
mod tests {
    use super::*;

    /// **A rota do PAINEL é a lei que shipava, verbatim** (BUGS_vector #26).
    ///
    /// ⚠️ `None` não é "um default razoável": é a política de LINHA — 25% com piso E teto — e
    /// mudá-la re-dimensionaria todo slider do app. O gate a escreve por extenso, para que
    /// alterá-la exija alterar as duas coisas.
    #[test]
    fn without_an_override_the_track_is_the_panel_law() {
        for across in [0.0, 4.0, 8.0, 28.0, 32.0, 96.0, 400.0] {
            assert_eq!(
                track_thickness(None, across),
                (across * 0.25).clamp(TRACK_MIN_PX, TRACK_MAX_PX), // CLAMP-OK: mirrors the law under test
                "a politica de linha mudou em across={across}"
            );
        }
    }

    /// **O TETO é do painel; o PISO é do pintor.** Quem informa a espessura escapa ao teto de
    /// linha — e não escapa ao piso, porque uma trilha abaixo dele é invisível seja quem for o
    /// chamador.
    #[test]
    fn an_override_escapes_the_ceiling_but_never_the_floor() {
        assert_eq!(track_thickness(Some(40.0), 160.0), 40.0);
        assert!(
            track_thickness(None, 160.0) < 40.0,
            "a fixture nao contem o fenomeno: o teto de linha nao mordeu em across=160"
        );
        assert_eq!(track_thickness(Some(0.0), 160.0), TRACK_MIN_PX);
        assert_eq!(track_thickness(Some(-3.0), 160.0), TRACK_MIN_PX);
    }

    /// **No ponto de operação do painel os dois caminhos COINCIDEM.** É esta igualdade que faz
    /// da pele de canvas uma continuação da lei, e não uma segunda lei.
    #[test]
    fn at_a_panel_row_the_override_and_the_law_agree() {
        let row = ph2d_tokens::ROW_H_PX;
        assert_eq!(
            track_thickness(Some(row * 0.25), row),
            track_thickness(None, row)
        );
    }

    fn fixture() -> Slider {
        Slider::new(NodeId(1), "Opacity")
    }

    #[test]
    fn defaults_match_spec() {
        let s = fixture();
        assert_eq!(s.id, NodeId(1));
        assert_eq!(s.label, "Opacity");
        assert!((s.value - 0.5).abs() < f32::EPSILON);
        assert_eq!(s.state, SliderState::Normal);
        assert_eq!(s.orientation, SliderOrientation::Horizontal);
        assert!(!s.accent);
        assert!(s.ticks.is_empty());
    }

    #[test]
    fn set_value_clamps_below_zero() {
        let mut s = fixture();
        s.set_value(-0.5);
        assert_eq!(s.value, 0.0);
    }

    #[test]
    fn set_value_clamps_above_one() {
        let mut s = fixture();
        s.set_value(1.5);
        assert_eq!(s.value, 1.0);
    }

    #[test]
    fn ticks_setter_round_trips() {
        let s = fixture().ticks(vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        assert_eq!(s.ticks.len(), 5);
    }

    #[test]
    fn a11y_node_has_slider_role_and_value() {
        let s = fixture();
        let node = s.build_a11y(0.0, 0.0, 100.0, 30.0);
        assert_eq!(node.role(), Role::Slider);
        assert_eq!(node.label(), Some("Opacity"));
        assert_eq!(node.numeric_value(), Some(0.5));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(1.0));
    }

    fn smoke(slider: Slider, rect: Rect, theme: Theme) {
        let mut scene = VectorScene::new();
        paint_slider(&slider, rect, &mut scene, theme);
    }

    #[test]
    fn paint_smoke_horizontal_default() {
        smoke(fixture(), Rect::new(0.0, 0.0, 200.0, 24.0), Theme::Forge);
    }

    #[test]
    fn paint_smoke_horizontal_zero() {
        let mut s = fixture();
        s.set_value(0.0);
        smoke(s, Rect::new(0.0, 0.0, 200.0, 24.0), Theme::Forge);
    }

    #[test]
    fn paint_smoke_horizontal_one() {
        let mut s = fixture();
        s.set_value(1.0);
        smoke(s, Rect::new(0.0, 0.0, 200.0, 24.0), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_vertical_half() {
        smoke(
            fixture().orientation(SliderOrientation::Vertical),
            Rect::new(0.0, 0.0, 24.0, 200.0),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_dragging_with_ticks() {
        let s = fixture()
            .accent(true)
            .ticks(vec![0.0, 0.25, 0.5, 0.75, 1.0])
            .state(SliderState::Dragging);
        smoke(s, Rect::new(0.0, 0.0, 200.0, 24.0), Theme::Forge);
    }

    #[test]
    fn paint_smoke_focused_draws_ring() {
        smoke(
            fixture().state(SliderState::Focused),
            Rect::new(0.0, 0.0, 200.0, 24.0),
            Theme::Workshop,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            fixture().state(SliderState::Disabled),
            Rect::new(0.0, 0.0, 200.0, 24.0),
            Theme::Forge,
        );
    }
}
