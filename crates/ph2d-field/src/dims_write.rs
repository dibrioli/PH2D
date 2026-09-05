//! ⭐ **ESCREVER num número de uma forma** — a metade de escrita do [`super`], onde vivem as leis
//! que decidem o que a porta **coage** e o que ela **recusa**.
//!
//! # Por que ela saiu do `dims.rs`
//!
//! O irmão responde a *«o que esta forma mede»* (a [`super::Dim`], a [`super::Span`] e a tabela por
//! primitiva); aqui responde-se a *«o que acontece quando alguém escreve»*. A W103/W104
//! acrescentaram cinco formas e o arquivo passou dos **700** do gate de LOC. ⚠️ **A cura é partir
//! para irmão, nunca uma entrada na allowlist.**
//!
//! ⚠️ O `pub use` no [`super`] mantém `ph2d_field::set_dim` e `ph2d_field::scale_primitive` — cortar
//! um arquivo não pode custar uma reescrita em cada sítio que o chamava.

use super::{Span, dims};
use crate::{FieldError, Primitive, round_limit};

/// ⭐ **Escreve uma dimensão**, ou recusa.
///
/// # ⚠️ Encolher uma forma ENCOLHE o filete dela, e não é recusado
///
/// Um `round` que deixa de caber quando a caixa encolhe é a situação normal, não um erro: o artista
/// pediu o tamanho, e o filete é o que **decorre** dele. Recusar obrigaria a desfazer o filete
/// primeiro — dois gestos onde há um — e é o que todo CAD resolve limitando o filete em silêncio.
///
/// ⚠️ **Em silêncio, mas não invisível**: o número do filete é uma linha do mesmo painel, e ela
/// muda à vista. Um valor que muda sozinho **sem aparecer** seria outra coisa.
///
/// # Errors
/// [`FieldError::NonPositive`] para um valor não-finito ou ≤ 0, e para um índice que não é desta
/// forma. [`FieldError::RoundTooLarge`] quando é o próprio filete que não cabe.
pub fn set_dim(p: &mut Primitive, node: u32, index: usize, value: f32) -> Result<(), FieldError> {
    let bad = |what: &'static str| FieldError::NonPositive { node, what };
    // ⭐⭐ **QUEM DECIDE SE O ZERO PASSA É A FAIXA DECLARADA** (W101), e não uma excepção escrita
    // aqui.
    //
    // ⚠️ Esta guarda dizia `value <= 0.0` para tudo, e isso tornava o **cone fechado**
    // indigitável: o raio de topo zero é a forma que dá nome à primitiva. Uma excepção
    // `if é_cone && index == 1` curaria o caso e não a família — a próxima grandeza cujo zero
    // significa alguma coisa voltaria a bater na mesma linha. A [`Span`] já sabe a resposta
    // (`FromZero`), e ela vem do mesmo sítio que o painel lê.
    let faixa = dims(p).get(index).map(|d| d.span);
    let zero_ok = matches!(
        faixa,
        // ⭐ A `WallFromZero` é a faixa dos DOIS RECUOS de uma aresta, e o zero **é** a aresta viva —
        // o estado de nascimento do chanfro e o destino de quem quer desfazer um filete. Ver o doc
        // dela para o defeito pré-existente que isto cura.
        Some(Span::FromZero | Span::WallFromZero(_) | Span::Free)
    );
    // ⭐⭐⭐ **E QUEM DECIDE SE O NEGATIVO PASSA É A MESMA FAIXA** (W119).
    //
    // ⛔⛔ **Defeito PRÉ-EXISTENTE**, apanhado ao ligar o segmento de círculo: o `cut` da esfera
    // cortada e o da cúpula oca declaram [`Span::Free`], cuja doc diz com todas as letras que *«a de
    // baixo é negativa»* — e esta guarda recusava **tudo** o que fosse `< 0`. ⇒ o slider descia
    // abaixo de zero e o número parava lá **sem dizer porquê**: exactamente a affordance que mente
    // que a [`Span::WallFromZero`] foi criada para curar, um campo ao lado.
    //
    // ⚠️ **A cura é a mesma da W101**: quem sabe a resposta é a faixa declarada, e não uma excepção
    // escrita aqui — uma `if é_esfera_cortada && index == 1` curaria o caso e não a família.
    let negativo_ok = matches!(faixa, Some(Span::Free));
    if !value.is_finite() || (value < 0.0 && !negativo_ok) || (value == 0.0 && !zero_ok) {
        return Err(bad("dim"));
    }
    let half = value * 0.5;
    match (p, index) {
        (Primitive::Box { half: h, .. }, i @ 0..=2) => h[i] = half,
        (Primitive::Sphere { radius }, 0) | (Primitive::Cylinder { radius, .. }, 0) => {
            *radius = value;
        }
        (Primitive::Cylinder { half_height, .. }, 1)
        | (Primitive::Extrude { half_height, .. }, 0) => *half_height = half,
        (Primitive::Torus { major, .. }, 0) => *major = value,
        (Primitive::Torus { minor, .. }, 1) => *minor = value,
        (Primitive::Cone { bottom, .. }, 0) => *bottom = value,
        // ⚠️ **O único destino de um zero neste arquivo** — ver a guarda acima.
        (Primitive::Cone { top, .. }, 1) => *top = value,
        (Primitive::Cone { half_height, .. }, 2) | (Primitive::Capsule { half_height, .. }, 1) => {
            *half_height = half
        }
        (Primitive::Capsule { radius, .. }, 0) | (Primitive::TorusArc { major: radius, .. }, 0) => {
            *radius = value;
        }
        (Primitive::Prism { bottom, .. }, 1) => *bottom = value,
        (Primitive::Prism { top, .. }, 2) => *top = value,
        (Primitive::Prism { half_height, .. }, 3) => *half_height = half,
        (Primitive::Wedge { half: h, .. }, i @ 0..=2) => h[i] = half,
        (Primitive::TorusArc { minor, .. }, 1) => *minor = value,
        // ⚠️ **COAGE no teto**, como a contagem de lados: acima de uma volta o sector não existe.
        (Primitive::TorusArc { angle, .. }, 2) => *angle = value.min(std::f32::consts::TAU),
        (Primitive::Ellipsoid { radii }, i @ 0..=2)
        | (Primitive::BoxFrame { half: radii, .. }, i @ 0..=2) => radii[i] = half,
        (
            Primitive::BoxFrame {
                half, thickness, ..
            },
            3,
        ) => {
            *thickness = keep_below(value, half[0].min(half[1]).min(half[2]));
        }
        // ⭐⭐ **A ÚNICA parede deste arquivo que é OUTRO CAMPO**, e ela coage nos dois sentidos.
        //
        // ⚠️ Recusar seria a resposta errada aqui, e não por conforto: o slider do vale pára no raio
        // da ponta (a [`Span::Wall`] di-lo), então o valor de fora só chega por um arrasto do OUTRO
        // controlo — baixar a ponta até passar o vale. *Uma recusa nesse gesto pararia o arrasto sem
        // dizer porquê, num campo em que o artista nem estava a tocar.*
        (Primitive::Star { outer, inner, .. }, 1) => *outer = keep_above(value, *inner),
        (Primitive::Star { outer, inner, .. }, 2) => *inner = keep_below(value, *outer),
        (Primitive::Star { half_height, .. }, 3) => *half_height = half,
        (Primitive::Star { points, .. }, 0) => {
            // ⚠️ Coage como a contagem de lados do prisma, e pela mesma razão.
            *points = (value.round() as u32).clamp(crate::MIN_STAR_POINTS, crate::MAX_STAR_POINTS);
        }
        // ─────────────────────────── W106 ───────────────────────────
        // ⚠️ **A ORDEM tem de bater com a do `dims_table.rs`** — o índice é a identidade da linha, e
        // um desalinho faz um arrasto escrever noutro número, em silêncio.
        //
        // ⚠️ E o que a tabela mostra **DOBRADO** (uma largura, uma altura) escreve-se como `half`;
        // o que ela mostra cru (um raio, um ângulo) escreve-se como `value`.
        (Primitive::Octahedron { radius, .. }, 0)
        | (Primitive::CutSphere { radius, .. }, 0)
        | (Primitive::HollowDome { radius, .. }, 0)
        | (Primitive::SolidAngle { radius, .. }, 0)
        | (Primitive::Link { major: radius, .. }, 0)
        | (Primitive::Moon { radius, .. }, 0)
        | (Primitive::Drop { radius, .. }, 0)
        | (Primitive::Pie { radius, .. }, 0)
        | (Primitive::Vesica { radius, .. }, 0) => *radius = value,
        (Primitive::RoundCone { bottom, .. }, 0) => *bottom = value,
        (Primitive::RoundCone { top, .. }, 1) => *top = value,
        (Primitive::RoundCone { half_height, .. }, 2) => *half_height = half,
        // ⚠️ O corte é uma POSIÇÃO: pode ser negativo, e a guarda de cima deixa-o passar porque a
        // faixa dele é `Span::Free`.
        (Primitive::CutSphere { cut, .. }, 1) | (Primitive::HollowDome { cut, .. }, 1) => {
            *cut = value;
        }
        (
            Primitive::HollowDome {
                radius, thickness, ..
            },
            2,
        ) => *thickness = keep_below(value, *radius * 2.0),
        (Primitive::Link { major, minor, .. }, 1) => *minor = keep_below(value, *major),
        (Primitive::Link { length, .. }, 2) => *length = half,
        (Primitive::SolidAngle { angle, .. }, 1) | (Primitive::Pie { angle, .. }, 1) => {
            *angle = value.min(std::f32::consts::PI);
        }
        (Primitive::Gear { teeth, .. }, 0) => {
            // ⚠️ **COAGE, não recusa** — a lei do prisma e da estrela, e pela mesma razão.
            *teeth = (value.round() as u32).clamp(crate::MIN_GEAR_TEETH, crate::MAX_GEAR_TEETH);
        }
        (Primitive::Gear { root, outer, .. }, 1) => *root = keep_below(value, *outer),
        (Primitive::Gear { root, outer, .. }, 2) => *outer = keep_above(value, *root),
        (Primitive::Gear { tooth, .. }, 3) => *tooth = keep_below(value, 1.0),
        (Primitive::Gear { half_height, .. }, 4) => *half_height = half,
        (Primitive::Cross { arm, .. }, 0) => *arm = half,
        (Primitive::Cross { arm, width, .. }, 1) => *width = keep_below(half, *arm),
        (Primitive::Cross { half_height, .. }, 2) => *half_height = half,
        (Primitive::Heart { size, .. }, 0) => *size = value,
        (Primitive::Heart { half_height, .. }, 1) => *half_height = half,
        (Primitive::Moon { bite, .. }, 1) => *bite = value,
        (Primitive::Moon { offset, .. }, 2) => *offset = value,
        (Primitive::Moon { half_height, .. }, 3) => *half_height = half,
        (Primitive::Drop { height, radius, .. }, 1) => *height = keep_above(value, *radius),
        (Primitive::Drop { half_height, .. }, 2)
        | (Primitive::Pie { half_height, .. }, 2)
        | (Primitive::Vesica { half_height, .. }, 2) => *half_height = half,
        (Primitive::Trapezoid { bottom, .. }, 0) => *bottom = half,
        (Primitive::Trapezoid { top, .. }, 1) => *top = half,
        (Primitive::Trapezoid { half_width, .. }, 2) => *half_width = half,
        (Primitive::Trapezoid { half_height, .. }, 3) => *half_height = half,
        (Primitive::Vesica { radius, offset, .. }, 1) => *offset = keep_below(value, *radius),
        // ─────────────────────────── W119 ───────────────────────────
        // ⚠️ **A ORDEM tem de bater com a do `dims_table.rs`** — o índice É a linha.
        (Primitive::Arrow { heads, .. }, 0) => {
            // ⚠️ **COAGE, não recusa** — a lei da contagem de lados do prisma, e pela mesma razão.
            *heads = (value.round() as u32).clamp(crate::MIN_ARROW_HEADS, crate::MAX_ARROW_HEADS);
        }
        (Primitive::Arrow { half_length, .. }, 1) => *half_length = half,
        // ⭐ A parede é a LARGURA DA PONTA, e ela coage: o slider da haste pára ali, então um valor
        // de fora só chega por um arrasto do OUTRO controlo. É a lei do vale da estrela.
        (Primitive::Arrow { shaft, head, .. }, 2) => *shaft = keep_below(half, *head),
        // ⚠️ **A ponta tem parede, e ela é o COMPRIMENTO da seta**: mais larga do que a peça é
        // comprida, ela deixa de se ler como uma seta — e as duas rectas do flanco ficam
        // quase paralelas, com o campo a subir a `1,16`.
        (
            Primitive::Arrow {
                shaft,
                head,
                half_length,
                ..
            },
            3,
        ) => *head = keep_below(keep_above(half, *shaft), *half_length),
        (
            Primitive::Arrow {
                head_length,
                half_length,
                ..
            },
            4,
        ) => *head_length = keep_below(value, *half_length * 2.0),
        (Primitive::Arrow { half_height, .. }, 5)
        | (Primitive::Chevron { half_height, .. }, 3)
        | (Primitive::BentArrow { half_height, .. }, 5)
        | (Primitive::Rhombus { half_height, .. }, 2)
        | (Primitive::Tube { half_height, .. }, 3)
        | (Primitive::CircleSegment { half_height, .. }, 2) => *half_height = half,
        (Primitive::Chevron { half_length, .. }, 0) => *half_length = half,
        (
            Primitive::Chevron {
                half_span,
                thickness,
                ..
            },
            1,
        ) => {
            *half_span = keep_above(half, *thickness);
        }
        (
            Primitive::Chevron {
                thickness,
                half_span,
                ..
            },
            2,
        ) => *thickness = keep_below(value, *half_span),
        (Primitive::BentArrow { run, .. }, 0) => *run = half,
        (Primitive::BentArrow { rise, .. }, 1) => *rise = half,
        (Primitive::BentArrow { shaft, head, .. }, 2) => *shaft = keep_below(half, *head),
        (Primitive::BentArrow { shaft, head, .. }, 3) => *head = keep_above(half, *shaft),
        (
            Primitive::BentArrow {
                head_length, rise, ..
            },
            4,
        ) => *head_length = keep_below(value, *rise * 2.0),
        (Primitive::Rhombus { half_width, .. }, 0) => *half_width = half,
        (Primitive::Rhombus { half_span, .. }, 1) => *half_span = half,
        (Primitive::Tube { outer, inner, .. }, 0) => *outer = keep_above(value, *inner),
        (Primitive::Tube { outer, inner, .. }, 1) => *inner = keep_below(value, *outer),
        // ⚠️ **COAGE no teto, e o teto É o anel fechado** — ver [`ph2d_field_eval::ops_plates`]:
        // em `π` o sector sai da árvore em vez de degenerar num corte de espessura zero.
        (Primitive::Tube { angle, .. }, 2) => *angle = value.min(std::f32::consts::PI),
        (Primitive::CircleSegment { radius, .. }, 0) => *radius = value,
        // ⚠️ **A corda é uma POSIÇÃO** e passa negativa — ver a guarda do [`Span::Free`] acima.
        (Primitive::CircleSegment { cut, .. }, 1) => *cut = value,
        // ─────────────────────────── W120 ───────────────────────────
        // ⚠️ **A ORDEM tem de bater com a do `dims_table_signs.rs`** — o índice É a linha.
        (Primitive::SpeechRect { half_width, .. }, 0)
        | (Primitive::SpeechOval { half_width, .. }, 0)
        | (Primitive::Bolt { half_width, .. }, 0)
        | (Primitive::Tag { half_width, .. }, 0)
        | (Primitive::Check { half_width, .. }, 0)
        | (Primitive::Banner { half_width, .. }, 0)
        | (Primitive::Cloud { half_width, .. }, 1) => *half_width = half,
        (Primitive::SpeechRect { half_span, .. }, 1)
        | (Primitive::SpeechOval { half_span, .. }, 1)
        | (Primitive::Bolt { half_span, .. }, 1)
        | (Primitive::Tag { half_span, .. }, 1)
        | (Primitive::Check { half_span, .. }, 1)
        | (Primitive::Banner { half_span, .. }, 1)
        | (Primitive::Cloud { half_span, .. }, 2)
        | (Primitive::Brace { half_span, .. }, 0) => *half_span = half,
        (Primitive::SpeechRect { tail, .. }, 2) | (Primitive::SpeechOval { tail, .. }, 2) => {
            *tail = value;
        }
        // ⚠️ **A cauda da nuvem COAGE na parede dela** — ver o doc da linha em `dims_table_signs`.
        (
            Primitive::Cloud {
                tail, half_span, ..
            },
            3,
        ) => *tail = keep_below(value, *half_span * 1.4),
        (Primitive::SpeechRect { half_height, .. }, 3)
        | (Primitive::SpeechOval { half_height, .. }, 3)
        | (Primitive::Cloud { half_height, .. }, 4)
        | (Primitive::Bolt { half_height, .. }, 2)
        | (Primitive::Shield { half_height, .. }, 2)
        | (Primitive::Tag { half_height, .. }, 4)
        | (Primitive::Check { half_height, .. }, 3)
        | (Primitive::Banner { half_height, .. }, 3)
        | (Primitive::Brace { half_height, .. }, 2) => *half_height = half,
        (Primitive::Cloud { lobes, .. }, 0) => {
            // ⚠️ **COAGE, não recusa** — a lei da contagem de lados do prisma.
            *lobes = (value.round() as u32).clamp(crate::MIN_CLOUD_LOBES, crate::MAX_CLOUD_LOBES);
        }
        // ⭐ **As duas metades da cerca do escudo**, e ela coage nos dois sentidos: a largura pára
        // em `2 × span`, e subir o span nunca pode deixar a largura para trás.
        (
            Primitive::Shield {
                half_width,
                half_span,
                ..
            },
            0,
        ) => *half_width = keep_below(half, *half_span / 0.9),
        (
            Primitive::Shield {
                half_width,
                half_span,
                ..
            },
            1,
        ) => *half_span = keep_above(half, *half_width * 0.9),
        (
            Primitive::Tag {
                point, half_width, ..
            },
            2,
        ) => *point = keep_below(value, *half_width * 2.0),
        (
            Primitive::Tag {
                hole,
                half_width,
                half_span,
                ..
            },
            3,
        ) => *hole = keep_below(value, (*half_width * 0.3).min(*half_span)),
        (
            Primitive::Check {
                thickness,
                half_width,
                half_span,
                ..
            },
            2,
        ) => *thickness = keep_below(value, half_width.min(*half_span)),
        (
            Primitive::Banner {
                notch, half_width, ..
            },
            2,
        ) => *notch = keep_below(value, *half_width),
        (
            Primitive::Brace {
                thickness,
                half_span,
                ..
            },
            1,
        ) => *thickness = keep_below(value, *half_span * 0.5),
        (Primitive::Prism { sides, .. }, 0) => {
            // ⚠️ **COAGE, não recusa** — a lei do `Unary::Taper`, e pela mesma razão: a faixa já
            // não oferece nada fora de `[MIN, MAX]`, então um valor de fora só chega por outra
            // porta (um ficheiro estragado), e recusar ali rejeitaria a peça inteira. É o
            // **documento** quem arredonda: um valor fracionário vindo de fora vira uma contagem,
            // não meio lado.
            *sides = (value.round() as u32).clamp(crate::MIN_PRISM_SIDES, crate::MAX_PRISM_SIDES);
        }
        // O filete é o último de cada forma que o tem — e ele passa pela lei do filete, que já
        // sabe recusar o que não cabe.
        (
            p @ (Primitive::Box { .. }
            | Primitive::Cylinder { .. }
            | Primitive::Extrude { .. }
            | Primitive::Cone { .. }
            | Primitive::Prism { .. }
            | Primitive::Wedge { .. }
            | Primitive::Star { .. }
            | Primitive::BoxFrame { .. }
            | Primitive::Octahedron { .. }
            | Primitive::CutSphere { .. }
            | Primitive::HollowDome { .. }
            | Primitive::SolidAngle { .. }
            | Primitive::Gear { .. }
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
            | Primitive::TorusArc { .. }),
            i,
        ) if Some(i) == round_index(p) => {
            return set_round(p, node, value);
        }
        // ⭐ **O chanfro entra pelo MESMO portão do filete** — a mesma pergunta à [`dims`], o mesmo
        // braço exaustivo, a mesma parede. ⚠️ E ele vem DEPOIS na guarda porque as duas fileiras têm
        // índices distintos: a de cima só casa a do filete, e sem esta o slider do chanfro cairia no
        // `_ => Err(bad("dim"))` — o slider pinta, arrasta, e o `let _ =` do shell engole o erro.
        (p, i) if Some(i) == chamfer_index(p) => {
            return set_chamfer(p, node, value);
        }
        _ => return Err(bad("dim")),
    }
    Ok(())
}

