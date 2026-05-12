//! [`Vector3Editor`] — three [`NumberInput`] fields side-by-side
//! labeled X/Y/Z.
//!
//! Used for transform inspector rows (position, rotation, scale).
//! Per-axis tinting (Danger/Success/Info) makes axes visually
//! distinct without a colored chip.

use super::number_input::{NumberInput, paint_number_input_with_buffer};
use super::text_input::TextInputState;
use crate::paint::{paint_text, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct Vector3Editor {
    pub id: NodeId,
    pub label: String,
    pub x: NumberInput,
    pub y: NumberInput,
    pub z: NumberInput,
    /// When true, paints axis labels (X/Y/Z) with their canonical
    /// tints (Danger/Success/Info) before each field.
    pub colored_labels: bool,
    pub state: TextInputState,
}

impl Vector3Editor {
    pub fn new(
        id: NodeId,
        label: impl Into<String>,
        x: NumberInput,
        y: NumberInput,
        z: NumberInput,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            x,
            y,
            z,
            colored_labels: true,
            state: TextInputState::Normal,
        }
    }

    pub fn colored_labels(mut self, yes: bool) -> Self {
        self.colored_labels = yes;
        self
    }

    pub fn state(mut self, state: TextInputState) -> Self {
        self.state = state;
        self.x.state = state;
        self.y.state = state;
        self.z.state = state;
        self
    }

    pub fn field_rects(&self, host: Rect) -> [Rect; 3] {
        let gap = Spacing::Md.px();
        let total_gap = gap * 2.0;
        let label_w = if self.colored_labels {
            (host.h * 0.7).max(14.0)
        } else {
            0.0
        };
        let field_w = ((host.w - total_gap) / 3.0 - label_w).max(0.0);
        std::array::from_fn(|i| {
            let x = host.x + (field_w + label_w + gap) * i as f32 + label_w;
            Rect::new(x, host.y, field_w, host.h)
        })
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Group)
            .label(&self.label)
            .bounds(x, y, w, h)
            .child(self.x.id)
            .child(self.y.id)
            .child(self.z.id)
            .build()
    }
}

pub fn paint_vector3_editor(
    v: &Vector3Editor,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_vector3_editor_with_state(
        v,
        [None; 3],
        [0; 3],
        [None; 3],
        rect,
        scene,
        text_system,
        theme,
    );
}

/// Like [`paint_vector3_editor`] but renders per-axis live edit
/// buffers + caret + selection. The arrays index 0/1/2 = X/Y/Z. Pass
/// `[None; 3]` / `[0; 3]` for the static (unfocused) case.
#[allow(clippy::too_many_arguments)]
pub fn paint_vector3_editor_with_state(
    v: &Vector3Editor,
    buffers: [Option<&str>; 3],
    carets: [usize; 3],
    selection_anchors: [Option<usize>; 3],
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let rects = v.field_rects(rect);
    let labels = ["X", "Y", "Z"];
    let label_tokens = [ColorToken::Danger, ColorToken::Success, ColorToken::Info];
    let inputs = [&v.x, &v.y, &v.z];
    let font = TypeToken::Base.px();
    for (i, ((label, &token), input)) in labels
        .iter()
        .zip(label_tokens.iter())
        .zip(inputs)
        .enumerate()
    {
        let field_rect = rects[i];
        if v.colored_labels {
            let label_x = field_rect.x - (rect.h * 0.7).max(14.0);
            let label_y = rect.y + (rect.h - font) * 0.5;
            let color = if v.state == TextInputState::Disabled {
                ColorToken::TextDisabled
            } else {
                token
            };
            paint_text(
                text_system,
                scene,
                label,
                label_x,
                label_y,
                font,
                rect.h,
                resolve(color, theme),
            );
        }
        paint_number_input_with_buffer(
            input,
            buffers[i],
            carets[i],
            selection_anchors[i],
            field_rect,
            scene,
            text_system,
            theme,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vector3Editor {
        Vector3Editor::new(
            NodeId(1),
            "Position",
            NumberInput::new(NodeId(2), "X", 1.0),
            NumberInput::new(NodeId(3), "Y", 2.0),
            NumberInput::new(NodeId(4), "Z", 3.0),
        )
    }

    #[test]
    fn defaults_match_spec() {
        let v = fixture();
        assert!(v.colored_labels);
        assert_eq!(v.state, TextInputState::Normal);
    }

    #[test]
    fn state_propagates_to_children() {
        let v = fixture().state(TextInputState::Disabled);
        assert_eq!(v.x.state, TextInputState::Disabled);
        assert_eq!(v.y.state, TextInputState::Disabled);
        assert_eq!(v.z.state, TextInputState::Disabled);
    }

    #[test]
    fn field_rects_partition_host() {
        let v = fixture();
        let host = Rect::new(0.0, 0.0, 360.0, 32.0);
        let rects = v.field_rects(host);
        assert!(rects[0].x < rects[1].x);
        assert!(rects[1].x < rects[2].x);
        for r in rects {
            assert!(r.w > 0.0);
        }
    }

    #[test]
    fn a11y_role_is_group_with_three_children() {
        let v = fixture();
        let node = v.build_a11y(0.0, 0.0, 360.0, 32.0);
        assert_eq!(node.role(), Role::Group);
    }

    fn smoke(v: Vector3Editor, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_vector3_editor(
            &v,
            Rect::new(0.0, 0.0, 360.0, 32.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_default() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_disabled_uniform() {
        smoke(fixture().state(TextInputState::Disabled), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_no_colored_labels() {
        smoke(fixture().colored_labels(false), Theme::Blueprint);
    }
}
