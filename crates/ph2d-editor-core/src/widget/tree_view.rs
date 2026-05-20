//! [`TreeView`] — hierarchical list with expand/collapse chevrons.
//!
//! Each [`TreeNode`] carries an `id`, a label, an optional icon and
//! a list of children. The view flattens the visible nodes (parents +
//! expanded subtrees) into rows; expand/collapse and selection state
//! live on the parent [`TreeView`] so consumers can wire input
//! handling without per-node mutation.

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub id: NodeId,
    pub label: String,
    pub icon: Option<IconId>,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            children: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn children(mut self, kids: Vec<TreeNode>) -> Self {
        self.children = kids;
        self
    }
}

#[derive(Clone, Debug)]
pub struct TreeView {
    pub id: NodeId,
    pub label: String,
    pub roots: Vec<TreeNode>,
    pub expanded: BTreeSet<NodeId>,
    pub selected: BTreeSet<NodeId>,
    pub multi_select: bool,
}

impl TreeView {
    pub fn new(id: NodeId, label: impl Into<String>, roots: Vec<TreeNode>) -> Self {
        Self {
            id,
            label: label.into(),
            roots,
            expanded: BTreeSet::new(),
            selected: BTreeSet::new(),
            multi_select: false,
        }
    }

    pub fn multi_select(mut self, yes: bool) -> Self {
        self.multi_select = yes;
        self
    }

    pub fn expand(&mut self, node: NodeId) {
        self.expanded.insert(node);
    }

    pub fn collapse(&mut self, node: NodeId) {
        self.expanded.remove(&node);
    }

    pub fn toggle_expand(&mut self, node: NodeId) {
        if !self.expanded.insert(node) {
            self.expanded.remove(&node);
        }
    }

    /// Add `node` to the selection (clearing the rest unless
    /// [`Self::multi_select`] is true).
    pub fn select(&mut self, node: NodeId) {
        if !self.multi_select {
            self.selected.clear();
        }
        self.selected.insert(node);
    }

    /// Visible rows (depth, &TreeNode) in paint order.
    pub fn visible_rows(&self) -> Vec<(usize, &TreeNode)> {
        let mut out = Vec::new();
        for root in &self.roots {
            push_row(&mut out, root, 0, &self.expanded);
        }
        out
    }

    /// Rect of the `visible_index`th row given the host rect and row
    /// height. Hosts use this to register a per-row hit zone so a
    /// click selects the entity at that row.
    pub fn row_rect(&self, host: Rect, visible_index: usize, row_h: f32) -> Rect {
        Rect::new(host.x, host.y + row_h * visible_index as f32, host.w, row_h)
    }

    /// Rect of the expand/collapse chevron for the `visible_index`th
    /// row, given the host rect, depth, and row height. Hosts use
    /// this to register a dedicated hit so the chevron toggles
    /// expansion without triggering selection.
    pub fn chevron_rect(&self, host: Rect, visible_index: usize, depth: usize, row_h: f32) -> Rect {
        let pad_x = Spacing::Md.px();
        let indent = Spacing::Lg.px();
        let chev_size = (row_h * 0.6).clamp(10.0, 16.0); // LITERAL-PX-OK: TreeView chev sized as 60% of row height with min/max
        let y = host.y + row_h * visible_index as f32;
        let x = host.x + pad_x + indent * depth as f32;
        Rect::new(x, y + (row_h - chev_size) * 0.5, chev_size, chev_size)
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Tree)
            .label(&self.label)
            .bounds(x, y, w, h)
            .children(self.visible_rows().iter().map(|(_, n)| n.id))
            .build()
    }

    pub fn build_node_a11y(&self, node: &TreeNode, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::TreeItem)
            .label(&node.label)
            .bounds(x, y, w, h)
            .focusable(true)
            .action(Action::Click)
            .build()
    }
}

fn push_row<'a>(
    out: &mut Vec<(usize, &'a TreeNode)>,
    node: &'a TreeNode,
    depth: usize,
    expanded: &BTreeSet<NodeId>,
) {
    out.push((depth, node));
    if expanded.contains(&node.id) {
        for child in &node.children {
            push_row(out, child, depth + 1, expanded);
        }
    }
}