/// `value`, mantido **estritamente abaixo** de `ceiling` — a folga é uma fração do próprio tecto,
/// pela razão do [`ROUND_MARGIN`] (num alvo de `0,01` um épsilon fixo seria o tecto inteiro).
fn keep_below(value: f32, ceiling: f32) -> f32 {
    value.min(ceiling * (1.0 - ROUND_MARGIN))
}

/// `value`, mantido **estritamente acima** de `floor` — a irmã do [`keep_below`].
fn keep_above(value: f32, floor: f32) -> f32 {
    value.max(floor / (1.0 - ROUND_MARGIN))
}

/// Onde fica o filete na lista desta forma, se ela tiver um.
fn round_index(p: &Primitive) -> Option<usize> {
    dims(p).iter().position(|d| d.key == "field.dim.round")
}

/// Onde fica o **chanfro**, se ela tiver um.
///
/// ⭐ **A pergunta é feita à [`dims`], e não a uma lista escrita à mão** — é isso que faz uma forma
/// nova receber o slider sem uma linha aqui, e é a razão de este arquivo não ter uma segunda
/// enumeração das 21 primitivas com aresta.
fn chamfer_index(p: &Primitive) -> Option<usize> {
    dims(p).iter().position(|d| d.key == "field.dim.chamfer")
}

