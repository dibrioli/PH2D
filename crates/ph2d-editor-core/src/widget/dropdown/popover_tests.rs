use super::*;

/// A linha com o foco do teclado é uma tinta que se vê sobre o popover e nunca o bloco do acento;
/// o anel é o do foco.
#[test]
fn the_focused_row_is_a_neutral_tint_a_step_above_the_popover() {
    for theme in ph2d_tokens::Theme::ALL {
        let tint = ColorToken::Bg3.resolve(theme);
        assert_ne!(
            tint,
            ColorToken::BgElev.resolve(theme),
            "{theme:?}: some no popover"
        );
        assert_ne!(
            tint,
            ColorToken::Accent.resolve(theme),
            "{theme:?}: o bloco cheio"
        );
        assert_ne!(
            ColorToken::BorderEmph.resolve(theme),
            tint,
            "{theme:?}: o anel some na tinta"
        );
    }
}

fn fixture() -> Dropdown<&'static str> {
    Dropdown::new(
        NodeId(1),
        "Tool",
        vec![
            DropdownOption::new(NodeId(2), "brush", "Brush"),
            DropdownOption::new(NodeId(3), "erase", "Erase"),
            DropdownOption::new(NodeId(4), "smudge", "Smudge"),
        ],
    )
}

#[test]
fn defaults_match_spec() {
    let d = fixture();
    assert!(d.selected.is_none());
    assert!(!d.open);
    assert_eq!(d.placeholder, "Select…");
}

#[test]
fn select_known_value() {
    let mut d = fixture();
    d.select("erase");
    assert_eq!(d.selected_label(), Some("Erase"));
}

#[test]
fn select_unknown_value_silent() {
    let mut d = fixture();
    d.select("nope");
    assert!(d.selected.is_none());
}

#[test]
fn a11y_role_is_combobox() {
    let node = fixture().build_a11y(0.0, 0.0, 200.0, 32.0);
    assert_eq!(node.role(), Role::ComboBox);
}

#[test]
fn a11y_option_role_is_listbox_option() {
    let node = fixture()
        .build_option_a11y(0, 0.0, 0.0, 200.0, 32.0)
        .unwrap();
    assert_eq!(node.role(), Role::ListBoxOption);
}

/// Um título (`header`) anuncia-se como `Heading`, sem foco nem clique; as opções não mudam.
#[test]
fn a11y_header_is_a_heading_not_an_option() {
    let mut d = fixture();
    d.options
        .insert(0, DropdownOption::new(NodeId(5), "t", "Tools").header(true));
    let h = d.build_option_a11y(0, 0.0, 0.0, 200.0, 32.0).unwrap();
    assert_eq!(h.role(), Role::Heading);
    assert!(!h.supports_action(Action::Click), "um título não se clica");
    let o = d.build_option_a11y(1, 0.0, 32.0, 200.0, 32.0).unwrap();
    assert_eq!(o.role(), Role::ListBoxOption);
    assert!(o.supports_action(Action::Click));
    assert!(
        !fixture().options.iter().any(|o| o.header),
        "header é `false` por omissão"
    );
    smoke(d.open(true), Theme::Forge);
}

fn smoke(d: Dropdown<&'static str>, theme: Theme) {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_dropdown(
        &d,
        Rect::new(0.0, 0.0, 200.0, 32.0),
        &mut scene,
        &mut text,
        theme,
    );
}

#[test]
fn popover_rect_sits_below_chip_with_gap() {
    let d = fixture();
    let chip = Rect::new(10.0, 20.0, 200.0, 32.0);
    let pop = d.popover_rect(chip);
    assert!(pop.y > chip.y + chip.h, "popover must be below chip");
    // Gap is non-zero so the two surfaces don't fuse into one.
    assert!((pop.y - (chip.y + chip.h)) >= 1.0);
    assert_eq!(pop.x, chip.x);
    assert_eq!(pop.w, chip.w);
}

#[test]
fn option_rect_inset_inside_popover() {
    let d = fixture();
    let chip = Rect::new(10.0, 20.0, 200.0, 32.0);
    let pop = d.popover_rect(chip);
    let r0 = d.option_rect(chip, 0);
    // Rows must live INSIDE the popover panel.
    assert!(r0.x > pop.x);
    assert!(r0.x + r0.w < pop.x + pop.w);
    assert!(r0.y >= pop.y);
    assert!(r0.y + r0.h <= pop.y + pop.h + 0.001);
}

#[test]
fn paint_smoke_closed_empty() {
    smoke(fixture(), Theme::Forge);
}

#[test]
fn paint_smoke_open_with_selection() {
    smoke(fixture().selected("erase").open(true), Theme::Sunstone);
}

#[test]
fn paint_smoke_focused() {
    smoke(fixture().state(DropdownState::Focused), Theme::Blueprint);
}

#[test]
fn paint_smoke_disabled() {
    smoke(fixture().state(DropdownState::Disabled), Theme::Workshop);
}

#[test]
fn content_height_counts_every_row() {
    let dd = fixture(); // 3 options
    let row_h = 20.0;
    let expected = row_h * 3.0 + POPOVER_PANEL_PAD_Y * 2.0;
    assert!((dd.content_height(row_h) - expected).abs() < 0.001);
}

#[test]
fn option_rect_scrolled_shifts_the_row_up_by_the_offset() {
    let dd = fixture();
    let chip = Rect::new(0.0, 0.0, 120.0, 20.0);
    let panel = dd.popover_rect(chip);
    let r0 = dd.option_rect_in_scrolled(chip, panel, 2, 0.0);
    let r1 = dd.option_rect_in_scrolled(chip, panel, 2, 15.0);
    assert!(
        (r0.y - r1.y - 15.0).abs() < 0.001,
        "scroll moves the row up by exactly the offset"
    );
    // scroll 0 matches the un-scrolled layout.
    assert!((dd.option_rect_in(chip, panel, 2).y - r0.y).abs() < 0.001);
}
