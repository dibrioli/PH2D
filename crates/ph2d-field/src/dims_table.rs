//! ⭐ **O QUE CADA FORMA MEDE** — a tabela que dá as linhas do painel, uma por primitiva.
//!
//! # Por que ela saiu do [`super::dims`]
//!
//! O irmão responde *o que uma dimensão É* (o [`Param`], o [`Dim`], a [`Span`] e a lei das
//! meias-extensões); este responde *o que cada forma TEM*. A W106 acrescentou catorze primitivas e
//! o arquivo passou dos **700** que o gate de LOC da workspace fixa.
//!
//! ⚠️ **A cura é partir para irmão, nunca uma entrada na allowlist** — ela existe para os que já
//! estavam acima em 2026-06-20, e o objectivo dela é descer. É a **sexta** vez que esta casa paga
//! este corte, e sempre pelo mesmo motivo: as tabelas medidas e as recusas escritas ao lado das
//! constantes são o valor, não o peso.

use super::{Dim, Span};
use crate::{Primitive, round_limit};

/// **O que esta forma mede.** Vazio quando ela não tem número nenhum a mexer.
///
/// ⚠️ A ordem é a que o painel mostra, e ela é parte da identidade: [`set_dim`] recebe o **índice**,
/// porque é o que um controle cunhado às cegas consegue guardar. Reordenar aqui reordena os
/// controles — e um arrasto a meio passaria a escrever noutro número.
#[must_use]
pub fn dims(p: &Primitive) -> Vec<Dim> {
    // ⭐⭐⭐ **O CHANFRO, ao lado do filete e ANTES dele** (Enio, 2026-08-30: *«poderíamos ter os
    // 2, com chamfer antes de fillet para a possibilidade de arredondar as bordas geradas por
    // chamfer»*). A ordem na lista É a ordem em que as duas operações acontecem na forma.
    //
    // ⚠️ **A MESMA parede do filete**, e não uma escolhida: os dois recuam a superfície a partir da
    // mesma quina, e um chanfro maior do que o `round_limit` come a forma pelo mesmo mecanismo. Uma
    // segunda parede aqui seria uma segunda resposta a *«até onde esta aresta aguenta»*.
    let chamfer_dim = |value: f32| Dim {
        key: "field.dim.chamfer",
        value,
        span: round_limit(p).map_or(Span::FromZero, Span::WallFromZero),
    };
    let round_dim = |value: f32| Dim {
        key: "field.dim.round",
        value,
        // ⚠️ `Positive` só sobra para uma forma que tenha filete e não tenha meia-extensão de onde
        // derivar a parede — não existe hoje, e a alternativa (um `expect`) transformaria uma
        // primitiva nova num pânico em vez de num slider sem teto.
        span: round_limit(p).map_or(Span::FromZero, Span::WallFromZero),
    };
    match p {
        Primitive::Box {
            half,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: half[0] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half[1] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.depth",
                value: half[2] * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Sphere { radius } => vec![Dim {
            key: "field.dim.radius",
            value: *radius,
            span: Span::Positive,
        }],
        Primitive::Cylinder {
            radius,
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
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Torus { major, minor } => vec![
            Dim {
                key: "field.dim.radius",
                value: *major,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.thickness",
                value: *minor,
                span: Span::Positive,
            },
        ],
        Primitive::Extrude {
            half_height,
            round,
            chamfer,
            ..
        } => vec![
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        // ⚠️ Um torno **não tem dimensões próprias**: a forma dele é o contorno desenhado, e um
        // número aqui prometeria mexer numa coisa que só o editor vetorial autora. Ver
        // [`Primitive::Revolve`].
        Primitive::Revolve { .. } => Vec::new(),
        // ⭐⭐ **O topo é o único número deste arquivo cujo piso é ZERO** (W101) — `Span::Positive`
        // proíbe o zero, e o zero é o **cone fechado**. `Span::Walls` também não serve (aceitaria
        // negativo), então a faixa é `From { lo: 0 }`: *a forma que dá nome à primitiva não pode
        // ser indigitável*.
        Primitive::Cone {
            bottom,
            top,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius_bottom",
                value: *bottom,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.radius_top",
                value: *top,
                span: Span::FromZero,
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Capsule {
            radius,
            half_height,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            // ⚠️ **A altura é a do SEGMENTO, não a da peça** — a cápsula mede `2·(h + r)` de ponta a
            // ponta. Publicar o total faria o número saltar ao mexer no raio, que é a linha de
            // cima: *um controle que se mexe sozinho quando se mexe noutro é o que faz o artista
            // deixar de confiar no painel.*
            Dim {
                key: "field.dim.length",
                value: half_height * 2.0,
                span: Span::Positive,
            },
        ],
        // ⭐⭐ **As duas pontas fazem dele a pirâmide** (W102) — a mesma lista do
        // [`Primitive::Cone`], com a contagem de lados à frente.
        Primitive::Prism {
            sides,
            bottom,
            top,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.sides",
                value: *sides as f32,
                span: Span::Count {
                    min: crate::MIN_PRISM_SIDES,
                    max: crate::MAX_PRISM_SIDES,
                },
            },
            Dim {
                key: "field.dim.radius_bottom",
                value: *bottom,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.radius_top",
                value: *top,
                span: Span::FromZero,
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Wedge {
            half,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: half[0] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.depth",
                value: half[1] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half[2] * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::TorusArc {
            major,
            minor,
            angle,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *major,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.thickness",
                value: *minor,
                span: Span::Positive,
            },
            // ⚠️ **Uma PAREDE do documento, e não do gesto**: acima de uma volta o sector deixa de
            // ser exprimível, e o número deixaria de querer dizer alguma coisa.
            Dim {
                key: "field.dim.angle",
                value: *angle,
                span: Span::Wall(std::f32::consts::TAU),
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        // ⭐⭐ **DOIS RAIOS, como o Illustrator** (W103) — e não um raio mais uma razão. ⚠️ A razão
        // seria o único número adimensional deste arquivo, e o artista que quer *«a ponta a 40 e o
        // vale a 15»* teria de fazer a divisão de cabeça. O preço é o `inner` ter uma **parede que
        // é outro campo**, e é o único caso — ver a coerção no [`set_dim`].
        Primitive::Star {
            points,
            outer,
            inner,
            half_height,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.points",
                value: *points as f32,
                span: Span::Count {
                    min: crate::MIN_STAR_POINTS,
                    max: crate::MAX_STAR_POINTS,
                },
            },
            Dim {
                key: "field.dim.radius",
                value: *outer,
                span: Span::Positive,
            },
            // ⚠️ **A parede é o raio da PONTA**, e é do documento: com o vale por fora dela a
            // estrela deixa de ser uma estrela (ver a validação).
            Dim {
                key: "field.dim.radius_inner",
                value: *inner,
                span: Span::Wall(*outer),
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::BoxFrame {
            half,
            thickness,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.width",
                value: half[0] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half[1] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.depth",
                value: half[2] * 2.0,
                span: Span::Positive,
            },
            // ⚠️ **A parede é a MENOR meia-extensão**: acima dela as vigas opostas encontram-se e a
            // gaiola vira uma caixa maciça — o miolo vazio é a forma.
            Dim {
                key: "field.dim.thickness",
                value: *thickness,
                span: Span::Wall(half[0].min(half[1]).min(half[2])),
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        // ⚠️ **As mesmas três chaves da caixa**, e a escolha é deliberada: o artista mede uma peça
        // pela extensão dela, e um elipsóide que mostrasse «semi-eixo» pediria a divisão por dois
        // que nenhuma outra forma deste painel pede.
        Primitive::Ellipsoid { radii } => vec![
            Dim {
                key: "field.dim.width",
                value: radii[0] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: radii[1] * 2.0,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.depth",
                value: radii[2] * 2.0,
                span: Span::Positive,
            },
        ],
        // ─────────────────────────── W106 ───────────────────────────
        // ⚠️ **A ORDEM é a identidade da linha** — o [`set_dim`] recebe o ÍNDICE. Reordenar aqui
        // reordena os controlos, e um arrasto a meio passaria a escrever noutro número.
        //
        // ⚠️ **O `round` é sempre o ÚLTIMO** de quem o tem: é a convenção que o `round_index`
        // assume, e quebrá-la faz o filete de uma forma escrever noutro campo.
        Primitive::Octahedron {
            radius,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::RoundCone {
            bottom,
            top,
            half_height,
        } => vec![
            Dim {
                key: "field.dim.radius_bottom",
                value: *bottom,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.radius_top",
                value: *top,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.height",
                value: half_height * 2.0,
                span: Span::Positive,
            },
        ],
        Primitive::CutSphere {
            radius,
            cut,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.cut",
                value: *cut,
                span: Span::Free,
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::HollowDome {
            radius,
            cut,
            thickness,
            round,
            chamfer,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *radius,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.cut",
                value: *cut,
                span: Span::Free,
            },
            Dim {
                key: "field.dim.thickness",
                value: *thickness,
                span: Span::Wall(*radius * 2.0),
            },
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        Primitive::Link {
            major,
            minor,
            length,
        } => vec![
            Dim {
                key: "field.dim.radius",
                value: *major,
                span: Span::Positive,
            },
            Dim {
                key: "field.dim.thickness",
                value: *minor,
                span: Span::Wall(*major),
            },
            Dim {
                key: "field.dim.length",
                value: length * 2.0,
                span: Span::Positive,
            },
        ],
        Primitive::SolidAngle {
            radius,
            angle,
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
            chamfer_dim(*chamfer),
            round_dim(*round),
        ],
        // ⭐⭐ **AS CHAPAS delegam ao irmão** — ver [`super::dims_table_plates`].
        //
        // ⚠️ **A corrente NÃO se perde:** este `match` continua exaustivo, então uma primitiva nova
        // é **erro de compilação** aqui até alguém dizer a que família ela pertence. O que o irmão
        // recebe é sempre uma das nomeadas nesta linha.
        p @ (Primitive::Gear { .. }
        | Primitive::Cross { .. }
        | Primitive::Heart { .. }
        | Primitive::Moon { .. }
        | Primitive::Drop { .. }
        | Primitive::Pie { .. }
        | Primitive::Trapezoid { .. }
        | Primitive::Vesica { .. }
        | Primitive::Arrow { .. }
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
        | Primitive::Brace { .. }
        | Primitive::Parallelogram { .. }
        | Primitive::Delay { .. }
        | Primitive::Display { .. }
        | Primitive::OffPage { .. }) => super::dims_table_plates::dims_plate(p),
    }
}
