//! Gates da AÇÃO — os dois números, e a coerência que a porta impõe.

use super::*;
use crate::gamepad::GamepadButton;
use crate::keyboard::Key;

fn act() -> InputAction {
    InputAction::new(ActionId(0), "jump")
}

/// **O invariante que a porta existe para impor.**
///
/// ⚠️ Pedir `press_point` **abaixo** da `dead_zone` não é um erro do autor a rejeitar — é um
/// pedido a **coagir**, porque o estado que ele descreve (*"premida" com força zero*) não é
/// desenhável nem consumível. A coerção acontece na derivação, e por isso o estado incoerente
/// **não existe**, em vez de existir e ser filtrado em cada leitor.
#[test]
fn the_press_point_never_falls_below_the_dead_zone() {
    let a = act().with_zone(0.60, 0.10);
    assert!(
        a.press_point >= a.dead_zone,
        "press_point {} caiu abaixo da dead_zone {}",
        a.press_point,
        a.dead_zone
    );
    assert_eq!(a.dead_zone, 0.60, "a dead_zone pedida e' respeitada");
    assert_eq!(
        a.press_point, 0.60,
        "e o press_point sobe ate' ela, em vez de a acao nascer incoerente"
    );
}

/// A `dead_zone` de `1` seria uma divisão por zero na normalização — com cara de configuração
/// inocente, que é o pior disfarce que um defeito pode ter.
#[test]
fn a_dead_zone_of_one_can_never_be_authored() {
    let a = act().with_zone(1.0, 1.0);
    assert!(
        a.dead_zone < 1.0,
        "dead_zone {} tornaria `1 - dead_zone` um divisor zero",
        a.dead_zone
    );
    assert!((1.0 - a.dead_zone) > 0.0);
}

/// `NaN` num dos dois números viajaria até à **força** de uma acção, e daí para dentro de toda
/// subtracção que a lesse — `f32::clamp` devolve `NaN` para `NaN`, então o clamp sozinho não
/// chega.
#[test]
fn a_non_finite_zone_becomes_zero_not_a_poisoned_strength() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let a = act().with_zone(bad, bad);
        assert!(
            a.dead_zone.is_finite() && a.press_point.is_finite(),
            "{bad} sobreviveu a' porta: dz={} pp={}",
            a.dead_zone,
            a.press_point
        );
    }
}

/// Os dois vivem em `0..1`, das duas pontas.
#[test]
fn both_numbers_live_inside_the_unit_range() {
    let low = act().with_zone(-5.0, -5.0);
    assert_eq!((low.dead_zone, low.press_point), (0.0, 0.0));
    let high = act().with_zone(9.0, 9.0);
    assert!(high.dead_zone <= 0.99 && high.press_point <= 1.0);
}

/// **Zero ligações é um estado LEGÍTIMO**, e o painel depende disso: é o que lhe permite oferecer
/// *"agora escolha a tecla"*. Uma acção que só pudesse existir já ligada tornaria esse passo
/// inalcançável.
#[test]
fn an_action_is_born_with_no_bindings_and_that_is_valid() {
    assert!(act().bindings.is_empty());
}

/// N ligações, e são a **mesma** acção — teclado, comando, e a segunda tecla do canhoto.
#[test]
fn one_action_carries_many_bindings() {
    let a = act()
        .with(Binding::Key(Key(0x20)))
        .with(Binding::PadButton(GamepadButton::South))
        .with(Binding::Key(Key(0x5A)));
    assert_eq!(a.bindings.len(), 3);
}
