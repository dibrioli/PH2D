//! [`Toggle`] — binary on/off switch (snap-to-grid, lock-axis, etc).
//!
//! Same pattern as [`crate::widget::Button`]: data + state enum +
//! token-resolved colors + AccessKit `Role::Switch` node + paint
//! helper colocated. Maps the boolean `on` to `Toggled::True/False`
//! so screen readers announce "on" / "off".

// ⚠️ A tinta deste widget mudou-se para o pintor da caixa de verificação em 2026-09-03 (ver
// [`paint_toggle`]), então o que sobra aqui é o MODELO: estado, `on`, e o nó de acessibilidade.
use crate::widget::checkbox::{CheckboxState, CheckboxValue};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role, Toggled};
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ToggleState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct Toggle {
    pub id: NodeId,
    pub label: String,
    pub on: bool,
    pub state: ToggleState,
    /// Quanto do hover está PRESENTE (`0`..=`1`); [`crate::motion::SETTLED`] = assente no estado.
    pub hover_t: f32,
}

impl Toggle {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            on: false,
            state: ToggleState::Normal,
            hover_t: crate::motion::SETTLED,
        }
    }

    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    /// **O par visual do store numa chamada** — o irmão exacto do [`super::Button::visual`]; o
    /// `.on(..)` continua a vir do MODELO.
    #[must_use]
    pub fn visual(self, v: (ToggleState, f32)) -> Self {
        self.state(v.0).hover_t(v.1)
    }

    /// Quanto do hover está presente. O neutro [`crate::motion::SETTLED`] pinta o traço DURO.
    #[must_use]
    pub fn hover_t(mut self, t: f32) -> Self {
        self.hover_t = t.clamp(0.0, 1.0);
        self
    }

    pub fn state(mut self, state: ToggleState) -> Self {
        self.state = state;
        self
    }

    /// Flip the current state. Convenience for click handlers that
    /// don't know or care about the prior value.
    pub fn toggle(&mut self) {
        self.on = !self.on;
    }

    /// Build the AccessKit node. `Role::Switch` is the AccessKit
    /// canonical for binary on/off (vs. CheckBox which is tri-state).
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Switch)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != ToggleState::Disabled)
            .action(Action::Click)
            .toggled(if self.on {
                Toggled::True
            } else {
                Toggled::False
            })
            .build()
    }
}

// ⛔ **O `thumb_circle` SAIU com o interruptor deslizante** (2026-09-03). Ele media o curso do
// disco — e o que defendia era real: numa moldura QUADRADA o curso era zero (ON e OFF no mesmo
// pixel, um controlo incapaz de mostrar o próprio estado) e numa moldura EM PÉ era **−30**, com
// a semântica invertida e o disco 24 px fora do corpo. Os dois gates que o guardavam foram
// removidos com ele.
//
// ⚠️ **Nada disso reaparece na marca nova**: uma caixa quadrada não tem curso para percorrer, e
// a `paint_boolean_mark` limita o lado a `min(box, rect.h)` e encosta-a à direita. *Um gate que
// defende geometria que já não se desenha não é uma protecção — é uma âncora.*

