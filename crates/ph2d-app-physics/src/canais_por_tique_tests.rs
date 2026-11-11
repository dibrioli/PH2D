//! **A curva da forma chega ao solver por TIQUE** (GEF02 onda 8 item 3): o canal FISICA (Height
//! do cartão) passa pelo `TimelineScene` em cada tique devido, como a escala do `Transform`.

use crate::bake::TimelineScene;
use crate::canais_corpo::{ConducaoFisica, aplicar, dono_da_chave};
use crate::physics_tests::registry;
use ph2d_core::Vec2;
use ph2d_ecs::canal::familia::FISICA;
use ph2d_ecs::canal_fisica as f;
use ph2d_ecs::{Entity, FieldKey, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge};
use ph2d_timeline::{PropKind, TimelineDoc};

const DT: f64 = 1.0 / 60.0;

fn cena() -> (SimWorld, Entity, TimelineDoc) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, -0.5)),
        ph2d_physics_ecs::RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 5.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
    ));
    let c = sim
        .world_mut()
        .spawn((
            Name::new("Crate"),
            Transform::from_translation(Vec2::new(0.0, 0.5)),
        ))
        .id();
    ph2d_physics_ecs::torna_corpo(sim.world_mut(), c, BodyKind::Dynamic, [0.5, 0.5], false);
    let prop = PropKind::Field(FieldKey::new(FISICA, f::MEIO_Y, 0, 0));
    let dono = dono_da_chave(sim.world(), c.to_bits(), prop);
    let mut doc = TimelineDoc::new();
    for (t, v) in [(1.0, 1.0), (1.5, 2.0), (2.0, 1.0), (2.5, 2.0)] {
        doc.insert_key(
            dono,
            prop,
            ph2d_anim::RationalTime::from_seconds(t),
            ph2d_anim::AnimValue::Float(v),
            ph2d_anim::Interp::Linear,
        );
    }
    (sim, c, doc)
}

/// Um quadro do app: a timeline no instante do alvo, os canais, a física até ao alvo.
fn quadro(
    sim: &mut SimWorld,
    doc: &mut TimelineDoc,
    b: &mut PhysicsBridge,
    cond: &mut ConducaoFisica,
    alvo: u64,
    joga: bool,
) {
    let reg = registry();
    ph2d_timeline::apply_from_doc(sim.world_mut(), doc, alvo as f64 * DT);
    aplicar(sim, &reg, cond, doc);
    let mut cena = TimelineScene { doc, fixed_dt: DT };
    b.dispatch_with_scene(sim, joga, alvo, &mut cena);
}

fn corre(
    passo: u64,
) -> (
    Vec<(u64, (f32, f32, f32))>,
    SimWorld,
    Entity,
    TimelineDoc,
    PhysicsBridge,
) {
    let (mut sim, c, mut doc) = cena();
    let mut b = PhysicsBridge::new();
    let mut cond = ConducaoFisica::default();
    let mut poses = Vec::new();
    let mut t = 0;
    while t < 180 {
        t += passo;
        quadro(&mut sim, &mut doc, &mut b, &mut cond, t, true);
        poses.push((t, b.body_pose(c).expect("caixote")));
    }
    (poses, sim, c, doc, b)
}

/// ⭐ Três tiques por quadro (máquina lenta) dão a corrida de um por quadro. Antes: 4,3 cm de desvio
/// do caixote (a forma só mudava por quadro); com o passo por tique e sem o 1.º tique antes do
/// reconcile, 0,65 cm.
#[test]
fn a_curva_da_forma_da_a_mesma_corrida_com_varios_tiques_por_quadro() {
    let (um, ..) = corre(1);
    let (tres, ..) = corre(3);
    for (t, p) in tres {
        let q = um[t as usize - 1].1;
        assert!(
            (p.1 - q.1).abs() < 1e-6,
            "tique {t}: {:.5} m contra {:.5}",
            p.1,
            q.1
        );
    }
}

/// Voltar atrás dá a pose do Play daquele tique.
#[test]
fn o_scrub_da_curva_da_forma_da_a_pose_do_play() {
    let (um, mut sim, c, mut doc, mut b) = corre(1);
    let mut cond = ConducaoFisica::default();
    for alvo in [150u64, 110, 80] {
        quadro(&mut sim, &mut doc, &mut b, &mut cond, alvo, false);
        assert_eq!(
            b.body_pose(c).expect("caixote"),
            um[alvo as usize - 1].1,
            "o salto a {alvo}"
        );
    }
}
