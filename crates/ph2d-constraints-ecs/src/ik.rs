//! A IK como item da pilha (W2, `docs/BlenderConstraints/02_W2_ossos.md` §2): escreve os ossos da
//! corrente acima do dono e devolve a pose LOCAL nova do dono, para a pilha continuar abaixo dela.

use std::collections::BTreeSet;

use bevy_ecs::entity::Entity;
use ph2d_constraints::{Ik, solve_chain};
use ph2d_ecs::{ChildOf, SimWorld, Transform, parent_world_transform, world_transform};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};
use ph2d_skeleton_ecs::Bone;
use ph2d_skeleton_live::bone::{aim_rotation_under, limited};
use ph2d_skeleton_live::goal::{aponta_com, governed, joints_of};

use crate::Estado;
use crate::avalia::base;

/// `alvo` está na corrente ou debaixo dela: mover a corrente moveria o alvo.
pub(crate) fn realimenta(sim: &SimWorld, alvo: Entity, corrente: &[Entity]) -> bool {
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

#[expect(
    clippy::cast_possible_truncation,
    reason = "o `Transform` da casa é f32"
)]
pub(crate) fn f32_de(v: f64) -> f32 {
    v as f32
}

/// Avalia o item IK do osso `e` cujo estado na pilha é `local_e`. `avaliadas` = as entidades que a
/// pilha já avaliou neste quadro (a pose delas é a de agora; as outras partem do autorado).
pub(crate) fn avalia(
    sim: &mut SimWorld,
    preview: &mut PreviewDrive,
    e: Entity,
    local_e: Transform,
    (ik, alvo, forca): (&Ik, Option<Entity>, f64),
    avaliadas: &mut BTreeSet<u64>,
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
    let desvio = if aponta_com(corrente.len()) {
        ik.offset
    } else {
        0.0
    };
    let mix = forca.clamp(0.0, 1.0);
    let mut dono = local_e;
    for (i, &(c, agora, partida)) in vistas.iter().enumerate() {
        // Um osso só OLHA: mira o próprio alvo (a porta do Look At), não a ponta que o alcance pôs.
        let ponto = if vistas.len() == 1 { goal } else { mira[i + 1] };
        let pai = parent_world_transform(sim.world(), c);
        let mut nova = olha(sim, c, (pai, partida), ponto, desvio, mix);
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
        // Escrito neste quadro: a próxima IK que o tem na corrente parte daqui, não do autorado.
        avaliadas.insert(c.to_bits());
    }
    Ok(dono)
}

/// ⭐ **Olhar para o alvo** — a lei do Look At na fase, objecto ou osso, e a da IK em cada osso da
/// corrente: o eixo +X DESENHADO aponta a `alvo` (a mira da casa, certa sob espelho e escala), mais o
/// desvio, misturado pelo ângulo no local e aparado pelo limite de junta. Devolve a pose local nova;
/// com o alvo sobre a origem, a de entrada.
pub(crate) fn olha(
    sim: &SimWorld,
    e: Entity,
    (pai, local): (Transform, Transform),
    alvo: [f64; 2],
    desvio: f64,
    forca: f64,
) -> Transform {
    let mut nova = local;
    if let Some(rot) = aim_rotation_under(pai, local, alvo) {
        let rot = ph2d_skeleton::blend_angle(f64::from(local.rotation), rot + desvio, forca);
        nova.rotation = f32_de(limited(sim, e, rot));
    }
    nova
}

/// Declara ao ledger o que a IK escreveu num osso da corrente (o dono é escrito pela pilha).
pub(crate) fn escreve(
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
        preview.driven(
            c,
            Driven::ConstraintPose(agora),
            Driven::ConstraintPose(nova),
        );
    }
}

/// As entidades de correntes de IK que se SOBREPÕEM (duas IKs a puxar ossos comuns), com as pontas:
/// a fase repete-as no mesmo quadro até assentarem (`docs/BlenderConstraints/02_W2_ossos.md` §2).
pub(crate) fn acopladas(world: &ph2d_ecs::World, ordem: &[Entity]) -> BTreeSet<u64> {
    let correntes: Vec<(u64, BTreeSet<u64>)> = ordem
        .iter()
        .filter_map(|&e| {
            let c = world.get::<crate::Constraints>(e)?;
            let mut s: BTreeSet<u64> = correntes(world, e, c).iter().map(|x| x.to_bits()).collect();
            (!s.is_empty()).then(|| {
                s.insert(e.to_bits());
                (e.to_bits(), s)
            })
        })
        .collect();
    let mut out = BTreeSet::new();
    for (i, (_, a)) in correntes.iter().enumerate() {
        for (_, b) in &correntes[i + 1..] {
            if !a.is_disjoint(b) {
                out.extend(a.iter().chain(b.iter()));
            }
        }
    }
    out
}

/// Os ossos que cada item IK activo escreve (a corrente sem a ponta), para a ordem.
pub(crate) fn correntes(world: &ph2d_ecs::World, e: Entity, c: &crate::Constraints) -> Vec<Entity> {
    if world.get::<Bone>(e).is_none() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for it in c.items.iter().filter(|i| i.enabled && i.influence > 0.0) {
        let chain = match it.kind {
            ph2d_constraints::Kind::Ik(ik) => ik.chain,
            ph2d_constraints::Kind::SplineIk(sp) => sp.chain,
            _ => continue,
        };
        {
            out.extend(
                ph2d_skeleton_live::goal::governed_in(world, e, chain)
                    .into_iter()
                    .filter(|&x| x != e),
            );
        }
    }
    out
}
