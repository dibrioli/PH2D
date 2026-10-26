//! A Spline IK como item da pilha (W3, `docs/BlenderConstraints/03_W3_poderosas.md` §2): deita a
//! corrente acima do dono na curva do alvo e devolve a pose LOCAL nova do dono, como a IK.

use std::collections::BTreeSet;

use bevy_ecs::entity::Entity;
use ph2d_constraints::{Ajuste, SplineIk, lay_on_curve};
use ph2d_ecs::{SimWorld, StableId, Transform, parent_world_transform};
use ph2d_preview_drive::PreviewDrive;
use ph2d_skeleton_ecs::Bone;
use ph2d_skeleton_live::goal::{governed, joints_of};

use crate::Estado;
use crate::avalia::{base, caminho_no_mundo};
use crate::ik::{escreve, f32_de, olha, realimenta};

/// Avalia o item Spline IK do osso `e` cujo estado na pilha é `local_e`.
pub(crate) fn avalia(
    sim: &mut SimWorld,
    preview: &mut PreviewDrive,
    e: Entity,
    local_e: Transform,
    (sp, alvo, forca): (&SplineIk, Option<(Entity, StableId)>, f64),
    avaliadas: &mut BTreeSet<u64>,
) -> Result<Transform, Estado> {
    if sim.world().get::<Bone>(e).is_none() {
        return Err(Estado::NaoEOsso);
    }
    let (alvo, id) = alvo.ok_or(Estado::SemAlvo)?;
    let corrente = governed(sim, e, sp.chain);
    if realimenta(sim, alvo, &corrente) {
        return Err(Estado::Ciclo);
    }
    let curva = caminho_no_mundo(sim, alvo, id).ok_or(Estado::SemAlvo)?;
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
    let (_, comps) = joints_of(sim, &corrente).ok_or(Estado::NaoEOsso)?;
    let juntas = lay_on_curve(&comps, &curva, sp.ajuste).ok_or(Estado::SemAlvo)?;
    let mix = forca.clamp(0.0, 1.0);
    let mut dono = local_e;
    // O esticar herda-se pelo `scale.x` do pai: cada osso pede o seu a dividir pelo do de cima.
    let mut herdado = 1.0;
    for (i, &(c, agora, partida)) in vistas.iter().enumerate() {
        let pai = parent_world_transform(sim.world(), c);
        let mut local = partida;
        if i == 0 {
            let mut mundo = Transform::compose(pai, partida);
            let (x, y) = (
                f64::from(mundo.translation.x),
                f64::from(mundo.translation.y),
            );
            mundo.translation.x = f32_de(x + (juntas[0][0] - x) * mix);
            mundo.translation.y = f32_de(y + (juntas[0][1] - y) * mix);
            local = Transform::inverse_compose(pai, mundo).unwrap_or(partida);
        }
        let mut nova = olha(sim, c, (pai, local), juntas[i + 1], 0.0, mix);
        if sp.ajuste == Ajuste::Esticar && comps[i] > 0.0 {
            let (a, b) = (juntas[i], juntas[i + 1]);
            let k = ((b[0] - a[0]).hypot(b[1] - a[1]) / comps[i]).powf(mix);
            nova.scale.x = f32_de(f64::from(partida.scale.x) * k / herdado);
            herdado = k;
        }
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(c) {
            *t = nova;
        }
        if c == e {
            dono = nova;
            continue;
        }
        escreve(sim, preview, c, agora, partida, nova);
        avaliadas.insert(c.to_bits());
    }
    Ok(dono)
}
