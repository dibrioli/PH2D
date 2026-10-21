//! ⭐⭐ **A família JUNTA nos canais** (plano 35 §3.0) — os campos do `PhysicsJoint` que a §12 do
//! Inspector edita: [`publicar`] o que o componente vale em `CanaisLidos`, [`aplicar`] o que a
//! curva mandou em `CanaisEscritos` (com o eco).
//!
//! ⭐ **A escrita é a da edição do painel**: cada campo vira a MESMA [`JointFieldEdit`] que o
//! Inspector emite e passa pelo funil único [`crate::joint::joint_with_edit`] (as conversões de
//! unidade, os pisos, o `clamped()`). Daí ao rapier o caminho é o de toda escrita no componente —
//! a do `PropKind::JointMotorTarget` da timeline incluída: o `reconcile_joints` da ponte reafina o
//! joint no lugar (`retune_joint`) e o `drive_joint_params` empurra-o por tique. A escrita da curva
//! é pré-visualização: declara-se ao [`PreviewDrive`] como `JointParams` (o joint inteiro, o
//! driver que as curvas de joint já usam).
//!
//! O canal guarda o valor do COMPONENTE (radianos numa dobradiça, metros num curso); o painel e o
//! funil falam a unidade do Inspector, e a conversão para lá é a porta `*_out` de cada um.

use ph2d_ecs::canal::familia::JUNTA;
use ph2d_ecs::canal_junta as j;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, Entity, FieldKey, SimWorld};
use ph2d_editor_core::JointFieldEdit;
use ph2d_physics_ecs::{AxisMode, CustomAxis, MotorMode, PhysicsJoint};
use ph2d_preview_drive::{Driven, PreviewDrive};

const fn k(campo: u16) -> FieldKey {
    FieldKey::new(JUNTA, campo, 0, 0)
}

/// Os campos de uma junta — todos, em qualquer tipo (o painel mostra os do tipo; o documento tem
/// todos, e trocar de tipo devolve os que o artista tinha).
#[must_use]
pub fn campos() -> Vec<FieldKey> {
    let mut out: Vec<FieldKey> = [
        j::MOTOR_LIGADO,
        j::MOTOR_MODO,
        j::MOTOR_FORCA,
        j::MOTOR_EIXO,
        j::QUEBRA_LIGADA,
        j::QUEBRA_FORCA,
        j::QUEBRA_TORQUE,
        j::LIMITES_LIGADOS,
        j::LIMITE_MIN,
        j::LIMITE_MAX,
        j::SOLDA_MOLE,
        j::RIGIDEZ,
        j::AMORTECIMENTO,
    ]
    .map(k)
    .to_vec();
    for a in 0..j::N_EIXOS {
        out.extend([j::EIXO_MODO, j::EIXO_MIN, j::EIXO_MAX].map(|b| k(j::do_eixo(b, a))));
    }
    out
}

