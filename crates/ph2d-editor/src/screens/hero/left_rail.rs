//! LeftRail painter — vertical strip of transform/space/history
//! tools, highlighting whichever tool's `ButtonState` is `Pressed`.

use super::HeroLayout;
use super::ids;
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::widget::{ButtonState, ToolRail, ToolRailEntry, paint_tool_rail};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme};
use ph2d_vector::VectorScene;

/// Register every LeftRail tool button into the [`WidgetStore`].
/// Translate is the default-pressed tool (matches the M13 fixture).
pub fn populate(store: &mut WidgetStore) {
    for id in [
        ids::TOOL_TRANSLATE,
        ids::TOOL_ROTATE,
        ids::TOOL_SCALE,
        ids::TOOL_PIVOT,
        ids::TOOL_SPACE,
        ids::TOOL_PROJECTION,
        ids::TOOL_HOME,
        ids::TOOL_UNDO,
        ids::TOOL_REDO,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    if let Some(InteractiveState::Button { state }) = store.get_mut(ids::TOOL_TRANSLATE) {
        *state = ButtonState::Pressed;
    }
}

/// Apply a [`WidgetEvent`] against LeftRail widgets. Returns true
/// iff the event was consumed. Stub for now — switching the active
/// transform tool is a follow-up PR.
pub fn apply_event(_store: &mut WidgetStore, _event: WidgetEvent) -> bool {
    false
}

pub fn paint_left_rail(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let entries = [
        (ids::TOOL_TRANSLATE, "Translate", IconId::Transform),
        (ids::TOOL_ROTATE, "Rotate", IconId::Rotate),
        (ids::TOOL_SCALE, "Scale", IconId::Scale),
        (ids::TOOL_PIVOT, "Pivot", IconId::Pivot),
    ];
    let mut rail_entries: Vec<ToolRailEntry> = entries
        .iter()
        .map(|(id, label, icon)| {
            let mut e = ToolRailEntry::icon(*id, *label, *icon);
            if matches!(store.button_state(*id), Some(ButtonState::Pressed)) {
                e = e.active();
            }
            e
        })
        .collect();
    rail_entries.push(ToolRailEntry::Divider);
    rail_entries.push(ToolRailEntry::compound(
        ids::TOOL_SPACE,
        "Coordinate space",
        "Global",
        "SPACE",
    ));
    rail_entries.push(ToolRailEntry::compound(
        ids::TOOL_PROJECTION,
        "Camera projection",
        "Persp",
        "PROJ",
    ));
    rail_entries.push(ToolRailEntry::compound(
        ids::TOOL_HOME,
        "Frame to home",
        "Home",
        "VIEW",
    ));
    rail_entries.push(ToolRailEntry::Divider);
    rail_entries.push(ToolRailEntry::icon(ids::TOOL_UNDO, "Undo", IconId::Undo));
    rail_entries.push(ToolRailEntry::icon(ids::TOOL_REDO, "Redo", IconId::Redo));

    let rail = ToolRail::new(NodeId(200), "Editor tools", rail_entries);
    let rail_rect = Rect::new(
        layout.left_rail.x,
        layout.left_rail.y,
        layout.left_rail.w,
        rail.preferred_height(),
    );
    paint_tool_rail(&rail, rail_rect, scene, text_system, theme);

    let mut y = rail_rect.y;
    let gap = Spacing::Xs.px();
    let chip_x = rail_rect.x + (rail_rect.w - crate::widget::TOOL_CHIP_PX) * 0.5;
    for (i, entry) in rail.entries.iter().enumerate() {
        if i > 0 {
            y += gap;
        }
        match entry {
            ToolRailEntry::Icon { id, .. } => {
                let chip = Rect::new(
                    chip_x,
                    y,
                    crate::widget::TOOL_CHIP_PX,
                    crate::widget::TOOL_CHIP_PX,
                );
                hit_index.register(*id, chip);
                y += crate::widget::TOOL_CHIP_PX;
            }
            ToolRailEntry::Compound { id, .. } => {
                let chip = Rect::new(
                    chip_x,
                    y,
                    crate::widget::TOOL_CHIP_PX,
                    crate::widget::TOOL_CHIP_PX,
                );
                hit_index.register(*id, chip);
                y += crate::widget::COMPOUND_TOTAL_H_PX;
            }
            ToolRailEntry::Divider => {
                y += crate::widget::DIVIDER_GAP_PX * 2.0 + 1.0;
            }
        }
    }
}
