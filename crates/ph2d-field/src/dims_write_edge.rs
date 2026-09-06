//! ⭐⭐ **OS DOIS RECUOS DE UMA ARESTA, do lado da ESCRITA** (W122) — o chanfro, o filete, e o que
//! acontece a ambos quando a peça encolhe.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::dims_write`] responde *«escreve esta DIMENSÃO»*; este responde *«escreve este
//! RECUO»*, que é a mesma pergunta feita a uma aresta e não a uma medida. O irmão passou as `700`
//! linhas do gate de LOC da workspace ao receber as quatro formas do fluxograma.
//! ⛔ *Split, nunca allowlist.*

use crate::{FieldError, Primitive, round_limit};

/// ⭐⭐⭐ **Escreve o CHANFRO de uma forma** (Enio, 2026-08-30).
///
/// ⚠️ **A parede é a MESMA do filete** ([`round_limit`]), e não uma escolhida: os dois recuam a
/// superfície a partir da mesma quina.
///
/// ⚠️ **EXAUSTIVO, pela razão que a [`set_round`] já pagou na W101**: uma primitiva nova com aresta
/// que caísse num braço vazio teria o slider a mexer-se sem escrever nada — *a falha mais cara de
/// diagnosticar, porque não deixa rasto*.
pub(super) fn set_chamfer(p: &mut Primitive, node: u32, value: f32) -> Result<(), FieldError> {
    let limit = round_limit(p).ok_or(FieldError::NonPositive {
        node,
        what: "chamfer",
    })?;
    if value >= limit {
        return Err(FieldError::RoundTooLarge {
            node,
            round: value,
            limit,
        });
    }
    match p {
        Primitive::Box { chamfer, .. }
        | Primitive::Helix { chamfer, .. }
        | Primitive::Gyroid { chamfer, .. }
        | Primitive::Spiral { chamfer, .. }
        | Primitive::Document { chamfer, .. }
        | Primitive::Parallelogram { chamfer, .. }
        | Primitive::Delay { chamfer, .. }
        | Primitive::Display { chamfer, .. }
        | Primitive::OffPage { chamfer, .. }
        | Primitive::Cylinder { chamfer, .. }
        | Primitive::Extrude { chamfer, .. }
        | Primitive::Cone { chamfer, .. }
        | Primitive::Prism { chamfer, .. }
        | Primitive::Wedge { chamfer, .. }
        | Primitive::Star { chamfer, .. }
        | Primitive::BoxFrame { chamfer, .. }
        | Primitive::Octahedron { chamfer, .. }
        | Primitive::CutSphere { chamfer, .. }
        | Primitive::HollowDome { chamfer, .. }
        | Primitive::SolidAngle { chamfer, .. }
        | Primitive::Gear { chamfer, .. }
        | Primitive::Cross { chamfer, .. }
        | Primitive::Heart { chamfer, .. }
        | Primitive::Moon { chamfer, .. }
        | Primitive::Drop { chamfer, .. }
        | Primitive::Pie { chamfer, .. }
        | Primitive::Trapezoid { chamfer, .. }
        | Primitive::Vesica { chamfer, .. }
        | Primitive::Arrow { chamfer, .. }
        | Primitive::Chevron { chamfer, .. }
        | Primitive::BentArrow { chamfer, .. }
        | Primitive::Rhombus { chamfer, .. }
        | Primitive::Tube { chamfer, .. }
        | Primitive::CircleSegment { chamfer, .. }
        | Primitive::SpeechRect { chamfer, .. }
        | Primitive::SpeechOval { chamfer, .. }
        | Primitive::Cloud { chamfer, .. }
        | Primitive::Bolt { chamfer, .. }
        | Primitive::Shield { chamfer, .. }
        | Primitive::Tag { chamfer, .. }
        | Primitive::Check { chamfer, .. }
        | Primitive::Banner { chamfer, .. }
        | Primitive::Brace { chamfer, .. }
        | Primitive::TorusArc { chamfer, .. } => *chamfer = value,
        Primitive::Sphere { .. }
        | Primitive::RoundedCylinder { .. }
        | Primitive::Superquadric { .. }
        | Primitive::Superformula { .. }
        | Primitive::Torus { .. }
        | Primitive::Revolve { .. }
        | Primitive::Capsule { .. }
        | Primitive::RoundCone { .. }
        | Primitive::Link { .. }
        | Primitive::Ellipsoid { .. } => {
            return Err(FieldError::NonPositive {
                node,
                what: "chamfer",
            });
        }
    }
    Ok(())
}

