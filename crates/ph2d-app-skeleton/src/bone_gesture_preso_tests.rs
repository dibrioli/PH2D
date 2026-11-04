//! O gesto sobre um cartão preso: de que osso são os pedidos do painel.

use super::*;
use ph2d_ecs::Transform;

/// ⭐ T36: um gesto sobre um cartão preso a um osso é desse osso; preso a um objecto que não é osso,
/// da seleção. E a shell pergunta por esta porta: ela perdeu-a no porte da linha de teste, e o preso
/// mostrava um osso e escrevia noutro. *Mutação: `preso` ignorado, ou a shell com `selected_bone`.*
#[test]
fn the_requests_of_a_gesture_on_a_pinned_card_are_of_its_bone() {
    let mut sim = SimWorld::default();
    let preso = create(&mut sim, None, [0.0, 0.0], [10.0, 0.0]).expect("preso");
    let foco = create(&mut sim, None, [0.0, 5.0], [10.0, 5.0]).expect("foco");
    let objecto = sim.world_mut().spawn(Transform::default()).id().to_bits();
    assert_eq!(osso_do_gesto(&sim, Some(preso), [foco]), Some(preso));
    assert_eq!(osso_do_gesto(&sim, None, [foco]), Some(foco));
    assert_eq!(osso_do_gesto(&sim, Some(objecto), [foco]), Some(foco));
    assert_eq!(osso_do_gesto(&sim, Some(objecto), []), None);
    let dreno = include_str!("../../../shells/desktop/src/render_loop/fase_bus_drain.rs");
    assert!(
        dreno.contains("osso_selecionado: crate::bone_gesture::osso_do_gesto("),
        "o dreno da shell deixou de perguntar pelo preso"
    );
    // A shell tem ~20 `return` antes do `dispatch_key`: um atalho larga o preso no topo do `key_input`.
    let tecla = include_str!("../../../shells/desktop/src/input_dispatch/keyboard_bind_capture.rs");
    assert!(
        tecla.contains("hero.store.note_card_key();"),
        "a tecla da shell deixou de largar o preso"
    );
}
