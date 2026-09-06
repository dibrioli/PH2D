//! ⭐ **ESCALAR um SINAL, uma forma de fluxograma, uma curva ou uma rede** (W124) — a metade do
//! [`super::dims_scale`] que responde pelas famílias novas.
//!
//! # Por que um arquivo irmão
//!
//! O irmão passou as `700` linhas do gate de LOC da workspace ao receber a mola e o gyroid, e o
//! corte é por assunto: ali estão os sólidos e as chapas de geometria, aqui as formas que as waves
//! W119–W124 acrescentaram. ⛔ *Split, nunca allowlist.*
//!
//! ⚠️ **O braço `_` no fim é inalcançável pelo caminho do produto** — o único chamador nomeia cada
//! variante, e o `match` dele continua exaustivo.

use crate::Primitive;

/// **Escala um sinal, uma forma de fluxograma, uma curva ou uma rede.**
pub(super) fn scale_sign(p: &mut Primitive, factor: f32) {
    match p {
        // ─────────────────────────── W119 ───────────────────────────
        // ⚠️ **A contagem de pontas NÃO escala** (o `heads`), e o **ângulo** do sector também não:
        // os dois são adimensionais, e multiplicá-los mudaria a FORMA em vez do tamanho.
        Primitive::Arrow {
            heads: _,
            half_length,
            shaft,
            head,
            head_length,
            half_height,
            round,
            chamfer,
        } => {
            for v in [
                half_length,
                shaft,
                head,
                head_length,
                half_height,
                round,
                chamfer,
            ] {
                *v *= factor;
            }
        }
        Primitive::Chevron {
            half_length,
            half_span,
            thickness,
            half_height,
            round,
            chamfer,
        } => {
            for v in [
                half_length,
                half_span,
                thickness,
                half_height,
                round,
                chamfer,
            ] {
                *v *= factor;
            }
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
            for v in [
                run,
                rise,
                shaft,
                head,
                head_length,
                half_height,
                round,
                chamfer,
            ] {
                *v *= factor;
            }
        }
        Primitive::Rhombus {
            half_width,
            half_span,
            half_height,
            round,
            chamfer,
        } => {
            for v in [half_width, half_span, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        Primitive::Tube {
            outer,
            inner,
            angle: _,
            half_height,
            round,
            chamfer,
        } => {
            for v in [outer, inner, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        Primitive::CircleSegment {
            radius,
            cut,
            half_height,
            round,
            chamfer,
        } => {
            for v in [radius, cut, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        // ─────────────────────────── W120 ───────────────────────────
        // ⚠️ **A contagem de bossas NÃO escala** — ela é adimensional, e multiplicá-la mudaria a
        // FORMA em vez do tamanho.
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
            for v in [half_width, half_span, tail, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        Primitive::Cloud {
            lobes: _,
            half_width,
            half_span,
            tail,
            half_height,
            round,
            chamfer,
        } => {
            for v in [half_width, half_span, tail, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        Primitive::Bolt {
            half_width,
            half_span,
            half_height,
            round,
            chamfer,
        }
        | Primitive::Shield {
            half_width,
            half_span,
            half_height,
            round,
            chamfer,
        } => {
            for v in [half_width, half_span, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        Primitive::Parallelogram {
            half_width,
            half_span,
            skew,
            half_height,
            round,
            chamfer,
        } => {
            // ⚠️ **O `skew` é um COMPRIMENTO** (o escorregamento da base de cima), logo escala com
            // a peça — e escalar as duas juntas deixa a razão `skew/half_span` intacta, que é o que
            // a cerca dela mede.
            for v in [half_width, half_span, skew, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        Primitive::Delay {
            half_width,
            half_span,
            half_height,
            round,
            chamfer,
        } => {
            for v in [half_width, half_span, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        Primitive::Display {
            half_width,
            half_span,
            point,
            half_height,
            round,
            chamfer,
        }
        | Primitive::OffPage {
            half_width,
            half_span,
            point,
            half_height,
            round,
            chamfer,
        } => {
            for v in [half_width, half_span, point, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        // ⚠️ **O `turns` NÃO escala** — é uma contagem, não um comprimento; escalá-lo daria uma
        // espiral com mais voltas ao ampliar a peça.
        Primitive::Spiral {
            radius,
            pitch,
            turns: _,
            thickness,
            half_height,
            round,
            chamfer,
        } => {
            for v in [radius, pitch, thickness, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        Primitive::Document {
            half_width,
            half_span,
            wave,
            half_height,
            round,
            chamfer,
        } => {
            for v in [half_width, half_span, wave, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        // ⚠️ **Nem `turns` nem a CÉLULA escalam como comprimento?** — o `turns` é uma contagem e
        // fica; a `cell` **é** um comprimento e escala, senão ampliar a peça daria mais células.
        Primitive::Helix {
            radius,
            pitch,
            turns: _,
            thickness,
            round,
            chamfer,
        } => {
            for v in [radius, pitch, thickness, round, chamfer] {
                *v *= factor;
            }
        }
        Primitive::Gyroid {
            half,
            cell,
            thickness,
            round,
            chamfer,
        } => {
            for v in half.iter_mut() {
                *v *= factor;
            }
            for v in [cell, thickness, round, chamfer] {
                *v *= factor;
            }
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
            for v in [
                half_width,
                half_span,
                point,
                hole,
                half_height,
                round,
                chamfer,
            ] {
                *v *= factor;
            }
        }
        Primitive::Check {
            half_width,
            half_span,
            thickness,
            half_height,
            round,
            chamfer,
        } => {
            for v in [
                half_width,
                half_span,
                thickness,
                half_height,
                round,
                chamfer,
            ] {
                *v *= factor;
            }
        }
        Primitive::Banner {
            half_width,
            half_span,
            notch,
            half_height,
            round,
            chamfer,
        } => {
            for v in [half_width, half_span, notch, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        Primitive::Brace {
            half_span,
            thickness,
            half_height,
            round,
            chamfer,
        } => {
            for v in [half_span, thickness, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        _ => {}
    }
}
