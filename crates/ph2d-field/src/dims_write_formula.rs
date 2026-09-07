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
        // ─────────────────────────── W134 ───────────────────────────
        (Primitive::TorusKnot { radius, .. }, 0) => *radius = value,
        // ⚠️ **COAGE, não recusa** — a lei desta casa: a faixa já não oferece nada acima da parede,
        // então um valor de fora só chega por outra porta, e recusar ali rejeitaria a peça inteira.
        (Primitive::TorusKnot { tube, radius, .. }, 1) => *tube = keep_below(value, *radius),
        (
            Primitive::TorusKnot {
                cord,
                radius,
                tube,
                winds,
                loops,
            },
            2,
        ) => {
            *cord = keep_below(
                value,
                crate::knot_cord_ceiling(*radius, *tube, *winds, *loops),
            )
        }
        // ⭐⭐ **A contagem muda e a CORDA pode deixar de caber** — a parede dela depende de `p` e
        // de `q`, então subir uma volta com a corda no tecto fundiria os fios. ⚠️ *A porta repõe a
        // invariante*: é a lei que a W127 pagou (uma escrita que deixa a peça inválida apaga a
        // cena inteira).
        // ⭐⭐ **AS CONTAGENS só se coagem à faixa DELAS.**
        //
        // ⚠️⚠️ **E o re-assentar da corda NÃO se escreve aqui — três linhas que o faziam
        // SOBREVIVERAM a uma mutação.** A parede da corda depende de `p` e de `q`, e subir uma
        // contagem com a corda no tecto deixaria a peça inválida — mas quem repõe isso é a coerção
        // GERAL ([`super::dims_clamp::clamp_dims`]), que a porta corre depois de **toda** escrita e
        // que lê a mesma tabela de faixas. *Duas leis a fazer a mesma coisa divergem no dia em que
        // uma delas é corrigida* — e o gate `raising_a_count_reseats_the_cord` mede a propriedade,
        // que é o que interessa, e não qual das duas a produziu.
        (Primitive::TorusKnot { winds: n, .. }, 3) => {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                *n = (value.round().max(0.0) as u32)
                    .clamp(crate::MIN_KNOT_WINDS, crate::MAX_KNOT_WINDS);
            }
        }
        (
            Primitive::TorusKnot {
                loops: n,
                winds: outro,
                ..
            },
            4,
        ) => {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                *n = (value.round().max(0.0) as u32)
                    .clamp(crate::MIN_KNOT_LOOPS, crate::max_knot_loops(*outro));
            }
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
