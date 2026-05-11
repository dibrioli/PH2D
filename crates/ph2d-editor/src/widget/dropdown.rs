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

/// Vertical gap between the chip's bottom edge and the open
/// popover panel's top edge. Keeps the two surfaces visually
/// distinct (without it the chip's border merges into the panel
/// border on themes where both use the same Border token).
const POPOVER_GAP: f32 = 4.0;
/// Inner padding inside the open popover panel — option rows live
/// inside this margin. Tight on purpose so the list doesn't look
/// "shifted down" inside an oversized panel (per the user's report).
const POPOVER_PANEL_PAD_X: f32 = 2.0;
const POPOVER_PANEL_PAD_Y: f32 = 2.0;

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

    /// Floating popover panel that wraps every option row when the
    /// dropdown is open. Sits just below the chip with a small
    /// breathing gap so the chip's bottom border + the panel's top
    /// border don't fuse into a single line. Tight: only the
    /// `POPOVER_PANEL_PAD_Y` of internal vertical room — option rows
    /// touch the panel chrome edge-to-edge minus that small pad.
    pub fn popover_rect(&self, chip: Rect) -> Rect {
        let row_h = chip.h;
        let count = self.options.len().max(1) as f32;
        let h = row_h * count + POPOVER_PANEL_PAD_Y * 2.0;
        Rect::new(chip.x, chip.y + chip.h + POPOVER_GAP, chip.w, h)
    }

    /// Open-list option row rect. The host rect is the *closed* chip;
    /// rows live INSIDE the popover panel (see [`Self::popover_rect`]).
    /// Tight horizontal inset so each row reads as part of the panel,
    /// not a free-floating sub-rect.
    pub fn option_rect(&self, chip: Rect, index: usize) -> Rect {
        let panel = self.popover_rect(chip);
        let row_h = chip.h;
        Rect::new(
            panel.x + POPOVER_PANEL_PAD_X,
            panel.y + POPOVER_PANEL_PAD_Y + row_h * index as f32,
            (panel.w - POPOVER_PANEL_PAD_X * 2.0).max(0.0),
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

/// Convenience: paint the chip AND, if open, the popover in one
/// call. Most call sites use this. The Inspector splits the two via
/// [`paint_dropdown_chip`] + [`paint_dropdown_popover`] so the
/// popover lands on top of every later section (paint order matters
/// — see `docs/UI_Bugs/README.md` §9.16).
pub fn paint_dropdown<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_dropdown_chip(dd, rect, scene, text_system, theme);
    if dd.open {
        paint_dropdown_popover(dd, rect, scene, text_system, theme);
    }
}

/// Paint just the chip (no popover) so the caller can defer the
/// popover to a later z-order pass.
pub fn paint_dropdown_chip<T: Clone + PartialEq>(
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
}

/// Paint only the open option list (no chip). Caller is responsible
/// for invoking this AFTER any other content that might sit at the
/// same Y so the popover stays on top (the inspector calls this in
/// its second pass, after every section has painted).
///
/// No-op when `!dd.open`.
pub fn paint_dropdown_popover<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    chip_rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if !dd.open {
        return;
    }
    // Floating popover panel: opaque `BgElev` fill + `Border`
    // stroke + Md radius so the list reads unambiguously as a
    // separate surface hovering above whatever sits behind it.
    let panel = dd.popover_rect(chip_rect);
    let panel_radius = Radius::Md.px();
    fill_rounded_rect(scene, panel, panel_radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(
        scene,
        panel,
        panel_radius,
        1.0,
        resolve(ColorToken::Border, theme),
    );
    let font_size = TypeToken::Base.px();
    for (i, opt) in dd.options.iter().enumerate() {
        let r = dd.option_rect(chip_rect, i);
        let is_selected = dd.selected.as_ref() == Some(&opt.value);
        // Selected row gets `AccentSoft`; rows inherit the panel
        // `BgElev` otherwise (no per-row fill so the panel reads
        // as a single contiguous surface).
        if is_selected {
            fill_rounded_rect(
                scene,
                r,
                Radius::Sm.px(),
                resolve(ColorToken::AccentSoft, theme),
            );
        }
        let fg = if is_selected {
            ColorToken::Accent
        } else {
            ColorToken::Text1
        };
        paint_text_centered(text_system, scene, &opt.label, r, font_size, resolve(fg, theme));
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
    fn popover_rect_sits_below_chip_with_gap() {
        let d = fixture();
        let chip = Rect::new(10.0, 20.0, 200.0, 32.0);
        let pop = d.popover_rect(chip);
        assert!(pop.y > chip.y + chip.h, "popover must be below chip");
        // Gap is non-zero so the two surfaces don't fuse into one.
        assert!((pop.y - (chip.y + chip.h)) >= 1.0);
        assert_eq!(pop.x, chip.x);
        assert_eq!(pop.w, chip.w);
    }

    #[test]
    fn option_rect_inset_inside_popover() {
        let d = fixture();
        let chip = Rect::new(10.0, 20.0, 200.0, 32.0);
        let pop = d.popover_rect(chip);
        let r0 = d.option_rect(chip, 0);
        // Rows must live INSIDE the popover panel.
        assert!(r0.x > pop.x);
        assert!(r0.x + r0.w < pop.x + pop.w);
        assert!(r0.y >= pop.y);
        assert!(r0.y + r0.h <= pop.y + pop.h + 0.001);
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