/// ⭐⭐⭐ **Escreve o CHANFRO de uma forma** (Enio, 2026-08-30).
///
/// ⚠️ **A parede é a MESMA do filete** ([`round_limit`]), e não uma escolhida: os dois recuam a
/// superfície a partir da mesma quina.
///
/// ⚠️ **EXAUSTIVO, pela razão que a [`set_round`] já pagou na W101**: uma primitiva nova com aresta
/// que caísse num braço vazio teria o slider a mexer-se sem escrever nada — *a falha mais cara de
/// diagnosticar, porque não deixa rasto*.
fn set_chamfer(p: &mut Primitive, node: u32, value: f32) -> Result<(), FieldError> {
    let limit = round_limit(p).ok_or(FieldError::NonPositive {
        node,
        what: "chamfer",
    })?;
    if value >= limit {
        return Err(FieldError::RoundTooLarge {
            node,
            round: value,
            limit,
        });
    }
    match p {
        Primitive::Box { chamfer, .. }
        | Primitive::Cylinder { chamfer, .. }
        | Primitive::Extrude { chamfer, .. }
        | Primitive::Cone { chamfer, .. }
        | Primitive::Prism { chamfer, .. }
        | Primitive::Wedge { chamfer, .. }
        | Primitive::Star { chamfer, .. }
        | Primitive::BoxFrame { chamfer, .. }
        | Primitive::Octahedron { chamfer, .. }
        | Primitive::CutSphere { chamfer, .. }
        | Primitive::HollowDome { chamfer, .. }
        | Primitive::SolidAngle { chamfer, .. }
        | Primitive::Gear { chamfer, .. }
        | Primitive::Cross { chamfer, .. }
        | Primitive::Heart { chamfer, .. }
        | Primitive::Moon { chamfer, .. }
        | Primitive::Drop { chamfer, .. }
        | Primitive::Pie { chamfer, .. }
        | Primitive::Trapezoid { chamfer, .. }
        | Primitive::Vesica { chamfer, .. }
        | Primitive::Arrow { chamfer, .. }
        | Primitive::Chevron { chamfer, .. }
        | Primitive::BentArrow { chamfer, .. }
        | Primitive::Rhombus { chamfer, .. }
        | Primitive::Tube { chamfer, .. }
        | Primitive::CircleSegment { chamfer, .. }
        | Primitive::SpeechRect { chamfer, .. }
        | Primitive::SpeechOval { chamfer, .. }
        | Primitive::Cloud { chamfer, .. }
        | Primitive::Bolt { chamfer, .. }
        | Primitive::Shield { chamfer, .. }
        | Primitive::Tag { chamfer, .. }
        | Primitive::Check { chamfer, .. }
        | Primitive::Banner { chamfer, .. }
        | Primitive::Brace { chamfer, .. }
        | Primitive::TorusArc { chamfer, .. } => *chamfer = value,
        Primitive::Sphere { .. }
        | Primitive::Torus { .. }
        | Primitive::Revolve { .. }
        | Primitive::Capsule { .. }
        | Primitive::RoundCone { .. }
        | Primitive::Link { .. }
        | Primitive::Ellipsoid { .. } => {
            return Err(FieldError::NonPositive {
                node,
                what: "chamfer",
            });
        }
    }
    Ok(())
}

