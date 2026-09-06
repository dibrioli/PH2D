//! ⭐ **O QUE CADA FORMA RECUSA** — a validação por primitiva.
//!
//! # Por que ela saiu do `lib.rs`
//!
//! O `lib.rs` desta crate responde por três coisas: o que uma forma é, como uma árvore se monta, e
//! o que o documento recusa. A W106 acrescentou catorze primitivas e o arquivo passou dos **700**
//! do gate de LOC da workspace. ⚠️ **Partir para irmão, nunca uma entrada na allowlist.**
//!
//! # A invariante que estas linhas defendem
//!
//! ⭐ *Um documento que EXISTE está válido.* É ela que faz o painel poder oferecer o que oferece:
//! cada `set_*` revalida e devolve o documento intacto se recusar, então nenhum controle precisa de
//! saber a regra — ele pergunta ao [`crate::round_limit`], que é a **mesma** função.
//!
//! ⚠️ **Recusar em voz alta é o ponto.** Uma primitiva com números impossíveis não produz «quase» a
//! forma: produz um campo que deixou de ser uma distância, e o sintoma aparece três camadas abaixo,
//! na malha, como uma superfície rasgada que ninguém liga ao número que a causou.

use crate::{
    FieldError, MAX_GEAR_TEETH, MAX_PRISM_SIDES, MAX_STAR_POINTS, MIN_GEAR_TEETH, MIN_PRISM_SIDES,
    MIN_STAR_POINTS, Primitive, round_limit,
};

