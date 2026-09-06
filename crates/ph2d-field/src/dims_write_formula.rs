//! ⭐ **AS FORMAS POR FÓRMULA escrevem aqui** (W125–W128) — o cilindro com bojo, a superquadrática
//! e a superfórmula.
//!
//! # Por que elas saíram do irmão
//!
//! O [`super::dims_write`] responde por *o que acontece quando alguém escreve*; ele chegou aos
//! **700** do gate de LOC quando a superfórmula trouxe **onze** linhas de painel. ⚠️ **A cura é
//! partir para irmão, nunca uma entrada na allowlist** — e o corte é por responsabilidade: estas
//! três são as que não têm filete e cujos números são **adimensionais**.
//!
//! ⚠️ **A ordem dos índices É a identidade da linha** (o painel manda o ÍNDICE), e a tabela que a
//! fixa é a [`super::dims_table_flow::dims_exact`].

use super::dims_write::keep_below;
use crate::{FieldError, Primitive};

/// As arms de escrita das três — ver [`super::set_dim`], que é a porta.
///
/// # Errors
/// [`FieldError::NonPositive`] para um índice que não é desta forma.
pub(super) fn write_formula(
    p: &mut Primitive,
    node: u32,
    index: usize,
    value: f32,
) -> Result<(), FieldError> {
    let half = value * 0.5;
    match (p, index) {
        // ─────────────────────────── W125 ───────────────────────────
        (Primitive::RoundedCylinder { radius, .. }, 0) => *radius = value,
        (
            Primitive::RoundedCylinder {
                bulge,
                radius,
                half_height,
            },
            1,
        ) => *bulge = keep_below(value, radius.min(*half_height)),
        (Primitive::RoundedCylinder { half_height, .. }, 2) => *half_height = half,
        // ─────────────────────────── W127 ───────────────────────────
        (Primitive::Superquadric { half: h, .. }, i @ 0..=2) => h[i] = half,
        // ⚠️ **COAGE, não recusa** — a lei do `Unary::Taper` e do prisma: a faixa já não oferece
        // nada fora de `[MIN, MAX]`, então um valor de fora só chega por outra porta, e recusar ali
        // rejeitaria a peça inteira.
        (Primitive::Superquadric { exponent_top, .. }, 3) => {
            *exponent_top = value.clamp(
                crate::MIN_SUPERQUADRIC_EXPONENT,
                crate::MAX_SUPERQUADRIC_EXPONENT,
            );
        }
        // ─────────────────────────── W128 ───────────────────────────
        (Primitive::Superformula { half: h, .. }, i @ 0..=2) => h[i] = half,
        (
            Primitive::Superformula {
                top_symmetry: m, ..
            },
            3,
        )
        | (
            Primitive::Superformula {
                side_symmetry: m, ..
            },
            7,
        ) => {
            // ⚠️ **INTEIRA, e coagida** — ver `MIN_SUPERFORMULA_SYMMETRY`: um `m` fraccionário racha
            // a peça na costura do `atan2`.
            #[allow(clippy::cast_precision_loss)]
            {
                *m = value.round().clamp(
                    crate::MIN_SUPERFORMULA_SYMMETRY as f32,
                    crate::MAX_SUPERFORMULA_SYMMETRY as f32,
                );
            }
        }
        (Primitive::Superformula { top_n1: n, .. }, 4)
        | (Primitive::Superformula { side_n1: n, .. }, 8) => {
            *n = value.clamp(crate::MIN_SUPERFORMULA_N1, crate::MAX_SUPERFORMULA_N1);
        }
        (Primitive::Superformula { top_n2: n, .. }, 5)
        | (Primitive::Superformula { top_n3: n, .. }, 6)
        | (Primitive::Superformula { side_n2: n, .. }, 9)
        | (Primitive::Superformula { side_n3: n, .. }, 10) => {
            *n = value.clamp(crate::MIN_SUPERFORMULA_N, crate::MAX_SUPERFORMULA_N);
        }
        (Primitive::Superquadric { exponent_side, .. }, 4) => {
            *exponent_side = value.clamp(
                crate::MIN_SUPERQUADRIC_EXPONENT,
                crate::MAX_SUPERQUADRIC_EXPONENT,
            );
        }
        _ => {
            return Err(FieldError::NonPositive { node, what: "dim" });
        }
    }
    Ok(())
}
