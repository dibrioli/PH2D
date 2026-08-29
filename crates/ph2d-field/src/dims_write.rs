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
    let zero_ok = matches!(dims(p).get(index).map(|d| d.span), Some(Span::FromZero));
    if !value.is_finite() || value < 0.0 || (value == 0.0 && !zero_ok) {
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
            | Primitive::TorusArc { .. }),
            i,
        ) if Some(i) == round_index(p) => {
            return set_round(p, node, value);
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
        | Primitive::TorusArc { round, .. } => *round = value,
        Primitive::Sphere { .. }
        | Primitive::Torus { .. }
        | Primitive::Revolve { .. }
        | Primitive::Capsule { .. }
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
        Primitive::Box { round, .. }
        | Primitive::Cylinder { round, .. }
        | Primitive::Extrude { round, .. }
        | Primitive::Cone { round, .. }
        | Primitive::Prism { round, .. }
        | Primitive::Wedge { round, .. }
        | Primitive::Star { round, .. }
        | Primitive::BoxFrame { round, .. }
        | Primitive::TorusArc { round, .. } => {
            if *round > ceiling {
                *round = ceiling.max(0.0);
                return true;
            }
            false
        }
        Primitive::Sphere { .. }
        | Primitive::Torus { .. }
        | Primitive::Revolve { .. }
        | Primitive::Capsule { .. }
        | Primitive::Ellipsoid { .. } => false,
    }
}

/// A folga entre o filete máximo e a parede, em fração da parede. Ver [`clamp_round`].
const ROUND_MARGIN: f32 = 1.0e-3;

/// ⭐ **Escala uma primitiva multiplicando as DIMENSÕES dela**, e não a pose.
///
/// # Por que uma folha não usa `Xform::scale`
///
/// ⚠️ **Uma folha escalada teria DUAS verdades sobre o mesmo tamanho visível**: a largura que o
/// painel mostra e o fator da pose. Uma caixa de 1 de largura escalada 2× mede 2 na tela e continua
/// a dizer «1» — e o artista não tem como saber qual das duas o próximo gesto vai mexer.
///
/// Multiplicar as dimensões dá **exatamente a mesma forma** (a escala uniforme é isso, aplicada ao
/// campo) com **um** número a mudar — o que o painel já mostra.
///
/// ⚠️ Um grupo é o contrário: ele **não tem dimensões próprias**, então o fator da pose é a única
/// resposta, e ali ele não compete com nada.
///
/// Devolve `false` para um fator não-positivo ou não-finito, sem tocar na forma.
pub fn scale_primitive(p: &mut Primitive, factor: f32) -> bool {
    if !factor.is_finite() || factor <= 0.0 {
        return false;
    }
    match p {
        Primitive::Box { half, round } => {
            for h in half.iter_mut() {
                *h *= factor;
            }
            *round *= factor;
        }
        Primitive::Sphere { radius } => *radius *= factor,
        Primitive::Cylinder {
            radius,
            half_height,
            round,
        } => {
            *radius *= factor;
            *half_height *= factor;
            *round *= factor;
        }
        Primitive::Torus { major, minor } => {
            *major *= factor;
            *minor *= factor;
        }
        Primitive::Extrude {
            half_height, round, ..
        } => {
            // ⚠️ O **perfil** não é escalado: ele é o desenho, e o dono dele é o editor vetorial. O
            // que esta escala mexe é a altura da extrusão e o aro — as duas grandezas que este
            // módulo autora. Escalar um perfil aqui seria reescrever, em silêncio, um documento de
            // outro módulo.
            *half_height *= factor;
            *round *= factor;
        }
        // Um torno é só o perfil: não há nada aqui que este módulo possua.
        Primitive::Revolve { .. } => return false,
        Primitive::Cone {
            bottom,
            top,
            half_height,
            round,
        } => {
            *bottom *= factor;
            // ⚠️ **O topo escala como os outros, e o zero fica zero** — é o que mantém um cone
            // fechado fechado ao redimensionar. Uma escala que somasse seria a que o abriria.
            *top *= factor;
            *half_height *= factor;
            *round *= factor;
        }
        Primitive::Capsule {
            radius,
            half_height,
        } => {
            *radius *= factor;
            *half_height *= factor;
        }
        Primitive::Prism {
            sides: _,
            bottom,
            top,
            half_height,
            round,
        } => {
            // ⚠️ **A contagem NÃO escala** — ela não é um comprimento. Multiplicá-la faria um
            // hexágono virar um dodecágono ao aumentar a peça, que é mudar a forma e não o tamanho.
            *bottom *= factor;
            *top *= factor;
            *half_height *= factor;
            *round *= factor;
        }
        Primitive::Wedge { half, round } => {
            for h in half.iter_mut() {
                *h *= factor;
            }
            *round *= factor;
        }
        Primitive::TorusArc {
            major,
            minor,
            angle: _,
            round,
        } => {
            *round *= factor;
            // ⚠️ **O ÂNGULO não escala** — ele é adimensional. Multiplicá-lo faria o arco fechar-se
            // ao aumentar a peça, que é mudar a forma e não o tamanho (a mesma lei da contagem).
            *major *= factor;
            *minor *= factor;
        }
        Primitive::Star {
            points: _,
            outer,
            inner,
            half_height,
            round,
        } => {
            // ⚠️ **A contagem de pontas NÃO escala** — a lei da contagem de lados: multiplicá-la
            // faria uma estrela de 5 virar uma de 10 ao aumentar a peça, que é mudar a forma.
            *outer *= factor;
            *inner *= factor;
            *half_height *= factor;
            *round *= factor;
        }
        Primitive::BoxFrame {
            half,
            thickness,
            round,
        } => {
            for h in half.iter_mut() {
                *h *= factor;
            }
            // ⚠️ **A espessura escala com o resto**, e é o que mantém a proporção da gaiola: uma
            // moldura ampliada com a viga fina seria outra peça, não a mesma maior.
            *thickness *= factor;
            *round *= factor;
        }
        Primitive::Ellipsoid { radii } => {
            for r in radii.iter_mut() {
                *r *= factor;
            }
        }
    }
    true
}
