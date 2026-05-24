//! [`NumberInput`] — numeric value with up/down step buttons.
//!
//! Reuses the [`TextInput`](super::text_input) border palette to stay
//! visually consistent. Steppers are 16x16 chips painted on the right
//! edge; clicking them adjusts the value by `step`. Hit testing for
//! the steppers is shell-side via [`NumberInput::up_rect`] /
//! [`NumberInput::down_rect`].

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::widget::text_input::{TextInputState, border_token, fill_token};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct NumberInput {
    pub id: NodeId,
    pub label: String,
    pub value: f64,
    pub step: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub state: TextInputState,
}

impl NumberInput {
    pub fn new(id: NodeId, label: impl Into<String>, value: f64) -> Self {
        Self {
            id,
            label: label.into(),
            value,
            step: 1.0,
            min: None,
            max: None,
            state: TextInputState::Normal,
        }
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self.value = self.value.max(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self.value = self.value.min(max);
        self
    }

    pub fn state(mut self, state: TextInputState) -> Self {
        self.state = state;
        self
    }

    /// Bump value by `+step`, clamped to `max`.
    pub fn increment(&mut self) {
        self.value = self.clamp(self.value + self.step);
    }

    /// Bump value by `-step`, clamped to `min`.
    pub fn decrement(&mut self) {
        self.value = self.clamp(self.value - self.step);
    }

    fn clamp(&self, v: f64) -> f64 {
        let mut v = v;
        if let Some(min) = self.min {
            v = v.max(min);
        }
        if let Some(max) = self.max {
            v = v.min(max);
        }
        v
    }

    pub fn up_rect(&self, host: Rect) -> Rect {
        stepper_up_rect(host)
    }

    pub fn down_rect(&self, host: Rect) -> Rect {
        stepper_down_rect(host)
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut builder = NodeBuilder::new(Role::NumberInput)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != TextInputState::Disabled)
            .action(Action::Focus)
            .numeric_value(self.value);
        if let Some(min) = self.min {
            builder = builder.numeric_value_min(min);
        }
        if let Some(max) = self.max {
            builder = builder.numeric_value_max(max);
        }
        builder.build()
    }
}

/// Width of the up/down stepper column carved out of the right edge
/// of every NumberInput / chip hit rect. Sized 60% of the host height,
/// clamped to 16-22 px so it stays clickable on dense rows but doesn't
/// dominate compact chips.
///
/// Exposed `pub` so `slider_with_chip::paint_number_chip` and the
/// stepper hit-test in `dispatch::number_input` use the SAME formula —
/// the chip canon (post-2026-05-24) paints arrows in this exact column,
/// and the dispatch's `apply_number_stepper_if_hit` carves this same
/// rect for the click→step affordance.
pub fn stepper_width(host: Rect) -> f32 {
    (host.h * 0.6).clamp(16.0, 22.0) // LITERAL-PX-OK: stepper column sized 60% of input height with min/max
}

/// Rect of the `up` arrow (top half of the stepper column). Standalone
/// fn so `paint_number_chip` can paint identical arrows without a
/// `NumberInput` instance.
pub fn stepper_up_rect(host: Rect) -> Rect {
    let chip_w = stepper_width(host);
    Rect::new(host.x + host.w - chip_w, host.y, chip_w, host.h * 0.5)
}

/// Rect of the `down` arrow (bottom half of the stepper column).
pub fn stepper_down_rect(host: Rect) -> Rect {
    let chip_w = stepper_width(host);
    Rect::new(
        host.x + host.w - chip_w,
        host.y + host.h * 0.5,
        chip_w,
        host.h * 0.5,
    )
}

pub fn paint_number_input(
    input: &NumberInput,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_number_input_with_buffer(input, None, 0, None, rect, scene, text_system, theme)
}

