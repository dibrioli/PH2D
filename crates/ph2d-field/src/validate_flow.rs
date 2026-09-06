//! ⭐ **O QUE CADA FORMA DE FLUXOGRAMA RECUSA** (W122) — o terceiro irmão do
//! [`super::validate_primitive`].
//!
//! ⚠️ **O braço `_` no fim é inalcançável pelo caminho do produto** — o único chamador nomeia as
//! quatro variantes, e o `match` dele continua exaustivo.

use crate::{FieldError, Primitive, round_limit};

/// **O que esta forma do fluxograma recusa.**
pub(super) fn validate_flow(p: &Primitive, idx: u32) -> Result<(), FieldError> {
    let positive = |v: f32, what: &'static str| -> Result<(), FieldError> {
        if !v.is_finite() || v <= 0.0 {
            Err(FieldError::NonPositive { node: idx, what })
        } else {
            Ok(())
        }
    };
    let nao_negativo = |v: f32, what: &'static str| -> Result<(), FieldError> {
        if !v.is_finite() || v < 0.0 {
            Err(FieldError::NonPositive { node: idx, what })
        } else {
            Ok(())
        }
    };
    let maior_ou_igual = |v: f32, piso: f32, what: &'static str| -> Result<(), FieldError> {
        if !v.is_finite() || v < piso {
            Err(FieldError::NonPositive { node: idx, what })
        } else {
            Ok(())
        }
    };
    let um_recuo = |round: f32, limit: f32| -> Result<(), FieldError> {
        if !round.is_finite() || round < 0.0 || round >= limit {
            Err(FieldError::RoundTooLarge {
                node: idx,
                round,
                limit,
            })
        } else {
            Ok(())
        }
    };
    let round_fits = |round: f32, chamfer: f32, limit: f32| -> Result<(), FieldError> {
        um_recuo(chamfer, limit)?;
        um_recuo(round, limit)
    };
    match *p {
        // ⭐ **O `skew` é a única grandeza desta família sem piso**: negativo e zero são formas.
        // A parede dele é do documento — ver [`crate::MAX_PARALLELOGRAM_SKEW`].
        Primitive::Parallelogram {
            half_width,
            half_span,
            skew,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_width, "half_width")?;
            positive(half_span, "half_span")?;
            positive(half_height, "half_height")?;
            if !skew.is_finite() || skew.abs() > half_span * crate::MAX_PARALLELOGRAM_SKEW {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "skew",
                });
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⚠️ **`half_width ≥ half_span` é a cerca**, e ela é uma IGUALDADE aceite: em `w = s` a
        // peça é meio-disco com a face reta, que é uma forma. Ver a tabela em
        // [`crate::DELAY_SPAN_OVER_WIDTH`].
        Primitive::Delay {
            half_width,
            half_span,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_width, "half_width")?;
            positive(half_span, "half_span")?;
            positive(half_height, "half_height")?;
            maior_ou_igual(
                half_width * crate::DELAY_SPAN_OVER_WIDTH,
                half_span,
                "half_width",
            )?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Display {
            half_width,
            half_span,
            point,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_width, "half_width")?;
            positive(half_span, "half_span")?;
            positive(half_height, "half_height")?;
            maior_ou_igual(
                half_width * crate::DELAY_SPAN_OVER_WIDTH,
                half_span,
                "half_width",
            )?;
            nao_negativo(point, "point")?;
            maior_ou_igual(
                crate::dims::display_point_wall(half_width, half_span),
                point,
                "point",
            )?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::OffPage {
            half_width,
            half_span,
            point,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_width, "half_width")?;
            positive(half_span, "half_span")?;
            positive(half_height, "half_height")?;
            nao_negativo(point, "point")?;
            maior_ou_igual(half_span * 2.0 * crate::MAX_OFFPAGE_POINT, point, "point")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        _ => Ok(()),
    }
}

