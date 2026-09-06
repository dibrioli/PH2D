//! ⭐ **O QUE CADA SINAL MEDE** — as setas (W119), os balões e os símbolos (W120).
//!
//! # Por que um terceiro arquivo
//!
//! O [`super::dims_table`] responde pelos SÓLIDOS, o [`super::dims_table_plates`] pelas chapas da
//! W106, e este pela família que as duas últimas waves acrescentaram. Cada corte foi pago quando o
//! anterior passou as `700` linhas do gate de LOC. ⚠️ **Partir para irmão, nunca uma entrada na
//! allowlist.**
//!
//! ⚠️ **O braço `_` no fim tem o mesmo censo do irmão** —
//! `every_primitive_offers_at_least_one_dimension` recusa uma forma sem número nenhum, e
//! `every_row_of_every_primitive_can_be_written` recusa uma que tenha linha e não a saiba escrever.
//! *Um braço `_` sem censo ao lado é uma licença.*

use super::{Dim, Span};
use crate::{Primitive, round_limit};

/// O chanfro de um sinal — a mesma parede do filete.
fn chamfer_dim(p: &Primitive, value: f32) -> Dim {
    Dim {
        key: "field.dim.chamfer",
        value,
        span: round_limit(p).map_or(Span::FromZero, Span::WallFromZero),
    }
}

/// O filete de um sinal.
fn round_dim(p: &Primitive, value: f32) -> Dim {
    Dim {
        key: "field.dim.round",
        value,
        span: round_limit(p).map_or(Span::FromZero, Span::WallFromZero),
    }
}

/// **O que este sinal mede.** ⚠️ Só é chamada com as variantes que o irmão nomeia.
pub(super) fn dims_sign(p: &Primitive) -> Vec<Dim> {
    let chamfer_dim = |v: f32| chamfer_dim(p, v);
    let round_dim = |v: f32| round_dim(p, v);
    match p {
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
                span: Span::Wall(half_length * 2.0),
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
                // ⚠️ **O cotovelo tem de caber no braço** (06/09): a validação exige
                // `percurso > haste`, senão a haste sai da caixa da peça.
                span: Span::Floor(shaft * 2.0),
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
                // ⚠️ **A validação já dizia `|corte| ≥ raio`** e a faixa dizia «livre» (06/09).
                span: Span::Walls(*radius),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        // ─────────────────────────── W120 ───────────────────────────
        // ⚠️ **A ORDEM é a identidade da linha** — o `set_dim` recebe o ÍNDICE.
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
                key: "field.dim.tail",
                value: *tail,
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
        Primitive::Cloud {
            lobes,
            half_width,
            half_span,
            tail,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.lobes",
                value: *lobes as f32,
                span: Span::Count {
                    min: crate::MIN_CLOUD_LOBES,
                    max: crate::MAX_CLOUD_LOBES,
                },
            },
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
            // ⭐ **O ZERO é a porta *Cloud*** — a fieira do pensamento desaparece, e é por isso que
            // esta faixa é a `FromZero` e não a `Positive`.
            //
            // ⛔⛔ **E ela tem PAREDE, porque sem uma o controlo não se alcança** (report do Enio,
            // 05/09: *«Tail não tem efeito»*). Sem parede o teto do slider vem da **vista**, e o
            // intervalo útil da cauda — `0` a `0,35` numa peça de `0,5` — cabia nos primeiros
            // por cento do curso: arrastar não parecia fazer nada. *Um controlo cujo intervalo útil
            // não chega a um pixel não oferece o que promete* — a lei que a banda do `Bend` já
            // pagou.
            //
            // ⚠️ **A parede é MEDIDA na própria forma**: a maior bolha da fieira mede `0,30 × tail`,
            // e a menor bossa do corpo `0,42 × half_span`. Em `tail = 1,4 × half_span` as duas são
            // iguais — acima disso a «fieira» tem bolhas maiores que a nuvem, e deixa de ser uma
            // fieira.
            Dim {
                key: "field.dim.tail",
                value: *tail,
                span: Span::WallFromZero(half_span * 1.4),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Bolt {
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
        // ⚠️ **A largura tem PAREDE, e ela é da geometria**: acima de `2 × half_span` o centro do
        // arco cai do outro lado e os lados do escudo curvam para dentro.
        Primitive::Shield {
            half_width,
            half_span,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: half_width * 2.0,
                span: Span::Wall(half_span / 0.45),
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
        Primitive::Tag {
            half_width,
            half_span,
            point,
            hole,
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
                key: "field.dim.point",
                value: *point,
                span: Span::Wall(half_width * 2.0),
            },
            Dim {
                key: "field.dim.hole",
                value: *hole,
                span: Span::Wall((half_width * 0.3).min(*half_span)),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Check {
            half_width,
            half_span,
            thickness,
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
                key: "field.dim.thickness",
                value: *thickness,
                span: Span::Wall(half_width.min(*half_span)),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Banner {
            half_width,
            half_span,
            notch,
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
                key: "field.dim.notch",
                value: *notch,
                span: Span::Wall(*half_width),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        // ⚠️ **A chave não tem largura**, e é uma decisão — ver o doc da variante.
        Primitive::Brace {
            half_span,
            thickness,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.span",
                value: half_span * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.thickness",
                value: *thickness,
                span: Span::Wall(half_span * 0.5),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        // ⚠️ **Inalcançável pelo caminho do produto** — ver o cabeçalho.
        _ => Vec::new(),
    }
}
