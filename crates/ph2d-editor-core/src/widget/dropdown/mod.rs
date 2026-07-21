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
    /// A **discreet leading glyph** for lists whose rows are not all the same KIND of thing.
    ///
    /// `None` — the default, and every existing caller — paints exactly as before: the label
    /// centred in the whole row. With one, the row grows a small left gutter and the label
    /// centres in what is left.
    ///
    /// It is for *kind*, never for decoration: a list where every row would carry the same
    /// glyph has learned nothing and spent a gutter saying so.
    pub icon: Option<IconId>,
}

impl<T> DropdownOption<T> {
    pub fn new(id: NodeId, value: T, label: impl Into<String>) -> Self {
        Self {
            id,
            value,
            label: label.into(),
            icon: None,
        }
    }

    /// Mark what KIND of thing this row is (see [`Self::icon`]).
    #[must_use]
    pub fn with_icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// **Where a row's glyph goes, and what is left for its name.** One function so the two can
/// never disagree about the gutter — a label centred over the full row would sit under its
/// own icon.
///
/// The glyph's box is the row's own FONT SIZE, not a fraction of its height: it is a mark
/// beside a name, so it should be the size of the name. That also keeps it a token
/// ([`TypeToken::Base`], the same one the label is painted at) instead of a ratio somebody
/// would have to re-tune the day row heights change.
fn option_icon_split(row: Rect, has_icon: bool) -> (Rect, Rect) {
    if !has_icon {
        return (Rect::new(row.x, row.y, 0.0, 0.0), row);
    }
    let g = TypeToken::Base.px();
    let pad = POPOVER_PANEL_PAD_X + Spacing::Xs.px();
    (
        Rect::new(row.x + pad, row.y + (row.h - g) * 0.5, g, g),
        Rect::new(row.x + pad + g, row.y, (row.w - pad - g).max(0.0), row.h),
    )
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

    /// The full height the option list wants (every row + panel padding) — the **scroll content
    /// height**. When it exceeds the (clamped) popover height the list scrolls.
    #[must_use]
    pub fn content_height(&self, row_h: f32) -> f32 {
        row_h * self.options.len().max(1) as f32 + POPOVER_PANEL_PAD_Y * 2.0
    }

    /// [`Self::option_rect_in`] shifted up by `scroll` (the popover's scroll offset, `>= 0`). The hit
    /// rect a scrolled row occupies; the caller clips it to the popover.
    pub fn option_rect_in_scrolled(
        &self,
        chip: Rect,
        panel: Rect,
        index: usize,
        scroll: f32,
    ) -> Rect {
        let mut r = self.option_rect_in_panel(panel, chip.h, index);
        r.y -= scroll;
        r
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

/// **The OPEN list** — popover panel, option rows, and the scrolled variant.
///
/// The sub-folder split the 500-LOC allowlist entry promised as a follow-up since 2026-06
/// (HR-18), taken in the shape that note prescribed. It is also the honest slice: the closed
/// CHIP and the open LIST are two surfaces with two layouts, and everything above this line
/// is about the chip and the geometry both share. Adding the per-row kind glyph (ADR-0133's
/// source selector) is the caller that made the split due.
mod popover;
pub use popover::{
    DROPDOWN_SCROLLBAR_ID, opaque, paint_dropdown_popover, paint_dropdown_popover_in_viewport,
    paint_dropdown_popover_scrolled,
};
