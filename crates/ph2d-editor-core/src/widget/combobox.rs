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
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
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

    /// Rect of the inline "clear ✕" icon sitting at the right edge.
    /// Returns `None` when the query is empty (nothing to clear) or
    /// the widget is disabled. Hosts use this both for hit-testing
    /// (the dispatch checks if a Down landed inside) and as the
    /// canonical geometry the painter draws into.
    pub fn clear_button_rect(&self, host: Rect) -> Option<Rect> {
        if self.query.is_empty() || self.state == ComboboxState::Disabled {
            return None;
        }
        let pad_x = Spacing::Lg.px();
        let size = (host.h * 0.5).clamp(14.0, 18.0); // LITERAL-PX-OK: clear-icon sized 50% of host height with min/max
        Some(Rect::new(
            host.x + host.w - pad_x * 0.5 - size,
            host.y + (host.h - size) * 0.5,
            size,
            size,
        ))
    }
}

pub fn paint_combobox(
    cb: &Combobox,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_combobox_with_state(cb, 0, None, rect, scene, text_system, theme);
}

/// Like [`paint_combobox`] but draws the focused-state caret + an
/// active selection range read from the caller's
/// [`crate::interaction::WidgetStore`] entry. Pass `caret = 0` /
/// `selection_anchor = None` for the unfocused / non-interactive
/// case (or just call [`paint_combobox`]).
#[allow(clippy::too_many_arguments)]
pub fn paint_combobox_with_state(
    cb: &Combobox,
    caret: usize,
    selection_anchor: Option<usize>,
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
    let icon_size = (rect.h * 0.5).clamp(14.0, 18.0); // LITERAL-PX-OK: search icon scales 50% of host with min/max
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
        StrokeToken::Default.px(),
    );
    let font_size = TypeToken::Base.px();
    let inner_x = rect.x + pad_x + icon_size + Spacing::Md.px();
    let inner_y = rect.y + (rect.h - font_size) * 0.5;
    // Reserve room for the inline clear-✕ icon at the right edge so
    // the text/caret/selection never slide under it. `clear_button_rect`
    // is the single source of truth — returns `None` when the query
    // is empty (no icon painted, no reserved gutter).
    let clear_rect = cb.clear_button_rect(rect);
    let right_reserve = clear_rect
        .map(|r| (rect.x + rect.w) - r.x + Spacing::Sm.px())
        .unwrap_or(pad_x);
    let inner_w = (rect.w - (inner_x - rect.x) - right_reserve).max(0.0);
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
    let focused = cb.state == ComboboxState::Focused;
    // Selection background drawn under the text when focused +
    // there's a non-empty selection range. Paint before the text
    // so it sits behind the glyphs.
    if focused
        && !cb.query.is_empty()
        && let Some(anchor) = selection_anchor
        && anchor != caret
    {
        let (sel_start, sel_end) = if anchor < caret {
            (anchor, caret)
        } else {
            (caret, anchor)
        };
        let sel_start = sel_start.min(cb.query.len());
        let sel_end = sel_end.min(cb.query.len());
        let prefix_w = text_system.prefix_width(&cb.query[..sel_start], font_size);
        let mid_w = if sel_start == sel_end {
            0.0
        } else {
            text_system.prefix_width(&cb.query[sel_start..sel_end], font_size)
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
        display,
        inner_x,
        inner_y,
        font_size,
        inner_w,
        resolve(label_color, theme),
    );
    // Caret line at the user's typing position, sized via real
    // text measurement so it lands between glyphs (not on top of
    // them — see `docs/UI_Bugs/README.md` §3.3).
    if focused {
        let caret_byte = caret.min(cb.query.len());
        let prefix = &cb.query[..caret_byte];
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

    // Clear-✕ icon at the right edge when the query is non-empty.
    // Single Close glyph in `Text2`; dispatch routes a Down landing
    // here through `Combobox::clear_button_rect` and zeroes the
    // query (see `dispatch::clear_combobox_if_button_hit`).
    if let Some(cr) = clear_rect {
        paint_icon(
            scene,
            IconId::Close,
            cr,
            resolve(ColorToken::Text2, theme),
            StrokeToken::Default.px(),
        );
    }

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
    fn clear_button_rect_none_when_empty() {
        let host = Rect::new(0.0, 0.0, 240.0, 32.0);
        assert!(fixture().clear_button_rect(host).is_none());
    }

    #[test]
    fn clear_button_rect_some_with_query() {
        let host = Rect::new(0.0, 0.0, 240.0, 32.0);
        let cb = fixture().query("spike");
        let r = cb.clear_button_rect(host).expect("expected clear rect");
        // Must sit on the right half of the pill.
        assert!(r.x > host.x + host.w * 0.5);
        // Vertically centered.
        let mid_y = r.y + r.h * 0.5;
        assert!((mid_y - (host.y + host.h * 0.5)).abs() < 1.0);
    }

    #[test]
    fn clear_button_rect_none_when_disabled() {
        let host = Rect::new(0.0, 0.0, 240.0, 32.0);
        let cb = fixture().query("spike").state(ComboboxState::Disabled);
        assert!(cb.clear_button_rect(host).is_none());
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
        let mut text = TextSystem::without_system_fonts();
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
        smoke(fixture(), Theme::Forge);
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
