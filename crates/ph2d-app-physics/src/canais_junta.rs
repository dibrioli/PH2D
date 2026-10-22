//! ⭐⭐ **A família JUNTA nos canais** (plano 35 §3.0) — todo campo que a §12 do Inspector (a junta)
//! e a §13 (a roldana de uma Pulley) editam: [`publicar`] o que o documento vale em `CanaisLidos`,
//! [`aplicar`] o que a curva mandou em `CanaisEscritos` (com o eco).
//!
//! ⭐ **A escrita é a da edição do painel**: cada campo vira a MESMA [`JointFieldEdit`] /
//! [`WheelFieldEdit`] que o Inspector emite e passa pelo funil único
//! ([`crate::joint::joint_with_edit`], [`crate::joint_wheel::wheel_with_edit`]: as conversões de
//! unidade, os pisos, o `clamped()`); os dois MARCADORES (o lado B no mundo, a talha de Weston)
//! pelas portas que o painel também chama. Daí ao rapier o caminho é o de toda escrita no
//! componente: o `reconcile_joints` da ponte relê juntas e roldanas a cada despacho. A escrita da
//! curva é pré-visualização: declara-se ao [`PreviewDrive`] (`JointParams`, `PontaNoMundo`,
//! `Roldana`).
//!
//! O canal guarda o valor do COMPONENTE (radianos numa dobradiça, metros num curso, `order`
//! 0-based); o painel e o funil falam a unidade do Inspector, e a conversão para lá é a porta
//! `*_out` de cada um.

use ph2d_ecs::canal::familia::JUNTA;
use ph2d_ecs::canal_junta as j;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, Entity, FieldKey, SimWorld, World};
use ph2d_editor_core::{JointFieldEdit, WheelFieldEdit};
use ph2d_physics_ecs::{
    AxisMode, CustomAxis, JointKind, JointWorldAnchor, MotorMode, PhysicsJoint, PulleyWheel,
    WestonAxle, WrapSide,
};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};

const fn k(campo: u16) -> FieldKey {
    FieldKey::new(JUNTA, campo, 0, 0)
}

/// Os campos do `PhysicsJoint` — todos, em qualquer tipo (o painel mostra os do tipo; o documento
/// tem todos, e trocar de tipo devolve os que o artista tinha).
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
        j::TIPO,
        j::ATIVA,
        j::COLIDE_PAR,
    ]
    .map(k)
    .to_vec();
    for a in 0..j::N_EIXOS {
        out.extend([j::EIXO_MODO, j::EIXO_MIN, j::EIXO_MAX].map(|b| k(j::do_eixo(b, a))));
    }
    out
}

/// Os campos de uma roldana (`PulleyWheel` + o marcador `WestonAxle`).
#[must_use]
pub fn campos_roda() -> Vec<FieldKey> {
    [
        j::RODA_RAIO,
        j::RODA_RAIO_SAIDA,
        j::RODA_WESTON,
        j::RODA_ORDEM,
        j::RODA_LADO,
        j::RODA_MOTOR,
        j::RODA_QUEBRA_LIGADA,
        j::RODA_QUEBRA_FORCA,
    ]
    .map(k)
    .to_vec()
}

const fn e_da_roda(campo: u16) -> bool {
    campo >= j::RODA_RAIO && campo <= j::RODA_QUEBRA_FORCA
}

fn bit(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

/// Quanto vale `kk` no `PhysicsJoint` — `None` fora destes campos.
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
        j::TIPO => f32::from(crate::joint::tag_of(pj.kind)),
        j::ATIVA => bit(pj.active),
        j::COLIDE_PAR => bit(pj.collide_connected),
        _ => return None,
    })
}

/// Quanto vale `kk` numa roldana (`weston` = o marcador está lá).
#[must_use]
pub fn ler_roda(w: &PulleyWheel, weston: bool, kk: FieldKey) -> Option<f32> {
    if kk.familia != JUNTA || kk.i != 0 || kk.j != 0 {
        return None;
    }
    Some(match kk.campo {
        j::RODA_RAIO => w.radius,
        j::RODA_RAIO_SAIDA => w.radius_out,
        j::RODA_WESTON => bit(weston),
        j::RODA_ORDEM => f32::from(w.order),
        j::RODA_LADO => f32::from(w.wrap.tag()),
        j::RODA_MOTOR => w.motor_speed,
        j::RODA_QUEBRA_LIGADA => bit(w.break_enabled),
        j::RODA_QUEBRA_FORCA => w.break_force,
        _ => return None,
    })
}

