//! Gates do [`super`] (`maos.rs`) — o que a mão do vetor segura.

use super::*;
use ph2d_ecs::MaosDasFamilias;

/// Sem gesto vivo, a mão está vazia — mesmo com selecção (seleccionar não é segurar).
#[test]
fn sem_gesto_a_mao_esta_vazia() {
    let mut vec = VecState::default();
    vec.entities.insert(3, 0x30);
    vec.pen.select_many(&[3]);
    assert!(!gesto_vivo(&vec));
    assert!(maos(&vec).is_empty());
}

/// Um gesto que age sobre a SELECÇÃO (o texto no caminho) segura as seleccionadas; o da gaiola do
/// Envelope segura a entidade dele, seleccionada ou não. (Mutação: `maos` sem a selecção ⇒ vazio.)
#[test]
fn o_gesto_segura_o_alvo_e_a_seleccao() {
    let mut vec = VecState::default();
    vec.entities.insert(3, 0x30);
    vec.pen.select_many(&[3]);
    vec.textpath_handle_drag = true;
    assert_eq!(maos(&vec), vec![0x30]);
    vec.textpath_handle_drag = false;
    vec.envelope_drag = Some((0x99, 2));
    assert_eq!(maos(&vec), vec![0x30, 0x99]);
}

/// Publicar põe a mão do vetor no mundo e o soltar esvazia-a (o quadro seguinte publica nada).
#[test]
fn publicar_poe_e_tira_a_mao_do_mundo() {
    let mut sim = SimWorld::new();
    let mut vec = VecState {
        envelope_drag: Some((0x99, 0)),
        ..VecState::default()
    };
    publicar(&vec, &mut sim);
    let r = sim.world().get_resource::<MaosDasFamilias>().cloned();
    assert_eq!(r.map(|m| m.todas().collect::<Vec<_>>()), Some(vec![0x99]));
    vec.envelope_drag = None;
    publicar(&vec, &mut sim);
    let r = sim.world().get_resource::<MaosDasFamilias>().cloned();
    assert!(r.is_some_and(|m| !m.alguma()));
}
