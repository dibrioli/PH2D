//! LeftRail painter — vertical strip of editor tools.
//!
//! Two faces, selected by mode:
//! - **Object mode** (default): panel toggles · Translate/Rotate/Scale/Pivot ·
//!   Coordinate-space + Frame-view · Undo/Redo.
//! - **Painter mode** (the Painter image-edit tool is active): the transform
//!   block is replaced by the paint tools (Brush · Eyedropper · Eraser · Clone ·
//!   Smear · Blur · Mask · Inpaint · Shapes). The Shapes button reveals a flyout
//!   of shape options to its right. Panel toggles + space/view + Undo/Redo stay.
//!
//! Highlights whichever tool's `ButtonState` is `Pressed` (the radio selection
//! lives in the store; dispatch in `chrome/rail_tools.rs` + `chrome/rail_painter_tools.rs`).

use super::HeroLayout;
use super::ids;
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve};
use crate::widget::{ButtonState, CHIP_X_OFFSET_PX, ToolRail, ToolRailEntry, paint_tool_rail};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Painter-mode paint tools, in rail order: `(id, a11y label, icon, vertical
/// sub-label)`. Replaces the object-mode transform block while the Painter tool
/// is active. Exclusive radio selection (`chrome/rail_painter_tools.rs`); the
/// last entry (Shapes) also owns the shape flyout ([`PAINTER_SHAPES`]).
const PAINTER_TOOLS: [(NodeId, &str, IconId, &str); 9] = [
    (ids::PAINTER_RAIL_BRUSH, "Brush", IconId::Painter, "BRUSH"),
    (
        ids::PAINTER_RAIL_EYEDROPPER,
        "Eyedropper",
        IconId::Eyedropper,
        "PICK",
    ),
    (ids::PAINTER_RAIL_ERASER, "Eraser", IconId::Erase, "ERASE"),
    (ids::PAINTER_RAIL_CLONE, "Clone", IconId::Clone, "CLONE"),
    (ids::PAINTER_RAIL_SMEAR, "Smear", IconId::Smear, "SMEAR"),
    (ids::PAINTER_RAIL_BLUR, "Blur", IconId::Blur, "BLUR"),
    (ids::PAINTER_RAIL_MASK, "Mask", IconId::Mask, "MASK"),
    (
        ids::PAINTER_RAIL_INPAINT,
        "Inpaint",
        IconId::Inpaint,
        "INPNT",
    ),
    (
        ids::PAINTER_RAIL_SHAPES,
        "Shapes",
        IconId::VectorShape,
        "SHAPE",
    ),
];

/// The Shapes flyout options, in flyout order: `(id, a11y label, icon, sub)`.
/// Same chip appearance as the rail tools (icon + short vertical sub-label, same
/// background) — only the icon differs. Revealed right of the Shapes button.
const PAINTER_SHAPES: [(NodeId, &str, IconId, &str); 5] = [
    (
        ids::PAINTER_RAIL_SHAPE_FREEHAND,
        "Free Hand",
        IconId::VectorPencil,
        "FREE",
    ),
    (ids::PAINTER_RAIL_SHAPE_LINE, "Line", IconId::Line, "LINE"),
    (
        ids::PAINTER_RAIL_SHAPE_CURVE,
        "Curve",
        IconId::VectorPen,
        "CURVE",
    ),
    (
        ids::PAINTER_RAIL_SHAPE_ELLIPSE,
        "Ellipse",
        IconId::Circle,
        "ELLI",
    ),
    (
        ids::PAINTER_RAIL_SHAPE_POLYGON,
        "Polygon",
        IconId::Polygon,
        "POLY",
    ),
];

