//! **A escuta também apanha o COMANDO** (plano `docs/InputMap/01_plano.md` §3 «Escuta»): enquanto a
//! escuta partilhada está armada para uma acção da aba, o botão ou a haste viram a ligação dela,
//! pela mesma ranhura e pelo mesmo `INPUT_MAP_BIND_CAPTURED` da tecla. ⚠️ A BORDA (`pressed`):
//! um botão já em baixo ao armar não se liga; o botão ganha à haste (o comando parado tem resíduo).

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::ids::INPUT_MAP_BIND_CAPTURED;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_input::{Binding, GamepadAxis, GamepadButton, GamepadState};

/// Meio curso: limiar de GESTO («empurrou de propósito?»), não o `press_point` da acção.
pub const AXIS_PUSHED: f32 = 0.5;

/// A ligação do comando neste quadro: o 1.º botão com BORDA, senão a 1.ª haste `|v| ≥` [`AXIS_PUSHED`].
#[must_use]
pub fn pad_binding(pad: &GamepadState) -> Option<Binding> {
    GamepadButton::ALL
        .iter()
        .copied()
        .find(|b| pad.pressed(*b))
        .map(Binding::PadButton)
        .or_else(|| {
            GamepadAxis::ALL.iter().copied().find_map(|axis| {
                let v = pad.axis(axis);
                (v.abs() >= AXIS_PUSHED).then_some(Binding::PadAxis {
                    axis,
                    positive: v > 0.0,
                })
            })
        })
}

/// Entrega a ligação do comando à escuta armada de uma acção do mapa (não à do Trigger); `true` =
/// ligou. A shell chama-o uma vez por quadro, ANTES de resolver o Play.
pub fn deliver_pad(hero: &mut HeroScreen, pad: &GamepadState) -> bool {
    if hero.store.input_map_listening().is_none() || hero.store.input_map_listen_replaces() {
        return false;
    }
    let Some(b) = pad_binding(pad) else {
        return false;
    };
    hero.store.capture_binding(b);
    hero.apply_event(WidgetEvent::Click(INPUT_MAP_BIND_CAPTURED))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pad() -> GamepadState {
        let mut p = GamepadState::new();
        p.begin_frame();
        p
    }

    #[test]
    fn a_button_edge_is_a_pad_button() {
        let mut p = pad();
        p.handle_button_down(GamepadButton::South);
        assert_eq!(
            pad_binding(&p),
            Some(Binding::PadButton(GamepadButton::South))
        );
        // Ainda em baixo no quadro seguinte: sem borda, nada.
        p.begin_frame();
        assert_eq!(pad_binding(&p), None);
    }

    #[test]
    fn an_axis_push_picks_its_half_by_sign() {
        let mut p = pad();
        p.handle_axis(GamepadAxis::LeftStickX, -0.9);
        assert_eq!(
            pad_binding(&p),
            Some(Binding::PadAxis {
                axis: GamepadAxis::LeftStickX,
                positive: false
            })
        );
        p.handle_axis(GamepadAxis::LeftStickX, 0.7);
        assert_eq!(
            pad_binding(&p),
            Some(Binding::PadAxis {
                axis: GamepadAxis::LeftStickX,
                positive: true
            })
        );
    }

    #[test]
    fn below_the_threshold_is_nothing() {
        let mut p = pad();
        p.handle_axis(GamepadAxis::LeftStickY, AXIS_PUSHED - 0.01);
        p.handle_axis(GamepadAxis::RightStickX, -(AXIS_PUSHED - 0.01));
        assert_eq!(pad_binding(&p), None);
    }

    #[test]
    fn a_button_beats_an_axis() {
        let mut p = pad();
        p.handle_axis(GamepadAxis::LeftStickX, 1.0);
        p.handle_button_down(GamepadButton::East);
        assert_eq!(
            pad_binding(&p),
            Some(Binding::PadButton(GamepadButton::East))
        );
    }

    #[test]
    fn deliver_binds_only_for_a_map_listening_not_the_triggers() {
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        let id = hero.input_map.create("fire");
        let mut p = pad();
        p.handle_button_down(GamepadButton::South);
        hero.store.listen_for_binding_replacing(id);
        assert!(
            !deliver_pad(&mut hero, &p),
            "a escuta do Trigger é só de teclado"
        );
        hero.store.listen_for_binding(id);
        assert!(deliver_pad(&mut hero, &p));
        assert!(hero.store.input_map_listening().is_none(), "desarmou");
        let a = hero.input_map.get(id).expect("acção");
        assert!(
            a.bindings
                .contains(&Binding::PadButton(GamepadButton::South))
        );
    }
}
