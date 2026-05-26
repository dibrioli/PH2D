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
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
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
const POPOVER_GAP: f32 = Spacing::Xs.px();
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

    /// Popover rect below the chip. Use [`Self::popover_rect_clamped`]
    /// when the chip can sit near the screen edge.
    pub fn popover_rect(&self, chip: Rect) -> Rect {
        let h = chip.h * self.options.len().max(1) as f32 + POPOVER_PANEL_PAD_Y * 2.0;
        Rect::new(chip.x, chip.y + chip.h + POPOVER_GAP, chip.w, h)
    }

    /// Like [`Self::popover_rect`] but flips ABOVE the chip when below
    /// overflows `viewport`. Falls back to the side with more room +
    /// clamped height when neither fits.
    pub fn popover_rect_clamped(&self, chip: Rect, viewport: Rect) -> Rect {
        let row_h = chip.h;
        let wanted_h = row_h * self.options.len().max(1) as f32 + POPOVER_PANEL_PAD_Y * 2.0;
        let space_below = (viewport.y + viewport.h) - (chip.y + chip.h + POPOVER_GAP);
        let space_above = (chip.y - POPOVER_GAP) - viewport.y;
        let min_h = row_h + POPOVER_PANEL_PAD_Y * 2.0;
        if wanted_h <= space_below {
            Rect::new(chip.x, chip.y + chip.h + POPOVER_GAP, chip.w, wanted_h)
        } else if wanted_h <= space_above {
            Rect::new(chip.x, chip.y - POPOVER_GAP - wanted_h, chip.w, wanted_h)
        } else if space_below >= space_above {
            Rect::new(
                chip.x,
                chip.y + chip.h + POPOVER_GAP,
                chip.w,
                space_below.max(min_h),
            )
        } else {
            let h = space_above.max(min_h);
            Rect::new(chip.x, chip.y - POPOVER_GAP - h, chip.w, h)
        }
    }

    /// Open-list option row rect. The host rect is the *closed* chip;
    /// rows live INSIDE the popover panel (see [`Self::popover_rect`]).
    /// Tight horizontal inset so each row reads as part of the panel,
    /// not a free-floating sub-rect.
    pub fn option_rect(&self, chip: Rect, index: usize) -> Rect {
        self.option_rect_in_panel(self.popover_rect(chip), chip.h, index)
    }

    /// Same as [`Self::option_rect`] but takes a pre-computed panel
    /// rect — use this when the popover was placed via
    /// [`Self::popover_rect_clamped`] so the option rows land inside
    /// the actual (possibly flipped) popover, not the default below-
    /// chip one.
    pub fn option_rect_in(&self, chip: Rect, panel: Rect, index: usize) -> Rect {
        self.option_rect_in_panel(panel, chip.h, index)
    }

    fn option_rect_in_panel(&self, panel: Rect, row_h: f32, index: usize) -> Rect {
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
    let chevron_size = (rect.h * 0.6).clamp(14.0, 20.0); // LITERAL-PX-OK: chevron sized 60% of host height with min/max
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
    // Hard-clip ao chip rect — `paint_text` pode wrap/overflow em
    // colunas estreitas (sem clip, a label "2 Levels" escapava pra
    // fora do chip do CEQ — Enio 2026-05-26).
    let clip = ph2d_vector::Rect::new(
        rect.x as f64,
        rect.y as f64,
        (rect.x + rect.w) as f64,
        (rect.y + rect.h) as f64,
    );
    scene.push_clip(&clip);
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
    scene.pop_layer();
    let chevron_color = resolve(ColorToken::Text2, theme);
    let icon = if dd.open {
        IconId::ChevronUp
    } else {
        IconId::ChevronDown
    };
    paint_icon(
        scene,
        icon,
        chevron_rect,
        chevron_color,
        StrokeToken::Default.px(),
    );
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
    paint_dropdown_popover_in_viewport(dd, chip_rect, None, scene, text_system, theme);
}

/// Variant of [`paint_dropdown_popover`] that flips above when below
/// overflows `viewport`. Panels paint chips near the bottom of the
/// screen pass `PaintCtx::viewport` here so option lists stay on-screen.
pub fn paint_dropdown_popover_in_viewport<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    chip_rect: Rect,
    viewport: Option<Rect>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if !dd.open {
        return;
    }
    // Floating popover panel — fully opaque. The theme tokens
    // (`BgElev`, `AccentSoft`, …) carry alpha < 255 on several themes
    // so the list let underlying widgets bleed through; the user's
    // call was "tire toda transparência da lista do dropdown".
    // `opaque(token)` forces alpha to 255 while keeping the token's
    // RGB so the color still tracks the theme.
    let panel = match viewport {
        Some(vp) => dd.popover_rect_clamped(chip_rect, vp),
        None => dd.popover_rect(chip_rect),
    };
    let panel_radius = Radius::Md.px();
    fill_rounded_rect(
        scene,
        panel,
        panel_radius,
        opaque(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        scene,
        panel,
        panel_radius,
        1.0,
        opaque(ColorToken::Border, theme),
    );
    let font_size = TypeToken::Base.px();
    for (i, opt) in dd.options.iter().enumerate() {
        let r = dd.option_rect_in(chip_rect, panel, i);
        let is_selected = dd.selected.as_ref() == Some(&opt.value);
        if is_selected {
            // Use the saturated `Accent` (alpha-255 by token) for
            // the selected row instead of `AccentSoft` (which is
            // intentionally semi-transparent for inline overlays).
            fill_rounded_rect(scene, r, Radius::Sm.px(), opaque(ColorToken::Accent, theme));
        }
        let fg = if is_selected {
            ColorToken::AccentFg
        } else {
            ColorToken::Text1
        };
        paint_text_centered(
            text_system,
            scene,
            &opt.label,
            r,
            font_size,
            opaque(fg, theme),
        );
    }
}

/// Resolve `token` against `theme` and force the alpha channel to
/// `255`. Used by the dropdown popover so the floating list never
/// shows the content behind it through token-level alpha (e.g.
/// `AccentSoft` is alpha 0x29 by design — fine for inline overlays,
/// wrong for an opaque surface).
fn opaque(token: ColorToken, theme: Theme) -> ph2d_vector::Color {
    let c = token.resolve(theme);
    ph2d_vector::Color::from_rgba8(c.r, c.g, c.b, 0xFF) // LITERAL-COLOR-OK: token-bridge with forced-opaque alpha (popover must occlude content behind it)
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
        let mut text = TextSystem::without_system_fonts();
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
        smoke(fixture(), Theme::Forge);
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
        smoke(fixture().state(DropdownState::Disabled), Theme::Workshop);
    }
}
