//! [`ListItem`] — single row in a vertical list (asset browser,
//! hierarchy panel, context menus).
//!
//! Layout: optional left icon + label + optional right value text +
//! optional chevron. Selected rows fill with `AccentSoft`. Density
//! is consumer-driven (the host sets row height via `ph2d-tokens`
//! `ROW_H_PX[Density]`).

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ListItemState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct ListItem {
    pub id: NodeId,
    pub label: String,
    /// Right-aligned secondary text (key shortcut, file size, etc).
    pub value: Option<String>,
    pub leading_icon: Option<IconId>,
    pub trailing_chevron: bool,
    pub selected: bool,
    pub state: ListItemState,
}

impl ListItem {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            value: None,
            leading_icon: None,
            trailing_chevron: false,
            selected: false,
            state: ListItemState::Normal,
        }
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = Some(v.into());
        self
    }

    pub fn chevron(mut self, yes: bool) -> Self {
        self.trailing_chevron = yes;
        self
    }

    pub fn selected(mut self, yes: bool) -> Self {
        self.selected = yes;
        self
    }

    pub fn state(mut self, state: ListItemState) -> Self {
        self.state = state;
        self
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::ListItem)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != ListItemState::Disabled)
            .action(Action::Click)
            .build()
    }
}

pub fn paint_list_item(
    item: &ListItem,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Xs.px();
    let bg = match (item.selected, item.state) {
        (true, _) => Some(ColorToken::AccentSoft),
        (false, ListItemState::Hovered | ListItemState::Focused) => Some(ColorToken::Bg2),
        (false, ListItemState::Pressed) => Some(ColorToken::AccentPress),
        _ => None,
    };
    if let Some(token) = bg {
        fill_rounded_rect(scene, rect, radius, resolve(token, theme));
    }

    let pad_x = Spacing::Lg.px();
    let icon_w = (rect.h * 0.6).clamp(14.0, 20.0); // LITERAL-PX-OK: list icon sized 60% of row height with min/max
    let mut cursor_x = rect.x + pad_x;
    if let Some(icon) = item.leading_icon {
        let icon_rect = Rect::new(cursor_x, rect.y + (rect.h - icon_w) * 0.5, icon_w, icon_w);
        let icon_color = if item.state == ListItemState::Disabled {
            ColorToken::TextDisabled
        } else if item.selected {
            ColorToken::Accent
        } else {
            ColorToken::Text2
        };
        paint_icon(
            scene,
            icon,
            icon_rect,
            resolve(icon_color, theme),
            StrokeToken::Default.px(),
        );
        cursor_x += icon_w + Spacing::Md.px();
    }

    let chevron_w = if item.trailing_chevron { icon_w } else { 0.0 };
    // Real text measurement for the right-aligned value pill — the
    // `len * 7` heuristic underestimated wide glyphs and overlapped
    // the label on proportional fonts (`docs/UI_Bugs/README.md` §3.3).
    let value_font = TypeToken::Sm.px();
    let value_w = item
        .value
        .as_ref()
        .map(|v| text_system.layout(v, value_font, f32::INFINITY).width())
        .unwrap_or(0.0);
    let label_w =
        (rect.x + rect.w - cursor_x - pad_x - chevron_w - value_w - Spacing::Lg.px()).max(0.0);
    let font_size = TypeToken::Base.px();
    let label_y = rect.y + (rect.h - font_size) * 0.5;
    let label_color = if item.state == ListItemState::Disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text1
    };
    paint_text(
        text_system,
        scene,
        &item.label,
        cursor_x,
        label_y,
        font_size,
        label_w,
        resolve(label_color, theme),
    );

    if let Some(value) = &item.value {
        let value_x = rect.x + rect.w - pad_x - chevron_w - value_w;
        paint_text(
            text_system,
            scene,
            value,
            value_x,
            label_y,
            value_font,
            value_w,
            resolve(ColorToken::Text2, theme),
        );
    }

    if item.trailing_chevron {
        let chev_rect = Rect::new(
            rect.x + rect.w - pad_x - icon_w,
            rect.y + (rect.h - icon_w) * 0.5,
            icon_w,
            icon_w,
        );
        paint_icon(
            scene,
            IconId::ChevronRight,
            chev_rect,
            resolve(ColorToken::Text3, theme),
            StrokeToken::Default.px(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ListItem {
        ListItem::new(NodeId(1), "Brush")
    }

    #[test]
    fn defaults_match_spec() {
        let i = fixture();
        assert!(i.value.is_none());
        assert!(i.leading_icon.is_none());
        assert!(!i.selected);
        assert!(!i.trailing_chevron);
    }

    #[test]
    fn builder_chain_sets_fields() {
        let i = fixture()
            .icon(IconId::Sprite)
            .value("⌘B")
            .chevron(true)
            .selected(true);
        assert_eq!(i.leading_icon, Some(IconId::Sprite));
        assert_eq!(i.value.as_deref(), Some("⌘B"));
        assert!(i.trailing_chevron);
        assert!(i.selected);
    }

    #[test]
    fn a11y_role_is_list_item() {
        let node = fixture().build_a11y(0.0, 0.0, 200.0, 26.0);
        assert_eq!(node.role(), Role::ListItem);
    }

    fn smoke(item: ListItem, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_list_item(
            &item,
            Rect::new(0.0, 0.0, 240.0, 26.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_plain() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_full_decoration() {
        smoke(
            fixture()
                .icon(IconId::Sprite)
                .value("⌘B")
                .chevron(true)
                .selected(true),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_hovered() {
        smoke(fixture().state(ListItemState::Hovered), Theme::Blueprint);
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(fixture().state(ListItemState::Disabled), Theme::Workshop);
    }
}
