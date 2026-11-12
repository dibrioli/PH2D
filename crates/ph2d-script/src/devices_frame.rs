//! A entrega dos dispositivos de UM quadro ao script: o gamepad, o teclado e o rato, e o mapa de acções
//! que o Play resolveu (`ph2d.action_*`). Quem limpa as bordas depois é o chamador (a shell).

use crate::ScriptHost;
use ph2d_input::{ActionState, InputMap, InputState};

impl ScriptHost {
    /// Entrega o quadro: `mouse_world` = o ponto do mundo sob o rato; `actions` = o mapa e o
    /// `ActionState` já ticado neste quadro (`None` sem mapa).
    pub fn feed_devices(
        &self,
        input: &InputState,
        mouse_world: Option<[f64; 2]>,
        actions: Option<(&InputMap, &ActionState)>,
    ) {
        self.clear_input();
        for button in input.gamepad.iter_held() {
            self.provide_input(&format!("gamepad.held.{}", button.as_lua_key()), 1.0);
        }
        for (axis, value) in input.gamepad.iter_axes() {
            self.provide_input(
                &format!("gamepad.axis.{}", axis.as_lua_key()),
                f64::from(value),
            );
        }
        self.devices()
            .feed(&input.keyboard, &input.mouse, mouse_world);
        self.devices().feed_actions(actions);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Edge;
    use ph2d_input::Event;

    /// ⭐ O caminho do APP: o Play tica o `ActionState` com o mapa, e a porta entrega ESSE estado
    /// ao script no mesmo quadro (`ph2d.action_*`).
    #[test]
    fn a_accao_que_o_play_resolveu_chega_ao_script() {
        let host = ScriptHost::new().expect("vm");
        let map = InputMap::with_player_defaults();
        let mut actions = ActionState::new();
        let mut input = InputState::new();
        let up = ph2d_input::Key::named_in_script("up").expect("up")[0];
        input.apply_event(Event::KeyDown(up));
        actions.tick(&map, &input);
        host.feed_devices(&input, None, Some((&map, &actions)));
        assert_eq!(host.devices().action("jump", Edge::Pressed), Some(true));
        assert_eq!(host.devices().action("dash", Edge::Down), Some(false));
        assert_eq!(
            host.devices().action("nada", Edge::Down),
            None,
            "fora do mapa"
        );
        host.feed_devices(&input, None, None);
        assert_eq!(
            host.devices().action("jump", Edge::Down),
            None,
            "sem mapa, nenhuma acção"
        );
    }
}
