//! [`RadioGroup`] — single-select among `T` options.
//!
//! Same pattern as [`crate::widget::Button`]: data + per-option a11y
//! `Role::RadioButton` + `paint_radio_group` colocated. Options carry
//! their own `NodeId` so each radio gets its own a11y node, but the
//! group itself is identified by the outer `id`.
//!
//! Layout splits the host rect by `orientation` — Horizontal divides
//! width evenly, Vertical divides height. Selected paints with
//! `AccentPrimary`; unselected with `Surface`. Borders are faked via
//! a thin gap (no stroke API on `VectorScene`'s convenience layer).

use crate::paint::{rect_to_vello, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum RadioOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub struct RadioOption<T> {
    pub value: T,
    pub label: String,
    pub id: NodeId,
}

impl<T> RadioOption<T> {
    pub fn new(id: NodeId, value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RadioGroup<T: Clone + PartialEq> {
    pub id: NodeId,
    pub label: String,
    pub options: Vec<RadioOption<T>>,
    pub selected: Option<T>,
    pub orientation: RadioOrientation,
}

impl<T: Clone + PartialEq> RadioGroup<T> {
    pub fn new(id: NodeId, label: impl Into<String>, options: Vec<RadioOption<T>>) -> Self {
        Self {
            id,
            label: label.into(),
            options,
            selected: None,
            orientation: RadioOrientation::Horizontal,
        }
    }

    pub fn orientation(mut self, o: RadioOrientation) -> Self {
        self.orientation = o;
        self
    }

    pub fn selected(mut self, value: T) -> Self {
        self.select(value);
        self
    }

    /// Set the selection to `value` iff it appears in `options`.
    /// Silently no-ops on an unknown value (per Procreate UX: invalid
    /// state shouldn't crash the editor).
    pub fn select(&mut self, value: T) {
        if self.options.iter().any(|opt| opt.value == value) {
            self.selected = Some(value);
        }
    }

    /// Build the group container a11y node. Per ADR-0023 §10:
    /// `Role::RadioGroup` with each option as a child `Role::RadioButton`.
    /// The group itself takes the host rect; per-option nodes are built
    /// separately by `build_option_a11y`.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::RadioGroup)
            .label(&self.label)
            .bounds(x, y, w, h)
            .children(self.options.iter().map(|o| o.id))
            .build()
    }

    /// Build a single radio option's a11y node. Caller supplies the
    /// option's pixel rect (computed via `option_rect`).
    pub fn build_option_a11y(&self, index: usize, x: f64, y: f64, w: f64, h: f64) -> Option<Node> {
        let opt = self.options.get(index)?;
        let is_selected = self.selected.as_ref() == Some(&opt.value);
        Some(
            NodeBuilder::new(Role::RadioButton)
                .label(&opt.label)
                .bounds(x, y, w, h)
                .focusable(true)
                .action(Action::Click)
                .toggled(if is_selected {
                    ph2d_a11y::Toggled::True
                } else {
                    ph2d_a11y::Toggled::False
                })
                .build(),
        )
    }

    /// Compute the sub-rect for the option at `index` inside `host`.
    pub fn option_rect(&self, host: Rect, index: usize) -> Rect {
        let count = self.options.len().max(1) as f32;
        match self.orientation {
            RadioOrientation::Horizontal => {
                let w = host.w / count;
                Rect::new(host.x + w * index as f32, host.y, w, host.h)
            }
            RadioOrientation::Vertical => {
                let h = host.h / count;
                Rect::new(host.x, host.y + h * index as f32, host.w, h)
            }
        }
    }
}

/// Each option becomes a sub-rect filled with `AccentPrimary` if
/// selected, else `Surface`. A 1 px gap of `Border` color between
/// options stands in for a real stroke (which `VectorScene` doesn't
/// expose at the convenience layer).
pub fn paint_radio_group<T: Clone + PartialEq>(
    group: &RadioGroup<T>,
    rect: Rect,
    scene: &mut VectorScene,
    theme: Theme,
) {
    // Backdrop painted with Border color so the seams between options
    // read as separators when we leave a 1 px gap on each option.
    scene.fill_rect(rect_to_vello(rect), resolve(ColorToken::Border, theme));

    for (i, opt) in group.options.iter().enumerate() {
        let r = group.option_rect(rect, i);
        // Inset 1 px so the Border backdrop shows through as a seam.
        let inset = Rect::new(
            r.x + 1.0,
            r.y + 1.0,
            (r.w - 2.0).max(0.0),
            (r.h - 2.0).max(0.0),
        );
        let token = if group.selected.as_ref() == Some(&opt.value) {
            ColorToken::Accent
        } else {
            ColorToken::Bg1
        };
        scene.fill_rect(rect_to_vello(inset), resolve(token, theme));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_group() -> RadioGroup<&'static str> {
        RadioGroup::new(
            NodeId(10),
            "Mode",
            vec![
                RadioOption::new(NodeId(11), "draw", "Draw"),
                RadioOption::new(NodeId(12), "erase", "Erase"),
                RadioOption::new(NodeId(13), "smudge", "Smudge"),
            ],
        )
    }

    #[test]
    fn defaults_are_unselected_and_horizontal() {
        let g = sample_group();
        assert_eq!(g.selected, None);
        assert_eq!(g.orientation, RadioOrientation::Horizontal);
        assert_eq!(g.options.len(), 3);
    }

    #[test]
    fn select_known_value_takes_effect() {
        let mut g = sample_group();
        g.select("erase");
        assert_eq!(g.selected, Some("erase"));
    }

    #[test]
    fn select_unknown_value_is_silently_ignored() {
        let mut g = sample_group();
        g.select("nope");
        assert_eq!(g.selected, None);
    }

    #[test]
    fn select_can_be_changed() {
        let mut g = sample_group();
        g.select("draw");
        g.select("smudge");
        assert_eq!(g.selected, Some("smudge"));
    }

    #[test]
    fn orientation_horizontal_splits_width() {
        let g = sample_group();
        let host = Rect::new(0.0, 0.0, 300.0, 30.0);
        let r0 = g.option_rect(host, 0);
        let r1 = g.option_rect(host, 1);
        let r2 = g.option_rect(host, 2);
        assert!((r0.w - 100.0).abs() < f32::EPSILON);
        assert_eq!(r0.x, 0.0);
        assert_eq!(r1.x, 100.0);
        assert_eq!(r2.x, 200.0);
        assert_eq!(r0.h, 30.0);
    }

    #[test]
    fn orientation_vertical_splits_height() {
        let g = sample_group().orientation(RadioOrientation::Vertical);
        let host = Rect::new(0.0, 0.0, 100.0, 90.0);
        let r0 = g.option_rect(host, 0);
        let r1 = g.option_rect(host, 1);
        assert_eq!(r0.h, 30.0);
        assert_eq!(r0.y, 0.0);
        assert_eq!(r1.y, 30.0);
        assert_eq!(r1.w, 100.0);
    }

    #[test]
    fn group_a11y_has_radio_group_role_and_children() {
        let g = sample_group();
        let node = g.build_a11y(0.0, 0.0, 300.0, 30.0);
        assert_eq!(node.role(), Role::RadioGroup);
        assert_eq!(node.label(), Some("Mode"));
    }

    #[test]
    fn option_a11y_has_radio_button_role_and_toggled() {
        let mut g = sample_group();
        g.select("draw");
        let node = g
            .build_option_a11y(0, 0.0, 0.0, 100.0, 30.0)
            .expect("option exists");
        assert_eq!(node.role(), Role::RadioButton);
        assert_eq!(node.label(), Some("Draw"));
        assert_eq!(node.toggled(), Some(ph2d_a11y::Toggled::True));

        let off = g
            .build_option_a11y(1, 100.0, 0.0, 100.0, 30.0)
            .expect("option exists");
        assert_eq!(off.toggled(), Some(ph2d_a11y::Toggled::False));
    }

    #[test]
    fn paint_smoke_unselected() {
        let g = sample_group();
        let mut scene = VectorScene::new();
        paint_radio_group(
            &g,
            Rect::new(0.0, 0.0, 100.0, 30.0),
            &mut scene,
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn paint_smoke_selected_vertical() {
        let mut g = sample_group().orientation(RadioOrientation::Vertical);
        g.select("smudge");
        let mut scene = VectorScene::new();
        paint_radio_group(
            &g,
            Rect::new(0.0, 0.0, 100.0, 90.0),
            &mut scene,
            Theme::Sunstone,
        );
    }
}
