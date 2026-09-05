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
        // ─────────────────────────── W119 ───────────────────────────
        // ⚠️ **A ORDEM é a identidade da linha** — o `set_dim` recebe o ÍNDICE, e um desalinho entre
        // esta tabela e o `dims_write.rs` faz um arrasto escrever noutro número, em silêncio.
        Primitive::Arrow {
            heads,
            half_length,
            shaft,
            head,
            head_length,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.heads",
                value: *heads as f32,
                span: Span::Count {
                    min: crate::MIN_ARROW_HEADS,
                    max: crate::MAX_ARROW_HEADS,
                },
            },
            Dim {
                key: "field.dim.length",
                value: half_length * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.shaft",
                value: shaft * 2.0,
                // ⚠️ **A parede é a LARGURA DA PONTA**: uma haste tão larga como a ponta é uma seta
                // sem farpa, e a partir dali a forma deixa de ser uma seta.
                span: Span::Wall(*head * 2.0),
            },
            Dim {
                key: "field.dim.head_width",
                value: head * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.head_length",
                value: *head_length,
                span: Span::Wall(half_length * 2.0),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Chevron {
            half_length,
            half_span,
            thickness,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.length",
                value: half_length * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.span",
                value: half_span * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.thickness",
                value: *thickness,
                // A banda não pode ser mais grossa do que a meia-envergadura, senão fecha o vazio.
                span: Span::Wall(*half_span),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::BentArrow {
            run,
            rise,
            shaft,
            head,
            head_length,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.length",
                value: run * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.span",
                value: rise * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.shaft",
                value: shaft * 2.0,
                span: Span::Wall(*head * 2.0),
            },
            Dim {
                key: "field.dim.head_width",
                value: head * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.head_length",
                value: *head_length,
                span: Span::Wall(rise * 2.0),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Rhombus {
            half_width,
            half_span,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: half_width * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.span",
                value: half_span * 2.0,
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
        Primitive::Tube {
            outer,
            inner,
            angle,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius_outer",
                value: *outer,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.radius_inner",
                value: *inner,
                span: Span::Wall(*outer),
            },
            // ⚠️ **A faixa vai até `π`, e o topo dela é o anel FECHADO** — ver [`sd_tube`]: ali o
            // sector sai da árvore. Uma faixa que parasse antes deixaria o tubo inalcançável pelo
            // controlo que o descreve.
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
        Primitive::CircleSegment {
            radius,
            cut,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            // ⚠️ **A corda é uma POSIÇÃO**, e pode ser negativa — é a faixa do corte da esfera
            // cortada, pela mesma razão.
            Dim {
                key: "field.dim.cut",
                value: *cut,
                span: Span::Free,
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        // ⚠️ **Inalcançável pelo caminho do produto** — ver o cabeçalho: o único chamador nomeia as
        // catorze chapas. Um `Vec` vazio e não um `panic`: uma forma nova esquecida aqui tem de
        // aparecer como um painel sem linhas no gate que a conta, e não derrubar o app.
        _ => Vec::new(),
    }
}