fn bit(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

/// Quanto vale `kk` na junta — `None` fora do vocabulário.
#[must_use]
pub fn ler(pj: &PhysicsJoint, kk: FieldKey) -> Option<f32> {
    if kk.familia != JUNTA || kk.i != 0 || kk.j != 0 {
        return None;
    }
    if let Some((a, base)) = j::eixo_de(kk.campo) {
        let ax = pj.custom.axis(CustomAxis::from_tag(a as u8));
        return Some(match base {
            j::EIXO_MODO => f32::from(ax.mode.tag()),
            j::EIXO_MIN => ax.min,
            _ => ax.max,
        });
    }
    Some(match kk.campo {
        j::MOTOR_LIGADO => bit(pj.motor_enabled),
        j::MOTOR_MODO => bit(pj.motor_mode == MotorMode::Position),
        j::MOTOR_FORCA => pj.motor_max_force,
        j::MOTOR_EIXO => f32::from(pj.custom.motor_axis.tag()),
        j::QUEBRA_LIGADA => bit(pj.break_enabled),
        j::QUEBRA_FORCA => pj.break_force,
        j::QUEBRA_TORQUE => pj.break_torque,
        j::LIMITES_LIGADOS => bit(pj.limits_enabled),
        j::LIMITE_MIN => pj.limit_min,
        j::LIMITE_MAX => pj.limit_max,
        j::SOLDA_MOLE => bit(pj.soft),
        j::RIGIDEZ => pj.stiffness,
        j::AMORTECIMENTO => pj.damping,
        _ => return None,
    })
}

/// O índice de uma escolha discreta: arredonda e prende a `0..=max`.
fn tag(v: f32, max: u8) -> u8 {
    if v.is_finite() {
        v.round().clamp(0.0, f32::from(max)) as u8
    } else {
        0
    }
}

/// ⭐ **A edição do Inspector que escreve `v` em `kk`** — o valor do componente convertido para a
/// unidade do painel (a inversa da que o funil aplica).
#[must_use]
pub fn edicao(pj: &PhysicsJoint, kk: FieldKey, v: f32) -> Option<JointFieldEdit> {
    ler(pj, kk)?;
    if let Some((a, base)) = j::eixo_de(kk.campo) {
        let ax = a as u8;
        let ui = if CustomAxis::from_tag(ax).in_metres() {
            v
        } else {
            v.to_degrees()
        };
        return Some(match base {
            j::EIXO_MODO => JointFieldEdit::AxisMode(ax, tag(v, AxisMode::Locked.tag())),
            j::EIXO_MIN => JointFieldEdit::AxisMin(ax, ui),
            _ => JointFieldEdit::AxisMax(ax, ui),
        });
    }
    let on = v >= 0.5;
    Some(match kk.campo {
        j::MOTOR_LIGADO => JointFieldEdit::MotorEnabled(on),
        j::MOTOR_MODO => JointFieldEdit::MotorMode(tag(v, 1)),
        j::MOTOR_FORCA => JointFieldEdit::MotorMaxForce(v),
        j::MOTOR_EIXO => JointFieldEdit::MotorAxis(tag(v, CustomAxis::Rotation.tag())),
        j::QUEBRA_LIGADA => JointFieldEdit::BreakEnabled(on),
        j::QUEBRA_FORCA => JointFieldEdit::BreakForce(v),
        j::QUEBRA_TORQUE => JointFieldEdit::BreakTorque(v),
        j::LIMITES_LIGADOS => JointFieldEdit::LimitsEnabled(on),
        j::LIMITE_MIN => JointFieldEdit::LimitMin(crate::joint::limit_out(pj.kind, v)),
        j::LIMITE_MAX => JointFieldEdit::LimitMax(crate::joint::limit_out(pj.kind, v)),
        j::SOLDA_MOLE => JointFieldEdit::Soft(on),
        j::RIGIDEZ => JointFieldEdit::Stiffness(v),
        j::AMORTECIMENTO => JointFieldEdit::Damping(v),
        _ => return None,
    })
}

/// Escreve `v` em `kk` pelo funil do painel; devolve a junta como ficou.
#[must_use]
pub fn escrever(pj: PhysicsJoint, kk: FieldKey, v: f32) -> PhysicsJoint {
    edicao(&pj, kk, v)
        .and_then(|ed| crate::joint::joint_with_edit(pj, ed))
        .unwrap_or(pj)
}

/// ⭐ **Publica o que cada junta vale AGORA** (através do eco), esquecido antes o que cada entidade
/// publicou desta família (a junta tirada deixa de ter campos).
pub fn publicar(sim: &mut SimWorld, entidades: &[u64]) {
    let lotes: Vec<(u64, PhysicsJoint)> = entidades
        .iter()
        .filter_map(|&b| {
            let e = Entity::try_from_bits(b)?;
            let w = sim.world();
            w.get_entity(e).ok()?;
            Some((b, *w.get::<PhysicsJoint>(e)?))
        })
        .collect();
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for &b in entidades {
        lidos.esquecer(b, JUNTA);
    }
    for (b, pj) in lotes {
        for kk in campos() {
            if let Some(v) = ler(&pj, kk) {
                lidos.publicar(b, kk, v);
            }
        }
    }
}

/// ⭐ **Aplica às juntas o que a curva mandou neste quadro**, regista o eco e declara a escrita à
/// pré-visualização. Devolve `true` se alguma junta mudou.
pub fn aplicar(sim: &mut SimWorld, drive: &mut PreviewDrive) -> bool {
    let Some(mut escritas) = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(JUNTA))
    else {
        return false;
    };
    if escritas.is_empty() {
        return false;
    }
    // As escolhas antes dos números: trocar o eixo do motor ou o modo de um eixo muda o que os
    // números significam (o funil re-semeia o alvo do motor).
    escritas.sort_by_key(|&(e, kk, _)| (e, kk.info().is_none_or(|i| !i.discreto), kk));
    let mut mudou = false;
    let mut ecos = Vec::with_capacity(escritas.len());
    for lote in escritas.chunk_by(|a, b| a.0 == b.0) {
        let Some(e) = Entity::try_from_bits(lote[0].0) else {
            continue;
        };
        let Some(&antes) = sim
            .world()
            .get_entity(e)
            .ok()
            .and_then(|r| r.get::<PhysicsJoint>())
        else {
            continue;
        };
        let mut pj = antes;
        for &(b, kk, v) in lote {
            pj = escrever(pj, kk, v);
            if let Some(ficou) = ler(&pj, kk) {
                ecos.push((b, kk, v, ficou));
            }
        }
        if pj != antes {
            if let Some(mut c) = sim.world_mut().get_mut::<PhysicsJoint>(e) {
                *c = pj;
            }
            drive.driven(e, Driven::JointParams(antes), Driven::JointParams(pj));
            mudou = true;
        }
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (b, kk, v, f) in ecos {
        lidos.eco(b, kk, v, f);
    }
    mudou
}

#[cfg(test)]
#[path = "canais_junta_tests.rs"]
mod tests;
