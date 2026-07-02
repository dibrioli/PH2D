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
    let size = RailButtonSize::default();
    let h = fixture().preferred_height(size);
    assert!(h > size.chip_px() * 5.0);
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
fn swatch_active_setter_flips_active() {
    let entry = ToolRailEntry::swatch(NodeId(1), "Fill", [10, 20, 30, 255])
        .with_sub("FILL")
        .active();
    match entry {
        ToolRailEntry::Swatch {
            active, color, sub, ..
        } => {
            assert!(active);
            assert_eq!(color, [10, 20, 30, 255]);
            assert_eq!(sub, "FILL");
        }
        _ => panic!("expected Swatch"),
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
    let mut text = TextSystem::without_system_fonts();
    let rail = fixture();
    let size = RailButtonSize::default();
    let host = Rect::new(0.0, 0.0, TOOL_RAIL_WIDTH_PX, rail.preferred_height(size));
    let store = crate::interaction::WidgetStore::with_capacity(0);
    paint_tool_rail(&rail, host, &mut scene, &mut text, Theme::Forge, &store);
}

#[test]
fn paint_smoke_swatch_entry() {
    // A rail with a colour-swatch chip paints without panic (the Fill button's box).
    let rail = ToolRail::new(
        NodeId(1),
        "Swatch",
        vec![ToolRailEntry::swatch(NodeId(2), "Fill", [200, 40, 40, 255]).with_sub("FILL")],
    );
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let store = crate::interaction::WidgetStore::with_capacity(0);
    let size = RailButtonSize::default();
    paint_tool_rail(
        &rail,
        Rect::new(0.0, 0.0, TOOL_RAIL_WIDTH_PX, rail.preferred_height(size)),
        &mut scene,
        &mut text,
        Theme::Blueprint,
        &store,
    );
}

#[test]
fn paint_smoke_minimal_rail() {
    let rail = ToolRail::new(
        NodeId(1),
        "Tiny",
        vec![ToolRailEntry::icon(NodeId(2), "x", IconId::Add)],
    );
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let store = crate::interaction::WidgetStore::with_capacity(0);
    let size = RailButtonSize::default();
    paint_tool_rail(
        &rail,
        Rect::new(0.0, 0.0, TOOL_RAIL_WIDTH_PX, rail.preferred_height(size)),
        &mut scene,
        &mut text,
        Theme::Sunstone,
        &store,
    );
}
