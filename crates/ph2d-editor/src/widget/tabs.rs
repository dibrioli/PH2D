//! [`Tabs`] — horizontal tab strip.
//!
//! Two variants: ghost (default — selected tab gets an `Accent`
//! underline 2 px tall) and segmented (pill background, like
//! [`super::radio_group::RadioOrientation::Segmented`] but tab-shaped).
//! AccessKit `Role::TabList` (parent) + `Role::Tab` (per option).

use crate::paint::{
    fill_rounded_rect, paint_text_centered, rect_to_vello, resolve, stroke_rounded_rect,
};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role, Toggled};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TabsVariant {
    /// Selected tab gets a thin `Accent` underline; siblings ghost.
    #[default]
    Ghost,
    /// Selected tab fills with `AccentSoft`; group sits in a `Bg2`
    /// pill. Mirrors the iOS segmented control look.
    Segmented,
}

#[derive(Clone, Debug)]
pub struct TabItem {
    pub id: NodeId,
    pub label: String,
}

impl TabItem {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tabs {
    pub id: NodeId,
    pub label: String,
    pub items: Vec<TabItem>,
    pub selected: usize,
    pub variant: TabsVariant,
}

impl Tabs {
    pub fn new(id: NodeId, label: impl Into<String>, items: Vec<TabItem>) -> Self {
        Self {
            id,
            label: label.into(),
            items,
            selected: 0,
            variant: TabsVariant::Ghost,
        }
    }

    pub fn selected(mut self, idx: usize) -> Self {
        self.selected = idx.min(self.items.len().saturating_sub(1));
        self
    }

    pub fn variant(mut self, v: TabsVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn select(&mut self, idx: usize) {
        if idx < self.items.len() {
            self.selected = idx;
        }
    }

    pub fn tab_rect(&self, host: Rect, index: usize) -> Rect {
        let count = self.items.len().max(1) as f32;
        let w = host.w / count;
        Rect::new(host.x + w * index as f32, host.y, w, host.h)
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::TabList)
            .label(&self.label)
            .bounds(x, y, w, h)
            .children(self.items.iter().map(|t| t.id))
            .build()
    }

