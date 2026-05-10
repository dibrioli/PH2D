//! [`Combobox`] — editable dropdown. TextInput on top + filtered
//! [`Dropdown`]-style list when `open`.
//!
//! Filter logic is case-insensitive substring match against
//! `query`. The widget itself is data-only — keystroke handling is
//! shell-side; we expose [`Combobox::filtered`] so the paint pass and
//! hit-test path share one source of truth.

use crate::icons::IconId;
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text, paint_text_centered, resolve, stroke_rounded_rect,
};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct ComboboxOption {
    pub id: NodeId,
    pub label: String,
}

impl ComboboxOption {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ComboboxState {
    #[default]
    Normal,
    Focused,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct Combobox {
    pub id: NodeId,
    pub label: String,
    pub options: Vec<ComboboxOption>,
    pub query: String,
    pub state: ComboboxState,
    pub open: bool,
}

impl Combobox {
    pub fn new(id: NodeId, label: impl Into<String>, options: Vec<ComboboxOption>) -> Self {
        Self {
            id,
            label: label.into(),
            options,
            query: String::new(),
            state: ComboboxState::Normal,
            open: false,
        }
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn state(mut self, state: ComboboxState) -> Self {
        self.state = state;
        self
    }

    /// Indices of options matching `query` (case-insensitive
    /// substring). Empty query matches all.
    pub fn filtered(&self) -> Vec<usize> {
        if self.query.is_empty() {
            return (0..self.options.len()).collect();
        }
        let needle = self.query.to_lowercase();
        self.options
            .iter()
            .enumerate()
            .filter(|(_, opt)| opt.label.to_lowercase().contains(&needle))
            .map(|(i, _)| i)
            .collect()
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::ComboBox)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != ComboboxState::Disabled)
            .action(Action::Focus)
            .children(self.options.iter().map(|o| o.id))
            .build()
    }

    pub fn build_option_a11y(&self, index: usize, x: f64, y: f64, w: f64, h: f64) -> Option<Node> {
        let opt = self.options.get(index)?;
        Some(
            NodeBuilder::new(Role::ListBoxOption)
                .label(&opt.label)
                .bounds(x, y, w, h)
                .focusable(true)
                .action(Action::Click)
                .build(),
        )
    }

    pub fn option_rect(&self, chip: Rect, visible_index: usize) -> Rect {
        let row_h = chip.h;
        Rect::new(
            chip.x,
            chip.y + chip.h + row_h * visible_index as f32,
            chip.w,
            row_h,
        )
    }
}

pub fn paint_combobox(
    cb: &Combobox,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    let fill = if cb.state == ComboboxState::Disabled {
        ColorToken::Bg2
    } else {
        ColorToken::Bg1
    };
    fill_rounded_rect(scene, rect, radius, resolve(fill, theme));
    let border = if cb.state == ComboboxState::Focused {
        ColorToken::Accent
    } else {
        ColorToken::Border
    };
    let stroke_w = if cb.state == ComboboxState::Focused {
        2.0
    } else {
        1.0
    };
    stroke_rounded_rect(scene, rect, radius, stroke_w, resolve(border, theme));

    let pad_x = Spacing::Lg.px();
    let icon_size = (rect.h * 0.5).clamp(14.0, 18.0);
    let search_rect = Rect::new(
        rect.x + pad_x * 0.5,
        rect.y + (rect.h - icon_size) * 0.5,
        icon_size,
        icon_size,
    );
    paint_icon(
        scene,
        IconId::Search,
        search_rect,
        resolve(ColorToken::Text2, theme),
        1.5,
    );
    let font_size = TypeToken::Base.px();
    let inner_x = rect.x + pad_x + icon_size + Spacing::Md.px();
    let inner_y = rect.y + (rect.h - font_size) * 0.5;
    let inner_w = (rect.w - (inner_x - rect.x) - pad_x).max(0.0);
    let label_color = if cb.query.is_empty() {
        ColorToken::Text3
    } else {
        ColorToken::Text1
    };
    let display = if cb.query.is_empty() {
        "Search…"
    } else {
        cb.query.as_str()
    };
    paint_text(
        text_system,
        scene,
        display,
        inner_x,
        inner_y,
        font_size,
        inner_w,
        resolve(label_color, theme),
    );

    if cb.open {
        for (visible, &index) in cb.filtered().iter().enumerate() {
            let r = cb.option_rect(rect, visible);
            fill_rounded_rect(scene, r, radius, resolve(ColorToken::BgElev, theme));
            paint_text_centered(
                text_system,
                scene,
                &cb.options[index].label,
                r,
                font_size,
                resolve(ColorToken::Text1, theme),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Combobox {
        Combobox::new(
            NodeId(1),
            "Asset",
            vec![
                ComboboxOption::new(NodeId(2), "spike.gltf"),
                ComboboxOption::new(NodeId(3), "spike-tex.png"),
                ComboboxOption::new(NodeId(4), "block.gltf"),
                ComboboxOption::new(NodeId(5), "block-tex.png"),
            ],
        )
    }

    #[test]
    fn empty_query_returns_all() {
        assert_eq!(fixture().filtered().len(), 4);
    }

    #[test]
    fn query_filters_substring_case_insensitive() {
        let cb = fixture().query("SPIKE");
        let f = cb.filtered();
        assert_eq!(f.len(), 2);
        assert!(f.contains(&0));
        assert!(f.contains(&1));
    }

    #[test]
    fn query_with_no_matches() {
        let cb = fixture().query("zzz");
        assert_eq!(cb.filtered().len(), 0);
    }

    #[test]
    fn a11y_role_is_combobox() {
        let node = fixture().build_a11y(0.0, 0.0, 200.0, 32.0);
        assert_eq!(node.role(), Role::ComboBox);
    }

    fn smoke(c: Combobox, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_combobox(
            &c,
            Rect::new(0.0, 0.0, 240.0, 32.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_empty_closed() {
        smoke(fixture(), Theme::ForgeSdf);
    }

    #[test]
    fn paint_smoke_open_with_query() {
        smoke(
            fixture()
                .query("spike")
                .open(true)
                .state(ComboboxState::Focused),
            Theme::Sunstone,
        );
    }
}
