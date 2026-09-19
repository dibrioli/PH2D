//! Os gates do [`super`] — o primitivo BOTÃO, testado campo a campo.
//!
//! ⚠️ **Eles vivem num irmão `#[path]` e continuam módulo FILHO** (logo alcançam o que é privado),
//! exactamente como os do `command_palette` — e pela mesma razão: manter o ficheiro do primitivo
//! sob o tecto de LOC. *Um tecto vermelho cura-se por CORTE, nunca por um número novo.*

use super::*;

fn fixture() -> Button {
    Button::new(NodeId(1), "Save")
}

#[test]
fn default_button_uses_text_primary_for_fg() {
    let b = fixture();
    assert_eq!(
        b.fg_color(Theme::Forge),
        ColorToken::Text1.resolve(Theme::Forge)
    );
}

#[test]
fn accent_button_paints_accent_fg() {
    let b = fixture().accent();
    assert_eq!(
        b.fg_color(Theme::Forge),
        ColorToken::AccentFg.resolve(Theme::Forge)
    );
}

#[test]
fn danger_button_paints_danger_bg() {
    let b = fixture().danger();
    assert_eq!(
        b.bg_color(Theme::Forge),
        Some(ColorToken::Danger.resolve(Theme::Forge))
    );
}

#[test]
fn danger_hover_softens() {
    let b = fixture().danger().state(ButtonState::Hovered);
    assert_eq!(
        b.bg_color(Theme::Forge),
        Some(ColorToken::DangerSoft.resolve(Theme::Forge))
    );
}

#[test]
fn icon_only_uses_icon_kind() {
    let b = fixture().icon_only(IconId::Save);
    assert!(matches!(
        b.kind,
        ButtonKind::IconOnly { icon: IconId::Save }
    ));
}

#[test]
fn disabled_overrides_fg() {
    let b = fixture().accent().state(ButtonState::Disabled);
    assert_eq!(
        b.fg_color(Theme::Sunstone),
        ColorToken::TextDisabled.resolve(Theme::Sunstone)
    );
}

#[test]
fn default_normal_has_no_bg() {
    assert!(fixture().bg_color(Theme::Forge).is_none());
}

#[test]
fn default_hover_lifts_to_bg_elev() {
    let b = fixture().state(ButtonState::Hovered);
    assert_eq!(
        b.bg_color(Theme::Forge),
        Some(ColorToken::BgElev.resolve(Theme::Forge))
    );
}

#[test]
fn focus_ring_only_when_focused() {
    assert!(!fixture().focus_ring());
    assert!(fixture().state(ButtonState::Focused).focus_ring());
}

#[test]
fn radius_uses_md_token() {
    assert_eq!(fixture().radius(Theme::Forge), Radius::Md.px());
}

#[test]
fn a11y_node_has_button_role_and_click() {
    let node = fixture().build_a11y(0.0, 0.0, 80.0, 32.0);
    assert_eq!(node.role(), Role::Button);
    assert_eq!(node.label(), Some("Save"));
    assert!(node.supports_action(Action::Click));
}

fn smoke(button: Button, theme: Theme) {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_button(
        &button,
        Rect::new(0.0, 0.0, 120.0, 32.0),
        &mut scene,
        &mut text,
        theme,
    );
}

#[test]
fn paint_smoke_normal() {
    smoke(fixture(), Theme::Forge);
}

#[test]
fn paint_smoke_hovered() {
    smoke(fixture().state(ButtonState::Hovered), Theme::Forge);
}

#[test]
fn paint_smoke_pressed() {
    smoke(fixture().state(ButtonState::Pressed), Theme::Forge);
}

#[test]
fn paint_smoke_focused() {
    smoke(fixture().state(ButtonState::Focused), Theme::Forge);
}

#[test]
fn paint_smoke_disabled() {
    smoke(fixture().state(ButtonState::Disabled), Theme::Forge);
}

#[test]
fn paint_smoke_accent() {
    smoke(fixture().accent(), Theme::Sunstone);
}

#[test]
fn paint_smoke_danger() {
    smoke(fixture().danger(), Theme::Sunstone);
}

#[test]
fn paint_smoke_icon_only() {
    smoke(fixture().icon_only(IconId::Settings), Theme::Forge);
}

#[test]
fn paint_smoke_loading_renders_spinner() {
    smoke(fixture().accent().state(ButtonState::Loading), Theme::Forge);
}
