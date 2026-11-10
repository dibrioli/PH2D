//! O teclado de notebook no Luau (InputMap W1, plano `docs/InputMap/01_plano.md` §13.1).

use super::*;

const NUMERICO: &str = r#"
function update(self)
  self.n = (self.n or 0) + 1
  if ph2d.key_down("kp7") then ph2d.set(self.id, "x", self.n) end
  if ph2d.key_down("7") then ph2d.set(self.id, "y", self.n) end
end
"#;

/// ⛔ Numpad 7 ≠ 7: segurar o do numérico acende `key_down("kp7")` e deixa `key_down("7")` apagado.
#[test]
fn kp7_is_down_while_numpad_7_is_held_and_7_is_not() {
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, "numerico.luau", NUMERICO);
    let mut host = ScriptHost::new().expect("vm");
    let (mut kb, mouse) = (KeyboardState::new(), MouseState::default());
    kb.begin_frame();
    kb.handle_key_down(Key::named_in_script("kp7").expect("o numérico")[0]);
    host.devices().feed(&kb, &mouse, None);
    tick(&mut host, &mut sim);
    let p = pose(sim.world(), e);
    assert_eq!(p.translation.x, 1.0, "kp7 em baixo");
    assert_eq!(p.translation.y, 0.0, "o 7 da fileira não desceu");
}
