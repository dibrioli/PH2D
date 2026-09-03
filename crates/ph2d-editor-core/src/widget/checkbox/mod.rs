//! [`Checkbox`] — tri-state on / off / indeterminate.
//!
//! AccessKit `Role::CheckBox` with `Toggled::True/False/Mixed`. The
//! Indeterminate state is for "some children selected" cases (a tree
//! row representing a group with mixed-selected leaves, etc).

// ⚠️ O que sobra aqui é o MODELO — o desenho mudou-se para `mark.rs` com o tecto de LOC, e levou
// consigo os `use` da tinta.
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role, Toggled};
use ph2d_tokens::CHECKBOX_BOX_PX as CHROME_CHECKBOX_BOX;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum CheckboxState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum CheckboxValue {
    #[default]
    Unchecked,
    Checked,
    /// Some children selected. Painted with a horizontal dash.
    Indeterminate,
}

#[derive(Clone, Debug)]
pub struct Checkbox {
    pub id: NodeId,
    pub label: String,
    pub state: CheckboxState,
    /// Quanto do hover está PRESENTE (`0`..=`1`); [`crate::motion::SETTLED`] = assente no estado.
    pub hover_t: f32,
    pub value: CheckboxValue,
    /// Aresta da caixa, em px. **`None` é o token** ([`CHECKBOX_BOX_PX`]) — a lei que todo
    /// painel usa e a razão de um formulário ler como formulário: cada checkbox do app tem
    /// exactamente o mesmo tamanho.
    ///
    /// ⚠️ **Isto não é um canal de ESTILO, é o TAMANHO** — a moldura já o comunica para dez dos
    /// doze widgets do catálogo, e este é um dos dois que a recusavam. Existe um consumidor e
    /// um só: a pele de canvas ([`crate::widget::paint_widget_skin`]), onde a moldura é o que o
    /// artista desenhou e não a linha de um painel. Qualquer valor continua limitado pela
    /// altura da moldura — a caixa nunca transborda o que a contém.
    pub box_px: Option<f32>,
    /// **Reserva e desenha a coluna de animação à direita.**
    ///
    /// ⭐ Nasce a [`crate::widget::FORM_ROWS_SHOW_DECORATOR`] — ou seja, **ligada** —, e por isso
    /// os 81 chamadores não mudam uma linha: eles constroem por `Checkbox::new(..)`.
    ///
    /// ⚠️ **É um campo PRÓPRIO, e a 1.ª tentativa não o era.** Ela derivava a coluna de
    /// `box_px.is_none()`, aproveitando que a pele de canvas é o único sítio que define aquele
    /// campo — e isso **partiu o contrato do `box_px`**, que diz *pedir o próprio token é igual a
    /// não pedir nada*. O gate `without_an_override_the_box_is_the_token` apanhou-o na primeira
    /// corrida. *Um parâmetro com dois papéis torna a chamada errada defensável.*
    pub decorator: bool,
}

impl Checkbox {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            state: CheckboxState::Normal,
            hover_t: crate::motion::SETTLED,
            value: CheckboxValue::Unchecked,
            box_px: None,
            decorator: crate::widget::property_box::FORM_ROWS_SHOW_DECORATOR,
        }
    }

    /// **O par visual do store numa chamada** — o irmão exacto do [`super::Button::visual`].
    ///
    /// ⚠️ **É esta a porta, e não o `.state()` ao lado de um `.hover_t()`:** com dois métodos,
    /// esquecer o segundo é o estado natural, e o sítio nasce silenciosamente discreto. A fonte é
    /// [`crate::interaction::WidgetStore::checkbox_visual`]; o `.value(..)` continua a vir do
    /// MODELO, porque quem sabe se a caixa está marcada é o painel, não o store.
    #[must_use]
    pub fn visual(self, v: (CheckboxState, f32)) -> Self {
        self.state(v.0).hover_t(v.1)
    }

    /// Quanto do hover está presente. O neutro [`crate::motion::SETTLED`] pinta os tokens DUROS.
    #[must_use]
    pub fn hover_t(mut self, t: f32) -> Self {
        self.hover_t = t.clamp(0.0, 1.0);
        self
    }

    pub fn state(mut self, state: CheckboxState) -> Self {
        self.state = state;
        self
    }

    pub fn value(mut self, value: CheckboxValue) -> Self {
        self.value = value;
        self
    }

    /// Cycle Unchecked → Checked → Unchecked. Indeterminate is set
    /// programmatically only.
    pub fn toggle(&mut self) {
        self.value = match self.value {
            CheckboxValue::Unchecked | CheckboxValue::Indeterminate => CheckboxValue::Checked,
            CheckboxValue::Checked => CheckboxValue::Unchecked,
        };
    }

    /// Build the AccessKit node.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let toggled = match self.value {
            CheckboxValue::Checked => Toggled::True,
            CheckboxValue::Unchecked => Toggled::False,
            CheckboxValue::Indeterminate => Toggled::Mixed,
        };
        NodeBuilder::new(Role::CheckBox)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != CheckboxState::Disabled)
            .action(Action::Click)
            .toggled(toggled)
            .build()
    }
}

mod mark;
pub use mark::paint_checkbox;
// ⚠️ `pub(crate)`: o pintor da marca é a porta do **interruptor**, não da casa. Um `pub` aqui
// convidaria um painel a desenhar um booleano sem passar pelo `Checkbox`, que é onde vivem o
// estado, o valor e o nó de acessibilidade.
pub(crate) use mark::{BooleanMark, paint_boolean_mark};

/// Edge length of the box itself (label flows to the right).
/// Per tokens.json `chrome.checkbox-box`.
pub const CHECKBOX_BOX_PX: f32 = CHROME_CHECKBOX_BOX;

#[cfg(test)]
mod tests {
    //! ⚠️ **Só o MODELO.** Os testes de TINTA foram com o pintor para o `mark.rs` quando o tecto de
    //! LOC partiu este ficheiro — um teste que pinta pertence ao ficheiro que pinta.
    use super::*;

    /// A caixa de partida dos testes — um sítio só, para nenhum deles nascer com um estado
    /// que o vizinho não tem.
    fn fixture() -> Checkbox {
        Checkbox::new(NodeId(1), "Snap")
    }

    #[test]
    fn defaults_match_spec() {
        let c = fixture();
        assert_eq!(c.value, CheckboxValue::Unchecked);
        assert_eq!(c.state, CheckboxState::Normal);
    }

    #[test]
    fn toggle_cycles_unchecked_checked() {
        let mut c = fixture();
        c.toggle();
        assert_eq!(c.value, CheckboxValue::Checked);
        c.toggle();
        assert_eq!(c.value, CheckboxValue::Unchecked);
    }

    #[test]
    fn toggle_from_indeterminate_goes_to_checked() {
        let mut c = fixture().value(CheckboxValue::Indeterminate);
        c.toggle();
        assert_eq!(c.value, CheckboxValue::Checked);
    }

    #[test]
    fn a11y_role_is_checkbox_with_toggled() {
        let node = fixture()
            .value(CheckboxValue::Checked)
            .build_a11y(0.0, 0.0, 100.0, 18.0);
        assert_eq!(node.role(), Role::CheckBox);
        assert_eq!(node.toggled(), Some(Toggled::True));
    }

    #[test]
    fn a11y_indeterminate_is_mixed() {
        let node = fixture()
            .value(CheckboxValue::Indeterminate)
            .build_a11y(0.0, 0.0, 100.0, 18.0);
        assert_eq!(node.toggled(), Some(Toggled::Mixed));
    }
}
