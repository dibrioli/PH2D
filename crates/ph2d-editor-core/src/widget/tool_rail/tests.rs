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

/// **A altura é a SOMA das entradas, com um gap entre cada par.**
///
/// ⚠️ **Este gate afirmava `h > chip_px * 5.0` e chamava-se *"sums entries"*.** Com onze entradas
/// esse piso fica muito abaixo da altura real, então ele não media soma nenhuma: medido, apagar
/// **todos os gaps** do laço deixa-o VERDE.
///
/// ⚠️ **O oráculo é DIFERENCIAL, e é por isso que ele não re-implementa a função sob teste** —
/// somar as entradas aqui seria escrever o mesmo laço num segundo lugar, e um oráculo que computa
/// a expectativa com a mesma aritmética do produto concorda com ele mesmo estando os dois errados.
/// A propriedade *"soma as entradas com um gap entre pares"* é exactamente: acrescentar uma
/// entrada custa **ela mais UM gap**, e a lista vazia custa **zero** (nenhum gap de cabeça).
#[test]
fn preferred_height_sums_entries() {
    let size = RailButtonSize::default();
    let gap = Spacing::Xs.px();

    let empty = ToolRail::new(NodeId(1), "empty", vec![]);
    assert_eq!(
        empty.preferred_height(size),
        0.0,
        "uma barra vazia tem altura zero — um gap de cabeca seria um vao que ninguem pediu"
    );

    // Cada entrada nova custa ELA + um gap, e o teste percorre uma barra que cresce.
    let mut entries: Vec<ToolRailEntry> = Vec::new();
    let mut previous = 0.0_f32;
    for (n, e) in fixture().entries.into_iter().enumerate() {
        let own = e.height(size);
        entries.push(e);
        let now = ToolRail::new(NodeId(1), "growing", entries.clone()).preferred_height(size);
        let expected = if n == 0 { own } else { previous + gap + own };
        assert!(
            (now - expected).abs() < 1e-3,
            "com {} entradas a altura deu {now}, esperava {expected} (anterior {previous} + gap \
             {gap} + a propria {own})",
            n + 1
        );
        previous = now;
    }
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
    let host = Rect::new(0.0, 0.0, tool_rail_width_px(), rail.preferred_height(size));
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
        Rect::new(0.0, 0.0, tool_rail_width_px(), rail.preferred_height(size)),
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
        Rect::new(0.0, 0.0, tool_rail_width_px(), rail.preferred_height(size)),
        &mut scene,
        &mut text,
        Theme::Sunstone,
        &store,
    );
}
