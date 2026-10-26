//! **Look At é a porta única de «olhar para o alvo»** (decisão do dono 25/10, handoff W2 «Falta» 2):
//! num osso ele dá o bit da IK de um osso — a mira da casa (`aim_rotation`), o limite de junta, a
//! mistura pelo ângulo no local — e a IK `chain = 1` gravada converte-se nele.

use bevy_ecs::entity::Entity;
use ph2d_constraints::{Ik, Kind, LookAt};
use ph2d_constraints_ecs::{Constraints, converter_ik_de_um_osso, evaluate};
use ph2d_ecs::{ChildOf, SimWorld, StableId, Transform};
use ph2d_preview_drive::PreviewDrive;
use ph2d_skeleton_ecs::{Bone, BoneLimit};

struct Caso {
    nome: &'static str,
    pai_rot: f32,
    pai_escala: (f32, f32),
    limite: Option<(f64, f64)>,
    forca: f64,
    desvio: f64,
}

const CASOS: [Caso; 6] = [
    Caso {
        nome: "pai rodado",
        pai_rot: 30.0,
        pai_escala: (1.0, 1.0),
        limite: None,
        forca: 1.0,
        desvio: 0.0,
    },
    Caso {
        nome: "pai espelhado",
        pai_rot: 20.0,
        pai_escala: (-1.0, 1.0),
        limite: None,
        forca: 1.0,
        desvio: 0.0,
    },
    Caso {
        nome: "escala desigual",
        pai_rot: 15.0,
        pai_escala: (2.0, 0.5),
        limite: None,
        forca: 1.0,
        desvio: 0.0,
    },
    Caso {
        nome: "limite de junta",
        pai_rot: 0.0,
        pai_escala: (1.0, 1.0),
        limite: Some((-0.35, 0.35)),
        forca: 1.0,
        desvio: 0.0,
    },
    Caso {
        nome: "forca 50 %",
        pai_rot: 10.0,
        pai_escala: (1.0, 1.0),
        limite: None,
        forca: 0.5,
        desvio: 0.0,
    },
    Caso {
        nome: "desvio",
        pai_rot: 10.0,
        pai_escala: (1.0, 1.0),
        limite: None,
        forca: 1.0,
        desvio: 0.3,
    },
];

fn osso(sim: &mut SimWorld, id: u64, t: Transform, pai: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            StableId(id),
            t,
            Bone {
                length: 1.0,
                strength: 1.0,
                ..Default::default()
            },
        ))
        .id();
    if let Some(p) = pai {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

/// Raiz (o pai do caso) → o osso que olha, com o alvo em (−0,4; 2,7) no mundo.
fn cena(c: &Caso, kind: Kind) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let mut raiz = Transform::IDENTITY;
    raiz.translation.x = 0.5;
    raiz.translation.y = -0.25;
    raiz.rotation = c.pai_rot.to_radians();
    raiz.scale.x = c.pai_escala.0;
    raiz.scale.y = c.pai_escala.1;
    let r = osso(&mut sim, 1, raiz, None);
    let mut t = Transform::IDENTITY;
    t.translation.x = 1.0;
    t.rotation = 0.2;
    let b = osso(&mut sim, 2, t, Some(r));
    if let Some((min, max)) = c.limite {
        sim.world_mut().entity_mut(b).insert(BoneLimit { min, max });
    }
    let mut alvo = Transform::IDENTITY;
    alvo.translation.x = -0.4;
    alvo.translation.y = 2.7;
    sim.world_mut().spawn((StableId(10), alvo));
    let mut pilha = Constraints::default();
    let id = pilha.push(kind, Some(StableId(10)));
    pilha.item_mut(id).unwrap().influence = c.forca;
    sim.world_mut().entity_mut(b).insert(pilha);
    (sim, b)
}

fn avalia(c: &Caso, kind: Kind) -> Transform {
    let (mut sim, b) = cena(c, kind);
    let mut drive = PreviewDrive::default();
    evaluate(&mut sim, &mut drive);
    *sim.world().get::<Transform>(b).unwrap()
}

fn um_osso(desvio: f64) -> Kind {
    Kind::Ik(Ik {
        chain: 1,
        offset: desvio,
        ..Ik::default()
    })
}

#[test]
fn a_look_at_on_a_bone_is_the_one_bone_ik_to_the_bit() {
    let mut falhas = Vec::new();
    for c in &CASOS {
        let olha = avalia(c, Kind::LookAt(LookAt { offset: c.desvio }));
        let ik = avalia(c, um_osso(c.desvio));
        eprintln!(
            "{:16} look_at rot {:+.9} · ik rot {:+.9} · Δ {:+.3e}",
            c.nome,
            olha.rotation,
            ik.rotation,
            f64::from(olha.rotation) - f64::from(ik.rotation)
        );
        if olha != ik {
            falhas.push(c.nome);
        }
    }
    assert!(
        falhas.is_empty(),
        "o Look At no osso ≠ IK de 1 osso: {falhas:?}"
    );
}

