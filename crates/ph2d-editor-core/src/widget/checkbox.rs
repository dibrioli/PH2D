//! [`Checkbox`] — tri-state on / off / indeterminate.
//!
//! AccessKit `Role::CheckBox` with `Toggled::True/False/Mixed`. The
//! Indeterminate state is for "some children selected" cases (a tree
//! row representing a group with mixed-selected leaves, etc).

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role, Toggled};
use ph2d_text::TextSystem;
use ph2d_tokens::{
    CHECKBOX_BOX_PX as CHROME_CHECKBOX_BOX, ColorToken, Radius, Spacing, StrokeToken, Theme,
    TypeToken,
};
use ph2d_vector::VectorScene;

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
    pub value: CheckboxValue,
}

impl Checkbox {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        }
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

/// Edge length of the box itself (label flows to the right).
/// Per tokens.json `chrome.checkbox-box`.
pub const CHECKBOX_BOX_PX: f32 = CHROME_CHECKBOX_BOX;

/// Square box (left) + label (right). Box fills with `Accent` when
/// Checked, paints a check glyph; Indeterminate paints a dash glyph.
pub fn paint_checkbox(
    cb: &Checkbox,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let box_size = CHECKBOX_BOX_PX.min(rect.h);
    let box_y = rect.y + (rect.h - box_size) * 0.5;
    let box_rect = Rect::new(rect.x, box_y, box_size, box_size);

    let radius = Radius::Xs.px();
    let (bg_token, border_token) = match (cb.state, cb.value) {
        (CheckboxState::Disabled, _) => (ColorToken::Bg2, ColorToken::Border),
        (_, CheckboxValue::Checked | CheckboxValue::Indeterminate) => {
            (ColorToken::Accent, ColorToken::Accent)
        }
        (CheckboxState::Hovered | CheckboxState::Focused, _) => {
            (ColorToken::Bg2, ColorToken::BorderEmph)
        }
        _ => (ColorToken::Bg1, ColorToken::Border),
    };
    fill_rounded_rect(scene, box_rect, radius, resolve(bg_token, theme));
    stroke_rounded_rect(
        scene,
        box_rect,
        radius,
        if cb.state == CheckboxState::Focused {
            2.0
        } else {
            1.0
        },
        resolve(border_token, theme),
    );

    let glyph_color = if cb.state == CheckboxState::Disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::AccentFg
    };
    let g = resolve(glyph_color, theme);
    match cb.value {
        CheckboxValue::Checked => {
            paint_icon(scene, IconId::Check, box_rect, g, StrokeToken::Default.px())
        }
        // Indeterminate paints a horizontal dash (Minus glyph) — the
        // platform convention for "some children selected". Previous
        // code used `Plus` here, which painted a `+` and read as
        // "add" rather than "mixed".
        CheckboxValue::Indeterminate => paint_icon(scene, IconId::Minus, box_rect, g, 2.0),
        CheckboxValue::Unchecked => {}
    }

    if !cb.label.is_empty() && rect.w > box_size + Spacing::Md.px() {
        let lx = rect.x + box_size + Spacing::Md.px();
        let label_color = if cb.state == CheckboxState::Disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        let font_size = TypeToken::Base.px();
        let ly = rect.y + (rect.h - font_size) * 0.5;
        paint_text(
            text_system,
            scene,
            &cb.label,
            lx,
            ly,
            font_size,
            (rect.x + rect.w - lx).max(0.0),
            resolve(label_color, theme),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Checkbox {
        Checkbox::new(NodeId(1), "Snap to grid")
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

    fn smoke(c: Checkbox, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_checkbox(
            &c,
            Rect::new(0.0, 0.0, 200.0, 18.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_normal_unchecked() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_hovered_checked() {
        smoke(
            fixture()
                .value(CheckboxValue::Checked)
                .state(CheckboxState::Hovered),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_pressed_indeterminate() {
        smoke(
            fixture()
                .value(CheckboxValue::Indeterminate)
                .state(CheckboxState::Pressed),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_focused_unchecked() {
        smoke(fixture().state(CheckboxState::Focused), Theme::Workshop);
    }

    #[test]
    fn paint_smoke_disabled_checked() {
        smoke(
            fixture()
                .value(CheckboxValue::Checked)
                .state(CheckboxState::Disabled),
            Theme::Forge,
        );
    }
}
