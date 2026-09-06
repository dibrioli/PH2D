//! ⭐ **O QUE CADA CHAPA MEDE** — a metade do [`super::dims_table`] que responde pela família das
//! chapas (um contorno 2D de fórmula puxado em Z).
//!
//! # Por que um arquivo irmão
//!
//! O irmão responde *o que cada SÓLIDO mede*; este responde o mesmo para as chapas. A W119
//! acrescentou seis primitivas e o arquivo passou as `700` linhas do gate de LOC da workspace.
//! ⚠️ **Partir para irmão, nunca uma entrada na allowlist.**
//!
//! # ⚠️⚠️ O braço `_` no fim, e por que ele NÃO abre um buraco
//!
//! Quem chama esta função é **um** sítio: o braço `p @ (…)` do [`super::dims_table::dims`], que
//! nomeia as catorze chapas uma a uma. Esse `match` continua **exaustivo**, então uma primitiva
//! nova é erro de compilação **lá** — a corrente que o [`crate::PrimitiveKind`] fecha não se perde.
//!
//! ⛔ O que o `_` não apanha é uma primitiva nova posta na lista do braço `p @ (…)` e **esquecida
//! aqui**: o painel dela nasceria **vazio**, em silêncio. ⇒ é por isso que existe o gate
//! `every_primitive_offers_at_least_one_dimension`, que corre a lista derivada de
//! `PrimitiveKind::ALL` e recusa uma forma sem número nenhum. *Um braço `_` sem um censo ao lado é
//! uma licença.*

use super::{Dim, Span};
use crate::{Primitive, round_limit};

/// O chanfro de uma chapa — a mesma parede do filete, ver [`super::dims_table::dims`].
fn chamfer_dim(p: &Primitive, value: f32) -> Dim {
    Dim {
        key: "field.dim.chamfer",
        value,
        span: round_limit(p).map_or(Span::FromZero, Span::WallFromZero),
    }
}

/// O filete de uma chapa.
fn round_dim(p: &Primitive, value: f32) -> Dim {
    Dim {
        key: "field.dim.round",
        value,
        span: round_limit(p).map_or(Span::FromZero, Span::WallFromZero),
    }
}

/// **O que esta chapa mede.** ⚠️ Só é chamada com as variantes que o braço `p @ (…)` do irmão
/// nomeia — ver o cabeçalho deste módulo.
pub(super) fn dims_plate(p: &Primitive) -> Vec<Dim> {
    let chamfer_dim = |v: f32| chamfer_dim(p, v);
    let round_dim = |v: f32| round_dim(p, v);
    match p {
        Primitive::Gear {
            teeth,
            root,
            outer,
            tooth,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.teeth",
                value: *teeth as f32,
                span: Span::Wall(crate::MAX_GEAR_TEETH as f32),
            },
            Dim {
                key: "field.dim.radius",
                value: *root,
                span: Span::Wall(*outer),
            },
            Dim {
                key: "field.dim.radius_outer",
                value: *outer,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.tooth",
                value: *tooth,
                span: Span::Wall(1.0),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Cross {
            arm,
            width,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.length",
                value: arm * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.width",
                value: width * 2.0,
                span: Span::Wall(*arm * 2.0),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Heart {
            size,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.size",
                value: *size,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Moon {
            radius,
            bite,
            offset,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.bite",
                value: *bite,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.offset",
                value: *offset,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Drop {
            radius,
            height,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.length",
                value: *height,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Pie {
            radius,
            angle,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.angle",
                value: *angle,
                span: Span::Wall(std::f32::consts::PI),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Trapezoid {
            bottom,
            top,
            half_width,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: bottom * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.radius_top",
                value: top * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.length",
                value: half_width * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Vesica {
            radius,
            offset,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.offset",
                value: *offset,
                span: Span::Wall(*radius),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        // ⭐ **Os SINAIS delegam ao irmão** — ver [`super::dims_table_signs`]. A corrente não se
        // perde: quem chega aqui já foi nomeado pelo braço `p @ (…)` do [`super::dims_table::dims`],
        // que continua exaustivo.
        p @ (Primitive::Arrow { .. }
        | Primitive::Chevron { .. }
        | Primitive::BentArrow { .. }
        | Primitive::Rhombus { .. }
        | Primitive::Tube { .. }
        | Primitive::CircleSegment { .. }
        | Primitive::SpeechRect { .. }
        | Primitive::SpeechOval { .. }
        | Primitive::Cloud { .. }
        | Primitive::Bolt { .. }
        | Primitive::Shield { .. }
        | Primitive::Tag { .. }
        | Primitive::Check { .. }
        | Primitive::Banner { .. }
        | Primitive::Brace { .. }) => super::dims_table_signs::dims_sign(p),
        // ⭐ **E o FLUXOGRAMA delega ao terceiro irmão** — ver [`super::dims_table_flow`].
        p @ (Primitive::Parallelogram { .. }
        | Primitive::Delay { .. }
        | Primitive::Display { .. }
        | Primitive::OffPage { .. }) => super::dims_table_flow::dims_flow(p),
        // ⭐ **E as duas CURVAS** (W123) — ver [`super::dims_table_flow::dims_curve`].
        p @ (Primitive::Spiral { .. } | Primitive::Document { .. }) => {
            super::dims_table_flow::dims_curve(p)
        }
        // ⚠️ **Inalcançável pelo caminho do produto** — ver o cabeçalho: o único chamador nomeia as
        // catorze chapas. Um `Vec` vazio e não um `panic`: uma forma nova esquecida aqui tem de
        // aparecer como um painel sem linhas no gate que a conta, e não derrubar o app.
        _ => Vec::new(),
    }
}
