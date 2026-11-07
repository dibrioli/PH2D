//! ⭐ **As regras do mundo e da mesa voltam com o Desfazer** (F30 §6 item 8, F41): a fotografia leva
//! o AUTORADO da ponte da física e da mesa, um passo que só as muda é um passo, e repor devolve-as ao
//! bit — sem mudar a forma gravada do estado (o ficheiro leva-as nos campos dele).

use super::*;
use crate::undo::Regras;

fn mundo() -> (ph2d_physics_ecs::PhysicsBridge, SimWorld) {
    let mut sim = SimWorld::new();
    ph2d_app_audio::mesa_doc::instalar(&mut sim, ph2d_audio::Mesa::default());
    (ph2d_physics_ecs::PhysicsBridge::new(), sim)
}

#[test]
fn as_regras_do_mundo_e_da_mesa_voltam_com_o_desfazer() {
    let (mut ponte, mut sim) = mundo();
    let antes = Regras::de(&ponte, &sim);
    let mut s = ponte.settings();
    s.gravity_y = -3.5;
    ponte.set_settings(s);
    let mut m = ph2d_audio::Mesa::default();
    m.bus[1].ganho = 0.4;
    ph2d_app_audio::mesa_doc::instalar(&mut sim, m);
    ph2d_physics_ecs::LayerNames::rename(sim.world_mut(), 2, "Enemy");
    let depois = Regras::de(&ponte, &sim);
    assert_ne!(
        depois.camadas, antes.camadas,
        "controlo: renomear muda as regras"
    );
    assert_ne!(depois.physics, antes.physics, "controlo: a gravidade mudou");
    assert_ne!(depois.mesa, antes.mesa, "controlo: a mesa mudou");

    let base = capture(&mut SimWorld::new(), &VecScene::new(), &registry());
    let estado = |r: Regras| ProjectState {
        regras: r,
        ..base.clone()
    };
    let (a, b) = (estado(antes.clone()), estado(depois.clone()));
    assert_eq!(
        a.parts_that_differ(&b),
        ["regras"],
        "um passo que só muda as regras é um passo"
    );
    assert_eq!(
        postcard::to_allocvec(&a).unwrap(),
        postcard::to_allocvec(&b).unwrap(),
        "a forma gravada do estado não as leva"
    );

    assert!(a.regras.repor(&mut ponte, &mut sim), "a física voltou");
    assert_eq!(
        Regras::de(&ponte, &sim),
        antes,
        "o Desfazer devolve as regras ao bit"
    );
    assert!(
        !a.regras.repor(&mut ponte, &mut sim),
        "repor o que já está não mexe"
    );
    assert!(b.regras.repor(&mut ponte, &mut sim));
    assert_eq!(Regras::de(&ponte, &sim), depois, "e o Refazer também");
}

/// O recibo do Remove compara o degrau do topo: outro passo por cima, ou o desfazer dele, têm de o mudar.
#[test]
fn cada_passo_novo_muda_o_topo_da_fila_e_o_refazer_nao_volta_ao_de_antes() {
    let s = capture(&mut SimWorld::new(), &VecScene::new(), &registry());
    let m = crate::undo::SelectionMark::default;
    let mut fila = ProjectUndo::default();
    assert_eq!(fila.top_at(false), None);
    fila.push_undo(s.clone(), m());
    let remocao = fila.top_at(false);
    assert!(remocao.is_some());
    fila.push_undo(s.clone(), m());
    let por_cima = fila.top_at(false);
    assert_ne!(por_cima, remocao, "a edição por cima mudou o topo");
    let _ = fila.undo(s.clone(), m());
    assert_eq!(
        fila.top_at(false),
        remocao,
        "o passo de baixo volta ao topo"
    );
    let _ = fila.redo(s.clone(), m());
    assert_ne!(fila.top_at(false), remocao);
}