/// ⭐⭐⭐ **O interruptor deslizante SAIU** (Enio, 2026-09: *«as pílulas e o interruptor deslizante
/// podem sair»*). O que ele pinta agora é a **mesma marca booleana da caixa de verificação** —
/// [`super::checkbox::paint_boolean_mark`], um pintor só.
///
/// A razão é a do §5.2 da pesquisa `07`: *um interruptor custa ~2× a largura e diz o mesmo*. Num
/// painel de tablet essa largura é o recurso escasso.
///
/// ⛔ **O `Toggle` NÃO foi apagado, e isso é uma decisão com mecanismo:** o `WidgetKind::Toggle`
/// tem `code() == 2` e o código **viaja em documento** (`skin/kind.rs` proíbe reciclar códigos) ⇒
/// um painel autorado gravado antes disto continua a abrir, e a peça que ele nomeia continua a
/// existir. ⚠️ O `Role::Switch` do [`Toggle::build_a11y`] também fica: *fundir a tinta de dois
/// controlos não os torna o mesmo controlo*.
///
/// ⚠️ Os três chamadores (grid-snap · painter-layers · timeline) passam um rect com forma de
/// interruptor e pintam o rótulo eles próprios. A marca **encosta à direita** desse rect, que é
/// onde o polegar do «ligado» estava — logo nenhum deles muda uma linha.
pub fn paint_toggle(toggle: &Toggle, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let _ = super::checkbox::paint_boolean_mark(
        rect,
        super::checkbox::BooleanMark {
            value: if toggle.on {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            },
            state: match toggle.state {
                ToggleState::Normal => CheckboxState::Normal,
                ToggleState::Hovered => CheckboxState::Hovered,
                ToggleState::Pressed => CheckboxState::Pressed,
                ToggleState::Focused => CheckboxState::Focused,
                ToggleState::Disabled => CheckboxState::Disabled,
            },
            hover_t: toggle.hover_t,
            box_px: None,
            // ⛔ **O interruptor NÃO leva a coluna de animação**, e não é esquecimento: os três
            // chamadores dele (grid-snap · painter-layers · timeline) pintam o rótulo eles próprios
            // e passam um rect **com forma de interruptor** — uma faixa estreita no fim da linha,
            // não a linha. Reservar `14 px` ali espremia a marca sem pôr a bolinha na margem do
            // formulário, que é o único sítio onde ela diz alguma coisa.
            decorator: false,
        },
        scene,
        theme,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let t = Toggle::new(NodeId(1), "Snap");
        assert_eq!(t.id, NodeId(1));
        assert_eq!(t.label, "Snap");
        assert!(!t.on);
        assert_eq!(t.state, ToggleState::Normal);
    }

    #[test]
    fn toggle_flips_state() {
        let mut t = Toggle::new(NodeId(1), "x");
        assert!(!t.on);
        t.toggle();
        assert!(t.on);
        t.toggle();
        assert!(!t.on);
    }

    #[test]
    fn builder_chain_sets_on() {
        let t = Toggle::new(NodeId(1), "x").on(true);
        assert!(t.on);
    }

    #[test]
    fn a11y_node_has_switch_role_and_toggled_off() {
        let t = Toggle::new(NodeId(1), "Snap");
        let node = t.build_a11y(0.0, 0.0, 60.0, 24.0);
        assert_eq!(node.role(), Role::Switch);
        assert_eq!(node.label(), Some("Snap"));
        assert_eq!(node.toggled(), Some(Toggled::False));
    }

    #[test]
    fn a11y_node_toggled_true_when_on() {
        let t = Toggle::new(NodeId(1), "Snap").on(true);
        let node = t.build_a11y(0.0, 0.0, 60.0, 24.0);
        assert_eq!(node.toggled(), Some(Toggled::True));
    }

    fn smoke(t: Toggle, theme: Theme) {
        let mut scene = VectorScene::new();
        paint_toggle(&t, Rect::new(0.0, 0.0, 60.0, 24.0), &mut scene, theme);
    }

    #[test]
    fn paint_smoke_off() {
        smoke(Toggle::new(NodeId(1), "x"), Theme::Forge);
    }

    #[test]
    fn paint_smoke_on() {
        smoke(Toggle::new(NodeId(1), "x").on(true), Theme::Forge);
    }

    #[test]
    fn paint_smoke_hovered_on() {
        smoke(
            Toggle::new(NodeId(1), "x")
                .on(true)
                .state(ToggleState::Hovered),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_focused() {
        smoke(
            Toggle::new(NodeId(1), "x").state(ToggleState::Focused),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_pressed() {
        smoke(
            Toggle::new(NodeId(1), "x").state(ToggleState::Pressed),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            Toggle::new(NodeId(1), "x").state(ToggleState::Disabled),
            Theme::Forge,
        );
    }
}
