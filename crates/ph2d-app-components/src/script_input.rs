//! ⭐ **Os dispositivos que os scripts leem** (CodeEditor item 6, `docs/CodeEditor/05_api_que_funciona.md`
//! §3.5): o comando de jogo (`ph2d.input`), o teclado e o rato (`ph2d.key_*`, `ph2d.mouse_*`). Saiu da
//! shell, que só calcula o ponto do mundo sob o rato (a câmara é dela) e chama.

use ph2d_input::{ActionState, InputMap, InputState};
use ph2d_script::ScriptHost;

/// ⭐ **Um quadro dos dispositivos para os scripts.** `mouse_world` = o ponto do mundo sob o rato;
/// `actions` = o mapa e o `ActionState` que o Play resolveu NESTE quadro (`ph2d.action_*`), já
/// com o tique feito — `None` sem mapa.
///
/// ⛔⛔ **As bordas do teclado e do rato limpam-se AQUI, depois de entregues** — e não no início do
/// quadro: os eventos do winit chegam ENTRE quadros, e um `begin_frame` antes da entrega apagava-as
/// todas (auditoria do fecho, 20/10: `key_pressed`/`mouse_pressed` nunca disparavam no app). Sem
/// script também se limpam, senão acumulavam para sempre.
pub fn frame(
    host: Option<&ScriptHost>,
    input: &mut InputState,
    mouse_world: Option<[f64; 2]>,
    actions: Option<(&InputMap, &ActionState)>,
) {
    let mut mouse_world = mouse_world;
    if crate::code_smoke::piloto_armado() {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        mouse_world = Some(crate::code_smoke::piloto(input, n));
    }
    if let Some(host) = host {
        host.feed_devices(input, mouse_world, actions);
    }
    input.keyboard.begin_frame();
    input.mouse.begin_frame();
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_input::{Event, MouseButton};

    /// ⭐ O caminho do APP: o evento chega entre quadros (sem `begin_frame` antes), o script vê a
    /// borda num quadro, e no seguinte já não. Vermelho com a limpeza antes da entrega.
    #[test]
    fn a_borda_que_chega_entre_quadros_chega_ao_script_uma_vez() {
        let host = ScriptHost::new().expect("vm");
        let mut input = InputState::new();
        let space = [ph2d_input::Key(0x20)];
        input.apply_event(Event::KeyDown(space[0]));
        input.apply_event(Event::MouseButtonDown(MouseButton::Left));
        frame(Some(&host), &mut input, None, None);
        assert!(host.devices().key(&space, ph2d_script::Edge::Pressed));
        assert!(
            host.devices()
                .button(MouseButton::Left, ph2d_script::Edge::Pressed)
        );
        host.devices().edges_seen();
        frame(Some(&host), &mut input, None, None);
        assert!(
            !host.devices().key(&space, ph2d_script::Edge::Pressed),
            "uma vez só"
        );
        assert!(
            host.devices().key(&space, ph2d_script::Edge::Down),
            "e continua em baixo"
        );
    }
}