/// ⭐ **Quanto vale `kk` na entidade `e`** — a junta, o marcador do lado B, ou a roldana.
#[must_use]
pub fn ler_mundo(w: &World, e: Entity, kk: FieldKey) -> Option<f32> {
    if e_da_roda(kk.campo) {
        return ler_roda(
            w.get::<PulleyWheel>(e)?,
            w.get::<WestonAxle>(e).is_some(),
            kk,
        );
    }
    let pj = w.get::<PhysicsJoint>(e)?;
    if kk.campo == j::PONTA_NO_MUNDO && kk.familia == JUNTA && kk.i == 0 && kk.j == 0 {
        return Some(bit(w.get::<JointWorldAnchor>(e).is_some()));
    }
    ler(pj, kk)
}

/// `bits` é a corda de uma Pulley — o comprimento dela (`max_length`) cai da rota, não da mão.
#[must_use]
pub fn e_polia(w: &World, bits: u64) -> bool {
    Entity::try_from_bits(bits)
        .and_then(|e| w.get::<PhysicsJoint>(e))
        .is_some_and(|j| j.kind == JointKind::Pulley)
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
        j::TIPO => JointFieldEdit::Kind(tag(v, (JointKind::ALL.len() - 1) as u8)),
        j::ATIVA => JointFieldEdit::Active(on),
        j::COLIDE_PAR => JointFieldEdit::CollideConnected(on),
        _ => return None,
    })
}

