//! [`TextArea`] — multiline text field.
//!
//! Same border/focus/error palette as [`super::text_input::TextInput`]
//! but reserves at least 3 rows of vertical space and lets text wrap.
//! Caret rendering is omitted in v1; multiline carets need full
//! parley layout introspection that lands when input handling does.

use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::widget::text_input::{TextInputState, border_token, fill_token};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct TextArea {
    pub id: NodeId,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub state: TextInputState,
}

impl TextArea {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            value: String::new(),
            placeholder: String::new(),
            state: TextInputState::Normal,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
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
        NodeBuilder::new(Role::MultilineTextInput)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != TextInputState::Disabled)
            .action(Action::Focus)
            .build()
    }
}

/// Suggested minimum height = 3 rows at body font size.
pub fn min_height(font_size: f32) -> f32 {
    font_size * 3.0 + Spacing::Md.px() * 2.0
}

pub fn paint_text_area(
    area: &TextArea,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(fill_token(area.state), theme));
    let stroke_w = if area.state == TextInputState::Focused {
        2.0
    } else {
        1.0
    };
    stroke_rounded_rect(
        scene,
        rect,
        radius,
        stroke_w,
        resolve(border_token(area.state), theme),
    );

    let pad_x = Spacing::Lg.px();
    let pad_y = Spacing::Md.px();
    let inner_x = rect.x + pad_x;
    let inner_y = rect.y + pad_y;
    let inner_w = (rect.w - pad_x * 2.0).max(0.0);
    let font_size = TypeToken::Base.px();

    if area.value.is_empty() && !area.placeholder.is_empty() {
        paint_text(
            text_system,
            scene,
            &area.placeholder,
            inner_x,
            inner_y,
            font_size,
            inner_w,
            resolve(ColorToken::Text3, theme),
        );
    } else if !area.value.is_empty() {
        let color = if area.state == TextInputState::Disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        paint_text(
            text_system,
            scene,
            &area.value,
            inner_x,
            inner_y,
            font_size,
            inner_w,
            resolve(color, theme),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let a = TextArea::new(NodeId(1), "Notes");
        assert_eq!(a.value, "");
        assert_eq!(a.state, TextInputState::Normal);
    }

    #[test]
    fn min_height_is_3_rows_plus_padding() {
        let h = min_height(14.0);
        assert!(h >= 14.0 * 3.0);
    }

    #[test]
    fn a11y_role_is_multiline_text_input() {
        let node = TextArea::new(NodeId(1), "x").build_a11y(0.0, 0.0, 200.0, 80.0);
        assert_eq!(node.role(), Role::MultilineTextInput);
    }

    fn smoke(area: TextArea, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_text_area(
            &area,
            Rect::new(0.0, 0.0, 240.0, 96.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_empty_with_placeholder() {
        smoke(
            TextArea::new(NodeId(1), "x").placeholder("Notes…"),
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn paint_smoke_filled_focused() {
        smoke(
            TextArea::new(NodeId(1), "x")
                .value("multi\nline\nvalue")
                .state(TextInputState::Focused),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            TextArea::new(NodeId(1), "x")
                .value("read-only")
                .state(TextInputState::Disabled),
            Theme::Blueprint,
        );
    }
}