pub(super) fn set_round(p: &mut Primitive, node: u32, value: f32) -> Result<(), FieldError> {
    let limit = round_limit(p).ok_or(FieldError::NonPositive {
        node,
        what: "round",
    })?;
    if value >= limit {
        return Err(FieldError::RoundTooLarge {
            node,
            round: value,
            limit,
        });
    }
    // ⚠️ **EXAUSTIVO, e o `_ => {}` que estava aqui era uma armadilha** (W101): uma primitiva nova
    // COM filete caía no braço vazio, o `round_limit` dela respondia, o `set_round` dizia `Ok` — e
    // o número **nunca era escrito**. Um slider que se mexe e não faz nada é a falha mais cara de
    // diagnosticar, porque não deixa rasto. Com a lista fechada, a próxima é erro de compilação.
    match p {
        Primitive::Box { round, .. }
        | Primitive::Helix { round, .. }
        | Primitive::Gyroid { round, .. }
        | Primitive::Spiral { round, .. }
        | Primitive::Document { round, .. }
        | Primitive::Parallelogram { round, .. }
        | Primitive::Delay { round, .. }
        | Primitive::Display { round, .. }
        | Primitive::OffPage { round, .. }
        | Primitive::Cylinder { round, .. }
        | Primitive::Extrude { round, .. }
        | Primitive::Cone { round, .. }
        | Primitive::Prism { round, .. }
        | Primitive::Wedge { round, .. }
        | Primitive::Star { round, .. }
        | Primitive::BoxFrame { round, .. }
        | Primitive::Octahedron { round, .. }
        | Primitive::CutSphere { round, .. }
        | Primitive::HollowDome { round, .. }
        | Primitive::SolidAngle { round, .. }
        | Primitive::Gear { round, .. }
        | Primitive::Cross { round, .. }
        | Primitive::Heart { round, .. }
        | Primitive::Moon { round, .. }
        | Primitive::Drop { round, .. }
        | Primitive::Pie { round, .. }
        | Primitive::Trapezoid { round, .. }
        | Primitive::Vesica { round, .. }
        | Primitive::Arrow { round, .. }
        | Primitive::Chevron { round, .. }
        | Primitive::BentArrow { round, .. }
        | Primitive::Rhombus { round, .. }
        | Primitive::Tube { round, .. }
        | Primitive::CircleSegment { round, .. }
        | Primitive::SpeechRect { round, .. }
        | Primitive::SpeechOval { round, .. }
        | Primitive::Cloud { round, .. }
        | Primitive::Bolt { round, .. }
        | Primitive::Shield { round, .. }
        | Primitive::Tag { round, .. }
        | Primitive::Check { round, .. }
        | Primitive::Banner { round, .. }
        | Primitive::Brace { round, .. }
        | Primitive::TorusArc { round, .. } => *round = value,
        Primitive::Sphere { .. }
        | Primitive::RoundedCylinder { .. }
        | Primitive::Superquadric { .. }
        | Primitive::Superformula { .. }
        | Primitive::Torus { .. }
        | Primitive::Revolve { .. }
        | Primitive::Capsule { .. }
        | Primitive::RoundCone { .. }
        | Primitive::Link { .. }
        | Primitive::Ellipsoid { .. } => {}
    }
    Ok(())
}

