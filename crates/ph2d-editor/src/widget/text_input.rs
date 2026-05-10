//! [`TextInput`] — single-line text field.
//!
//! v1 paints layout-only: caret stays static at `caret_pos`, no IME
//! composing, no selection range. Real input handling lands when the
//! shell wires `winit::Event::KeyboardInput` into the editor (post
//! M13). The data shape here exists so widgets that compose
//! TextInput (NumberInput, Combobox) have a stable contract.

use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

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
        }
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

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::TextInput)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != TextInputState::Disabled)
            .action(Action::Focus)
            .build()
    }
}

/// Border tokens chosen by state. Promoted to a free function so
/// `text_area`/`number_input` can reuse the same palette.
pub(crate) fn border_token(state: TextInputState) -> ColorToken {
    match state {
        TextInputState::Disabled => ColorToken::Border,
        TextInputState::Hovered => ColorToken::BorderEmph,
        TextInputState::Focused => ColorToken::Accent,
        TextInputState::Error => ColorToken::Danger,
        TextInputState::Normal => ColorToken::Border,
    }
}

pub(crate) fn fill_token(state: TextInputState) -> ColorToken {
    match state {
        TextInputState::Disabled => ColorToken::Bg2,
        _ => ColorToken::Bg1,
    }
}

pub fn paint_text_input(
    input: &TextInput,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(fill_token(input.state), theme));
    let stroke_w = if input.state == TextInputState::Focused {
        2.0
    } else {
        1.0
    };
    stroke_rounded_rect(
        scene,
        rect,
        radius,
        stroke_w,
        resolve(border_token(input.state), theme),
    );

    let pad_x = Spacing::Lg.px();
    let pad_y = Spacing::Md.px();
    let font_size = TypeToken::Base.px();
    let inner_x = rect.x + pad_x;
    let inner_y = rect.y + (rect.h - font_size) * 0.5 - pad_y * 0.0;
    let inner_w = (rect.w - pad_x * 2.0).max(0.0);

    if input.value.is_empty() && !input.placeholder.is_empty() {
        paint_text(
            text_system,
            scene,
            &input.placeholder,
            inner_x,
            inner_y,
            font_size,
            inner_w,
            resolve(ColorToken::Text3, theme),
        );
    } else if !input.value.is_empty() {
        let color = if input.state == TextInputState::Disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        paint_text(
            text_system,
            scene,
            &input.value,
            inner_x,
            inner_y,
            font_size,
            inner_w,
            resolve(color, theme),
        );
    }

    if input.state == TextInputState::Focused {
        // Static 1px caret at end-of-value position. Real metrics
        // need parley layout; this approximation places the caret at
        // an em-width per char, accurate enough for monospace and
        // close enough for sans on inspector-width fields.
        let caret_byte = input.caret_byte.min(input.value.len());
        let prefix = &input.value[..caret_byte];
        let approx_advance = prefix.chars().count() as f32 * font_size * 0.55;
        let caret_x = inner_x + approx_advance;
        let caret = Rect::new(caret_x, rect.y + pad_y, 1.0, rect.h - pad_y * 2.0);
        scene.fill_rect(
            crate::paint::rect_to_vello(caret),
            resolve(ColorToken::Accent, theme),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> TextInput {
        TextInput::new(NodeId(1), "Project name")
    }

    #[test]
    fn defaults_match_spec() {
        let t = fixture();
        assert_eq!(t.value, "");
        assert_eq!(t.placeholder, "");
        assert_eq!(t.state, TextInputState::Normal);
        assert_eq!(t.caret_byte, 0);
    }

    #[test]
    fn value_seed_moves_caret_to_end() {
        let t = fixture().value("hello");
        assert_eq!(t.value, "hello");
        assert_eq!(t.caret_byte, 5);
    }

    #[test]
    fn a11y_role_is_text_input() {
        let node = fixture().build_a11y(0.0, 0.0, 200.0, 32.0);
        assert_eq!(node.role(), Role::TextInput);
    }

    fn smoke(t: TextInput, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_text_input(
            &t,
            Rect::new(0.0, 0.0, 240.0, 32.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_empty_with_placeholder() {
        smoke(fixture().placeholder("Untitled"), Theme::ForgeSdf);
    }

    #[test]
    fn paint_smoke_filled_focused() {
        smoke(
            fixture()
                .value("hello world")
                .state(TextInputState::Focused),
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn paint_smoke_hovered() {
        smoke(fixture().state(TextInputState::Hovered), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_error() {
        smoke(
            fixture().value("oops").state(TextInputState::Error),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            fixture().value("locked").state(TextInputState::Disabled),
            Theme::PaintStudio,
        );
    }
}
