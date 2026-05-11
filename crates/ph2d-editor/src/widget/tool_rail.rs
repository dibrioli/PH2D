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
use crate::interaction::WidgetStore;
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text_centered, paint_text_rotated_ccw, rect_to_vello,
    resolve, stroke_rounded_rect,
};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Width of the LeftRail. Holds a vertical sub-label column on the
/// left + a 44×44 chip on the right; sized to accommodate both
/// without crowding.
pub const TOOL_RAIL_WIDTH_PX: f32 = 68.0;
pub const TOOL_CHIP_PX: f32 = 44.0;
pub const COMPOUND_TOTAL_H_PX: f32 = TOOL_CHIP_PX; // sub-label moved to vertical-left
pub const DIVIDER_GAP_PX: f32 = 8.0;
/// Right margin reserved for the chip; the rest of the rail width is
/// the vertical sub-label column.
const CHIP_RIGHT_PAD_PX: f32 = 4.0;

#[derive(Clone, Debug)]
pub enum ToolRailEntry {
    Icon {
        id: NodeId,
        label: String,
        icon: IconId,
        active: bool,
        /// Short UPPERCASE tag painted vertically to the LEFT of
        /// the chip. Empty string means "no sub-label".
        sub: String,
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
            sub: String::new(),
        }
    }

    /// Builder shortcut for the Icon variant — sets the vertical
    /// sub-label tag (short uppercase, e.g. "MOVE", "ROT", "UNDO").
    pub fn sub(mut self, sub: impl Into<String>) -> Self {
        if let Self::Icon { sub: s, .. } = &mut self {
            *s = sub.into();
        }
        self
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
    store: &WidgetStore,
) {
    // Chip is right-anchored so the column to its LEFT can carry a
    // small vertical sub-label per entry.
    let chip_x = rect.x + rect.w - TOOL_CHIP_PX - CHIP_RIGHT_PAD_PX;
    let sub_font = (TypeToken::Xs.px() - 2.0).max(8.0);
    let gap = Spacing::Xs.px();
    let mut y = rect.y;
    for (i, entry) in rail.entries.iter().enumerate() {
        if i > 0 {
            y += gap;
        }
        match entry {
            ToolRailEntry::Icon {
                id,
                icon,
                active,
                sub,
                ..
            } => {
                let chip_rect = Rect::new(chip_x, y, TOOL_CHIP_PX, TOOL_CHIP_PX);
                let radius = Radius::Lg.px();
                let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
                let is_active = *active || state == ButtonState::Pressed;
                let bg = match state {
                    ButtonState::Hovered | ButtonState::Focused => ColorToken::BgElev,
                    ButtonState::Pressed => ColorToken::AccentSoft,
                    _ if is_active => ColorToken::AccentSoft,
                    _ => ColorToken::BgElev,
                };
                fill_rounded_rect(scene, chip_rect, radius, resolve(bg, theme));
                let (border, border_w) = match state {
                    ButtonState::Hovered | ButtonState::Focused => (ColorToken::BorderEmph, 1.0),
                    ButtonState::Pressed => (ColorToken::Accent, 1.5),
                    _ if is_active => (ColorToken::Accent, 1.5),
                    _ => (ColorToken::Border, 1.0),
                };
                stroke_rounded_rect(scene, chip_rect, radius, border_w, resolve(border, theme));
                let fg = match state {
                    ButtonState::Hovered | ButtonState::Focused => ColorToken::Text1,
                    ButtonState::Pressed => ColorToken::Accent,
                    _ if is_active => ColorToken::Accent,
                    _ => ColorToken::Text2,
                };
                paint_icon(scene, *icon, chip_rect, resolve(fg, theme), 1.5);
                paint_sub_label_vertical(
                    text_system,
                    scene,
                    sub,
                    sub_font,
                    rect.x,
                    chip_rect,
                    resolve(ColorToken::Text3, theme),
                );
                y += TOOL_CHIP_PX;
            }
            ToolRailEntry::Compound { id, face, sub, .. } => {
                let chip_rect = Rect::new(chip_x, y, TOOL_CHIP_PX, TOOL_CHIP_PX);
                let radius = Radius::Lg.px();
                let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
                let bg = match state {
                    ButtonState::Hovered | ButtonState::Focused => ColorToken::BgElev,
                    ButtonState::Pressed => ColorToken::AccentSoft,
                    _ => ColorToken::BgElev,
                };
                fill_rounded_rect(scene, chip_rect, radius, resolve(bg, theme));
                let (border, border_w) = match state {
                    ButtonState::Hovered | ButtonState::Focused => (ColorToken::BorderEmph, 1.0),
                    ButtonState::Pressed => (ColorToken::Accent, 1.5),
                    _ => (ColorToken::Border, 1.0),
                };
                stroke_rounded_rect(scene, chip_rect, radius, border_w, resolve(border, theme));
                let face_color = match state {
                    ButtonState::Pressed => ColorToken::Accent,
                    _ => ColorToken::Text1,
                };
                paint_text_centered(
                    text_system,
                    scene,
                    face,
                    chip_rect,
                    TypeToken::Xs.px(),
                    resolve(face_color, theme),
                );
                paint_sub_label_vertical(
                    text_system,
                    scene,
                    sub,
                    sub_font,
                    rect.x,
                    chip_rect,
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

/// Helper — paint a short uppercase tag vertically (CCW-rotated)
/// in the column to the LEFT of `chip_rect`. The text's baseline
/// sits at `rail_left_x + col_width / 2`; the rotation puts the
/// glyphs running bottom-to-top.
fn paint_sub_label_vertical(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    font_size: f32,
    rail_left_x: f32,
    chip_rect: Rect,
    color: ph2d_vector::Color,
) {
    if text.is_empty() {
        return;
    }
    let column_w = chip_rect.x - rail_left_x;
    // Anchor: bottom-left of the rotated text. After 90° CCW the
    // text's height (post-layout) becomes its horizontal extent,
    // which we want to fit inside the column. The baseline ends up
    // at `anchor_x` from the bottom of the chip going up.
    let anchor_x = rail_left_x + (column_w * 0.5) + font_size * 0.5;
    let anchor_y = chip_rect.y + chip_rect.h - 6.0;
    paint_text_rotated_ccw(
        text_system,
        scene,
        text,
        anchor_x,
        anchor_y,
        font_size,
        chip_rect.h,
        color,
    );
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
        let store = crate::interaction::WidgetStore::with_capacity(0);
        paint_tool_rail(&rail, host, &mut scene, &mut text, Theme::ForgeSdf, &store);
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
        let store = crate::interaction::WidgetStore::with_capacity(0);
        paint_tool_rail(
            &rail,
            Rect::new(0.0, 0.0, TOOL_RAIL_WIDTH_PX, rail.preferred_height()),
            &mut scene,
            &mut text,
            Theme::Sunstone,
            &store,
        );
    }
}