/// Like [`paint_number_input`] but renders the in-progress edit
/// buffer + caret line when the input is focused. Pass `Some(buffer)`
/// and a caret byte offset when reading live state from a
/// [`crate::interaction::WidgetStore`]; otherwise pass `None`.
/// `selection_anchor` paints a selection background when non-None
/// and the input is focused.
#[allow(clippy::too_many_arguments)]
pub fn paint_number_input_with_buffer(
    input: &NumberInput,
    buffer: Option<&str>,
    caret: usize,
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

    let chip_w = stepper_width(rect);
    let pad_x = Spacing::Lg.px();
    let value_text_owned;
    let value_text: &str = match buffer {
        Some(b) if input.state == TextInputState::Focused => b,
        _ => {
            value_text_owned = format_number(input.value);
            value_text_owned.as_str()
        }
    };
    let font_size = TypeToken::Base.px();
    let inner_x = rect.x + pad_x;
    let inner_y = rect.y + (rect.h - font_size) * 0.5;
    let inner_w = (rect.w - pad_x * 2.0 - chip_w).max(0.0);
    let label_color = if input.state == TextInputState::Disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text1
    };
    if input.state == TextInputState::Focused
        && buffer.is_some()
        && let Some(anchor) = selection_anchor
        && anchor != caret
    {
        let (sel_start, sel_end) = if anchor < caret {
            (anchor, caret)
        } else {
            (caret, anchor)
        };
        let sel_start = sel_start.min(value_text.len());
        let sel_end = sel_end.min(value_text.len());
        let prefix_w = text_system.prefix_width(&value_text[..sel_start], font_size);
        let mid_w = if sel_start == sel_end {
            0.0
        } else {
            text_system.prefix_width(&value_text[sel_start..sel_end], font_size)
        };
        let sel_x = (inner_x + prefix_w).min(inner_x + inner_w);
        let sel_w = mid_w.min(inner_x + inner_w - sel_x);
        if sel_w > 0.0 {
            let sel_top = rect.y + Spacing::Md.px();
            let sel_bot = rect.y + rect.h - Spacing::Md.px();
            let sel_rect = Rect::new(sel_x, sel_top, sel_w, (sel_bot - sel_top).max(2.0));
            fill_rounded_rect(scene, sel_rect, 1.0, resolve(ColorToken::AccentSoft, theme));
        }
    }
    paint_text(
        text_system,
        scene,
        value_text,
        inner_x,
        inner_y,
        font_size,
        inner_w,
        resolve(label_color, theme),
    );

    if input.state == TextInputState::Focused && buffer.is_some() {
        let caret_clamped = caret.min(value_text.len());
        let prefix = &value_text[..caret_clamped];
        let prefix_w = if prefix.is_empty() {
            0.0
        } else {
            text_system.prefix_width(prefix, font_size)
        };
        let caret_x = (inner_x + prefix_w).min(inner_x + inner_w);
        let caret_top = rect.y + Spacing::Md.px();
        let caret_bot = rect.y + rect.h - Spacing::Md.px();
        let caret_rect = Rect::new(
            caret_x,
            caret_top,
            StrokeToken::Default.px(),
            (caret_bot - caret_top).max(2.0),
        );
        fill_rounded_rect(scene, caret_rect, 0.75, resolve(ColorToken::Accent, theme)); // LITERAL-PX-OK: caret half-width radius
    }

    let icon_color = resolve(ColorToken::Text2, theme);
    paint_icon(
        scene,
        IconId::ChevronUp,
        input.up_rect(rect),
        icon_color,
        StrokeToken::Default.px(),
    );
    paint_icon(
        scene,
        IconId::ChevronDown,
        input.down_rect(rect),
        icon_color,
        StrokeToken::Default.px(),
    );
}

pub fn format_number(v: f64) -> String {
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v as i64)
    } else {
        format!("{v:.3}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let n = NumberInput::new(NodeId(1), "Width", 10.0);
        assert_eq!(n.value, 10.0);
        assert_eq!(n.step, 1.0);
        assert_eq!(n.min, None);
        assert_eq!(n.max, None);
    }

    #[test]
    fn min_max_clamp_initial_value() {
        let n = NumberInput::new(NodeId(1), "x", -5.0).min(0.0);
        assert_eq!(n.value, 0.0);
        let n = NumberInput::new(NodeId(1), "x", 100.0).max(50.0);
        assert_eq!(n.value, 50.0);
    }

    #[test]
    fn increment_respects_max() {
        let mut n = NumberInput::new(NodeId(1), "x", 9.5).step(1.0).max(10.0);
        n.increment();
        assert_eq!(n.value, 10.0);
        n.increment();
        assert_eq!(n.value, 10.0);
    }

    #[test]
    fn decrement_respects_min() {
        let mut n = NumberInput::new(NodeId(1), "x", 0.5).step(1.0).min(0.0);
        n.decrement();
        assert_eq!(n.value, 0.0);
        n.decrement();
        assert_eq!(n.value, 0.0);
    }

    #[test]
    fn a11y_role_is_number_input_with_value() {
        let n = NumberInput::new(NodeId(1), "x", 7.0).min(0.0).max(10.0);
        let node = n.build_a11y(0.0, 0.0, 100.0, 32.0);
        assert_eq!(node.role(), Role::NumberInput);
        assert_eq!(node.numeric_value(), Some(7.0));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(10.0));
    }

    #[test]
    fn stepper_rects_split_vertically() {
        let n = NumberInput::new(NodeId(1), "x", 0.0);
        let host = Rect::new(0.0, 0.0, 100.0, 32.0);
        let up = n.up_rect(host);
        let down = n.down_rect(host);
        assert!(up.y < down.y);
        assert!((up.h - down.h).abs() < 0.01);
    }

    fn smoke(n: NumberInput, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_number_input(
            &n,
            Rect::new(0.0, 0.0, 120.0, 32.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_default() {
        smoke(NumberInput::new(NodeId(1), "x", 42.0), Theme::Forge);
    }

    #[test]
    fn paint_smoke_focused() {
        smoke(
            NumberInput::new(NodeId(1), "x", 1.5).state(TextInputState::Focused),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            NumberInput::new(NodeId(1), "x", 0.0).state(TextInputState::Disabled),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_error() {
        smoke(
            NumberInput::new(NodeId(1), "x", 99.0).state(TextInputState::Error),
            Theme::Workshop,
        );
    }
}
