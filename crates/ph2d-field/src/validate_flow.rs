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