/// Register every LeftRail widget (both faces) into the [`WidgetStore`].
/// Translate is the default-pressed object tool; Brush is the default-pressed
/// painter tool (the Shapes sub-radio starts with none pressed). The painter ids are always registered
/// (even in object mode) so the wiring-parity gate + the radio dispatch find
/// them; they are only painted/hit while the Painter tool is active.
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
    for (id, ..) in PAINTER_TOOLS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    for (id, ..) in PAINTER_SHAPES {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    store.register(ids::RAIL_BACKDROP, InteractiveState::Plain);
    // Default-pressed selections (each radio group is independent — only one
    // group is painted at a time, so co-pressing is fine). The Shapes sub-radio
    // starts with NONE pressed — a shape button looks unchecked until the artist
    // picks one (Enio); the Shapes rail button then adopts that shape's icon.
    for id in [ids::TOOL_TRANSLATE, ids::PAINTER_RAIL_BRUSH] {
        if let Some(InteractiveState::Button { state }) = store.get_mut(id) {
            *state = ButtonState::Pressed;
        }
    }
    // Both panels start visible → Show toggles start Pressed.
    for id in [ids::RAIL_SHOW_INSPECTOR, ids::RAIL_SHOW_HIERARCHY] {
        if let Some(InteractiveState::Button { state }) = store.get_mut(id) {
            *state = ButtonState::Pressed;
        }
    }
}

/// Apply a [`WidgetEvent`] against LeftRail widgets. Returns false
/// so other dispatch handlers (rail_tools, rail_painter_tools,
/// rail_panels, …) still react; this only side-effect-prints the
/// clicked chip's name (Enio 2026-05-25: "cada um dos componentes
/// deve ao click imprimir seu nome no console").
pub fn apply_event(_store: &mut WidgetStore, event: WidgetEvent) -> bool {
    if let WidgetEvent::Click(id) = event
        && let Some(name) = left_rail_chip_name(id)
    {
        println!("[rail] click: {name}");
    }
    false
}

fn left_rail_chip_name(id: NodeId) -> Option<&'static str> {
    if let Some((_, label, ..)) = PAINTER_TOOLS.iter().find(|(tid, ..)| *tid == id) {
        return Some(label);
    }
    if let Some((_, label, ..)) = PAINTER_SHAPES.iter().find(|(sid, ..)| *sid == id) {
        return Some(label);
    }
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
        x if x == ids::RAIL_BACKDROP => "Rail Backdrop",
        _ => return None,
    })
}

/// Build the rail entry for one `(id, label, icon, sub)` tool tuple, flagging it
/// `active` when its store button state is `Pressed`.
fn tool_entry(
    store: &WidgetStore,
    (id, label, icon, sub): (NodeId, &str, IconId, &str),
) -> ToolRailEntry {
    let mut e = ToolRailEntry::icon(id, label, icon).with_sub(sub);
    if matches!(store.button_state(id), Some(ButtonState::Pressed)) {
        e = e.active();
    }
    e
}

/// The Shapes-flyout entry (`label`, `icon`, `sub`) whose sub-radio is Pressed — so the Shapes rail
/// button can adopt the ACTIVE shape's icon in place of the generic Shapes icon. `None` until a shape
/// is picked (the button then keeps the last-picked shape's icon, like a Photoshop tool group).
fn active_shape(store: &WidgetStore) -> Option<(&'static str, IconId, &'static str)> {
    PAINTER_SHAPES.iter().find_map(|(id, label, icon, sub)| {
        matches!(store.button_state(*id), Some(ButtonState::Pressed))
            .then_some((*label, *icon, *sub))
    })
}

