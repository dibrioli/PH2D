//! [`ToolRail`] — vertical strip of editor tools.
//!
//! Three entry kinds:
//! - `Icon { id, icon, active }` — square 44x44 icon-only chip.
//! - `Compound { id, label, sub }` — chip with a body label (face)
//!   and a small uppercase mono sub-label below ("Global / SPACE",
//!   "Persp / PROJ", "Home / VIEW").
//! - `Divider` — 24x1 px line in `Border` color.
//!
//! AccessKit `Role::Toolbar` (vertical orientation hinted by the
//! parent layout — AccessKit doesn't expose orientation on Toolbar).

use crate::icons::IconId;
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text_centered, rect_to_vello, resolve, stroke_rounded_rect,
};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

pub const TOOL_RAIL_WIDTH_PX: f32 = 56.0;
pub const TOOL_CHIP_PX: f32 = 44.0;
pub const COMPOUND_TOTAL_H_PX: f32 = 60.0; // chip + sub-label gap
pub const DIVIDER_GAP_PX: f32 = 8.0;

#[derive(Clone, Debug)]
pub enum ToolRailEntry {
    Icon {
        id: NodeId,
        label: String,
        icon: IconId,
        active: bool,
    },
    Compound {
        id: NodeId,
        label: String,
        face: String,
        sub: String,
    },
    Divider,
}

impl ToolRailEntry {
    pub fn icon(id: NodeId, label: impl Into<String>, icon: IconId) -> Self {
        Self::Icon {
            id,
            label: label.into(),
            icon,
            active: false,
        }
    }

    pub fn compound(
        id: NodeId,
        label: impl Into<String>,
        face: impl Into<String>,
        sub: impl Into<String>,
    ) -> Self {
        Self::Compound {
            id,
            label: label.into(),
            face: face.into(),
            sub: sub.into(),
        }
    }

    /// Builder shortcut for the Icon variant — flips `active` true.
    pub fn active(mut self) -> Self {
        if let Self::Icon { active, .. } = &mut self {
            *active = true;
        }
        self
    }

