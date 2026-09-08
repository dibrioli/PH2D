//! ⭐⭐ **AS LINHAS DAS DUAS CURVAS COM ESPESSURA** (W136) — a Bezier e a onda em anel.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::dims_table_flow`] estava a `665` das `700` linhas do gate de LOC da workspace, e as
//! duas formas pedem `~65`. ⚠️ **Partir para irmão, nunca uma entrada na allowlist** — e o corte é
//! por responsabilidade: estas duas são as únicas cujo contorno é uma **curva aberta com
//! espessura**, e não um contorno fechado.
//!
//! ⚠️ **A ORDEM É A IDENTIDADE DA LINHA** — o painel manda o índice, não o nome.

use super::dims_table_flow::{chamfer_dim, round_dim};
use super::{Dim, Span};
use crate::Primitive;

/// As linhas das duas curvas.
///
/// ⚠️ **Inalcançável com outra primitiva** — quem chega aqui já foi nomeado pelo braço `p @ (…)` do
/// [`super::dims_table_plates::dims_plate`], que continua exaustivo.
#[must_use]
pub(crate) fn dims_stroke(p: &Primitive) -> Vec<Dim> {
    match p {
        // ⭐⭐ **Os SEIS primeiros índices são os três pontos**, e é isso que dá as alças do canvas
        // — ver [`crate::vertex_rows`], que lê `first_row: 0`.
        Primitive::Bezier {
            a,
            b,
            c,
            thickness,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.ax",
                value: a[0],
                span: Span::Free,
            },
            Dim {
                key: "field.dim.ay",
                value: a[1],
                span: Span::Free,
            },
            Dim {
                key: "field.dim.bx",
                value: b[0],
                span: Span::Free,
            },
            Dim {
                key: "field.dim.by",
                value: b[1],
                span: Span::Free,
            },
            Dim {
                key: "field.dim.cx",
                value: c[0],
                span: Span::Free,
            },
            Dim {
                key: "field.dim.cy",
                value: c[1],
                span: Span::Free,
            },
            Dim {
                key: "field.dim.thickness",
                value: *thickness,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(p, *chamfer),
            round_dim(p, *round),
        ],
        Primitive::CircleWave {
            radius,
            amplitude,
            lobes,
            thickness,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            // ⚠️ **A amplitude tem PAREDE no raio**: em `amplitude = radius` o vale da onda toca o
            // eixo e o divisor do campo perde o chão.
            Dim {
                key: "field.dim.wave",
                value: *amplitude,
                span: Span::Wall(*radius),
            },
            // ⭐⭐ **CONTAGEM, e não número** — ver [`crate::curve`]: só um inteiro fecha a volta.
            Dim {
                key: "field.dim.lobes",
                value: *lobes as f32,
                span: Span::Count {
                    min: crate::MIN_WAVE_LOBES,
                    max: crate::MAX_WAVE_LOBES,
                },
            },
            Dim {
                key: "field.dim.thickness",
                value: *thickness,
                span: Span::Wall(crate::wave_thickness_ceiling(*radius, *amplitude)),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(p, *chamfer),
            round_dim(p, *round),
        ],
        // Inalcançável — ver o doc acima.
        _ => Vec::new(),
    }
}
