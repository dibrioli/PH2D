//! ⭐ **Os gestos do esqueleto sobre o mundo** — fazer um osso, e onde está a ponta dele.
//!
//! Lei pura sobre o `SimWorld`: zero `App`, zero `gfx`. Ela vivia em
//! `shells/desktop/src/bone_gesture.rs` e `bone_pick.rs`, e saiu porque a cena
//! `PH2D_VEC_BONE_SMOKE` a chama — ver o cabeçalho do [`crate`].
//!
//! ⛔ Os dois módulos da shell FICAM, a delegar: os 24 sítios que os nomeiam não se movem.

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform};
use ph2d_skeleton_ecs::Bone;
use ph2d_vec_scene::Xform;

/// **Faz um osso** de `origin` a `tip` (mundo), filho de `parent`. Devolve os bits dele.
///
/// `None` se a pose do pai é singular (escala zero) — não há espaço local em que pôr o osso.
pub fn create(
    sim: &mut SimWorld,
    parent: Option<Entity>,
    origin: [f64; 2],
    tip: [f64; 2],
) -> Option<u64> {
    // O espaço do PAI. Sem pai, o mundo — e aí o inverso é a identidade.
    let pai_mundo = parent.map_or(Xform::IDENTITY, |p| {
        ph2d_vec_entities::transform::xform_of_transform(
            ph2d_vec_entities::transform::world_transform(sim, p),
        )
    });
    let inv = pai_mundo.inverse()?;
    let a = inv.apply(origin);
    let b = inv.apply(tip);
    let d = [b[0] - a[0], b[1] - a[1]];
    let length = d[0].hypot(d[1]);
    let rotation = d[1].atan2(d[0]);
    // ⭐⭐⭐ **A POSE EM QUE ELE NASCE É O REPOUSO DELE** (2026-09-19). ⚠️ Escrita a partir do MESMO
    // `Transform` que o spawn leva — e não de uma segunda montagem dos números —, senão os dois
    // divergem no dia em que alguém acrescentar um campo à pose do osso novo. *Um osso que nasce
    // sem repouso é um osso a que o `Reset Transform` não sabe responder.*
    let pose = Transform {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "o `Transform` da casa é f32; a geometria do documento é f64"
        )]
        translation: ph2d_core::Vec2::new(a[0] as f32, a[1] as f32),
        #[expect(
            clippy::cast_possible_truncation,
            reason = "idem — a rotação do `Transform` é f32"
        )]
        rotation: rotation as f32,
        ..Transform::IDENTITY
    };
    let e = sim
        .world_mut()
        .spawn((
            pose,
            ph2d_skeleton_ecs::BoneRest::de(&pose),
            Bone {
                length,
                ..Bone::default()
            },
        ))
        .id();
    // ⚠️ **O nome tem de ser ÚNICO**, e não é cosmética: a referência durável entre objectos neste
    // app é o NOME (`stable_name_id`, o hash do `Name`) — dois ossos chamados "Bone" seriam o mesmo
    // sujeito para a timeline. O índice da entidade é único entre as vivas, que é o mesmo critério
    // que o `vec_entities` usa para um caminho novo.
    sim.world_mut()
        .entity_mut(e)
        .insert(Name::new(format!("Bone {}", e.index())));
    match parent {
        Some(p) => {
            sim.world_mut().entity_mut(e).insert(ChildOf(p));
        }
        None => {
            // ⛔ `RootOrder` EXPLÍCITO — sem ele a árvore desempata por bits de alocação, e o undo
            // vira um passo espúrio por quadro (BUGS #15).
            let order = ph2d_vec_entities::entities::next_root_order(sim);
            sim.world_mut().entity_mut(e).insert(RootOrder(order));
        }
    }
    Some(e.to_bits())
}

/// **A ponta de um osso, em MUNDO** — para o encaixe do próximo nascer colado nela.
pub fn tip_of(sim: &SimWorld, bits: u64) -> Option<[f64; 2]> {
    crate::skin_live::bone_segments(sim)
        .into_iter()
        .find(|(b, _, _)| *b == bits)
        .map(|(_, _, tip)| tip)
}

/// ⭐ **A rotação LOCAL que aponta o +X do osso a um ponto do MUNDO** — a porta única do gesto (arrastar
/// a ponta) e da IK da pilha: os dois dão o mesmo bit.
///
/// A lei é a da composição da casa (`Transform::compose`: a rotação do filho SOMA-SE à do pai), que é
/// a mesma com que o osso se desenha: rumo no mundo − rotação mundial do pai − o ângulo que a escala e
/// o skew mundiais do osso dão ao eixo +X (π sob um espelho). ⛔ Levar o ponto ao espaço do pai pelo
/// afim inverso só concorda com isto sob escala uniforme positiva: sob `sx = −1` o osso apontava ao
/// lado errado e sob escala desigual torto (medido na W2, `bend_side_flips_with_the_mirror`).
/// `None` com o ponto sobre a origem do osso.
#[must_use]
pub fn aim_rotation(sim: &SimWorld, bone: Entity, world: [f64; 2]) -> Option<f64> {
    let local = *sim.world().get::<Transform>(bone)?;
    let pai = ph2d_ecs::parent_world_transform(sim.world(), bone);
    let mundo = Transform::compose(pai, local);
    let o = [f64::from(mundo.translation.x), f64::from(mundo.translation.y)];
    let d = [world[0] - o[0], world[1] - o[1]];
    if d[0].hypot(d[1]) < f64::EPSILON {
        return None;
    }
    let forma = ph2d_vec_entities::transform::xform_of_transform(Transform {
        translation: ph2d_core::Vec2::new(0.0, 0.0),
        rotation: 0.0,
        ..mundo
    });
    let [a, b, ..] = forma.0;
    Some(d[1].atan2(d[0]) - f64::from(pai.rotation) - b.atan2(a))
}

/// ⭐ **A rotação que o osso pode ter** — a pedida, aparada pelo [`ph2d_skeleton_ecs::BoneLimit`] (o
/// gesto e a IK); sem limite devolve `rot` ao bit.
#[must_use]
pub fn limited(sim: &SimWorld, bone: Entity, rot: f64) -> f64 {
    sim.world()
        .get::<ph2d_skeleton_ecs::BoneLimit>(bone)
        .map_or(rot, |l| ph2d_skeleton::clamp_to_limit(rot, l.min, l.max))
}
