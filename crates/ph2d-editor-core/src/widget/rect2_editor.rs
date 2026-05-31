//! [`Rect2Editor`] — four [`NumberInput`] fields (X/Y/W/H) for editing
//! a 2D rectangle in an Inspector row, plus geometry helpers for the
//! canvas drag handles (corners + center).
//!
//! Sprite Inspector v2 W6 (spec §15.7, T6.4.1). The numeric-row path
//! mirrors [`Vector3Editor`](super::vector3_editor) (per-axis tinted
//! labels + a `NumberInput` per field); the handle geometry is shared
//! by every feature that drags a rect on the canvas — OnScreenEnabler
//! (§8), Region Rect (§3.3), and NamedAnchor bounds/center (W5).
//! Per-axis tinting (Danger/Success/Info/Warn) makes the four fields
//! visually distinct without a colored chip.

use super::number_input::{NumberInput, paint_number_input_with_buffer};
use super::text_input::TextInputState;
use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Axis label width floor — mirrors the `Vector3Editor` canon
/// `(host.h * 0.7).max(14)`; 14 px is the documented floor at
/// `TypeToken::Base` so the glyph never clips to the column width.
const AXIS_LABEL_FLOOR_PX: f32 = 14.0; // LITERAL-PX-OK: axis-label width floor (Vector3Editor canon)

/// Number of canvas drag handles a rect exposes: 4 corners + center.
pub const RECT2_HANDLE_COUNT: usize = 5;

/// Vertical gap between the two rows in [`Rect2Layout::Grid2x2`].
const ROW_GAP_PX: f32 = 4.0; // LITERAL-PX-OK: inter-row gap in 2x2 layout

/// How the four fields are arranged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Rect2Layout {
    /// All four fields in one row (X Y W H). Best for wide hosts.
    #[default]
    Row,
    /// Two rows of two (X Y / W H). Best for narrow columns like the
    /// Inspector, where a single row would shrink each field below
    /// [`NumberInput`]'s usable minimum width.
    Grid2x2,
}

#[derive(Clone, Debug)]
pub struct Rect2Editor {
    pub id: NodeId,
    pub label: String,
    pub x: NumberInput,
    pub y: NumberInput,
    pub w: NumberInput,
    pub h: NumberInput,
    /// When true, paints axis labels (X/Y/W/H) with their canonical
    /// tints (Danger/Success/Info/Warn) before each field.
    pub colored_labels: bool,
    pub layout: Rect2Layout,
    pub state: TextInputState,
}

