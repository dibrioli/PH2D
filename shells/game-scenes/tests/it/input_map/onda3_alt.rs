//! Onda 3, item 4: Alt + tecla nunca liga sob a escuta partilhada.
//!
//! A shell empurra o Alt para o store em cada `ModifiersChanged` (`set_alt_held`, ao lado do `set_cmd_held`); a
//! escuta lê os dois. O acorde recusado SEGUE (o atalho do editor continua) e a escuta fica armada.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_editor_core::WidgetEvent;
use ph2d_editor_core::interaction::capture_if_listening;
use ph2d_input::{Binding, Key};

const Q: Key = Key('Q' as u32);

fn armada() -> (Mesa, ph2d_input::ActionId) {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let fire = m.hero.input_map.id("fire").expect("fire");
    m.hero.store.listen_for_binding(fire);
    (m, fire)
}

fn ligou(m: &mut Mesa, k: Key) -> bool {
    let e = capture_if_listening(&mut m.hero.store, k.0);
    matches!(e, Some(WidgetEvent::Click(id)) if id == ph2d_editor_core::ids::INPUT_MAP_BIND_CAPTURED)
}

#[test]
fn alt_q_sob_a_escuta_nao_liga_e_a_escuta_continua() {
    let (mut m, fire) = armada();
    m.hero.store.set_alt_held(true);
    assert!(
        capture_if_listening(&mut m.hero.store, Q.0).is_none(),
        "Alt+Q é do editor: segue"
    );
    assert_eq!(m.hero.store.take_captured_binding(), None);
    assert_eq!(m.hero.store.input_map_listening(), Some(fire));
}

#[test]
fn solto_o_alt_a_tecla_liga() {
    let (mut m, _) = armada();
    m.hero.store.set_alt_held(true);
    assert!(!ligou(&mut m, Q));
    m.hero.store.set_alt_held(false);
    assert!(ligou(&mut m, Q), "Alt solto: o Q liga");
    assert_eq!(m.hero.store.take_captured_binding(), Some(Binding::Key(Q)));
}

#[test]
fn alt_sozinho_segue_como_o_ctrl_sozinho() {
    let (mut m, fire) = armada();
    for n in ["left_alt", "right_alt", "left_ctrl", "right_ctrl"] {
        let k = Key::named_in_script(n).expect("tecla com nome")[0];
        assert!(!ligou(&mut m, k), "{n} sozinho é do editor");
    }
    assert_eq!(m.hero.store.input_map_listening(), Some(fire));
}
