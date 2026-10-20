//! ⭐ **Os dispositivos que os scripts leem** (CodeEditor item 6, `docs/CodeEditor/05_api_que_funciona.md`
//! §3.5): o comando de jogo (`ph2d.input`), o teclado e o rato (`ph2d.key_*`, `ph2d.mouse_*`). Saiu da
//! shell, que só calcula o ponto do mundo sob o rato (a câmara é dela) e chama.

use ph2d_input::{Event, InputState, MouseButton};
use ph2d_script::ScriptHost;

/// ⭐ **Um quadro dos dispositivos para os scripts.** `mouse_world` = o ponto do mundo sob o rato.
///
/// ⛔⛔ **As bordas do teclado e do rato limpam-se AQUI, depois de entregues** — e não no início do
/// quadro: os eventos do winit chegam ENTRE quadros, e um `begin_frame` antes da entrega apagava-as
/// todas (auditoria do fecho, 20/10: `key_pressed`/`mouse_pressed` nunca disparavam no app). Sem
/// script também se limpam, senão acumulavam para sempre.
pub fn frame(host: Option<&ScriptHost>, input: &mut InputState, mouse_world: Option<[f64; 2]>) {
    let mut mouse_world = mouse_world;
    if crate::code_smoke::piloto_armado() {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        mouse_world = Some(crate::code_smoke::piloto(input, n));
    }
    if let Some(host) = host {
        feed(host, input, mouse_world);
    }
    input.keyboard.begin_frame();
    input.mouse.begin_frame();
}

fn feed(host: &ScriptHost, input: &InputState, mouse_world: Option<[f64; 2]>) {
    host.clear_input();
    for button in input.gamepad.iter_held() {
        host.provide_input(&format!("gamepad.held.{}", button.as_lua_key()), 1.0);
    }
    for (axis, value) in input.gamepad.iter_axes() {
        host.provide_input(
            &format!("gamepad.axis.{}", axis.as_lua_key()),
            f64::from(value),
        );
    }
    host.devices()
        .feed(&input.keyboard, &input.mouse, mouse_world);
}

/// ⭐ **Um botão do rato para o JOGO**: só um clique que COMEÇA na cena conta (um clique no Play
/// não é um tiro); o largar passa sempre, senão um arrasto que sai da cena ficava preso.
pub fn game_mouse(
    input: &mut InputState,
    button: ph2d_host::PointerButton,
    down: bool,
    on_scene: bool,
) {
    let b = match button {
        ph2d_host::PointerButton::Primary => MouseButton::Left,
        ph2d_host::PointerButton::Secondary => MouseButton::Right,
        ph2d_host::PointerButton::Middle => MouseButton::Middle,
    };
    if !down {
        input.apply_event(Event::MouseButtonUp(b));
    } else if on_scene {
        input.apply_event(Event::MouseButtonDown(b));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ O caminho do APP: o evento chega entre quadros (sem `begin_frame` antes), o script vê a
    /// borda num quadro, e no seguinte já não. Vermelho com a limpeza antes da entrega.
    #[test]
    fn a_borda_que_chega_entre_quadros_chega_ao_script_uma_vez() {
        let host = ScriptHost::new().expect("vm");
        let mut input = InputState::new();
        let space = [ph2d_input::Key(0x20)];
        input.apply_event(Event::KeyDown(space[0]));
        game_mouse(&mut input, ph2d_host::PointerButton::Primary, true, true);
        frame(Some(&host), &mut input, None);
        assert!(host.devices().key(&space, ph2d_script::Edge::Pressed));
        assert!(
            host.devices()
                .button(MouseButton::Left, ph2d_script::Edge::Pressed)
        );
        host.devices().edges_seen();
        frame(Some(&host), &mut input, None);
        assert!(
            !host.devices().key(&space, ph2d_script::Edge::Pressed),
            "uma vez só"
        );
        assert!(
            host.devices().key(&space, ph2d_script::Edge::Down),
            "e continua em baixo"
        );
    }

    #[test]
    fn um_clique_fora_da_cena_nao_conta_e_o_largar_passa_sempre() {
        let mut input = InputState::new();
        input.begin_frame();
        game_mouse(&mut input, ph2d_host::PointerButton::Primary, true, false);
        assert!(
            !input.mouse.pressed(MouseButton::Left),
            "um clique num painel não é do jogo"
        );
        game_mouse(&mut input, ph2d_host::PointerButton::Primary, true, true);
        assert!(input.mouse.pressed(MouseButton::Left));
        input.begin_frame();
        game_mouse(&mut input, ph2d_host::PointerButton::Primary, false, false);
        assert!(
            input.mouse.released(MouseButton::Left),
            "o largar fora da cena passa"
        );
    }
}