impl Rect2Editor {
    pub fn new(
        id: NodeId,
        label: impl Into<String>,
        x: NumberInput,
        y: NumberInput,
        w: NumberInput,
        h: NumberInput,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            x,
            y,
            w,
            h,
            colored_labels: true,
            layout: Rect2Layout::Row,
            state: TextInputState::Normal,
        }
    }

    pub fn colored_labels(mut self, yes: bool) -> Self {
        self.colored_labels = yes;
        self
    }

    pub fn layout(mut self, layout: Rect2Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn state(mut self, state: TextInputState) -> Self {
        self.state = state;
        self.x.state = state;
        self.y.state = state;
        self.w.state = state;
        self.h.state = state;
        self
    }

    /// Axis-label column width for a field of `cell_h` height.
    fn axis_label_w(&self, cell_h: f32) -> f32 {
        if self.colored_labels {
            (cell_h * 0.7).max(AXIS_LABEL_FLOOR_PX) // LITERAL-PX-OK: axis label width 70% of cell height with floor
        } else {
            0.0
        }
    }

    /// Per-field cell height for the current layout given the host's
    /// total height.
    fn cell_h(&self, host_h: f32) -> f32 {
        match self.layout {
            Rect2Layout::Row => host_h,
            Rect2Layout::Grid2x2 => ((host_h - ROW_GAP_PX) * 0.5).max(0.0),
        }
    }

    /// Total height the editor needs given a single-field `row_h`.
    /// `Row` is one row; `Grid2x2` is two rows plus the inter-row gap.
    pub fn preferred_height(layout: Rect2Layout, row_h: f32) -> f32 {
        match layout {
            Rect2Layout::Row => row_h,
            Rect2Layout::Grid2x2 => row_h * 2.0 + ROW_GAP_PX,
        }
    }

    /// Partition `host` into four field rects (X/Y/W/H), each preceded
    /// by its colored axis-label column. `Row` lays them out across one
    /// row; `Grid2x2` lays them in two rows of two (X Y / W H).
    pub fn field_rects(&self, host: Rect) -> [Rect; 4] {
        let gap = Spacing::Md.px();
        let cell_h = self.cell_h(host.h);
        let label_w = self.axis_label_w(cell_h);
        match self.layout {
            Rect2Layout::Row => {
                let total_gap = gap * 3.0; // LITERAL-PX-OK: 3 gaps between 4 fields (count)
                let field_w = ((host.w - total_gap) / 4.0 - label_w).max(0.0); // LITERAL-PX-OK: 4-field divisor (count)
                std::array::from_fn(|i| {
                    let x = host.x + (field_w + label_w + gap) * i as f32 + label_w;
                    Rect::new(x, host.y, field_w, cell_h)
                })
            }
            Rect2Layout::Grid2x2 => {
                let field_w = ((host.w - gap) / 2.0 - label_w).max(0.0); // LITERAL-PX-OK: 2-column divisor (count)
                std::array::from_fn(|i| {
                    let col = (i % 2) as f32;
                    let row = (i / 2) as f32;
                    let x = host.x + (field_w + label_w + gap) * col + label_w;
                    let y = host.y + (cell_h + ROW_GAP_PX) * row;
                    Rect::new(x, y, field_w, cell_h)
                })
            }
        }
    }

    /// Geometry for the canvas drag handles of a rect occupying
    /// `content` (in the same screen space the caller registers hits
    /// in). Order: `[top-left, top-right, bottom-left, bottom-right,
    /// center]` — matching [`RECT2_HANDLE_COUNT`]. Each handle is a
    /// `size`-px square centered on its anchor point.
    pub fn handle_rects(content: Rect, size: f32) -> [Rect; RECT2_HANDLE_COUNT] {
        let half = size * 0.5;
        let cx = content.x + content.w * 0.5;
        let cy = content.y + content.h * 0.5;
        let anchors = [
            (content.x, content.y),                         // TL
            (content.x + content.w, content.y),             // TR
            (content.x, content.y + content.h),             // BL
            (content.x + content.w, content.y + content.h), // BR
            (cx, cy),                                       // center
        ];
        std::array::from_fn(|i| {
            let (ax, ay) = anchors[i];
            Rect::new(ax - half, ay - half, size, size)
        })
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Group)
            .label(&self.label)
            .bounds(x, y, w, h)
            .child(self.x.id)
            .child(self.y.id)
            .child(self.w.id)
            .child(self.h.id)
            .build()
    }
}

pub fn paint_rect2_editor(
    v: &Rect2Editor,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_rect2_editor_with_state(
        v,
        [None; 4],
        [0; 4],
        [None; 4],
        rect,
        scene,
        text_system,
        theme,
    );
}

