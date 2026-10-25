//! A IK como item da pilha (W2, `docs/BlenderConstraints/02_W2_ossos.md` §2): escreve os ossos da
//! corrente acima do dono e devolve a pose LOCAL nova do dono, para a pilha continuar abaixo dela.

use std::collections::BTreeSet;

use bevy_ecs::entity::Entity;
use ph2d_constraints::{Ik, solve_chain};
use ph2d_ecs::{ChildOf, SimWorld, Transform, world_transform};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};
use ph2d_skeleton_ecs::Bone;
use ph2d_skeleton_live::bone::{aim_rotation, limited};
use ph2d_skeleton_live::goal::{aponta_com, governed, joints_of};

use crate::Estado;
use crate::avalia::base;

/// `alvo` está na corrente ou debaixo dela: mover a corrente moveria o alvo.
fn realimenta(sim: &SimWorld, alvo: Entity, corrente: &[Entity]) -> bool {
    let mut x = Some(alvo);
    let mut guarda = 0;
    while let Some(a) = x {
        if corrente.contains(&a) {
            return true;
        }
        x = sim.world().get::<ChildOf>(a).map(ChildOf::parent);
        guarda += 1;
        if guarda > 4096 {
            return true;
        }
    }
    false
}

#[expect(clippy::cast_possible_truncation, reason = "o `Transform` da casa é f32")]
fn f32_de(v: f64) -> f32 {
    v as f32
}

/// Avalia o item IK do osso `e` cujo estado na pilha é `local_e`. `avaliadas` = as entidades que a
/// pilha já avaliou neste quadro (a pose delas é a de agora; as outras partem do autorado).
pub(crate) fn avalia(
    sim: &mut SimWorld,
    preview: &mut PreviewDrive,
    e: Entity,
    local_e: Transform,
    ik: &Ik,
    alvo: Option<Entity>,
    forca: f64,
    avaliadas: &BTreeSet<u64>,
) -> Result<Transform, Estado> {
    if sim.world().get::<Bone>(e).is_none() {
        return Err(Estado::NaoEOsso);
    }
    let alvo = alvo.ok_or(Estado::SemAlvo)?;
    let corrente = governed(sim, e, ik.chain);
    if realimenta(sim, alvo, &corrente) {
        return Err(Estado::Ciclo);
    }
    // A corrente parte da pose de antes da IK: o estado da pilha para o dono, o autorado para os
    // ossos que a pilha ainda não avaliou neste quadro.
    let mut vistas = Vec::with_capacity(corrente.len());
    for &c in &corrente {
        let agora = *sim.world().get::<Transform>(c).ok_or(Estado::NaoEOsso)?;
        let partida = if c == e {
            local_e
        } else if avaliadas.contains(&c.to_bits()) {
            agora
        } else {
            base(sim, preview, c).unwrap_or(agora)
        };
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(c) {
            *t = partida;
        }
        vistas.push((c, agora, partida));
    }
    let raiz_mundo = world_transform(sim.world(), corrente[0]).ok_or(Estado::NaoEOsso)?;
    let espelhado = raiz_mundo.scale.x * raiz_mundo.scale.y < 0.0;
    let goal = world_transform(sim.world(), alvo).ok_or(Estado::SemAlvo)?;
    let goal = [f64::from(goal.translation.x), f64::from(goal.translation.y)];
    let (mut juntas, comps) = joints_of(sim, &corrente).ok_or(Estado::NaoEOsso)?;
    let k = solve_chain(&mut juntas, &comps, goal, ik, espelhado);
    // Os ângulos miram a corrente NÃO esticada (triângulos semelhantes); o esticar entra pela raiz.
    let r = juntas[0];
    let mira: Vec<[f64; 2]> = juntas
        .iter()
        .map(|j| [r[0] + (j[0] - r[0]) / k, r[1] + (j[1] - r[1]) / k])
        .collect();
    let desvio = if aponta_com(corrente.len()) { ik.offset } else { 0.0 };
    let mix = forca.clamp(0.0, 1.0);
    let mut dono = local_e;
    for (i, &(c, agora, partida)) in vistas.iter().enumerate() {
        let mut nova = partida;
        if let Some(rot) = aim_rotation(sim, c, mira[i + 1]) {
            let rot = ph2d_skeleton::blend_angle(f64::from(partida.rotation), rot + desvio, mix);
            nova.rotation = f32_de(limited(sim, c, rot));
        }
        if i == 0 && k > 1.0 {
            nova.scale.x = f32_de(f64::from(partida.scale.x) * k.powf(mix));
        }
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(c) {
            *t = nova;
        }
        if c == e {
            dono = nova;
            continue;
        }
        escreve(sim, preview, c, agora, partida, nova);
    }
    Ok(dono)
}

/// Declara ao ledger o que a IK escreveu num osso da corrente (o dono é escrito pela pilha).
fn escreve(
    sim: &mut SimWorld,
    preview: &mut PreviewDrive,
    c: Entity,
    agora: Transform,
    partida: Transform,
    nova: Transform,
) {
    if nova == agora {
        preview.still_driving(c, Driver::ConstraintPose);
    } else if nova == partida {
        preview.release_to_authored(sim, c, Driver::ConstraintPose);
    } else {
        preview.driven(c, Driven::ConstraintPose(agora), Driven::ConstraintPose(nova));
    }
}

/// Os ossos que cada item IK activo escreve (a corrente sem a ponta), para a ordem.
pub(crate) fn correntes(world: &ph2d_ecs::World, e: Entity, c: &crate::Constraints) -> Vec<Entity> {
    if world.get::<Bone>(e).is_none() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for it in c.items.iter().filter(|i| i.enabled && i.influence > 0.0) {
        if let ph2d_constraints::Kind::Ik(ik) = it.kind {
            out.extend(
                ph2d_skeleton_live::goal::governed_in(world, e, ik.chain)
                    .into_iter()
                    .filter(|&x| x != e),
            );
        }
    }
    out
}
