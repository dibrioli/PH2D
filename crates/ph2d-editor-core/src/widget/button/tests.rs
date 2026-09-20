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

/// ⭐⭐⭐ **UM BOTÃO PARADO PINTA A SUPERFÍCIE DELE** — e este gate tinha a PREMISSA INVERTIDA.
///
/// ⛔⛔ Ele chamava-se `default_normal_has_no_bg` e afirmava `is_none()`. Era o **defeito escrito
/// como lei**: report do dono, 2026-09-20, com foto — *«a aparência deste tipo de botão precisa
/// mudar: veja como não se pode saber que é um botão pois só aparece o nome. Todo o app tem essa
/// aparência ruim»*.
///
/// ⚠️ O que o desmentiu não foi gosto: o
/// [`crate::widget::button_surface::flat_button_surface`] — a lei que CINCO sítios de pintura já
/// usavam — declara `repouso Bg2` e diz, no doc, ser *«as mesmas superfícies que o `Button`
/// canónico usa»*. **Não eram.** ⇒ havia duas respostas a *«que cor tem um botão parado?»*, e o
/// doc de uma afirmava ser a outra.
///
/// ⭐ E o `border_color` deste mesmo ficheiro já escrevia o report inteiro — *«without it a
/// Normal-state Cancel / Reset is bare text indistinguishable from a label»* —, só que o tema
/// moderno desliga o traço que ele prometia (`Widgets::inactive.bg_stroke`, que o Godot só traça
/// com *Draw Extra Borders*). *Uma promessa escrita num sítio e desligada noutro.*
///
/// *Mutação que sangra:* devolver `None` ao repouso do `Default`.
#[test]
fn default_normal_paints_its_surface() {
    assert_eq!(
        fixture().bg_color(Theme::Forge),
        Some(ColorToken::Bg2.resolve(Theme::Forge)),
        "um botão parado tem de ter superfície — sem ela ele é texto centrado, e o artista não \
         tem como saber que ali há um botão",
    );
    // ⚠️ **E o CONTROLO: ele continua a LEVANTAR sob o rato.** Sem esta metade, pintar o repouso
    //    com a cor do hover passaria o `assert` de cima e apagaria a resposta ao ponteiro.
    assert_ne!(
        fixture().bg_color(Theme::Forge),
        fixture().state(ButtonState::Hovered).bg_color(Theme::Forge),
        "o repouso e o hover não podem ser a mesma cor",
    );
}

/// ⛔ **E o ícone-só FICA sem superfície**, que é a decisão oposta e é deliberada: um chip de fila
/// de ferramentas é uma grelha densa de ícones, e dar fundo a cada um faz a fila virar um tabuleiro
/// de xadrez. ⚠️ Ali a afordância é o ÍCONE e a posição na fila, não uma superfície.
///
/// *Mutação que sangra:* dar `Bg2` ao repouso do `IconOnly` junto com o do `Default`.
#[test]
fn an_icon_only_chip_stays_frameless_at_rest() {
    let b = fixture().kind(ButtonKind::IconOnly {
        icon: crate::IconId::Spinner,
    });
    assert!(
        b.bg_color(Theme::Forge).is_none(),
        "o chip de ícone da fila não leva superfície de repouso",
    );
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
