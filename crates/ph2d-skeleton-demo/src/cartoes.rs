//! ⭐ A montagem da cena do painel Bones em cartões (`PH2D_VEC_BONE_SMOKE=9`): uma barra e um
//! esqueleto-objecto de três ossos; o da PONTA traz IK (corrente de 2), Angle Limit, `Segments` > 1 e
//! curva autorada, para os cartões abrirem cheios.

use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_vec_scene::{ShapeKind, VecPathId, VecScene, cook_tinted as shape};

/// Quantos ossos a corrente tem.
pub const OSSOS: u32 = 3;
/// A barra: comprimento e espessura, em metros de mundo.
const BARRA: (f64, f64) = (9.0, 1.2);
/// O Angle Limit do osso em foco, em graus `(min, max)`.
const LIMITE_GRAUS: (f64, f64) = (-35.0, 50.0);
/// Os sub-ossos do osso em foco.
pub const SEGMENTOS: u8 = 6;

/// O que a cena devolve a quem a prende e a põe em foco.
pub struct Montagem {
    pub barra: VecPathId,
    pub raiz: Entity,
    pub ponta: Entity,
}

/// A barra, o esqueleto e o que o osso da ponta carrega; `smart` = já com Smart Bone (a foto não
/// clica no `+`). `None` = a corrente não nasceu.
pub fn monta(
    scene: &mut VecScene,
    sim: &mut SimWorld,
    cor: [u8; 3],
    smart: bool,
) -> Option<Montagem> {
    let (l, t) = BARRA;
    let barra = scene.push_path(shape(
        ShapeKind::RoundRect,
        [-l / 2.0, -t / 2.0],
        [l / 2.0, t / 2.0],
        &[t / 2.0],
        cor,
    ));
    let esq = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new(ph2d_i18n::tr("object_add.skeleton")),
            ph2d_skeleton_ecs::Skeleton,
        ))
        .id();
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    let x0 = -l / 2.0 + t / 2.0;
    let passo = (l - t) / f64::from(OSSOS);
    let mut pai = esq;
    let mut raiz = None;
    for k in 0..OSSOS {
        let a = [x0 + passo * f64::from(k), 0.0];
        let b = [x0 + passo * f64::from(k + 1), 0.0];
        pai = Entity::from_bits(ph2d_skeleton_live::bone::create(sim, Some(pai), a, b)?);
        raiz.get_or_insert(pai);
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let ponta = pai;
    if let Some(mut o) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::Bone>(ponta) {
        o.segments = SEGMENTOS;
        o.curve = ph2d_skeleton::bend::Bend {
            inn: [0.0, passo * 0.12],
            out: [0.0, passo * 0.12],
        };
    }
    sim.world_mut()
        .entity_mut(ponta)
        .insert(ph2d_skeleton_ecs::BoneLimit {
            min: LIMITE_GRAUS.0.to_radians(),
            max: LIMITE_GRAUS.1.to_radians(),
        });
    if smart {
        sim.world_mut()
            .entity_mut(ponta)
            .insert(ph2d_skeleton_ecs::SmartBone::default());
    }
    ph2d_constraints_ecs::add_ik(sim, ponta)?;
    Some(Montagem {
        barra,
        raiz: raiz?,
        ponta,
    })
}