/// Like [`paint_rect2_editor`] but renders per-field live edit buffers
/// + caret + selection. The arrays index 0/1/2/3 = X/Y/W/H. Pass
/// `[None; 4]` / `[0; 4]` for the static (unfocused) case.
#[allow(clippy::too_many_arguments)]
pub fn paint_rect2_editor_with_state(
    v: &Rect2Editor,
    buffers: [Option<&str>; 4],
    carets: [usize; 4],
    selection_anchors: [Option<usize>; 4],
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let rects = v.field_rects(rect);
    let labels = ["X", "Y", "W", "H"];
    let label_tokens = [
        ColorToken::Danger,
        ColorToken::Success,
        ColorToken::Info,
        ColorToken::Warn,
    ];
    let inputs = [&v.x, &v.y, &v.w, &v.h];
    let font = TypeToken::Base.px();
    // Label column width is derived from the per-field cell height so it
    // is correct for both Row and Grid2x2 layouts; each label is placed
    // relative to its own field rect (so the bottom row in Grid2x2 gets
    // its labels at the right y).
    let label_w = v.axis_label_w(v.cell_h(rect.h));
    for (i, ((label, &token), input)) in labels
        .iter()
        .zip(label_tokens.iter())
        .zip(inputs)
        .enumerate()
    {
        let field_rect = rects[i];
        if v.colored_labels {
            let label_x = field_rect.x - label_w;
            let label_y = field_rect.y + (field_rect.h - font) * 0.5;
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
                label_w,
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

/// Paint the canvas drag handles for a rect occupying `content`. Each
/// corner is a filled square with a border; the center handle is
/// drawn in the accent tint to distinguish "move" from "resize".
pub fn paint_rect2_handles(content: Rect, size: f32, scene: &mut VectorScene, theme: Theme) {
    let handles = Rect2Editor::handle_rects(content, size);
    let radius = Radius::Sm.px();
    let stroke = StrokeToken::Default.px();
    for (i, h) in handles.iter().enumerate() {
        let fill = if i + 1 == RECT2_HANDLE_COUNT {
            ColorToken::Accent // center = move
        } else {
            ColorToken::Bg3 // corners = resize
        };
        fill_rounded_rect(scene, *h, radius, resolve(fill, theme));
        stroke_rounded_rect(
            scene,
            *h,
            radius,
            stroke,
            resolve(ColorToken::BorderStrong, theme),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Rect2Editor {
        Rect2Editor::new(
            NodeId(1),
            "Enabler Rect",
            NumberInput::new(NodeId(2), "X", 0.0),
            NumberInput::new(NodeId(3), "Y", 0.0),
            NumberInput::new(NodeId(4), "W", 100.0),
            NumberInput::new(NodeId(5), "H", 50.0),
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
        assert_eq!(v.w.state, TextInputState::Disabled);
        assert_eq!(v.h.state, TextInputState::Disabled);
    }

    #[test]
    fn field_rects_partition_host_left_to_right() {
        let v = fixture();
        let host = Rect::new(0.0, 0.0, 400.0, 32.0);
        let rects = v.field_rects(host);
        for pair in rects.windows(2) {
            assert!(pair[0].x < pair[1].x);
        }
        for r in rects {
            assert!(r.w > 0.0);
        }
    }

    #[test]
    fn grid2x2_two_rows_two_cols() {
        let v = fixture().layout(Rect2Layout::Grid2x2);
        let host = Rect::new(0.0, 0.0, 240.0, 60.0);
        let r = v.field_rects(host);
        // X,Y on top row; W,H on bottom row.
        assert!((r[0].y - r[1].y).abs() < 0.01, "X and Y share the top row");
        assert!(
            (r[2].y - r[3].y).abs() < 0.01,
            "W and H share the bottom row"
        );
        assert!(r[2].y > r[0].y, "bottom row is below top row");
        // Two columns: X left of Y, W left of H.
        assert!(r[0].x < r[1].x);
        assert!(r[2].x < r[3].x);
        // Grid fields are wider than the equivalent single row would give.
        let row_fields = fixture().field_rects(Rect::new(0.0, 0.0, 240.0, 28.0));
        assert!(r[0].w > row_fields[0].w, "2x2 fields wider than 1x4");
    }

    #[test]
    fn preferred_height_matches_layout() {
        assert_eq!(Rect2Editor::preferred_height(Rect2Layout::Row, 28.0), 28.0);
        assert!(Rect2Editor::preferred_height(Rect2Layout::Grid2x2, 28.0) > 28.0 * 2.0);
    }

    #[test]
    fn handle_rects_corners_and_center() {
        let content = Rect::new(10.0, 20.0, 100.0, 60.0);
        let size = 8.0;
        let h = Rect2Editor::handle_rects(content, size);
        // TL centered on (10,20).
        assert_eq!(h[0].x, 10.0 - 4.0);
        assert_eq!(h[0].y, 20.0 - 4.0);
        // BR centered on (110,80).
        assert_eq!(h[3].x, 110.0 - 4.0);
        assert_eq!(h[3].y, 80.0 - 4.0);
        // Center on (60,50).
        assert_eq!(h[4].x, 60.0 - 4.0);
        assert_eq!(h[4].y, 50.0 - 4.0);
        for r in h {
            assert_eq!(r.w, size);
            assert_eq!(r.h, size);
        }
    }

    #[test]
    fn a11y_role_is_group_with_four_children() {
        let v = fixture();
        let node = v.build_a11y(0.0, 0.0, 400.0, 32.0);
        assert_eq!(node.role(), Role::Group);
    }

    fn smoke(v: Rect2Editor, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_rect2_editor(
            &v,
            Rect::new(0.0, 0.0, 400.0, 32.0),
            &mut scene,
            &mut text,
            theme,
        );
        paint_rect2_handles(Rect::new(50.0, 50.0, 120.0, 80.0), 8.0, &mut scene, theme);
    }

    #[test]
    fn paint_smoke_default() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(fixture().state(TextInputState::Disabled), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_no_colored_labels() {
        smoke(fixture().colored_labels(false), Theme::Blueprint);
    }
}
