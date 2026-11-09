//! O modelo do campo de texto — [`TextInputState`], [`TextInput`] e o construtor dele —, irmão do
//! pintor (`mod.rs`) pelo tecto de LOC dos widgets.

use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TextInputState {
    #[default]
    Normal,
    Hovered,
    Focused,
    Disabled,
    /// Validation failed; border switches to `Danger`.
    Error,
}

#[derive(Clone, Debug)]
pub struct TextInput {
    pub id: NodeId,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub state: TextInputState,
    /// Byte offset of the caret within `value`. Out-of-range values
    /// are clamped at paint time. v1 draws the caret only when
    /// `state == Focused`.
    pub caret_byte: usize,
    /// Quanto do hover está presente. [`crate::motion::SETTLED`] = assente no estado que o campo
    /// diz ter, que é o mundo pré-UI-viva byte a byte.
    pub hover_t: f32,
    /// O foco pinta um anel FINO (`StrokeToken::Thin`) em vez da moldura de foco do tema — o campo de busca que
    /// mora dentro de uma lista aberta (dono 08/11: o anel grosso «ficou feio»). `false` = o de sempre.
    pub anel_fino: bool,
}

impl TextInput {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            value: String::new(),
            placeholder: String::new(),
            state: TextInputState::Normal,
            caret_byte: 0,
            hover_t: crate::motion::SETTLED,
            anel_fino: false,
        }
    }

    /// Ver [`Self::anel_fino`].
    #[must_use]
    pub fn anel_fino(mut self) -> Self {
        self.anel_fino = true;
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.caret_byte = self.value.len();
        self
    }

    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn state(mut self, state: TextInputState) -> Self {
        self.state = state;
        self
    }

    /// **O par que o store publica** — `(estado, quanto do hover está presente)`, numa pergunta só.
    /// Irmão exacto do [`crate::widget::Button::visual`].
    #[must_use]
    pub fn visual(self, v: (TextInputState, f32)) -> Self {
        self.state(v.0).hover_t(v.1)
    }

    #[must_use]
    pub fn hover_t(mut self, t: f32) -> Self {
        self.hover_t = t.clamp(0.0, 1.0);
        self
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let b = NodeBuilder::new(Role::TextInput)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != TextInputState::Disabled)
            .action(Action::Focus);
        super::compose::a11y_value(b, self).build()
    }
}
