//! ⭐⭐⭐ **O QUE A MÃO FAZ A UM OSSO** — irmão do [`super::bone_gesture`] pelo teto de 600 LOC do
//! HR-18, com o corte por RESPONSABILIDADE: ali mora *o que o dedo APONTA* (o hit-test, o hover, a
//! criação), aqui *o que a mão FAZ* depois de apontar.
//!
//! ⚠️ **A porta é um `match` EXAUSTIVO sobre [`ph2d_skeleton_render::BonePart`]**, e não uma cascata
//! de `if`: uma alça nova sem verbo é **erro de compilação** aqui, em vez de cair no braço do corpo
//! e girar o osso em silêncio — a família do *dreno de um braço só* que o `CLAUDE.md` §5 nomeia.

use ph2d_ecs::{ChildOf, Entity, SimWorld, Transform};
use ph2d_skeleton_ecs::Bone;
use ph2d_vec_scene::Xform;

use super::bone_gesture::{aim_rotation, reach_chain};

/// **POSAR um osso** — as duas metades do gesto, e por que são duas.
///
/// ⚠️ **O gizmo de sprite NÃO serve aqui, e isso foi medido:** ele dimensiona-se pela caixa da
/// geometria (`vec_gizmo_view::anchor_half` pede um `VecPathRef`), e um osso não tem geometria
/// nenhuma — a caixa sai `0×0` e as alças colapsam num ponto. ⇒ o osso posa-se **agarrando o osso**,
/// que é o gesto do Spine, do Moho e de todo pacote de rig.
///
/// ⚠️ **A lista das alças NÃO se escreve aqui** — ela é o [`ph2d_skeleton_render::BonePart`], que a
/// documenta variante a variante, e o `match` abaixo é EXAUSTIVO sobre ele (uma alça nova sem verbo
/// não compila). ⛔ Uma cópia em prosa envelhece à primeira wave, e envelheceu: esta nota
/// enumerava **três** (corpo · junta · alça da força) quando o `match` já tinha **seis** — a ponta
/// da IK e as duas paredes do limite entraram sem ela ser reconferida (auditoria de 2026-09-08).
///
/// *Duas coisas diferentes precisam de dois gestos*: sem o segundo, um esqueleto inteiro não se
/// move do sítio onde nasceu, e a única saída seria o painel de Transform.
pub(crate) fn pose(
    sim: &mut SimWorld,
    bone: Entity,
    world: [f64; 2],
    part: ph2d_skeleton_render::BonePart,
) -> bool {
    use ph2d_skeleton_render::BonePart;
    // ⭐⭐⭐ **UM `match` EXAUSTIVO, e não uma cascata de `if`** — a diferença é o que acontece
    // quando alguém acrescenta uma alça.
    //
    // ⛔⛔ A 1.ª redacção despachava por `if part == …` com o corpo do osso a apanhar tudo o que
    // sobrasse: uma variante nova de [`BonePart`] caía **no braço do `Body`** e girava o osso em
    // silêncio — a alça pintava, acendia sob o rato e fazia a coisa errada. É a família do *dreno
    // de um braço só* que o `CLAUDE.md` §5 nomeia, e nenhuma sonda deste repo a vê.
    //
    // ⇒ com o `match`, uma alça nova sem verbo é **erro de compilação** aqui.
    match part {
        BonePart::Influence => pose_influence(sim, bone, world),
        // ⭐⭐⭐ **A PONTA faz CINEMÁTICA INVERSA** — a corrente inteira dobra para o *end effector*
        // chegar onde a mão foi. É o degrau que separa um editor de esqueletos de um de animação.
        //
        // ⭐⭐ **Com ÂNCORA, o mesmo gesto move o ALVO em vez de posar a corrente**, e não é uma
        // conveniência: a restrição reescreve a pose dos ossos a cada quadro, então posá-los aqui
        // seria apagado no quadro seguinte e o artista veria o braço **voltar** debaixo do dedo. O
        // que ele arrasta passa a ser o objecto AUTORADO, que é o que o documento guarda — e é o
        // modelo do Blender e do Spine, onde um osso sob restrição não se posa à mão.
        BonePart::Tip => {
            if sim.world().get::<ph2d_skeleton_ecs::IkGoal>(bone).is_some() {
                crate::skeleton_goal::drag_anchor(sim, bone, world)
            } else {
                reach_chain(sim, bone, world)
            }
        }
        BonePart::Joint => pose_joint(sim, bone, world),
        // ⭐⭐⭐ **As duas alças do LIMITE DE ÂNGULO** — arrastá-las escreve a borda do arco.
        BonePart::LimitMin => crate::bone_limit::drag_edge(sim, bone, world, false),
        BonePart::LimitMax => crate::bone_limit::drag_edge(sim, bone, world, true),
        BonePart::Body => pose_body(sim, bone, world),
    }
}