    pub fn build_tab_a11y(&self, index: usize, x: f64, y: f64, w: f64, h: f64) -> Option<Node> {
        let item = self.items.get(index)?;
        Some(
            NodeBuilder::new(Role::Tab)
                .label(&item.label)
                .bounds(x, y, w, h)
                .focusable(true)
                .action(Action::Click)
                .toggled(if index == self.selected {
                    Toggled::True
                } else {
                    Toggled::False
                })
                .build(),
        )
    }
}

pub fn paint_tabs(
    tabs: &Tabs,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_tabs_with_hover(tabs, None, rect, scene, text_system, theme);
}

/// Like [`paint_tabs`] but draws a subtle hover background under the
/// hovered tab. Pass `hovered = None` for the unhovered case (or just
/// call [`paint_tabs`]). Used by call sites that read the hot id from
/// `WidgetStore` and want immediate hover feedback rather than waiting
/// for the click to commit the selection.
pub fn paint_tabs_with_hover(
    tabs: &Tabs,
    hovered: Option<usize>,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if tabs.variant == TabsVariant::Segmented {
        let radius = Radius::Md.px();
        fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
        stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    }

    for (i, item) in tabs.items.iter().enumerate() {
        let r = tabs.tab_rect(rect, i);
        let is_selected = i == tabs.selected;
        let is_hovered = hovered == Some(i);
        // Hover affordance for non-selected tabs — a subtle Bg2
        // fill (Ghost) or AccentSoft @ 50 % alpha equivalent
        // (Segmented). Selected tabs already have their own
        // emphasis, so we don't paint the hover layer on top.
        if is_hovered && !is_selected {
            let inset = Rect::new(
                r.x + 2.0,
                r.y + 2.0,
                (r.w - 4.0).max(0.0),
                (r.h - 4.0).max(0.0),
            );
            let radius = (Radius::Md.px() - 2.0).max(0.0);
            fill_rounded_rect(scene, inset, radius, resolve(ColorToken::Bg2, theme));
        }
        match tabs.variant {
            TabsVariant::Ghost => {
                let fg = if is_selected {
                    ColorToken::Text1
                } else if is_hovered {
                    ColorToken::Text1
                } else {
                    ColorToken::Text2
                };
                paint_text_centered(
                    text_system,
                    scene,
                    &item.label,
                    r,
                    TypeToken::Base.px(),
                    resolve(fg, theme),
                );
                if is_selected {
                    let underline =
                        Rect::new(r.x + 8.0, r.y + r.h - 2.0, (r.w - 16.0).max(0.0), 2.0);
                    scene.fill_rect(rect_to_vello(underline), resolve(ColorToken::Accent, theme));
                }
            }
            TabsVariant::Segmented => {
                if is_selected {
                    let inset = Rect::new(
                        r.x + 2.0,
                        r.y + 2.0,
                        (r.w - 4.0).max(0.0),
                        (r.h - 4.0).max(0.0),
                    );
                    fill_rounded_rect(
                        scene,
                        inset,
                        (Radius::Md.px() - 2.0).max(0.0),
                        resolve(ColorToken::AccentSoft, theme),
                    );
                }
                let fg = if is_selected {
                    ColorToken::Accent
                } else {
                    ColorToken::Text2
                };
                paint_text_centered(
                    text_system,
                    scene,
                    &item.label,
                    r,
                    TypeToken::Base.px(),
                    resolve(fg, theme),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Tabs {
        Tabs::new(
            NodeId(1),
            "Sections",
            vec![
                TabItem::new(NodeId(2), "General"),
                TabItem::new(NodeId(3), "Layers"),
                TabItem::new(NodeId(4), "History"),
            ],
        )
    }

    #[test]
    fn defaults_select_first() {
        assert_eq!(fixture().selected, 0);
    }

    #[test]
    fn selected_chain_clamps_to_range() {
        let t = fixture().selected(99);
        assert_eq!(t.selected, 2);
    }

    #[test]
    fn select_method_clamps() {
        let mut t = fixture();
        t.select(99);
        assert_eq!(t.selected, 0);
        t.select(2);
        assert_eq!(t.selected, 2);
    }

    #[test]
    fn tab_rect_splits_evenly() {
        let t = fixture();
        let host = Rect::new(0.0, 0.0, 300.0, 32.0);
        assert_eq!(t.tab_rect(host, 0).w, 100.0);
        assert_eq!(t.tab_rect(host, 1).x, 100.0);
        assert_eq!(t.tab_rect(host, 2).x, 200.0);
    }

    #[test]
    fn a11y_parent_is_tab_list() {
        let node = fixture().build_a11y(0.0, 0.0, 300.0, 32.0);
        assert_eq!(node.role(), Role::TabList);
    }

    #[test]
    fn a11y_tab_role_and_toggled() {
        let t = fixture().selected(1);
        let on = t.build_tab_a11y(1, 0.0, 0.0, 100.0, 32.0).unwrap();
        let off = t.build_tab_a11y(0, 0.0, 0.0, 100.0, 32.0).unwrap();
        assert_eq!(on.role(), Role::Tab);
        assert_eq!(on.toggled(), Some(Toggled::True));
        assert_eq!(off.toggled(), Some(Toggled::False));
    }

    fn smoke(t: Tabs, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_tabs(
            &t,
            Rect::new(0.0, 0.0, 300.0, 32.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_ghost() {
        smoke(fixture(), Theme::ForgeSdf);
    }

    #[test]
    fn paint_smoke_segmented_middle_selected() {
        smoke(
            fixture().selected(1).variant(TabsVariant::Segmented),
            Theme::Sunstone,
        );
    }
}
