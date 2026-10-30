//! ⭐⭐⭐ **O LIMITE DE UMA JUNTA** — irmão do [`super::bone_gesture`] pelo teto de 600 LOC do
//! HR-18, com o corte por RESPONSABILIDADE: ali mora *o que a mão faz a um osso*, aqui *até onde
//! ele pode ir*.
//!
//! A [`limited`] é a porta que as DUAS mãos atravessam (gesto e solver). O limite mora no OSSO
//! (Blender/Moho), não na restrição de IK (Godot): é anatomia e vale sem IK.

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_vec_scene::Xform;

/// ⭐⭐⭐ **DÁ A ESTA JUNTA UM LIMITE, em volta da pose que ela TEM.**
///
/// A faixa é capturada em volta da pose actual (no-op exacto: a pose fica no centro). O quarto de
/// volta é o valor de nascimento, não um tecto (a `drag_edge` alcança meia volta), e difere do
/// `Default` do [`ph2d_skeleton_ecs::BoneLimit`] (a volta inteira).
pub fn add_limit(sim: &mut SimWorld, bone: Entity) -> bool {
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
pub fn remove_limit(sim: &mut SimWorld, bone: Entity) -> bool {
    sim.world_mut()
        .entity_mut(bone)
        .take::<ph2d_skeleton_ecs::BoneLimit>()
        .is_some()
}

/// ⭐⭐⭐ **A ROTAÇÃO QUE ESTE OSSO PODE DE FACTO TER** — a pedida, aparada pelo limite dele.
///
/// Porta única do gesto e do solver; sem [`ph2d_skeleton_ecs::BoneLimit`] devolve `rot` ao bit.
pub fn limited(sim: &SimWorld, bone: Entity, rot: f64) -> f64 {
    ph2d_skeleton_live::bone::limited(sim, bone, rot)
}

/// Quantos segmentos o leque do arco tem por volta inteira.
///
/// Por VOLTA e não por arco, para a densidade não depender da largura da faixa.
const FAN_PER_TURN: usize = 48;
// O `48` é da resolução do ECRÃ: corda a `7,5°` ≈ `r/1170`, abaixo de meio pixel até 585 px.

/// ⭐⭐⭐ **O ARCO DE LIMITE de uma junta, em MUNDO** — o que se desenha e o que o dedo apanha.
///
/// O raio é o comprimento do osso; constrói-se no espaço do PAI e leva-se a mundo ponto a ponto
/// (sob um pai escalado num eixo o arco é uma elipse).
pub fn arc(
    sim: &SimWorld,
    bone: Entity,
    px_to_world: f64,
) -> Option<ph2d_skeleton_render::LimitArc> {
    let l = *sim.world().get::<ph2d_skeleton_ecs::BoneLimit>(bone)?;
    let comp = sim.world().get::<ph2d_skeleton_ecs::Bone>(bone)?.length;
    if !(comp.is_finite() && comp > 0.0) {
        return None;
    }
    let t = sim.world().get::<Transform>(bone)?;
    let o = [f64::from(t.translation.x), f64::from(t.translation.y)];
    let pai = sim
        .world()
        .get::<ph2d_ecs::ChildOf>(bone)
        .map(ph2d_ecs::ChildOf::parent);
    let pai_mundo = pai.map_or(Xform::IDENTITY, |p| {
        ph2d_vec_entities::transform::xform_of_transform(
            ph2d_vec_entities::transform::world_transform(sim, p),
        )
    });
    let ponto = |a: f64| pai_mundo.apply([o[0] + comp * a.cos(), o[1] + comp * a.sin()]);
    let (lo, hi) = (l.min.min(l.max), l.min.max(l.max));
    let faixa = (hi - lo).min(ph2d_skeleton::FULL_TURN);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a contagem sai de uma fracção de volta, sempre pequena e positiva"
    )]
    let n = ((faixa / ph2d_skeleton::FULL_TURN) * FAN_PER_TURN as f64).ceil() as usize;
    let n = n.max(1);
    let fan = (0..=n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "n é um punhado de segmentos")]
            let t = i as f64 / n as f64;
            ponto(lo + faixa * t)
        })
        .collect();
    // As alças saem para fora do alcance do osso: folga derivada em TELA (`BONE_HIT_PX` +
    // `LIMIT_HANDLE_R_PX`), por isso este passe precisa do zoom.
    let folga = (crate::bone_pick::BONE_HIT_PX + ph2d_skeleton_render::LIMIT_HANDLE_R_PX)
        * px_to_world.max(0.0);
    let fora = |a: f64| {
        pai_mundo.apply([
            o[0] + (comp + folga) * a.cos(),
            o[1] + (comp + folga) * a.sin(),
        ])
    };
    Some(ph2d_skeleton_render::LimitArc {
        apex: pai_mundo.apply(o),
        edge_min: ponto(lo),
        edge_max: ponto(hi),
        handle_min: fora(lo),
        handle_max: fora(hi),
        fan,
    })
}

/// ⭐⭐⭐ **ARRASTAR UMA PAREDE** — o ponteiro dá o ângulo, e ele vira a borda pedida.
///
/// Lê o ponteiro pela mesma porta do girar ([`crate::bone_gesture::aim_rotation`]); a borda
/// arrastada pára colada à outra, nunca a atravessa.
pub fn drag_edge(sim: &mut SimWorld, bone: Entity, world: [f64; 2], is_max: bool) -> bool {
    let Some(a) = crate::bone_gesture::aim_rotation(sim, bone, world) else {
        return false;
    };
    let Some(l) = sim
        .world()
        .get::<ph2d_skeleton_ecs::BoneLimit>(bone)
        .copied()
    else {
        return false;
    };
    // O ângulo vem em `(-π, π]` e a faixa vive em torno do centro — trazer a leitura para a volta
    // do OUTRO extremo é o que faz a parede seguir o dedo em vez de saltar meia volta.
    let outro = if is_max { l.min } else { l.max };
    let pedido = outro + ph2d_skeleton::wrap_pi(a - outro);
    let Some(mut m) = sim
        .world_mut()
        .get_mut::<ph2d_skeleton_ecs::BoneLimit>(bone)
    else {
        return false;
    };
    set_edge(&mut m, is_max, pedido);
    true
}

/// ⭐⭐⭐ **A PORTA ÚNICA DE MOVER UMA PAREDE** — e ela impede que uma paredes passe a outra.
///
/// Existe porque o gesto tinha o guarda e os campos não (`min > max` congelava a junta). A parede
/// empurrada além da vizinha pára NELA, nunca a atravessa.
pub fn set_edge(l: &mut ph2d_skeleton_ecs::BoneLimit, is_max: bool, pedido: f64) {
    if is_max {
        l.max = pedido.max(l.min);
    } else {
        l.min = pedido.min(l.max);
    }
}
