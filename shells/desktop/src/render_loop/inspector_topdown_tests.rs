//! Os gates da tradução painel ⇄ lei e do dreno.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{Collider, ColliderShape};

fn cena(kind: BodyKind, com_platformer: bool) -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let mut e = sim.world_mut().spawn((
        TopDownPlayer::default(),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Ball { radius: 0.3 },
            ..Collider::default()
        },
    ));
    if com_platformer {
        e.insert(PlatformPlayer::default());
    }
    let bits = e.id().to_bits();
    (sim, bits)
}

#[test]
fn a_traducao_painel_lei_e_uma_ida_e_volta_exacta() {
    // ⛔ Sem isto, reordenar um enum de um dos lados troca o que um clique escreve — e compila.
    for d in InspectorMoveDirections::ALL {
        assert_eq!(dir_para_painel(dir_para_lei(d)), d, "{d:?}");
    }
    for v in InspectorViewpoint::ALL {
        assert_eq!(view_para_painel(view_para_lei(v)), v, "{v:?}");
    }
    for f in InspectorFacing::ALL {
        assert_eq!(face_para_painel(face_para_lei(f)), f, "{f:?}");
    }
}

#[test]
fn e_no_outro_sentido_tambem() {
    for d in [
        DirectionMode::Free,
        DirectionMode::EightWay,
        DirectionMode::FourWay,
        DirectionMode::AxisX,
        DirectionMode::AxisY,
    ] {
        assert_eq!(dir_para_lei(dir_para_painel(d)), d, "{d:?}");
    }
    for v in Viewpoint::ALL {
        assert_eq!(view_para_lei(view_para_painel(v)), v, "{v:?}");
    }
    for r in RotationMode::ALL {
        assert_eq!(face_para_lei(face_para_painel(r)), r, "{r:?}");
    }
}

#[test]
fn o_instantaneo_acusa_um_corpo_que_nao_serve() {
    // ⭐ O aviso que esta wave pagou a descobrir: um corpo dinâmico é do SOLVER, e o mover fica
    // sem pose para escrever — com todos os números certos no ecrã.
    let (sim, bits) = cena(BodyKind::Dynamic, false);
    let i = build_topdown_info(sim.world(), bits, 1, true).expect("tem o componente");
    assert!(i.has_body);
    assert!(!i.body_is_kinematic, "um corpo dinamico nao serve");
    assert!(!i.conflicts_with_platformer);

    let (sim, bits) = cena(BodyKind::Kinematic, false);
    let i = build_topdown_info(sim.world(), bits, 1, true).unwrap();
    assert!(i.body_is_kinematic);
}

#[test]
fn e_acusa_o_CONFLITO_de_dois_movers() {
    let (sim, bits) = cena(BodyKind::Kinematic, true);
    let i = build_topdown_info(sim.world(), bits, 1, true).unwrap();
    assert!(
        i.conflicts_with_platformer,
        "dois movers, um Transform — o painel tem de o dizer"
    );
}

#[test]
fn quem_nao_tem_o_componente_nao_tem_seccao() {
    let mut sim = SimWorld::new();
    let bits = sim.world_mut().spawn(()).id().to_bits();
    assert!(build_topdown_info(sim.world(), bits, 1, true).is_none());
}

#[test]
fn o_dreno_escreve_e_as_cercas_seguram() {
    let (mut sim, bits) = cena(BodyKind::Kinematic, false);
    let w = sim.world_mut();
    assert!(apply_topdown_edit(w, bits, &TopDownFieldEdit::Speed(9.5)));
    assert!(apply_topdown_edit(
        w,
        bits,
        &TopDownFieldEdit::Viewpoint(InspectorViewpoint::Iso30)
    ));
    // ⚠️ As cercas do painel escritas TAMBÉM aqui: o campo é alcançável por outra rota.
    assert!(apply_topdown_edit(w, bits, &TopDownFieldEdit::Speed(-4.0)));
    assert!(apply_topdown_edit(
        w,
        bits,
        &TopDownFieldEdit::ViewpointAngle(300.0)
    ));
    assert!(apply_topdown_edit(w, bits, &TopDownFieldEdit::MaxSlides(0)));
    let c = w.get::<TopDownPlayer>(Entity::from_bits(bits)).unwrap();
    let law = c.law();
    assert_eq!(law.speed, 0.0, "a velocidade nao fica negativa");
    assert_eq!(law.viewpoint, Viewpoint::Isometric30);
    assert!((1.0..=89.0).contains(&law.viewpoint_angle_deg));
    assert!(
        law.max_slides >= 1,
        "com ZERO deslizes o corpo para em toda parede — o componente deixaria de fazer a unica \
         coisa que existe para fazer"
    );
}

#[test]
fn um_dreno_sobre_quem_nao_tem_o_componente_nao_toca_em_nada() {
    let mut sim = SimWorld::new();
    let bits = sim.world_mut().spawn(()).id().to_bits();
    assert!(!apply_topdown_edit(
        sim.world_mut(),
        bits,
        &TopDownFieldEdit::Speed(3.0)
    ));
}
