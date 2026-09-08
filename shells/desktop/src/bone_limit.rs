//! ⭐⭐⭐ **O LIMITE DE UMA JUNTA** — irmão do [`super::bone_gesture`] pelo teto de 600 LOC do
//! HR-18, com o corte por RESPONSABILIDADE: ali mora *o que a mão faz a um osso*, aqui *até onde
//! ele pode ir*.
//!
//! ⚠️ **A [`limited`] é a porta que as DUAS mãos atravessam** — o gesto que gira o osso e o solver
//! que o gira a cada quadro. Um limite que só uma delas honrasse é pior que limite nenhum: a junta
//! obedeceria ou não conforme quem a moveu, e o artista não conseguiria formar um modelo do que a
//! ferramenta faz.
//!
//! ⛔ **O Godot põe isto na RESTRIÇÃO de IK** (`ccdik_joint_constraint_angle_min`/`_max`), e nós
//! pomos no OSSO — o modelo do Blender e do Moho. A razão está no doc do
//! [`ph2d_skeleton_ecs::BoneLimit`]: *«este cotovelo não dobra para trás»* é uma afirmação sobre a
//! ANATOMIA, logo vale sem IK nenhuma e não pode evaporar num *Remove IK*.

use ph2d_ecs::{Entity, SimWorld, Transform};

/// ⭐⭐⭐ **DÁ A ESTA JUNTA UM LIMITE, em volta da pose que ela TEM.**
///
/// ⚠️ **A faixa é capturada, como o lado da dobra da âncora** — e é a mesma razão: uma faixa
/// escrita de fora (o `Default` do componente, a volta inteira) não move a pose mas também não diz
/// nada ao artista; uma faixa em volta da pose ACTUAL nasce com o osso no meio dela, então ele vê
/// para onde pode ir e aperta a partir daí.
///
/// ⭐ **É um no-op exacto na mesma**, e não por sorte: a pose está no CENTRO da faixa, logo dentro
/// dela — e a lei devolve `rot` ao bit quando ele está dentro.
///
/// ⚠️ O quarto de volta é o valor de nascimento e não um tecto de recurso: é a maior faixa cujos
/// dois extremos um arrasto alcança sem o osso dar meia-volta, e cobre com folga a amplitude das
/// juntas que um rig tem (um cotovelo faz ~150°, um joelho ~140°, um dedo ~90°).
pub(crate) fn add_limit(sim: &mut SimWorld, bone: Entity) -> bool {
    if sim.world().get::<ph2d_skeleton_ecs::Bone>(bone).is_none()
        || sim
            .world()
            .get::<ph2d_skeleton_ecs::BoneLimit>(bone)
            .is_some()
    {
        return false;
    }
    let Some(t) = sim.world().get::<Transform>(bone).copied() else {
        return false;
    };
    let centro = f64::from(t.rotation);
    let meia = ph2d_skeleton::FULL_TURN / 8.0; // o quarto de volta, metade de cada lado
    sim.world_mut()
        .entity_mut(bone)
        .insert(ph2d_skeleton_ecs::BoneLimit {
            min: centro - meia,
            max: centro + meia,
        });
    true
}

/// Tira o limite: a junta volta a girar livremente. ⛔ Ela **não** é reposta na pose de repouso —
/// tirar uma cerca não é desfazer o que se fez dentro dela.
pub(crate) fn remove_limit(sim: &mut SimWorld, bone: Entity) -> bool {
    sim.world_mut()
        .entity_mut(bone)
        .take::<ph2d_skeleton_ecs::BoneLimit>()
        .is_some()
}

/// ⭐⭐⭐ **A ROTAÇÃO QUE ESTE OSSO PODE DE FACTO TER** — a pedida, aparada pelo limite dele.
///
/// ⚠️ **Porta única de dois consumidores, exactamente como a [`aim_rotation`] ao lado**: o gesto que
/// gira um osso à mão e a restrição que o gira a cada quadro. É isso que faz *«este cotovelo não
/// dobra para trás»* valer nas duas — um limite que só o solver honrasse seria um limite que o dedo
/// atravessa, e o artista veria a junta obedecer ou não conforme quem a moveu.
///
/// ⚠️ **Sem [`ph2d_skeleton_ecs::BoneLimit`] ela devolve `rot` AO BIT** — a junta sem limite é a
/// esmagadora maioria, e ela não paga nada por esta lei existir.
pub(crate) fn limited(sim: &SimWorld, bone: Entity, rot: f64) -> f64 {
    sim.world()
        .get::<ph2d_skeleton_ecs::BoneLimit>(bone)
        .map_or(rot, |l| ph2d_skeleton::clamp_to_limit(rot, l.min, l.max))
}