/// **O que a espiral e o documento recusam** (W123).
pub(super) fn validate_curve(p: &Primitive, idx: u32) -> Result<(), FieldError> {
    let positive = |v: f32, what: &'static str| -> Result<(), FieldError> {
        if !v.is_finite() || v <= 0.0 {
            Err(FieldError::NonPositive { node: idx, what })
        } else {
            Ok(())
        }
    };
    let nao_passa = |v: f32, tecto: f32, what: &'static str| -> Result<(), FieldError> {
        if !v.is_finite() || v > tecto {
            Err(FieldError::NonPositive { node: idx, what })
        } else {
            Ok(())
        }
    };
    let um_recuo = |round: f32, limit: f32| -> Result<(), FieldError> {
        if !round.is_finite() || round < 0.0 || round >= limit {
            Err(FieldError::RoundTooLarge {
                node: idx,
                round,
                limit,
            })
        } else {
            Ok(())
        }
    };
    let round_fits = |round: f32, chamfer: f32, limit: f32| -> Result<(), FieldError> {
        um_recuo(chamfer, limit)?;
        um_recuo(round, limit)
    };
    match *p {
        // ⚠️ **`2·thickness ≤ pitch·MAX_SPIRAL_FILL` é a cerca que faz de uma fita uma fita**: com
        // as voltas a encostarem-se a peça é um disco com um risco, e o vale entre elas — que é
        // toda a forma — desaparece.
        Primitive::Spiral {
            radius,
            pitch,
            turns,
            thickness,
            half_height,
            round,
            chamfer,
        } => {
            positive(radius, "radius")?;
            positive(pitch, "pitch")?;
            positive(turns, "turns")?;
            positive(thickness, "thickness")?;
            positive(half_height, "half_height")?;
            nao_passa(turns, crate::MAX_SPIRAL_TURNS, "turns")?;
            nao_passa(2.0 * thickness, pitch * crate::MAX_SPIRAL_FILL, "thickness")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Document {
            half_width,
            half_span,
            wave,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_width, "half_width")?;
            positive(half_span, "half_span")?;
            positive(half_height, "half_height")?;
            if !wave.is_finite() || wave < 0.0 {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "wave",
                });
            }
            nao_passa(wave, half_span * crate::MAX_DOCUMENT_WAVE, "wave")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        _ => Ok(()),
    }
}

/// **O que a mola e o gyroid recusam** (W124).
pub(super) fn validate_lattice(p: &Primitive, idx: u32) -> Result<(), FieldError> {
    let positive = |v: f32, what: &'static str| -> Result<(), FieldError> {
        if !v.is_finite() || v <= 0.0 {
            Err(FieldError::NonPositive { node: idx, what })
        } else {
            Ok(())
        }
    };
    let nao_passa = |v: f32, tecto: f32, what: &'static str| -> Result<(), FieldError> {
        if !v.is_finite() || v > tecto {
            Err(FieldError::NonPositive { node: idx, what })
        } else {
            Ok(())
        }
    };
    let um_recuo = |round: f32, limit: f32| -> Result<(), FieldError> {
        if !round.is_finite() || round < 0.0 || round >= limit {
            Err(FieldError::RoundTooLarge {
                node: idx,
                round,
                limit,
            })
        } else {
            Ok(())
        }
    };
    let round_fits = |round: f32, chamfer: f32, limit: f32| -> Result<(), FieldError> {
        um_recuo(chamfer, limit)?;
        um_recuo(round, limit)
    };
    match *p {
        Primitive::Helix {
            radius,
            pitch,
            turns,
            thickness,
            round,
            chamfer,
        } => {
            positive(radius, "radius")?;
            positive(pitch, "pitch")?;
            positive(turns, "turns")?;
            positive(thickness, "thickness")?;
            nao_passa(turns, crate::MAX_SPIRAL_TURNS, "turns")?;
            nao_passa(2.0 * thickness, pitch * crate::MAX_SPIRAL_FILL, "thickness")?;
            // ⚠️ **O tubo não pode engolir o eixo**: com `thickness ≥ radius` a mola fecha-se sobre
            // ela própria e o divisor do campo perde o chão.
            nao_passa(2.0 * thickness, radius * 1.8, "thickness")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Gyroid {
            half,
            cell,
            thickness,
            round,
            chamfer,
        } => {
            for (i, h) in half.iter().enumerate() {
                positive(*h, ["half_x", "half_y", "half_z"][i])?;
            }
            positive(cell, "cell")?;
            positive(thickness, "thickness")?;
            let menor = half[0].min(half[1]).min(half[2]) * 2.0;
            nao_passa(cell * crate::MIN_GYROID_CELLS, menor, "cell")?;
            nao_passa(2.0 * thickness, cell * crate::MAX_GYROID_FILL, "thickness")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        _ => Ok(()),
    }
}