/// A FORÇA — ela não é uma pose (não toca no `Transform`), é uma propriedade do osso.
fn pose_influence(sim: &mut SimWorld, bone: Entity, world: [f64; 2]) -> bool {
    let Some((_, a, b)) = crate::skeleton_live::bone_segments(sim)
        .into_iter()
        .find(|(x, _, _)| *x == bone.to_bits())
    else {
        return false;
    };
    let comp = (b[0] - a[0]).hypot(b[1] - a[1]);
    if comp <= f64::EPSILON {
        return false;
    }
    let raio = ph2d_skeleton::dist2_to_segment(world, a, b).sqrt();
    let Some(mut osso) = sim.world_mut().get_mut::<Bone>(bone) else {
        return false;
    };
    // ⛔ Piso em zero e **sem tecto**: §0.0 — não há recurso nenhum a limitar o alcance de um
    // osso, e um tecto aqui seria o desenho a decidir pelo artista. Força zero é legal e
    // significa *"só alcança quem não tem mais ninguém"* (o desempate do órfão).
    osso.strength = (raio / comp).max(0.0);
    true
}

/// A JUNTA — desloca o osso, no espaço do PAI.
fn pose_joint(sim: &mut SimWorld, bone: Entity, world: [f64; 2]) -> bool {
    // O espaço do PAI — a pose local vive nele. Sem pai, o mundo.
    let pai = sim.world().get::<ChildOf>(bone).map(ChildOf::parent);
    let pai_mundo = pai.map_or(Xform::IDENTITY, |p| {
        crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(sim, p))
    });
    let Some(inv) = pai_mundo.inverse() else {
        return false;
    };
    let p = inv.apply(world);
    let Some(mut t) = sim.world_mut().get_mut::<Transform>(bone) else {
        return false;
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o `Transform` da casa é f32; a geometria do documento é f64"
    )]
    {
        t.translation = ph2d_core::Vec2::new(p[0] as f32, p[1] as f32);
    }
    true
}

/// O CORPO — gira o osso para ele apontar ao ponteiro.
fn pose_body(sim: &mut SimWorld, bone: Entity, world: [f64; 2]) -> bool {
    // ⛔ Sobre a própria origem não há DIRECÇÃO — a porta devolve `None` e o osso fica quieto, que
    // é a resposta certa (apontar para lá daria um ângulo arbitrário e ele saltaria).
    let Some(r) = aim_rotation(sim, bone, world) else {
        return false;
    };
    // ⭐ O limite da junta apara o que o DEDO pede, e não só o que o solver pede.
    let r = crate::bone_limit::limited(sim, bone, r);
    let Some(mut t) = sim.world_mut().get_mut::<Transform>(bone) else {
        return false;
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a rotação do `Transform` da casa é f32; a geometria do documento é f64"
    )]
    {
        t.rotation = r as f32;
    }
    true
}

#[cfg(test)]
#[path = "bone_pose_tests.rs"]
mod pose_tests;