pub fn paint_left_rail(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    painter_active: bool,
) {
    // Top section: panel visibility toggles (workspace-level controls).
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
    // Middle section: paint tools in Painter mode, else transform tools.
    if painter_active {
        for tool in PAINTER_TOOLS {
            // The Shapes button adopts the ACTIVE shape's icon/label (like a Photoshop tool group)
            // once a shape is picked — the generic Shapes icon only shows until then.
            let tool = if tool.0 == ids::PAINTER_RAIL_SHAPES
                && let Some((label, icon, sub)) = active_shape(store)
            {
                (tool.0, label, icon, sub)
            } else {
                tool
            };
            rail_entries.push(tool_entry(store, tool));
        }
    } else {
        let entries = [
            (ids::TOOL_TRANSLATE, "Translate", IconId::Transform, "MOVE"),
            (ids::TOOL_ROTATE, "Rotate", IconId::Rotate, "ROT"),
            (ids::TOOL_SCALE, "Scale", IconId::Scale, "SCALE"),
            (ids::TOOL_PIVOT, "Pivot", IconId::Pivot, "PIVOT"),
        ];
        for tool in entries {
            rail_entries.push(tool_entry(store, tool));
        }
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
    // Frosted-glass backing so the vertical labels have a stable contrast
    // platform; rounded right edge only (left edge flush with the viewport).
    let bg_rect = Rect::new(
        rail_rect.x,
        rail_rect.y - Spacing::Sm.px(),
        rail_rect.w,
        rail_rect.h + Spacing::Sm.px() * 2.0,
    );
    // Register backdrop hit FIRST so chip hits (registered below by the chip
    // loop) win when overlapping (HitIndex walks back-to-front).
    hit_index.register(ids::RAIL_BACKDROP, bg_rect);
    fill_rounded_rect(
        scene,
        bg_rect,
        Radius::Md.px(),
        resolve(ColorToken::RailBg, theme),
    );
    paint_tool_rail(&rail, rail_rect, scene, text_system, theme, store);

    // Hit-rects MUST mirror exactly what `paint_tool_rail` paints — same
    // `chip_px` and `CHIP_X_OFFSET_PX`. Capture the Shapes chip rect so the
    // flyout (if open) can anchor to it.
    let chip_px = store.rail_button_size().chip_px();
    let mut y = rail_rect.y;
    let gap = Spacing::Xs.px();
    let chip_x = rail_rect.x + CHIP_X_OFFSET_PX;
    let mut shapes_chip: Option<Rect> = None;
    for (i, entry) in rail.entries.iter().enumerate() {
        if i > 0 {
            y += gap;
        }
        match entry {
            ToolRailEntry::Icon { id, .. } => {
                let chip = Rect::new(chip_x, y, chip_px, chip_px);
                hit_index.register(*id, chip);
                if *id == ids::PAINTER_RAIL_SHAPES {
                    shapes_chip = Some(chip);
                }
                y += chip_px;
            }
            ToolRailEntry::Compound { id, .. } => {
                let chip = Rect::new(chip_x, y, chip_px, chip_px);
                hit_index.register(*id, chip);
                y += chip_px;
            }
            ToolRailEntry::Divider => {
                y += crate::widget::DIVIDER_GAP_PX * 2.0 + 1.0;
            }
        }
    }

    // Shapes flyout — a mini-rail of shape chips painted to the RIGHT of the
    // Shapes button while open (Painter mode only). Same chip appearance.
    if painter_active
        && store.painter_shapes_flyout_open()
        && let Some(anchor) = shapes_chip
    {
        paint_shapes_flyout(
            anchor,
            rail_rect,
            scene,
            text_system,
            theme,
            hit_index,
            store,
        );
    }
}

/// Paint the Shapes flyout column to the right of the Shapes chip and
/// hit-register its 5 shape chips. Mirrors the main rail's chip geometry.
#[allow(clippy::too_many_arguments)]
fn paint_shapes_flyout(
    anchor: Rect,
    rail_rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let entries: Vec<ToolRailEntry> = PAINTER_SHAPES
        .iter()
        .map(|t| tool_entry(store, *t))
        .collect();
    let rail = ToolRail::new(NodeId(201), "Shape options", entries);
    let size = store.rail_button_size();
    let flyout_rect = Rect::new(
        rail_rect.x + rail_rect.w + Spacing::Xs.px(),
        anchor.y,
        size.rail_width_px(),
        rail.preferred_height(size),
    );
    let bg_rect = Rect::new(
        flyout_rect.x,
        flyout_rect.y - Spacing::Sm.px(),
        flyout_rect.w,
        flyout_rect.h + Spacing::Sm.px() * 2.0,
    );
    fill_rounded_rect(
        scene,
        bg_rect,
        Radius::Md.px(),
        resolve(ColorToken::RailBg, theme),
    );
    paint_tool_rail(&rail, flyout_rect, scene, text_system, theme, store);
    let chip_px = size.chip_px();
    let gap = Spacing::Xs.px();
    let chip_x = flyout_rect.x + CHIP_X_OFFSET_PX;
    let mut y = flyout_rect.y;
    for (i, entry) in rail.entries.iter().enumerate() {
        if i > 0 {
            y += gap;
        }
        if let ToolRailEntry::Icon { id, .. } = entry {
            hit_index.register(*id, Rect::new(chip_x, y, chip_px, chip_px));
            y += chip_px;
        }
    }
}
