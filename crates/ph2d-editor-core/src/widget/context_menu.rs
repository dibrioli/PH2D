//! [`ContextMenu`] — vertical list of [`super::list_item::ListItem`]
//! entries with optional separators (rendered via
//! [`super::divider`]).
//!
//! Lives in `Layer::Overlay`. AccessKit `Role::Menu` (parent) +
//! `Role::MenuItem` per row.

use super::divider::{Divider, paint_divider};
use super::list_item::{ListItem, ListItemState, paint_list_item};
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub enum ContextMenuEntry {
    Item(ListItem),
    Separator(Divider),
}

#[derive(Clone, Debug)]
pub struct ContextMenu {
    pub id: NodeId,
    pub label: String,
    pub entries: Vec<ContextMenuEntry>,
}

impl ContextMenu {
    pub fn new(id: NodeId, label: impl Into<String>, entries: Vec<ContextMenuEntry>) -> Self {
        Self {
            id,
            label: label.into(),
            entries,
        }
    }

    /// Sum of every entry's preferred row height (separators count
    /// as 1px, items as `row_h`).
    pub fn preferred_height(&self, row_h: f32) -> f32 {
        let pad = Spacing::Md.px() * 2.0;
        let rows: f32 = self
            .entries
            .iter()
            .map(|e| match e {
                ContextMenuEntry::Item(_) => row_h,
                ContextMenuEntry::Separator(_) => 1.0 + Spacing::Xs.px() * 2.0,
            })
            .sum();
        rows + pad
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let item_ids = self.entries.iter().filter_map(|e| match e {
            ContextMenuEntry::Item(item) => Some(item.id),
            ContextMenuEntry::Separator(_) => None,
        });
        NodeBuilder::new(Role::Menu)
            .label(&self.label)
            .bounds(x, y, w, h)
            .children(item_ids)
            .build()
    }

    pub fn build_item_a11y(&self, index: usize, x: f64, y: f64, w: f64, h: f64) -> Option<Node> {
        match self.entries.get(index)? {
            ContextMenuEntry::Item(item) => Some(
                NodeBuilder::new(Role::MenuItem)
                    .label(&item.label)
                    .bounds(x, y, w, h)
                    .focusable(item.state != ListItemState::Disabled)
                    .action(Action::Click)
                    .build(),
            ),
            ContextMenuEntry::Separator(_) => None,
        }
    }

    /// Rect of the `index`th entry given the menu's host rect + row
    /// height. Returns `None` for out-of-range indices or separators
    /// (separators aren't interactive). Hosts use this to register
    /// hit rects without re-walking the layout the painter does.
    pub fn entry_rect(&self, host: Rect, index: usize, row_h: f32) -> Option<Rect> {
        let pad = Spacing::Md.px();
        let inner_x = host.x + pad;
        let inner_w = (host.w - pad * 2.0).max(0.0);
        let mut y = host.y + pad;
        for (i, entry) in self.entries.iter().enumerate() {
            match entry {
                ContextMenuEntry::Item(_) => {
                    if i == index {
                        return Some(Rect::new(inner_x, y, inner_w, row_h));
                    }
                    y += row_h;
                }
                ContextMenuEntry::Separator(_) => {
                    if i == index {
                        return None;
                    }
                    y += 1.0 + Spacing::Xs.px() * 2.0;
                }
            }
        }
        None
    }
}

pub fn paint_context_menu(
    menu: &ContextMenu,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    row_h: f32,
) {
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    let pad = Spacing::Md.px();
    let mut y = rect.y + pad;
    for entry in &menu.entries {
        match entry {
            ContextMenuEntry::Item(item) => {
                let row = Rect::new(rect.x + pad, y, rect.w - pad * 2.0, row_h);
                paint_list_item(item, row, scene, text_system, theme);
                y += row_h;
            }
            ContextMenuEntry::Separator(div) => {
                y += Spacing::Xs.px();
                let row = Rect::new(rect.x + pad, y, rect.w - pad * 2.0, 1.0);
                paint_divider(div, row, scene, theme);
                y += 1.0 + Spacing::Xs.px();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons::IconId;

    fn fixture() -> ContextMenu {
        ContextMenu::new(
            NodeId(1),
            "File",
            vec![
                ContextMenuEntry::Item(
                    ListItem::new(NodeId(2), "Open")
                        .icon(IconId::Open)
                        .value("⌘O"),
                ),
                ContextMenuEntry::Item(
                    ListItem::new(NodeId(3), "Save")
                        .icon(IconId::Save)
                        .value("⌘S"),
                ),
                ContextMenuEntry::Separator(Divider::new(NodeId(4))),
                ContextMenuEntry::Item(
                    ListItem::new(NodeId(5), "Quit")
                        .icon(IconId::Close)
                        .value("⌘Q"),
                ),
            ],
        )
    }

    #[test]
    fn preferred_height_includes_separators() {
        let h = fixture().preferred_height(26.0);
        assert!(h > 26.0 * 3.0); // 3 items at minimum
    }

    #[test]
    fn a11y_parent_is_menu() {
        let node = fixture().build_a11y(0.0, 0.0, 200.0, 200.0);
        assert_eq!(node.role(), Role::Menu);
    }

    #[test]
    fn a11y_item_is_menu_item() {
        let node = fixture().build_item_a11y(0, 0.0, 0.0, 200.0, 26.0).unwrap();
        assert_eq!(node.role(), Role::MenuItem);
    }

    #[test]
    fn separator_index_returns_none() {
        assert!(
            fixture()
                .build_item_a11y(2, 0.0, 0.0, 200.0, 26.0)
                .is_none()
        );
    }

    #[test]
    fn entry_rect_returns_item_position() {
        let m = fixture();
        let host = Rect::new(10.0, 20.0, 200.0, m.preferred_height(26.0));
        let r0 = m.entry_rect(host, 0, 26.0).unwrap();
        let r1 = m.entry_rect(host, 1, 26.0).unwrap();
        // Second item must sit one row below the first.
        assert!((r1.y - r0.y - 26.0).abs() < 0.001);
    }

    #[test]
    fn entry_rect_skips_separator() {
        let m = fixture();
        let host = Rect::new(0.0, 0.0, 200.0, m.preferred_height(26.0));
        assert!(m.entry_rect(host, 2, 26.0).is_none());
    }

    #[test]
    fn paint_smoke() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let menu = fixture();
        let host = Rect::new(0.0, 0.0, 200.0, menu.preferred_height(26.0));
        paint_context_menu(&menu, host, &mut scene, &mut text, Theme::Forge, 26.0);
    }
}