    /// Vertical extent this entry needs.
    pub fn height(&self) -> f32 {
        match self {
            Self::Icon { .. } => TOOL_CHIP_PX,
            Self::Compound { .. } => COMPOUND_TOTAL_H_PX,
            Self::Divider => 1.0 + DIVIDER_GAP_PX * 2.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ToolRail {
    pub id: NodeId,
    pub label: String,
    pub entries: Vec<ToolRailEntry>,
}

impl ToolRail {
    pub fn new(id: NodeId, label: impl Into<String>, entries: Vec<ToolRailEntry>) -> Self {
        Self {
            id,
            label: label.into(),
            entries,
        }
    }

    pub fn preferred_height(&self) -> f32 {
        let gap = Spacing::Xs.px();
        let mut total = 0.0_f32;
        for (i, e) in self.entries.iter().enumerate() {
            if i > 0 {
                total += gap;
            }
            total += e.height();
        }
        total
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let kids = self.entries.iter().filter_map(|e| match e {
            ToolRailEntry::Icon { id, .. } | ToolRailEntry::Compound { id, .. } => Some(*id),
            ToolRailEntry::Divider => None,
        });
        NodeBuilder::new(Role::Toolbar)
            .label(&self.label)
            .bounds(x, y, w, h)
            .children(kids)
            .build()
    }

    pub fn build_entry_a11y(&self, index: usize, x: f64, y: f64, w: f64, h: f64) -> Option<Node> {
        match self.entries.get(index)? {
            ToolRailEntry::Icon { id: _, label, .. } => Some(
                NodeBuilder::new(Role::Button)
                    .label(label)
                    .bounds(x, y, w, h)
                    .focusable(true)
                    .action(Action::Click)
                    .build(),
            ),
            ToolRailEntry::Compound { id: _, label, .. } => Some(
                NodeBuilder::new(Role::Button)
                    .label(label)
                    .bounds(x, y, w, h)
                    .focusable(true)
                    .action(Action::Click)
                    .build(),
            ),
            ToolRailEntry::Divider => None,
        }
    }
}

pub fn paint_tool_rail(
    rail: &ToolRail,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let gap = Spacing::Xs.px();
    let mut y = rect.y;
    for (i, entry) in rail.entries.iter().enumerate() {
        if i > 0 {
            y += gap;
        }
        match entry {
            ToolRailEntry::Icon { icon, active, .. } => {
                let chip_rect = Rect::new(
                    rect.x + (rect.w - TOOL_CHIP_PX) * 0.5,
                    y,
                    TOOL_CHIP_PX,
                    TOOL_CHIP_PX,
                );
                let radius = Radius::Lg.px();
                let bg = if *active {
                    ColorToken::AccentSoft
                } else {
                    ColorToken::BgElev
                };
                fill_rounded_rect(scene, chip_rect, radius, resolve(bg, theme));
                let border = if *active {
                    ColorToken::Accent
                } else {
                    ColorToken::Border
                };
                stroke_rounded_rect(
                    scene,
                    chip_rect,
                    radius,
                    if *active { 1.5 } else { 1.0 },
                    resolve(border, theme),
                );
                let fg = if *active {
                    ColorToken::Accent
                } else {
                    ColorToken::Text2
                };
                paint_icon(scene, *icon, chip_rect, resolve(fg, theme), 1.5);
                y += TOOL_CHIP_PX;
            }
            ToolRailEntry::Compound { face, sub, .. } => {
                let chip_rect = Rect::new(
                    rect.x + (rect.w - TOOL_CHIP_PX) * 0.5,
                    y,
                    TOOL_CHIP_PX,
                    TOOL_CHIP_PX,
                );
                let radius = Radius::Lg.px();
                fill_rounded_rect(scene, chip_rect, radius, resolve(ColorToken::BgElev, theme));
                stroke_rounded_rect(
                    scene,
                    chip_rect,
                    radius,
                    1.0,
                    resolve(ColorToken::Border, theme),
                );
                paint_text_centered(
                    text_system,
                    scene,
                    face,
                    chip_rect,
                    TypeToken::Xs.px(),
                    resolve(ColorToken::Text1, theme),
                );
                let sub_rect = Rect::new(
                    rect.x,
                    y + TOOL_CHIP_PX + 2.0,
                    rect.w,
                    COMPOUND_TOTAL_H_PX - TOOL_CHIP_PX - 2.0,
                );
                paint_text_centered(
                    text_system,
                    scene,
                    sub,
                    sub_rect,
                    TypeToken::Xs.px() - 2.0,
                    resolve(ColorToken::Text3, theme),
                );
                y += COMPOUND_TOTAL_H_PX;
            }
            ToolRailEntry::Divider => {
                y += DIVIDER_GAP_PX;
                let line = Rect::new(rect.x + (rect.w - 24.0) * 0.5, y, 24.0, 1.0);
                scene.fill_rect(rect_to_vello(line), resolve(ColorToken::Border, theme));
                y += 1.0 + DIVIDER_GAP_PX;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ToolRail {
        ToolRail::new(
            NodeId(1),
            "Editor tools",
            vec![
                ToolRailEntry::icon(NodeId(2), "Translate", IconId::Transform).active(),
                ToolRailEntry::icon(NodeId(3), "Rotate", IconId::Rotate),
                ToolRailEntry::icon(NodeId(4), "Scale", IconId::Scale),
                ToolRailEntry::icon(NodeId(5), "Pivot", IconId::Pivot),
                ToolRailEntry::Divider,
                ToolRailEntry::compound(NodeId(6), "Coordinate space", "Global", "SPACE"),
                ToolRailEntry::compound(NodeId(7), "Camera projection", "Persp", "PROJ"),
                ToolRailEntry::compound(NodeId(8), "Frame to home", "Home", "VIEW"),
                ToolRailEntry::Divider,
                ToolRailEntry::icon(NodeId(9), "Undo", IconId::Undo),
                ToolRailEntry::icon(NodeId(10), "Redo", IconId::Redo),
            ],
        )
    }

    #[test]
    fn preferred_height_sums_entries() {
        let h = fixture().preferred_height();
        assert!(h > TOOL_CHIP_PX * 5.0);
    }

    #[test]
    fn icon_active_setter_flips_active() {
        let entry = ToolRailEntry::icon(NodeId(1), "x", IconId::Add).active();
        match entry {
            ToolRailEntry::Icon { active, .. } => assert!(active),
            _ => panic!("expected Icon"),
        }
    }

    #[test]
    fn a11y_parent_is_toolbar() {
        let node = fixture().build_a11y(0.0, 0.0, 56.0, 600.0);
        assert_eq!(node.role(), Role::Toolbar);
    }

    #[test]
    fn a11y_entry_button_role() {
        let node = fixture().build_entry_a11y(0, 0.0, 0.0, 44.0, 44.0).unwrap();
        assert_eq!(node.role(), Role::Button);
    }

    #[test]
    fn a11y_divider_returns_none() {
        let rail = fixture();
        assert!(rail.build_entry_a11y(4, 0.0, 0.0, 44.0, 1.0).is_none());
    }

    #[test]
    fn paint_smoke_full_rail() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let rail = fixture();
        let host = Rect::new(0.0, 0.0, TOOL_RAIL_WIDTH_PX, rail.preferred_height());
        paint_tool_rail(&rail, host, &mut scene, &mut text, Theme::ForgeSdf);
    }

    #[test]
    fn paint_smoke_minimal_rail() {
        let rail = ToolRail::new(
            NodeId(1),
            "Tiny",
            vec![ToolRailEntry::icon(NodeId(2), "x", IconId::Add)],
        );
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_tool_rail(
            &rail,
            Rect::new(0.0, 0.0, TOOL_RAIL_WIDTH_PX, rail.preferred_height()),
            &mut scene,
            &mut text,
            Theme::Sunstone,
        );
    }
}