fn set_round(p: &mut Primitive, node: u32, value: f32) -> Result<(), FieldError> {
    let limit = round_limit(p).ok_or(FieldError::NonPositive {
        node,
        what: "round",
    })?;
    if value >= limit {
        return Err(FieldError::RoundTooLarge {
            node,
            round: value,
            limit,
        });
    }
    // ⚠️ **EXAUSTIVO, e o `_ => {}` que estava aqui era uma armadilha** (W101): uma primitiva nova
    // COM filete caía no braço vazio, o `round_limit` dela respondia, o `set_round` dizia `Ok` — e
    // o número **nunca era escrito**. Um slider que se mexe e não faz nada é a falha mais cara de
    // diagnosticar, porque não deixa rasto. Com a lista fechada, a próxima é erro de compilação.
    match p {
        Primitive::Box { round, .. }
        | Primitive::Cylinder { round, .. }
        | Primitive::Extrude { round, .. }
        | Primitive::Cone { round, .. }
        | Primitive::Prism { round, .. }
        | Primitive::Wedge { round, .. }
        | Primitive::Star { round, .. }
        | Primitive::BoxFrame { round, .. }
        | Primitive::Octahedron { round, .. }
        | Primitive::CutSphere { round, .. }
        | Primitive::HollowDome { round, .. }
        | Primitive::SolidAngle { round, .. }
        | Primitive::Gear { round, .. }
        | Primitive::Cross { round, .. }
        | Primitive::Heart { round, .. }
        | Primitive::Moon { round, .. }
        | Primitive::Drop { round, .. }
        | Primitive::Pie { round, .. }
        | Primitive::Trapezoid { round, .. }
        | Primitive::Vesica { round, .. }
        | Primitive::Arrow { round, .. }
        | Primitive::Chevron { round, .. }
        | Primitive::BentArrow { round, .. }
        | Primitive::Rhombus { round, .. }
        | Primitive::Tube { round, .. }
        | Primitive::CircleSegment { round, .. }
        | Primitive::SpeechRect { round, .. }
        | Primitive::SpeechOval { round, .. }
        | Primitive::Cloud { round, .. }
        | Primitive::Bolt { round, .. }
        | Primitive::Shield { round, .. }
        | Primitive::Tag { round, .. }
        | Primitive::Check { round, .. }
        | Primitive::Banner { round, .. }
        | Primitive::Brace { round, .. }
        | Primitive::TorusArc { round, .. } => *round = value,
        Primitive::Sphere { .. }
        | Primitive::Torus { .. }
        | Primitive::Revolve { .. }
        | Primitive::Capsule { .. }
        | Primitive::RoundCone { .. }
        | Primitive::Link { .. }
        | Primitive::Ellipsoid { .. } => {}
    }
    Ok(())
}

