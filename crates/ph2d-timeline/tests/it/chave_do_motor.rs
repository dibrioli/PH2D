//! **O motor da junta na Timeline** (GEF02 onda 8 item 4): a faixa guarda o número do cartão.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{JointKind, PhysicsJoint};
use ph2d_timeline::{PropKind, TimelineDoc};

/// ⭐ **A chave do motor na Timeline é o número do CARTÃO** (GEF02 onda 8 item 4, dono 10/11: «corrija
/// para graus»): graus e graus/s numa dobradiça, metros num trilho, o eixo do motor num Custom — o
/// mesmo número que a §12 mostra, no referencial do objeto. A curva chega ao componente convertida
/// e lê-se de volta igual.
#[test]
fn a_chave_do_motor_e_o_numero_do_cartao() {
    let casos = [
        (
            JointKind::Pin,
            None,
            90.0,
            180.0,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
        ),
        (JointKind::Slider, None, 0.5, 2.0, 0.5, 2.0),
        (
            JointKind::Custom,
            Some(ph2d_physics_ecs::CustomAxis::X),
            0.25,
            1.0,
            0.25,
            1.0,
        ),
    ];
    for (kind, eixo, alvo, vel, alvo_rad, vel_rad) in casos {
        let mut sim = SimWorld::new();
        let mut pj = PhysicsJoint {
            kind,
            ..PhysicsJoint::default()
        };
        if let Some(a) = eixo {
            pj.custom.motor_axis = a;
        }
        let e = sim.world_mut().spawn(pj).id();
        let mut doc = TimelineDoc::new();
        for (p, v) in [
            (PropKind::JointMotorTarget, alvo),
            (PropKind::JointMotorSpeed, vel),
        ] {
            doc.insert_key(
                e.to_bits(),
                p,
                RationalTime::from_seconds(0.0),
                AnimValue::Float(v),
                Interp::Linear,
            );
        }
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut doc, 0.0);
        let j = sim.world().get::<PhysicsJoint>(e).expect("junta");
        assert!(
            (j.motor_target - alvo_rad).abs() < 1e-5,
            "{kind:?}: alvo {}",
            j.motor_target
        );
        assert!(
            (j.motor_speed - vel_rad).abs() < 1e-5,
            "{kind:?}: velocidade {}",
            j.motor_speed
        );
        assert!(
            (j.motor_no_cartao(j.motor_target) - alvo).abs() < 1e-4,
            "{kind:?}: volta ao cartão"
        );
    }
}
