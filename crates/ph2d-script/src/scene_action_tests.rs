//! Os portões de `ph2d.action_*` (plano InputMap §5) — mesmos ajudantes do pai. O script lê a
//! acção pelo NOME, do mapa e `ActionState` que o Play tica.

use super::*;
use ph2d_input::{
    ActionState, Binding, Event, GamepadAxis, Input, InputMap, InputState, PLAYER_JUMP,
};

/// Um quadro do Play: o `ActionState` tica com o mapa, e a porta entrega-o ao script.
fn quadro(host: &ScriptHost, map: &InputMap, acts: &mut ActionState, dev: &InputState) {
    acts.tick(map, dev);
    host.devices().feed_actions(Some((map, acts)));
}

fn tecla(nome: &str) -> Key {
    Key::named_in_script(nome).expect("tecla")[0]
}

const SALTO: &str = r#"
function update(self)
  self.n = (self.n or 0) + 1
  if ph2d.action_pressed("jump") then ph2d.set(self.id, "x", self.n) end
  if ph2d.action_released("jump") then ph2d.set(self.id, "y", self.n) end
  if ph2d.action_down("jump") then ph2d.set(self.id, "rotation", self.n) end
end
"#;

/// A tecla do `jump` do mapa de fábrica, premida pelo mapa: `pressed` num tique só, `down`
/// enquanto segura, `released` ao largar.
#[test]
fn o_script_le_o_salto_pelo_nome_da_accao() {
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, "salto.luau", SALTO);
    let mut host = ScriptHost::new().expect("vm");
    let map = InputMap::with_player_defaults();
    let (mut acts, mut dev) = (ActionState::new(), InputState::new());
    quadro(&host, &map, &mut acts, &dev);
    tick(&mut host, &mut sim);
    dev.apply_event(Event::KeyDown(tecla("up")));
    quadro(&host, &map, &mut acts, &dev);
    tick(&mut host, &mut sim);
    quadro(&host, &map, &mut acts, &dev);
    tick(&mut host, &mut sim);
    let p = pose(sim.world(), e);
    assert_eq!(
        p.translation.x, 2.0,
        "pressed no tique em que desceu, e só nele"
    );
    assert_eq!(p.rotation, 3.0, "down enquanto segura");
    dev.apply_event(Event::KeyUp(tecla("up")));
    quadro(&host, &map, &mut acts, &dev);
    tick(&mut host, &mut sim);
    let p = pose(sim.world(), e);
    assert_eq!(p.translation.y, 4.0, "released ao largar");
    assert_eq!(p.rotation, 3.0, "largada já não está em baixo");
}

/// ⭐ O script lê a ACÇÃO, não a tecla: com o `jump` ligado ao J, a seta para cima não salta.
#[test]
fn a_accao_segue_o_mapa_e_nao_a_tecla() {
    let mut sim = SimWorld::new();
    let e = actor(
        sim.world_mut(),
        1,
        "j.luau",
        r#"function update(self) ph2d.set(self.id, "x", ph2d.action_down("jump") and 1 or 0) end"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    let mut map = InputMap::new();
    let id = map.create(PLAYER_JUMP);
    map.get_mut(id).expect("jump").bindings = vec![Binding::Key(tecla("j"))];
    let (mut acts, mut dev) = (ActionState::new(), InputState::new());
    dev.apply_event(Event::KeyDown(tecla("up")));
    quadro(&host, &map, &mut acts, &dev);
    tick(&mut host, &mut sim);
    assert_eq!(
        pose(sim.world(), e).translation.x,
        0.0,
        "a seta não é o salto"
    );
    dev.apply_event(Event::KeyDown(tecla("j")));
    quadro(&host, &map, &mut acts, &dev);
    tick(&mut host, &mut sim);
    assert_eq!(pose(sim.world(), e).translation.x, 1.0, "o J é");
}

/// Um nome que o mapa não tem pára o objecto com o nome errado e a lista das acções do mapa.
#[test]
fn um_nome_que_o_mapa_nao_tem_e_erro_que_lista_as_accoes() {
    let mut sim = SimWorld::new();
    let mau = actor(
        sim.world_mut(),
        1,
        "gralha.luau",
        "function update(self) ph2d.action_pressed('jmup') end",
    );
    let mut host = ScriptHost::new().expect("vm");
    let map = InputMap::with_player_defaults();
    let (mut acts, dev) = (ActionState::new(), InputState::new());
    quadro(&host, &map, &mut acts, &dev);
    tick(&mut host, &mut sim);
    let msg = host.scene_failure(mau).expect("pára").message.clone();
    assert!(msg.contains("unknown action 'jmup'"), "{msg}");
    for a in map.actions() {
        assert!(msg.contains(&format!("\"{}\"", a.name)), "{msg}");
    }
}

/// A força é a do resolvedor do `ph2d-input` (zona morta renormalizada), para um eixo sintético.
#[test]
fn a_forca_e_a_do_resolvedor() {
    let mut sim = SimWorld::new();
    let e = actor(
        sim.world_mut(),
        1,
        "forca.luau",
        r#"function update(self) ph2d.set(self.id, "x", ph2d.action_strength("acelera")) end"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    let mut map = InputMap::new();
    let id = map.create("acelera");
    let a = map.get_mut(id).expect("acelera");
    a.bindings = vec![Binding::PadAxis {
        axis: GamepadAxis::LeftStickX,
        positive: true,
    }];
    a.set_zone(0.2, 0.5);
    let (mut acts, mut dev) = (ActionState::new(), InputState::new());
    dev.gamepad.handle_axis(GamepadAxis::LeftStickX, 0.6);
    quadro(&host, &map, &mut acts, &dev);
    tick(&mut host, &mut sim);
    let esperado = Input::new(&map, &acts).strength("acelera");
    assert!(
        esperado > 0.0 && esperado < 1.0,
        "um valor do meio: {esperado}"
    );
    assert_eq!(pose(sim.world(), e).translation.x, esperado);
}
