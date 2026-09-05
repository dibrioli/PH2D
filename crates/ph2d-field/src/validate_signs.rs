//! ⭐ **O QUE CADA SINAL RECUSA** — a metade do [`super::validate_primitive`] que responde pelas
//! setas (W119), pelos balões e pelos símbolos (W120).
//!
//! # Por que um arquivo irmão
//!
//! O irmão responde pelos sólidos e pelas chapas; este pela família dos sinais. A W120 acrescentou
//! nove primitivas e o arquivo passou as `700` linhas do gate de LOC.
//! ⚠️ **Partir para irmão, nunca uma entrada na allowlist.**
//!
//! ⚠️ **O braço `_` no fim é inalcançável pelo caminho do produto** — o único chamador nomeia as
//! quinze variantes, e o `match` dele continua exaustivo. `Ok(())` e não um `panic`: uma forma nova
//! esquecida aqui tem de aparecer como uma peça que o documento aceita e o censo da marcha reprova,
//! e não como o app a cair.

use crate::{FieldError, Primitive, round_limit};

/// **O que este sinal recusa.** ⚠️ Só é chamada com as variantes que o irmão nomeia.
#[allow(clippy::too_many_lines)]
pub(super) fn validate_sign(p: &Primitive, idx: u32) -> Result<(), FieldError> {
    let positive = |v: f32, what: &'static str| -> Result<(), FieldError> {
        if !v.is_finite() || v <= 0.0 {
            Err(FieldError::NonPositive { node: idx, what })
        } else {
            Ok(())
        }
    };
    // ⭐ **Estritamente MAIOR que outro campo desta forma** — a cerca que faz de uma seta uma seta
    // (a ponta passa a haste) e de um tubo um tubo (o bordo passa o furo).
    let maior = |v: f32, piso: f32, what: &'static str| -> Result<(), FieldError> {
        if !v.is_finite() || v <= piso {
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
        // ─────────────────────────── W119 ───────────────────────────
        // ⚠️ **`head > shaft` é a cerca que faz de uma seta uma seta**: iguais, não há farpa, e a
        // lei da sobreposição ([`ph2d_field_eval::ops_arrows`]) fica sem o `x` onde a ponta tem a
        // largura da haste.
        Primitive::Arrow {
            heads,
            half_length,
            shaft,
            head,
            head_length,
            half_height,
            round,
            chamfer,
        } => {
            if !(crate::MIN_ARROW_HEADS..=crate::MAX_ARROW_HEADS).contains(&heads) {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "heads",
                });
            }
            positive(half_length, "half_length")?;
            positive(shaft, "shaft")?;
            positive(half_height, "half_height")?;
            maior(head, shaft, "head")?;
            positive(head_length, "head_length")?;
            maior(half_length * 2.0, head_length, "head_length")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Chevron {
            half_length,
            half_span,
            thickness,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_length, "half_length")?;
            positive(thickness, "thickness")?;
            positive(half_height, "half_height")?;
            // A banda mais grossa do que a meia-envergadura fecharia o vazio que a define.
            maior(half_span, thickness, "half_span")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::BentArrow {
            run,
            rise,
            shaft,
            head,
            head_length,
            half_height,
            round,
            chamfer,
        } => {
            positive(run, "run")?;
            positive(rise, "rise")?;
            positive(shaft, "shaft")?;
            positive(half_height, "half_height")?;
            maior(head, shaft, "head")?;
            positive(head_length, "head_length")?;
            maior(rise * 2.0, head_length, "head_length")?;
            // O cotovelo tem de caber nos dois braços, senão a haste sai da caixa da peça.
            maior(run, shaft, "run")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Rhombus {
            half_width,
            half_span,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_width, "half_width")?;
            positive(half_span, "half_span")?;
            positive(half_height, "half_height")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⚠️ **O furo é OBRIGATÓRIO**, e a cerca é o que impede a segunda fórmula: sem ele isto
        // seria a [`Primitive::Pie`], e duas primitivas para a mesma superfície é o defeito que o
        // [`Primitive::Cone`] evita desde a W101.
        Primitive::Tube {
            outer,
            inner,
            angle,
            half_height,
            round,
            chamfer,
        } => {
            positive(inner, "inner")?;
            positive(half_height, "half_height")?;
            maior(outer, inner, "outer")?;
            if !angle.is_finite() || angle <= 0.0 || angle > std::f32::consts::PI {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "angle",
                });
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⚠️ **A corda tem de cortar o disco**: em `|cut| ≥ radius` ela passa ao lado, e o que sobra
        // é o disco inteiro ou nada.
        Primitive::CircleSegment {
            radius,
            cut,
            half_height,
            round,
            chamfer,
        } => {
            positive(radius, "radius")?;
            positive(half_height, "half_height")?;
            if !cut.is_finite() || cut.abs() >= radius {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "cut",
                });
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ─────────────────────────── W120 ───────────────────────────
        Primitive::SpeechRect {
            half_width,
            half_span,
            tail,
            half_height,
            round,
            chamfer,
        }
        | Primitive::SpeechOval {
            half_width,
            half_span,
            tail,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_width, "half_width")?;
            positive(half_span, "half_span")?;
            positive(tail, "tail")?;
            positive(half_height, "half_height")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⚠️ **A cauda pode ser ZERO, e é ela que dá a porta *Cloud*** — ver o doc da variante.
        Primitive::Cloud {
            lobes,
            half_width,
            half_span,
            tail,
            half_height,
            round,
            chamfer,
        } => {
            if !(crate::MIN_CLOUD_LOBES..=crate::MAX_CLOUD_LOBES).contains(&lobes) {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "lobes",
                });
            }
            positive(half_width, "half_width")?;
            positive(half_span, "half_span")?;
            positive(half_height, "half_height")?;
            if !tail.is_finite() || tail < 0.0 {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "tail",
                });
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Bolt {
            half_width,
            half_span,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_width, "half_width")?;
            positive(half_span, "half_span")?;
            positive(half_height, "half_height")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⚠️ **`2·half_span > half_width` é uma cerca de GEOMETRIA** — abaixo dela o centro do arco
        // cai do outro lado e os lados do escudo curvam para dentro.
        Primitive::Shield {
            half_width,
            half_span,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_width, "half_width")?;
            positive(half_height, "half_height")?;
            maior(2.0 * half_span, half_width, "half_span")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Tag {
            half_width,
            half_span,
            point,
            hole,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_span, "half_span")?;
            positive(half_height, "half_height")?;
            positive(hole, "hole")?;
            maior(2.0 * half_width, point, "half_width")?;
            positive(point, "point")?;
            // O furo mora a `0,7 × half_width` do centro, e tem de caber entre o bordo e a peça.
            maior(half_width * 0.3, hole, "hole")?;
            maior(half_span, hole, "hole")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Check {
            half_width,
            half_span,
            thickness,
            half_height,
            round,
            chamfer,
        } => {
            positive(thickness, "thickness")?;
            positive(half_height, "half_height")?;
            maior(half_width, thickness, "half_width")?;
            maior(half_span, thickness, "half_span")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Banner {
            half_width,
            half_span,
            notch,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_span, "half_span")?;
            positive(half_height, "half_height")?;
            positive(notch, "notch")?;
            maior(half_width, notch, "half_width")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Brace {
            half_span,
            thickness,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_span, "half_span")?;
            positive(half_height, "half_height")?;
            positive(thickness, "thickness")?;
            maior(half_span * 0.5, thickness, "half_span")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⚠️ **Inalcançável pelo caminho do produto** — ver o cabeçalho.
        _ => Ok(()),
    }
}