/// A ponta DESENHADA do osso (a da casa, `skin_live`) fica no rumo do alvo, também sob espelho e escala
/// desigual — o rumo é o que o artista vê, não a soma das rotações.
#[test]
fn a_look_at_on_a_bone_points_the_drawn_tip_at_the_target() {
    for c in CASOS
        .iter()
        .filter(|c| c.limite.is_none() && c.forca >= 1.0 && c.desvio == 0.0)
    {
        let (mut sim, b) = cena(c, Kind::LookAt(LookAt::default()));
        let mut drive = PreviewDrive::default();
        evaluate(&mut sim, &mut drive);
        let bits = b.to_bits();
        let (_, cabeca, ponta) = ph2d_skeleton_live::skin_live::bone_segments(&sim)
            .into_iter()
            .find(|(x, ..)| *x == bits)
            .unwrap();
        let rumo = (ponta[1] - cabeca[1]).atan2(ponta[0] - cabeca[0]);
        let quer = (2.7 - cabeca[1]).atan2(-0.4 - cabeca[0]);
        let erro = ph2d_constraints::wrap_pi(rumo - quer).abs();
        eprintln!(
            "{:16} rumo desenhado − rumo do alvo = {erro:.3e} rad",
            c.nome
        );
        assert!(
            erro < 1e-5,
            "{}: a ponta não aponta ao alvo ({erro} rad)",
            c.nome
        );
    }
}

/// A IK `chain = 1` de um ficheiro (ou colagem) anterior vira um Look At no mesmo lugar da pilha, com o
/// mesmo id, alvo, força e desvio — e a pose não muda ao bit.
#[test]
fn a_saved_one_bone_ik_opens_as_a_look_at() {
    for c in &CASOS {
        let antes = avalia(c, um_osso(c.desvio));
        let (mut sim, b) = cena(c, um_osso(c.desvio));
        assert_eq!(converter_ik_de_um_osso(sim.world_mut()), 1);
        assert_eq!(converter_ik_de_um_osso(sim.world_mut()), 0, "idempotente");
        let pilha = sim.world().get::<Constraints>(b).unwrap().clone();
        assert_eq!(pilha.items.len(), 1);
        let it = pilha.items[0];
        assert_eq!(
            it.kind,
            Kind::LookAt(LookAt { offset: c.desvio }),
            "{}",
            c.nome
        );
        assert_eq!(
            (it.id, it.target, it.influence),
            (0, Some(StableId(10)), c.forca)
        );
        let mut drive = PreviewDrive::default();
        evaluate(&mut sim, &mut drive);
        assert_eq!(
            *sim.world().get::<Transform>(b).unwrap(),
            antes,
            "{}",
            c.nome
        );
    }
}

/// O mesmo num OBJECTO (sem `Bone`): sob um pai espelhado o eixo +X desenhado aponta o alvo. A lei
/// antiga apontava a soma das rotações e a frente do objecto olhava ao contrário (medido: π).
#[test]
fn a_look_at_on_a_mirrored_object_faces_the_target() {
    for (sx, sy) in [(-1.0_f32, 1.0_f32), (1.0, -1.0), (-2.0, 0.5)] {
        let mut sim = SimWorld::new();
        let mut pai = Transform::IDENTITY;
        pai.rotation = 0.4;
        pai.scale.x = sx;
        pai.scale.y = sy;
        let p = sim.world_mut().spawn((StableId(1), pai)).id();
        let mut t = Transform::IDENTITY;
        t.translation.x = 1.0;
        let o = sim.world_mut().spawn((StableId(2), t, ChildOf(p))).id();
        let mut alvo = Transform::IDENTITY;
        alvo.translation.x = 3.0;
        alvo.translation.y = 4.0;
        sim.world_mut().spawn((StableId(10), alvo));
        let mut pilha = Constraints::default();
        pilha.push(Kind::LookAt(LookAt::default()), Some(StableId(10)));
        sim.world_mut().entity_mut(o).insert(pilha);
        let mut drive = PreviewDrive::default();
        evaluate(&mut sim, &mut drive);
        let x = ph2d_vec_entities::transform::xform_of_transform(
            ph2d_vec_entities::transform::world_transform(&sim, o),
        );
        let [a, b, _, _, ox, oy] = x.0;
        let erro = ph2d_constraints::wrap_pi(b.atan2(a) - (4.0 - oy).atan2(3.0 - ox)).abs();
        assert!(
            erro < 1e-5,
            "pai ({sx}, {sy}): a frente desenhada erra {erro} rad"
        );
    }
}