/// A edição da §13 que escreve `v` em `kk` (graus no motor, `order` 1-based, como o painel).
#[must_use]
pub fn edicao_roda(kk: FieldKey, v: f32) -> Option<WheelFieldEdit> {
    ler_roda(&PulleyWheel::default(), false, kk)?;
    let on = v >= 0.5;
    Some(match kk.campo {
        j::RODA_RAIO => WheelFieldEdit::Radius(v),
        j::RODA_RAIO_SAIDA => WheelFieldEdit::RadiusOut(v),
        j::RODA_WESTON => WheelFieldEdit::Weston(on),
        j::RODA_ORDEM => {
            let o = if v.is_finite() {
                v.round().clamp(0.0, f32::from(u16::MAX))
            } else {
                0.0
            };
            WheelFieldEdit::Order(o as u32 + 1)
        }
        j::RODA_LADO => WheelFieldEdit::Wrap(tag(v, WrapSide::Under.tag())),
        j::RODA_MOTOR => WheelFieldEdit::MotorDegPerS(v.to_degrees()),
        j::RODA_QUEBRA_LIGADA => WheelFieldEdit::BreakEnabled(on),
        j::RODA_QUEBRA_FORCA => WheelFieldEdit::BreakForce(v),
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

/// Escreve `v` em `kk` numa roldana; devolve `(roldana, weston)` como ficaram. A talha de Weston é o
/// MARCADOR (o painel intercepta-a antes do funil de campo, e aqui também).
#[must_use]
pub fn escrever_roda(w: PulleyWheel, weston: bool, kk: FieldKey, v: f32) -> (PulleyWheel, bool) {
    match edicao_roda(kk, v) {
        Some(WheelFieldEdit::Weston(on)) => (w, on),
        Some(ed) => (
            crate::joint_wheel::wheel_with_edit(w, ed).unwrap_or(w),
            weston,
        ),
        None => (w, weston),
    }
}

/// ⭐ **Publica o que cada junta e cada roldana vale AGORA** (através do eco), esquecido antes o
/// que cada entidade publicou desta família (a junta tirada deixa de ter campos).
pub fn publicar(sim: &mut SimWorld, entidades: &[u64]) {
    let lotes: Vec<(u64, Vec<(FieldKey, f32)>)> = entidades
        .iter()
        .filter_map(|&b| {
            let e = Entity::try_from_bits(b)?;
            let w = sim.world();
            w.get_entity(e).ok()?;
            let mut chaves = Vec::new();
            if w.get::<PhysicsJoint>(e).is_some() {
                chaves.extend(campos());
                chaves.push(k(j::PONTA_NO_MUNDO));
            }
            if w.get::<PulleyWheel>(e).is_some() {
                chaves.extend(campos_roda());
            }
            let v: Vec<_> = chaves
                .into_iter()
                .filter_map(|kk| Some((kk, ler_mundo(w, e, kk)?)))
                .collect();
            Some((b, v))
        })
        .collect();
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for &b in entidades {
        lidos.esquecer(b, JUNTA);
    }
    for (b, valores) in lotes {
        for (kk, v) in valores {
            lidos.publicar(b, kk, v);
        }
    }
}

/// O que a pré-visualização guarda desta entidade, por driver.
fn fotografia(sim: &SimWorld, e: Entity) -> [Option<Driven>; 3] {
    [Driver::JointParams, Driver::PontaNoMundo, Driver::Roldana].map(|d| Driven::read(d, sim, e))
}

/// Escreve um campo na entidade; devolve `true` se mexeu na ROTA de uma corda (o painel re-abre
/// então o comprimento dela, `reseat_wheel_geometry`).
fn escrever_mundo(w: &mut World, e: Entity, kk: FieldKey, v: f32) -> bool {
    if e_da_roda(kk.campo) {
        let Some(&antes) = w.get::<PulleyWheel>(e) else {
            return false;
        };
        let weston = w.get::<WestonAxle>(e).is_some();
        let (roda, novo) = escrever_roda(antes, weston, kk, v);
        if roda != antes {
            *w.get_mut::<PulleyWheel>(e).expect("lida acima") = roda;
        }
        if novo != weston {
            crate::joint_wheel::weston_axle(w, e, novo);
        }
        return novo != weston || roda.route_differs(&antes);
    }
    if kk.campo == j::PONTA_NO_MUNDO {
        crate::joint::anchor_to_world(w, e, v >= 0.5);
        return false;
    }
    if let Some(&pj) = w.get::<PhysicsJoint>(e) {
        let novo = escrever(pj, kk, v);
        if novo != pj {
            *w.get_mut::<PhysicsJoint>(e).expect("lida acima") = novo;
        }
    }
    false
}

/// ⭐ **Aplica às juntas e às roldanas o que a curva mandou neste quadro**, regista o eco e declara
/// a escrita à pré-visualização. Devolve `true` se alguma mudou.
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
    // O TIPO primeiro e as escolhas antes dos números: trocar o tipo, o eixo do motor ou o modo de
    // um eixo muda o que os números significam (o funil re-semeia limites e motor).
    escritas.sort_by_key(|&(e, kk, _)| {
        let numero = kk.info().is_none_or(|i| !i.discreto);
        (e, kk.campo != j::TIPO, numero, kk)
    });
    let mut mudou = false;
    let mut ecos = Vec::with_capacity(escritas.len());
    for lote in escritas.chunk_by(|a, b| a.0 == b.0) {
        let Some(e) = Entity::try_from_bits(lote[0].0) else {
            continue;
        };
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let antes = fotografia(sim, e);
        let mut rota = false;
        for &(b, kk, v) in lote {
            rota |= escrever_mundo(sim.world_mut(), e, kk, v);
            if let Some(ficou) = ler_mundo(sim.world(), e, kk) {
                ecos.push((b, kk, v, ficou));
            }
        }
        // A rota mudou: o comprimento da corda re-abre, como na edição do painel — e a corda é
        // outra entidade, com a sua própria declaração.
        if rota {
            let corda = ph2d_physics_ecs::rope_joint_of(sim.world_mut(), e);
            let corda_antes = corda.and_then(|c| Driven::read(Driver::JointParams, sim, c));
            ph2d_physics_ecs::reseat_wheel_geometry(sim.world_mut(), e);
            if let (Some(c), Some(a)) = (corda, corda_antes)
                && let Some(d) = Driven::read(Driver::JointParams, sim, c)
                && a != d
            {
                drive.driven(c, a, d);
            }
        }
        for (a, d) in antes.into_iter().zip(fotografia(sim, e)) {
            if let (Some(a), Some(d)) = (a, d)
                && a != d
            {
                drive.driven(e, a, d);
                mudou = true;
            }
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
