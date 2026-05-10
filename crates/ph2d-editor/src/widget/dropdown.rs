//! [`Dropdown`] (a.k.a. Select) — single-select from a list.
//!
//! Closed: TextInput-shaped chip with a chevron at the right.
//! Open: [`super::popover`] would render the option list — but to
//! keep this widget self-contained, the dropdown owns the option
//! row layout and exposes [`Dropdown::option_rect`] for the caller
//! to hit-test.

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
pub struct DropdownOption<T> {
    pub id: NodeId,
    pub value: T,
    pub label: String,
}

impl<T> DropdownOption<T> {
    pub fn new(id: NodeId, value: T, label: impl Into<String>) -> Self {
        Self {
            id,
            value,
            label: label.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum DropdownState {
    #[default]
    Normal,
    Hovered,
    Focused,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct Dropdown<T: Clone + PartialEq> {
    pub id: NodeId,
    pub label: String,
    pub options: Vec<DropdownOption<T>>,
    pub selected: Option<T>,
    pub placeholder: String,
    pub state: DropdownState,
    pub open: bool,
}

impl<T: Clone + PartialEq> Dropdown<T> {
    pub fn new(id: NodeId, label: impl Into<String>, options: Vec<DropdownOption<T>>) -> Self {
        Self {
            id,
            label: label.into(),
            options,
            selected: None,
            placeholder: String::from("Select…"),
            state: DropdownState::Normal,
            open: false,
        }
    }

    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn selected(mut self, value: T) -> Self {
        self.select(value);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn state(mut self, state: DropdownState) -> Self {
        self.state = state;
        self
    }

    pub fn select(&mut self, value: T) {
        if self.options.iter().any(|o| o.value == value) {
            self.selected = Some(value);
        }
    }

    pub fn selected_label(&self) -> Option<&str> {
        let v = self.selected.as_ref()?;
        self.options
            .iter()
            .find(|o| &o.value == v)
            .map(|o| o.label.as_str())
    }

    /// Open-list option row rect. The host rect is the *closed* chip;
    /// rows stack downward starting at `chip.y + chip.h`.
    pub fn option_rect(&self, chip: Rect, index: usize) -> Rect {
        let row_h = chip.h;
        Rect::new(
            chip.x,
            chip.y + chip.h + row_h * index as f32,
            chip.w,
            row_h,
        )
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        // ComboBox is the AccessKit canonical for a select-style chip;
        // role flips to ListBox when the list is rendered separately.
        NodeBuilder::new(Role::ComboBox)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != DropdownState::Disabled)
            .action(Action::Click)
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
}

pub fn paint_dropdown<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    let fill = if dd.state == DropdownState::Disabled {
        ColorToken::Bg2
    } else {
        ColorToken::Bg1
    };
    fill_rounded_rect(scene, rect, radius, resolve(fill, theme));
    let border = match dd.state {
        DropdownState::Focused => ColorToken::Accent,
        DropdownState::Hovered => ColorToken::BorderEmph,
        _ => ColorToken::Border,
    };
    let stroke_w = if dd.state == DropdownState::Focused {
        2.0
    } else {
        1.0
    };
    stroke_rounded_rect(scene, rect, radius, stroke_w, resolve(border, theme));

    let pad_x = Spacing::Lg.px();
    let chevron_size = (rect.h * 0.6).clamp(14.0, 20.0);
    let chevron_rect = Rect::new(
        rect.x + rect.w - pad_x - chevron_size,
        rect.y + (rect.h - chevron_size) * 0.5,
        chevron_size,
        chevron_size,
    );
    let label_color = if dd.state == DropdownState::Disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text1
    };
    let placeholder_color = ColorToken::Text3;
    let font_size = TypeToken::Base.px();
    let inner_x = rect.x + pad_x;
    let inner_y = rect.y + (rect.h - font_size) * 0.5;
    let inner_w = (chevron_rect.x - inner_x - Spacing::Md.px()).max(0.0);
    if let Some(label) = dd.selected_label() {
        paint_text(
            text_system,
            scene,
            label,
            inner_x,
            inner_y,
            font_size,
            inner_w,
            resolve(label_color, theme),
        );
    } else {
        paint_text(
            text_system,
            scene,
            &dd.placeholder,
            inner_x,
            inner_y,
            font_size,
            inner_w,
            resolve(placeholder_color, theme),
        );
    }
    let chevron_color = resolve(ColorToken::Text2, theme);
    let icon = if dd.open {
        IconId::ChevronUp
    } else {
        IconId::ChevronDown
    };
    paint_icon(scene, icon, chevron_rect, chevron_color, 1.5);

    if dd.open {
        for (i, opt) in dd.options.iter().enumerate() {
            let r = dd.option_rect(rect, i);
            let row_token = if dd.selected.as_ref() == Some(&opt.value) {
                ColorToken::AccentSoft
            } else {
                ColorToken::BgElev
            };
            fill_rounded_rect(scene, r, radius, resolve(row_token, theme));
            paint_text_centered(
                text_system,
                scene,
                &opt.label,
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

    fn fixture() -> Dropdown<&'static str> {
        Dropdown::new(
            NodeId(1),
            "Tool",
            vec![
                DropdownOption::new(NodeId(2), "brush", "Brush"),
                DropdownOption::new(NodeId(3), "erase", "Erase"),
                DropdownOption::new(NodeId(4), "smudge", "Smudge"),
            ],
        )
    }

    #[test]
    fn defaults_match_spec() {
        let d = fixture();
        assert!(d.selected.is_none());
        assert!(!d.open);
        assert_eq!(d.placeholder, "Select…");
    }

    #[test]
    fn select_known_value() {
        let mut d = fixture();
        d.select("erase");
        assert_eq!(d.selected_label(), Some("Erase"));
    }

    #[test]
    fn select_unknown_value_silent() {
        let mut d = fixture();
        d.select("nope");
        assert!(d.selected.is_none());
    }

    #[test]
    fn a11y_role_is_combobox() {
        let node = fixture().build_a11y(0.0, 0.0, 200.0, 32.0);
        assert_eq!(node.role(), Role::ComboBox);
    }

    #[test]
    fn a11y_option_role_is_listbox_option() {
        let node = fixture()
            .build_option_a11y(0, 0.0, 0.0, 200.0, 32.0)
            .unwrap();
        assert_eq!(node.role(), Role::ListBoxOption);
    }

    fn smoke(d: Dropdown<&'static str>, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_dropdown(
            &d,
            Rect::new(0.0, 0.0, 200.0, 32.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_closed_empty() {
        smoke(fixture(), Theme::ForgeSdf);
    }

    #[test]
    fn paint_smoke_open_with_selection() {
        smoke(fixture().selected("erase").open(true), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_focused() {
        smoke(fixture().state(DropdownState::Focused), Theme::Blueprint);
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(fixture().state(DropdownState::Disabled), Theme::PaintStudio);
    }
}
