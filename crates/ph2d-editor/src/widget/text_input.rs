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
    paint_text_input_with_buffer(input, None, None, None, rect, scene, text_system, theme)
}

/// Like [`paint_text_input`] but draws an override `buffer` and
/// caret offset when the caller has a live
/// [`crate::interaction::WidgetStore`] entry for the input. Reading
/// from the store avoids per-frame allocations that would happen if
/// the caller copied `store.text(id)` into `TextInput.value`.
/// `selection_anchor` is the other end of an active selection (for
/// double-click "select all" + Shift+Arrow); when None, no selection
/// is drawn.
#[allow(clippy::too_many_arguments)]
pub fn paint_text_input_with_buffer(
    input: &TextInput,
    buffer: Option<&str>,
    caret: Option<usize>,
    selection_anchor: Option<usize>,
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

    let displayed: &str = buffer.unwrap_or(input.value.as_str());
    let displayed_caret = caret.unwrap_or(input.caret_byte);

    if input.state == TextInputState::Focused
        && let Some(anchor) = selection_anchor
        && anchor != displayed_caret
    {
        let (sel_start, sel_end) = if anchor < displayed_caret {
            (anchor, displayed_caret)
        } else {
            (displayed_caret, anchor)
        };
        let sel_start = sel_start.min(displayed.len());
        let sel_end = sel_end.min(displayed.len());
        let prefix_w = text_system.prefix_width(&displayed[..sel_start], font_size);
        let mid_w = if sel_start == sel_end {
            0.0
        } else {
            text_system.prefix_width(&displayed[sel_start..sel_end], font_size)
        };
        let sel_x = (inner_x + prefix_w).min(inner_x + inner_w);
        let sel_w = mid_w.min(inner_x + inner_w - sel_x);
        if sel_w > 0.0 {
            let sel_rect = Rect::new(sel_x, rect.y + pad_y, sel_w, rect.h - pad_y * 2.0);
            fill_rounded_rect(scene, sel_rect, 1.0, resolve(ColorToken::AccentSoft, theme));
        }
    }

    if displayed.is_empty() && !input.placeholder.is_empty() {
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
    } else if !displayed.is_empty() {
        let color = if input.state == TextInputState::Disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        paint_text(
            text_system,
            scene,
            displayed,
            inner_x,
            inner_y,
            font_size,
            inner_w,
            resolve(color, theme),
        );
    }

    if input.state == TextInputState::Focused {
        let caret_byte = displayed_caret.min(displayed.len());
        let prefix = &displayed[..caret_byte];
        let prefix_w = if prefix.is_empty() {
            0.0
        } else {
            text_system.prefix_width(prefix, font_size)
        };
        let caret_x = (inner_x + prefix_w).min(inner_x + inner_w);
        let caret_rect = Rect::new(caret_x, rect.y + pad_y, 1.0, rect.h - pad_y * 2.0);
        scene.fill_rect(
            crate::paint::rect_to_vello(caret_rect),
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

    #[test]
    fn paint_with_buffer_overrides_value() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let t = fixture().value("stale").state(TextInputState::Focused);
        // Pretend the WidgetStore has a freshly typed buffer.
        paint_text_input_with_buffer(
            &t,
            Some("live edit"),
            Some(4),
            None,
            Rect::new(0.0, 0.0, 240.0, 32.0),
            &mut scene,
            &mut text,
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn paint_with_buffer_handles_empty_caret_oob() {
        // Caret beyond buffer length should still paint without
        // panic (clamped at draw time).
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let t = fixture().state(TextInputState::Focused);
        paint_text_input_with_buffer(
            &t,
            Some(""),
            Some(99),
            None,
            Rect::new(0.0, 0.0, 240.0, 32.0),
            &mut scene,
            &mut text,
            Theme::Sunstone,
        );
    }
}