pub(crate) fn validate_primitive(idx: u32, p: &Primitive) -> Result<(), FieldError> {
    let positive = |v: f32, what: &'static str| -> Result<(), FieldError> {
        if !v.is_finite() || v <= 0.0 {
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
    // ⭐⭐ **OS DOIS RECUOS DE UMA ARESTA** (Enio, 2026-08-30) — o chanfro e o filete, contra a MESMA
    // parede. Os dois recuam a superfície a partir da mesma quina, e um chanfro que a atravessa
    // apaga a forma pelo mesmo mecanismo que um filete grande demais.
    //
    // ⚠️ **É uma porta e não duas chamadas em cada braço**: são 21 primitivas, e uma que esquecesse
    // a segunda linha aceitaria um número que o resto do módulo assume validado.
    let round_fits = |round: f32, chamfer: f32, limit: f32| -> Result<(), FieldError> {
        um_recuo(chamfer, limit)?;
        um_recuo(round, limit)
    };
    match *p {
        Primitive::Box {
            half,
            round,
            chamfer,
        } => {
            for h in half {
                positive(h, "half")?;
            }
            // ⚠️ O limite é a MENOR meia-extensão: a receita do arredondamento encolhe a caixa em
            // `round` nos três eixos, e uma delas ficando ≤ 0 não é "quase" — é uma caixa que
            // deixou de existir naquele eixo, e o campo que sai disso não é uma distância.
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Sphere { radius } => positive(radius, "radius"),
        Primitive::Cylinder {
            radius,
            half_height,
            round,
            chamfer,
        } => {
            positive(radius, "radius")?;
            positive(half_height, "half_height")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Torus { major, minor } => {
            positive(major, "major")?;
            positive(minor, "minor")
        }
        Primitive::Extrude {
            profile: _,
            half_height,
            round,
            chamfer,
        } => {
            positive(half_height, "half_height")?;
            // ⚠️ O limite é a meia-altura, e **só** ela. Um `round` maior do que a meia-largura do
            // perfil não é um erro: a receita (encolher a fonte, depois deslocar) é uma **abertura
            // morfológica**, e o que ela faz a um pescoço mais fino que `2·round` é exatamente o
            // que arredondar com esse raio deveria fazer — o pescoço desaparece. O campo continua a
            // ser um limite conservador de distância; a forma é a certa.
            //
            // Na altura não é assim: com `round ≥ half_height` o termo axial inverte de sinal e o
            // sólido deixa de existir — isso não é abertura, é uma forma que ninguém pediu.
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Revolve { ref profile } => {
            let min_x = profile.bounds().0[0];
            if min_x < 0.0 {
                return Err(FieldError::ProfileCrossesAxis { node: idx, min_x });
            }
            Ok(())
        }
        // ⚠️ **O `top` pode ser ZERO, e é o cone fechado** — só ele entre todos os números deste
        // arquivo. Exigir `> 0` proibiria a forma que dá nome à primitiva.
        Primitive::Cone {
            bottom,
            top,
            half_height,
            round,
            chamfer,
        } => {
            positive(bottom, "bottom")?;
            if !top.is_finite() || top < 0.0 {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "top",
                });
            }
            positive(half_height, "half_height")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Capsule {
            radius,
            half_height,
        } => {
            positive(radius, "radius")?;
            positive(half_height, "half_height")
        }
        Primitive::Prism {
            sides,
            bottom,
            top,
            half_height,
            round,
            chamfer,
        } => {
            // ⚠️ **A contagem é COAGIDA na porta, não recusada**: um prisma de 2 lados não é uma
            // forma degenerada que o artista queira ver recusada — é um valor que a UI nunca
            // oferece e que só um documento estragado traz. Recusar aqui rejeitaria a peça inteira.
            if !(MIN_PRISM_SIDES..=MAX_PRISM_SIDES).contains(&sides) {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "sides",
                });
            }
            positive(bottom, "bottom")?;
            // ⚠️ Zero é a **pirâmide** — a mesma excepção do [`Primitive::Cone`], e pela mesma razão.
            if !top.is_finite() || top < 0.0 {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "top",
                });
            }
            positive(half_height, "half_height")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Wedge {
            half,
            round,
            chamfer,
        } => {
            for h in half {
                positive(h, "half")?;
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::TorusArc {
            major,
            minor,
            angle,
            round,
            chamfer,
        } => {
            positive(major, "major")?;
            positive(minor, "minor")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))?;
            // ⚠️ **O ângulo é o único número deste arquivo cujo teto importa tanto quanto o piso**:
            // acima de `2π` o sector deixa de ser exprimível por semiplanos, e a porta coage em vez
            // de recusar (o slider pára lá, e só um documento estragado traz mais).
            if !angle.is_finite() || angle <= 0.0 {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "angle",
                });
            }
            Ok(())
        }
        Primitive::Star {
            points,
            outer,
            inner,
            half_height,
            round,
            chamfer,
        } => {
            // ⚠️ **COAGIDA na porta como a contagem de lados** — a UI nunca oferece fora da faixa,
            // então um valor de fora só chega por um documento estragado, e recusar ali rejeitaria
            // a peça inteira por causa de um número que o documento sabe arredondar.
            if !(MIN_STAR_POINTS..=MAX_STAR_POINTS).contains(&points) {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "points",
                });
            }
            positive(outer, "outer")?;
            positive(inner, "inner")?;
            // ⚠️ **O vale TEM de estar dentro da ponta**, e isto é validade e não gosto: com
            // `inner >= outer` as línguas invertem-se e a união devolve **o polígono dos vales** —
            // uma estrela que, ao arrastar um número, deixa de ser uma estrela **sem dizer nada**.
            if inner >= outer {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "inner",
                });
            }
            positive(half_height, "half_height")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::BoxFrame {
            half,
            thickness,
            round,
            chamfer,
        } => {
            for h in half {
                positive(h, "half")?;
            }
            positive(thickness, "thickness")?;
            // ⚠️ **Uma aresta mais grossa do que a meia-extensão fecha a gaiola** — as vigas
            // opostas encontram-se e a moldura vira uma caixa maciça. O `>` (e não `>=`) é
            // deliberado: com a igualdade elas tocam-se e o miolo some, que é a forma-limite.
            if thickness > half[0].min(half[1]).min(half[2]) {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "thickness",
                });
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Ellipsoid { radii } => {
            for r in radii {
                positive(r, "radius")?;
            }
            Ok(())
        }
        // ─────────────────────────── W106 ───────────────────────────
        // ⚠️ **Cada uma recusa em VOZ ALTA o que produziria a forma errada em silêncio.** Um
        // documento que existe está válido — é a invariante desta crate, e é ela que faz o painel
        // poder oferecer o que oferece.
        Primitive::Octahedron {
            radius,
            round,
            chamfer,
        } => {
            positive(radius, "radius")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⛔⛔ **A cerca que a fórmula EXIGE:** com `|bottom − top| >= 2·half_height` uma esfera
        // contém a outra, a tangente comum **não existe** e o `a` do denominador vai a zero. Não é
        // uma preferência de forma — é a fronteira em que a expressão deixa de ser definida.
        Primitive::RoundCone {
            bottom,
            top,
            half_height,
        } => {
            positive(bottom, "radius")?;
            positive(top, "radius")?;
            positive(half_height, "half_height")?;
            if (bottom - top).abs() >= 2.0 * half_height {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "half_height",
                });
            }
            Ok(())
        }
        // ⚠️ `cut >= radius` não deixa peça nenhuma; `cut <= −radius` é a esfera inteira e é
        // legítimo (o corte fica fora dela).
        Primitive::CutSphere {
            radius,
            cut,
            round,
            chamfer,
        } => {
            positive(radius, "radius")?;
            if cut >= radius {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "cut",
                });
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::HollowDome {
            radius,
            cut,
            thickness,
            round,
            chamfer,
        } => {
            positive(radius, "radius")?;
            positive(thickness, "thickness")?;
            // A casca tem de caber dentro da esfera: `thickness/2 < radius`, senão o miolo fecha.
            if thickness * 0.5 >= radius || cut >= radius {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "thickness",
                });
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Link {
            major,
            minor,
            length,
        } => {
            positive(major, "radius")?;
            positive(minor, "thickness")?;
            // ⚠️ `length` pode ser ZERO — aí o elo é um toro, que é a forma-limite e não um erro.
            if !length.is_finite() || length < 0.0 {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "length",
                });
            }
            // O tubo não pode fechar o buraco.
            if minor >= major {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "thickness",
                });
            }
            Ok(())
        }
        Primitive::SolidAngle {
            radius,
            angle,
            round,
            chamfer,
        } => {
            positive(radius, "radius")?;
            positive(angle, "angle")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⚠️ **A ponta do dente TEM de passar o corpo**, senão não há dente — e uma engrenagem sem
        // dentes é um disco com um nome errado.
        Primitive::Gear {
            teeth,
            root,
            outer,
            tooth,
            half_height,
            round,
            chamfer,
        } => {
            if !(MIN_GEAR_TEETH..=MAX_GEAR_TEETH).contains(&teeth) {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "teeth",
                });
            }
            positive(root, "radius")?;
            positive(half_height, "half_height")?;
            if outer <= root {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "radius_outer",
                });
            }
            if !tooth.is_finite() || tooth <= 0.0 || tooth >= 1.0 {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "tooth",
                });
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⚠️ **O braço tem de passar a largura dele**, senão a cruz é um quadrado.
        Primitive::Cross {
            arm,
            width,
            half_height,
            round,
            chamfer,
        } => {
            positive(arm, "arm")?;
            positive(width, "width")?;
            positive(half_height, "half_height")?;
            if width >= arm {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "width",
                });
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Heart {
            size,
            half_height,
            round,
            chamfer,
        } => {
            positive(size, "size")?;
            positive(half_height, "half_height")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⚠️ **A mordida tem de deixar crescente:** com `bite >= radius + offset` ela come o disco
        // inteiro e não sobra peça nenhuma.
        Primitive::Moon {
            radius,
            bite,
            offset,
            half_height,
            round,
            chamfer,
        } => {
            positive(radius, "radius")?;
            positive(bite, "radius")?;
            positive(half_height, "half_height")?;
            if !offset.is_finite() || bite >= radius + offset {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "bite",
                });
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⚠️ **A ponta tem de estar FORA da bolha** — de dentro dela não há tangente.
        Primitive::Drop {
            radius,
            height,
            half_height,
            round,
            chamfer,
        } => {
            positive(radius, "radius")?;
            positive(half_height, "half_height")?;
            if height <= radius {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "height",
                });
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Pie {
            radius,
            angle,
            half_height,
            round,
            chamfer,
        } => {
            positive(radius, "radius")?;
            positive(angle, "angle")?;
            positive(half_height, "half_height")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Trapezoid {
            bottom,
            top,
            half_width,
            half_height,
            round,
            chamfer,
        } => {
            positive(bottom, "width")?;
            positive(top, "width")?;
            positive(half_width, "half_width")?;
            positive(half_height, "half_height")?;
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⚠️ **O afastamento tem de ser MENOR que o raio**, senão os dois discos não se cruzam e a
        // lente é vazia.
        Primitive::Vesica {
            radius,
            offset,
            half_height,
            round,
            chamfer,
        } => {
            positive(radius, "radius")?;
            positive(half_height, "half_height")?;
            if !offset.is_finite() || offset < 0.0 || offset >= radius {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "offset",
                });
            }
            round_fits(round, chamfer, round_limit(p).unwrap_or(0.0))
        }
        // ⭐ **Os SINAIS delegam ao irmão** — ver [`super::validate_signs`]. ⚠️ A corrente não se
        // perde: este `match` continua exaustivo, então uma primitiva nova é **erro de compilação**
        // aqui até alguém dizer o que ela recusa.
        Primitive::Arrow { .. }
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
        | Primitive::Brace { .. } => super::validate_signs::validate_sign(p, idx),
        // ⭐ **E o FLUXOGRAMA ao terceiro** — ver [`super::validate_flow`].
        Primitive::Parallelogram { .. }
        | Primitive::Delay { .. }
        | Primitive::Display { .. }
        | Primitive::OffPage { .. } => super::validate_flow::validate_flow(p, idx),
        Primitive::Spiral { .. } | Primitive::Document { .. } => {
            super::validate_flow::validate_curve(p, idx)
        }
    }
}