pub fn paint_tree_view(
    tree: &TreeView,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    row_h: f32,
) {
    let pad_x = Spacing::Md.px();
    let indent = Spacing::Lg.px();
    let chev_size = (row_h * 0.6).clamp(10.0, 16.0); // LITERAL-PX-OK: chev 60% of row height with min/max
    let icon_size = (row_h * 0.6).clamp(12.0, 18.0); // LITERAL-PX-OK: icon 60% of row height with min/max
    let font = TypeToken::Base.px();

    for (visible_index, (depth, node)) in tree.visible_rows().iter().enumerate() {
        let y = rect.y + row_h * visible_index as f32;
        let row = Rect::new(rect.x, y, rect.w, row_h);
        if tree.selected.contains(&node.id) {
            fill_rounded_rect(
                scene,
                row,
                Radius::Xs.px(),
                resolve(ColorToken::AccentSoft, theme),
            );
            // Thin accent stroke so the selected row reads as
            // "active" rather than just "tinted" — matches the
            // hierarchy panel's row stroke and the picker's
            // channel-chip-active state.
            stroke_rounded_rect(
                scene,
                row,
                Radius::Xs.px(),
                1.0,
                resolve(ColorToken::Accent, theme),
            );
        }
        let depth_offset = indent * (*depth as f32);
        let mut cursor_x = row.x + pad_x + depth_offset;
        if !node.children.is_empty() {
            let chev_rect = Rect::new(
                cursor_x,
                y + (row_h - chev_size) * 0.5,
                chev_size,
                chev_size,
            );
            let icon = if tree.expanded.contains(&node.id) {
                IconId::ChevronDown
            } else {
                IconId::ChevronRight
            };
            paint_icon(
                scene,
                icon,
                chev_rect,
                resolve(ColorToken::Text2, theme),
                StrokeToken::Default.px(),
            );
            cursor_x += chev_size + Spacing::Xs.px();
        } else {
            cursor_x += chev_size + Spacing::Xs.px();
        }
        if let Some(icon) = node.icon {
            let icon_rect = Rect::new(
                cursor_x,
                y + (row_h - icon_size) * 0.5,
                icon_size,
                icon_size,
            );
            paint_icon(
                scene,
                icon,
                icon_rect,
                resolve(ColorToken::Text2, theme),
                StrokeToken::Default.px(),
            );
            cursor_x += icon_size + Spacing::Md.px();
        }
        let label_y = y + (row_h - font) * 0.5;
        paint_text(
            text_system,
            scene,
            &node.label,
            cursor_x,
            label_y,
            font,
            (rect.x + rect.w - cursor_x - pad_x).max(0.0),
            resolve(ColorToken::Text1, theme),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> TreeView {
        let leaf = TreeNode::new(NodeId(11), "Leaf").icon(IconId::Sprite);
        let mid = TreeNode::new(NodeId(10), "Mid")
            .icon(IconId::Folder)
            .children(vec![leaf]);
        let root = TreeNode::new(NodeId(2), "Scene")
            .icon(IconId::Scene)
            .children(vec![mid]);
        TreeView::new(NodeId(1), "Hierarchy", vec![root])
    }

    #[test]
    fn defaults_match_spec() {
        let t = fixture();
        assert!(t.expanded.is_empty());
        assert!(t.selected.is_empty());
        assert!(!t.multi_select);
    }

    #[test]
    fn visible_rows_collapsed_returns_only_roots() {
        let t = fixture();
        let rows = t.visible_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, 0);
    }

    #[test]
    fn visible_rows_expanded_returns_descendants() {
        let mut t = fixture();
        t.expand(NodeId(2));
        t.expand(NodeId(10));
        let rows = t.visible_rows();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].0, 0);
        assert_eq!(rows[1].0, 1);
        assert_eq!(rows[2].0, 2);
    }

    #[test]
    fn toggle_expand_round_trips() {
        let mut t = fixture();
        t.toggle_expand(NodeId(2));
        assert!(t.expanded.contains(&NodeId(2)));
        t.toggle_expand(NodeId(2));
        assert!(!t.expanded.contains(&NodeId(2)));
    }

    #[test]
    fn select_single_replaces_existing() {
        let mut t = fixture();
        t.select(NodeId(2));
        t.select(NodeId(11));
        assert_eq!(t.selected.len(), 1);
        assert!(t.selected.contains(&NodeId(11)));
    }

    #[test]
    fn select_multi_keeps_existing() {
        let mut t = fixture().multi_select(true);
        t.select(NodeId(2));
        t.select(NodeId(11));
        assert_eq!(t.selected.len(), 2);
    }

    #[test]
    fn a11y_role_is_tree() {
        let node = fixture().build_a11y(0.0, 0.0, 200.0, 200.0);
        assert_eq!(node.role(), Role::Tree);
    }

    #[test]
    fn a11y_node_role_is_tree_item() {
        let t = fixture();
        let node_ref = &t.roots[0];
        let node = t.build_node_a11y(node_ref, 0.0, 0.0, 200.0, 26.0);
        assert_eq!(node.role(), Role::TreeItem);
    }

    fn smoke(t: TreeView, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_tree_view(
            &t,
            Rect::new(0.0, 0.0, 240.0, 200.0),
            &mut scene,
            &mut text,
            theme,
            26.0,
        );
    }

    #[test]
    fn paint_smoke_collapsed() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_expanded_with_selection() {
        let mut t = fixture();
        t.expand(NodeId(2));
        t.expand(NodeId(10));
        t.select(NodeId(11));
        smoke(t, Theme::Sunstone);
    }
}