/// ⭐ **Limita o filete ao que a forma agora comporta** — ver a nota de [`set_dim`].
///
/// Devolve `true` se teve de o mexer, para quem chamar poder dizê-lo.
pub fn clamp_round(p: &mut Primitive) -> bool {
    let Some(limit) = round_limit(p) else {
        return false;
    };
    // ⚠️ **Estritamente abaixo**, e não «até»: a validação recusa `round >= limit`, e um filete
    // exatamente no limite encolheria a fonte a zero. A margem é uma fração do próprio limite, e
    // não um épsilon absoluto — numa peça de 0,01 um épsilon fixo seria o limite inteiro.
    let ceiling = limit * (1.0 - ROUND_MARGIN);
    // ⚠️ Exaustivo pela razão do [`set_round`] — um braço `_` deixaria a primitiva nova com um
    // filete que a peça já não comporta, e a validação recusaria o documento inteiro no gesto
    // seguinte.
    match p {
        Primitive::Box { round, chamfer, .. }
        | Primitive::Helix { round, chamfer, .. }
        | Primitive::Gyroid { round, chamfer, .. }
        | Primitive::Spiral { round, chamfer, .. }
        | Primitive::Document { round, chamfer, .. }
        | Primitive::Parallelogram { round, chamfer, .. }
        | Primitive::Delay { round, chamfer, .. }
        | Primitive::Display { round, chamfer, .. }
        | Primitive::OffPage { round, chamfer, .. }
        | Primitive::Cylinder { round, chamfer, .. }
        | Primitive::Extrude { round, chamfer, .. }
        | Primitive::Cone { round, chamfer, .. }
        | Primitive::Prism { round, chamfer, .. }
        | Primitive::Wedge { round, chamfer, .. }
        | Primitive::Star { round, chamfer, .. }
        | Primitive::BoxFrame { round, chamfer, .. }
        | Primitive::Octahedron { round, chamfer, .. }
        | Primitive::CutSphere { round, chamfer, .. }
        | Primitive::HollowDome { round, chamfer, .. }
        | Primitive::SolidAngle { round, chamfer, .. }
        | Primitive::Gear { round, chamfer, .. }
        | Primitive::Cross { round, chamfer, .. }
        | Primitive::Heart { round, chamfer, .. }
        | Primitive::Moon { round, chamfer, .. }
        | Primitive::Drop { round, chamfer, .. }
        | Primitive::Pie { round, chamfer, .. }
        | Primitive::Trapezoid { round, chamfer, .. }
        | Primitive::Vesica { round, chamfer, .. }
        | Primitive::Arrow { round, chamfer, .. }
        | Primitive::Chevron { round, chamfer, .. }
        | Primitive::BentArrow { round, chamfer, .. }
        | Primitive::Rhombus { round, chamfer, .. }
        | Primitive::Tube { round, chamfer, .. }
        | Primitive::CircleSegment { round, chamfer, .. }
        | Primitive::SpeechRect { round, chamfer, .. }
        | Primitive::SpeechOval { round, chamfer, .. }
        | Primitive::Cloud { round, chamfer, .. }
        | Primitive::Bolt { round, chamfer, .. }
        | Primitive::Shield { round, chamfer, .. }
        | Primitive::Tag { round, chamfer, .. }
        | Primitive::Check { round, chamfer, .. }
        | Primitive::Banner { round, chamfer, .. }
        | Primitive::Brace { round, chamfer, .. }
        | Primitive::TorusArc { round, chamfer, .. } => {
            // ⭐⭐ **OS DOIS RECUOS são limitados, e não só o filete** (Enio, 2026-08-30).
            //
            // ⛔ Sem a segunda metade, encolher uma peça chanfrada deixava o chanfro acima da parede
            // e a `validate_primitive` passava a **recusar o documento inteiro** no gesto seguinte —
            // exactamente a armadilha que o doc do `set_round` nomeia para um braço `_`.
            let mut mexeu = false;
            for v in [&mut *round, &mut *chamfer] {
                if *v > ceiling {
                    *v = ceiling.max(0.0);
                    mexeu = true;
                }
            }
            mexeu
        }
        Primitive::Sphere { .. }
        | Primitive::RoundedCylinder { .. }
        | Primitive::Superquadric { .. }
        | Primitive::Superformula { .. }
        | Primitive::Torus { .. }
        | Primitive::Revolve { .. }
        | Primitive::Capsule { .. }
        | Primitive::RoundCone { .. }
        | Primitive::Link { .. }
        | Primitive::Ellipsoid { .. } => false,
    }
}

/// A folga entre o filete máximo e a parede, em fração da parede. Ver [`clamp_round`].
pub(super) const ROUND_MARGIN: f32 = 1.0e-3;
