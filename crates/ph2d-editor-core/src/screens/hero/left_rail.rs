//! LeftRail painter — vertical strip of transform/space/history
//! tools, highlighting whichever tool's `ButtonState` is `Pressed`.

use super::HeroLayout;
use super::ids;
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve};
use crate::widget::{ButtonState, ToolRail, ToolRailEntry, paint_tool_rail};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Register every LeftRail tool button into the [`WidgetStore`].
/// Translate is the default-pressed tool (matches the M13 fixture).
pub fn populate(store: &mut WidgetStore) {
    for id in [
        ids::RAIL_SHOW_INSPECTOR,
        ids::RAIL_SHOW_HIERARCHY,
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
    // Both panels start visible → Show toggles start Pressed.
    for id in [ids::RAIL_SHOW_INSPECTOR, ids::RAIL_SHOW_HIERARCHY] {
        if let Some(InteractiveState::Button { state }) = store.get_mut(id) {
            *state = ButtonState::Pressed;
        }
    }
}

/// Apply a [`WidgetEvent`] against LeftRail widgets. Returns false
/// so other dispatch handlers (rail_tools, rail_panels, …) still
/// react; this only side-effect-prints the clicked chip's name
/// (Enio 2026-05-25: "cada um dos componentes deve ao click
/// imprimir seu nome no console").
pub fn apply_event(_store: &mut WidgetStore, event: WidgetEvent) -> bool {
    if let WidgetEvent::Click(id) = event
        && let Some(name) = left_rail_chip_name(id)
    {
        println!("[rail] click: {name}");
    }
    false
}

fn left_rail_chip_name(id: NodeId) -> Option<&'static str> {
    Some(match id {
        x if x == ids::RAIL_SHOW_INSPECTOR => "Show Inspector",
        x if x == ids::RAIL_SHOW_HIERARCHY => "Show Hierarchy",
        x if x == ids::TOOL_TRANSLATE => "Translate",
        x if x == ids::TOOL_ROTATE => "Rotate",
        x if x == ids::TOOL_SCALE => "Scale",
        x if x == ids::TOOL_PIVOT => "Pivot",
        x if x == ids::TOOL_SPACE => "Coordinate Space",
        x if x == ids::TOOL_PROJECTION => "Projection",
        x if x == ids::TOOL_HOME => "Frame View",
        x if x == ids::TOOL_UNDO => "Undo",
        x if x == ids::TOOL_REDO => "Redo",
        _ => return None,
    })
}

pub fn paint_left_rail(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    // Top section: panel visibility toggles. `Pressed` ⇒ the
    // corresponding side panel is currently visible. Sit above
    // Move/Translate with their own divider so they read as
    // workspace-level controls, not transform tools.
    // Short uppercase tags painted vertically alongside the chip
    // (4-5 chars max — anything longer overruns the 44-px chip
    // height once rotated 90° CCW).
    let panel_toggles = [
        (
            ids::RAIL_SHOW_INSPECTOR,
            "Show Inspector",
            IconId::Inspector,
            "INSP",
        ),
        (
            ids::RAIL_SHOW_HIERARCHY,
            "Show Hierarchy",
            IconId::Hierarchy,
            "HIER",
        ),
    ];
    let mut rail_entries: Vec<ToolRailEntry> = panel_toggles
        .iter()
        .map(|(id, label, icon, sub)| {
            let mut e = ToolRailEntry::icon(*id, *label, *icon).with_sub(*sub);
            if matches!(store.button_state(*id), Some(ButtonState::Pressed)) {
                e = e.active();
            }
            e
        })
        .collect();
    rail_entries.push(ToolRailEntry::Divider);
    let entries = [
        (ids::TOOL_TRANSLATE, "Translate", IconId::Transform, "MOVE"),
        (ids::TOOL_ROTATE, "Rotate", IconId::Rotate, "ROT"),
        (ids::TOOL_SCALE, "Scale", IconId::Scale, "SCALE"),
        (ids::TOOL_PIVOT, "Pivot", IconId::Pivot, "PIVOT"),
    ];
    for (id, label, icon, sub) in entries.iter() {
        let mut e = ToolRailEntry::icon(*id, *label, *icon).with_sub(*sub);
        if matches!(store.button_state(*id), Some(ButtonState::Pressed)) {
            e = e.active();
        }
        rail_entries.push(e);
    }
    rail_entries.push(ToolRailEntry::Divider);
    // Face label reflects the live store state: SPACE toggles
    // Global ↔ Local on click; VIEW cycles Selected → Camera → All.
    let space_face = if store.tool_space_local() {
        "Local"
    } else {
        "Global"
    };
    rail_entries.push(ToolRailEntry::compound(
        ids::TOOL_SPACE,
        "Coordinate space",
        space_face,
        "SPACE",
    ));
    // TOOL_PROJECTION ("Persp / PROJ") hidden — not used yet.
    // M14.7 polish: 3-mode cycle (Selected → Camera → All). Click
    // executes the current mode AND advances the label. F/Home key
    // is a shortcut that always runs Selected.
    let view_face = match store.tool_view_mode() {
        1 => "Camera",
        2 => "All",
        _ => "Selected",
    };
    rail_entries.push(ToolRailEntry::compound(
        ids::TOOL_HOME,
        "Frame view",
        view_face,
        "VIEW",
    ));
    rail_entries.push(ToolRailEntry::Divider);
    rail_entries.push(ToolRailEntry::icon(ids::TOOL_UNDO, "Undo", IconId::Undo).with_sub("UNDO"));
    rail_entries.push(ToolRailEntry::icon(ids::TOOL_REDO, "Redo", IconId::Redo).with_sub("REDO"));

    let rail = ToolRail::new(NodeId(200), "Editor tools", rail_entries);
    let rail_rect = Rect::new(
        layout.left_rail.x,
        layout.left_rail.y,
        layout.left_rail.w,
        rail.preferred_height(store.rail_button_size()),
    );
    // Frosted-glass backing so the vertical labels (INSP / HIER /
    // MOVE / …) have a stable contrast platform when the rail sits
    // directly on canvas content. Slight vertical inset matches the
    // chip column's breathing room; rounded right edge only — the
    // left edge is flush with the viewport.
    let bg_rect = Rect::new(
        rail_rect.x,
        rail_rect.y - Spacing::Sm.px(),
        rail_rect.w,
        rail_rect.h + Spacing::Sm.px() * 2.0,
    );
    fill_rounded_rect(
        scene,
        bg_rect,
        Radius::Md.px(),
        resolve(ColorToken::RailBg, theme),
    );
    paint_tool_rail(&rail, rail_rect, scene, text_system, theme, store);

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