/// ⭐ **Limita o filete ao que a forma agora comporta** — ver a nota de [`set_dim`].
///
/// Devolve `true` se teve de o mexer, para quem chamar poder dizê-lo.
pub fn clamp_round(p: &mut Primitive) -> bool {
    let Some(limit) = round_limit(p) else {
        return false;
    };
    // ⚠️ **Estritamente abaixo**, e não «até»: a validação recusa `round >= limit`, e um filete
    // exatamente no limite encolheria a fonte a zero. A margem é uma fração do próprio limite, e
    // não um épsilon absoluto — numa peça de 0,01 um épsilon fixo seria o limite inteiro.
    let ceiling = limit * (1.0 - ROUND_MARGIN);
    // ⚠️ Exaustivo pela razão do [`set_round`] — um braço `_` deixaria a primitiva nova com um
    // filete que a peça já não comporta, e a validação recusaria o documento inteiro no gesto
    // seguinte.
    match p {
        Primitive::Box { round, chamfer, .. }
        | Primitive::Cylinder { round, chamfer, .. }
        | Primitive::Extrude { round, chamfer, .. }
        | Primitive::Cone { round, chamfer, .. }
        | Primitive::Prism { round, chamfer, .. }
        | Primitive::Wedge { round, chamfer, .. }
        | Primitive::Star { round, chamfer, .. }
        | Primitive::BoxFrame { round, chamfer, .. }
        | Primitive::Octahedron { round, chamfer, .. }
        | Primitive::CutSphere { round, chamfer, .. }
        | Primitive::HollowDome { round, chamfer, .. }
        | Primitive::SolidAngle { round, chamfer, .. }
        | Primitive::Gear { round, chamfer, .. }
        | Primitive::Cross { round, chamfer, .. }
        | Primitive::Heart { round, chamfer, .. }
        | Primitive::Moon { round, chamfer, .. }
        | Primitive::Drop { round, chamfer, .. }
        | Primitive::Pie { round, chamfer, .. }
        | Primitive::Trapezoid { round, chamfer, .. }
        | Primitive::Vesica { round, chamfer, .. }
        | Primitive::Arrow { round, chamfer, .. }
        | Primitive::Chevron { round, chamfer, .. }
        | Primitive::BentArrow { round, chamfer, .. }
        | Primitive::Rhombus { round, chamfer, .. }
        | Primitive::Tube { round, chamfer, .. }
        | Primitive::CircleSegment { round, chamfer, .. }
        | Primitive::SpeechRect { round, chamfer, .. }
        | Primitive::SpeechOval { round, chamfer, .. }
        | Primitive::Cloud { round, chamfer, .. }
        | Primitive::Bolt { round, chamfer, .. }
        | Primitive::Shield { round, chamfer, .. }
        | Primitive::Tag { round, chamfer, .. }
        | Primitive::Check { round, chamfer, .. }
        | Primitive::Banner { round, chamfer, .. }
        | Primitive::Brace { round, chamfer, .. }
        | Primitive::TorusArc { round, chamfer, .. } => {
            // ⭐⭐ **OS DOIS RECUOS são limitados, e não só o filete** (Enio, 2026-08-30).
            //
            // ⛔ Sem a segunda metade, encolher uma peça chanfrada deixava o chanfro acima da parede
            // e a `validate_primitive` passava a **recusar o documento inteiro** no gesto seguinte —
            // exactamente a armadilha que o doc do `set_round` nomeia para um braço `_`.
            let mut mexeu = false;
            for v in [&mut *round, &mut *chamfer] {
                if *v > ceiling {
                    *v = ceiling.max(0.0);
                    mexeu = true;
                }
            }
            mexeu
        }
        Primitive::Sphere { .. }
        | Primitive::Torus { .. }
        | Primitive::Revolve { .. }
        | Primitive::Capsule { .. }
        | Primitive::RoundCone { .. }
        | Primitive::Link { .. }
        | Primitive::Ellipsoid { .. } => false,
    }
}

/// A folga entre o filete máximo e a parede, em fração da parede. Ver [`clamp_round`].
const ROUND_MARGIN: f32 = 1.0e-3;
