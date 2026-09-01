//! ⭐⭐⭐ **O QUE UM MODIFICADOR FAZ AO BORDO** — a lei de cada um, no eixo canónico dele.
//!
//! # Por que um módulo irmão
//!
//! O [`crate::bounds`] responde a duas perguntas: *que forma tem uma bola de bordo* (o tipo, os
//! construtores, a caixa) e *o que a ÁRVORE lhe faz* (a dobra dos filhos, a pose). Estas leis são a
//! terceira — uma por modificador, cada uma com a medição que a escolheu ao lado —, e com elas o
//! ficheiro passava dos **700** do gate de LOC da workspace.
//! ⚠️ **A cura é partir para irmão, nunca uma entrada na allowlist.**

use crate::bounds::Ball;
use ph2d_field::Unary;

/// ⭐⭐⭐ **O que UM modificador faz ao bordo** — a lei, num sítio só.
///
/// ⚠️ **Ela saiu do `with_mods` porque ganhou um SEGUNDO leitor** (2026-08-30): a pilha de
/// modificadores ([`crate::stacked`]) precisa de saber, a cada passo, quão longe do eixo a peça vai
/// — é disso que a torção tira o divisor que a mantém uma distância honesta. *Uma lei com dois
/// leitores é uma porta; escrita duas vezes, são duas respostas que começam a divergir no dia em que
/// alguém acrescentar um modificador.*
#[must_use]
pub fn step_mod(b: Ball, m: Unary) -> Ball {
    // ⭐⭐⭐ **A LEI ESTÁ ESCRITA NO EIXO CANÓNICO; O EIXO ESCOLHIDO ENTRA POR FORA** (Enio,
    // 2026-08-31) — a mesma conjugação que a `stack::conjugado` aplica à árvore, e **a mesma
    // permutação**. Escrever a lei uma vez por eixo daria três sítios onde um índice errado
    // devolve uma caixa que **corta a peça** em silêncio, que é o modo de falha que o topo deste
    // ficheiro diz que nunca pode acontecer.
    let s = axis_shift_of(m);
    if s == 0 {
        return canonical_step(b, m);
    }
    canonical_step(b.to_canonical(s), m).from_canonical(s)
}

/// De quantos passos a lei de `m` tem de rodar para o eixo escolhido cair no canónico dela.
///
/// ⚠️ **`0` para quem não tem eixo** — a casca e o afastamento são isotrópicos, e o espelho tem os
/// três eixos como três variantes (ver [`ph2d_field::FIELD_DOC_VERSION`], v16).
pub(crate) fn axis_shift_of(m: Unary) -> usize {
    use ph2d_field::mods::{ARRAY_AXIS, BEND_AXIS, RADIAL_AXIS, TAPER_AXIS, TWIST_AXIS};
    match m {
        Unary::Array { axis, .. } => axis.shift_to(ARRAY_AXIS),
        Unary::Taper { axis, .. } => axis.shift_to(TAPER_AXIS),
        Unary::Radial { axis, .. } => axis.shift_to(RADIAL_AXIS),
        Unary::Twist { axis, .. } => axis.shift_to(TWIST_AXIS),
        Unary::Bend { axis, .. } => axis.shift_to(BEND_AXIS),
        Unary::Shell { .. }
        | Unary::Offset { .. }
        | Unary::Mirror
        | Unary::MirrorY
        | Unary::MirrorZ => 0,
    }
}

/// ⭐⭐⭐ **Quão longe de um EIXO a bola chega** — a grandeza que a torção varre, que a repetição
/// radial varre e de que a inclinação tira o factor de secção. Uma lei, três leitores.
///
/// ⛔ **Cada um deles escrevia `‖(c_a, c_b)‖ + raio`, e o raio não é uma extensão de eixo**
/// (2026-09-01). Numa peça achatada no plano que interessa a esfera responde pela **espessura no
/// terceiro eixo**, que não participa: uma barra `0,05 × 0,05 × 0,80` centrada dizia `0,80` de braço
/// e o braço verdadeiro é `0,07` — **11×**, e o erro ia direito ao divisor da torção
/// (`σ = t/2 + √(1 + t²/4)` com `t = κ·R`), que é multiplicativo.
///
/// ⚠️ **As DUAS respostas são majorantes, e fica-se com a menor**: a da caixa ganha numa peça
/// achatada, a da esfera ganha numa peça longe do eixo (ali a caixa paga `√2` pelo canto).
#[must_use]
pub(crate) fn axis_distance(b: Ball, eixo: usize) -> f32 {
    let h = b.half();
    let (u, v) = ((eixo + 1) % 3, (eixo + 2) % 3);
    let pela_caixa = (b.center[u].abs() + h[u]).hypot(b.center[v].abs() + h[v]);
    let pela_esfera = b.center[u].hypot(b.center[v]) + b.radius.max(0.0);
    pela_caixa.min(pela_esfera)
}

/// A lei de cada modificador, **no eixo canónico dele** — ver [`step_mod`], que é a porta.
fn canonical_step(b: Ball, m: Unary) -> Ball {
    match m {
        // A parede é centrada na superfície: metade cresce para fora.
        Unary::Shell { thickness } => b.expanded_by(thickness.abs() * 0.5),
        Unary::Offset { distance } => b.expanded_by(distance.max(0.0)),
        // O espelho é num plano do eixo LOCAL: a cópia está com aquela coordenada trocada de
        // sinal. ⚠️ **Uma função, três eixos** — três braços com a conta escrita à mão seriam
        // três sítios onde um índice errado dá uma caixa que **corta a peça** em silêncio.
        Unary::Mirror | Unary::MirrorY | Unary::MirrorZ => {
            let k = match m {
                Unary::Mirror => 0,
                Unary::MirrorY => 1,
                _ => 2,
            };
            let mut c = b.center;
            c[k] = -c[k];
            // ⚠️ A cópia tem a **mesma** caixa — só o centro é que reflecte —, e o `merge` sabe
            // fundir as duas.
            b.merge(Ball::of(c, b.radius, b.half()))
        }
        // A matriz linear anda ao longo do X local.
        Unary::Array {
            count,
            spacing,
            joint,
            ..
        } => {
            let span = f32::from(u16::try_from(count.saturating_sub(1)).unwrap_or(u16::MAX))
                * spacing.abs();
            // ⭐ **A junta ACRESCENTA material no vinco** — um bordo que não a conte recorta a
            // peça na marcha e na exportação, que é o defeito que a inclinação já custou a esta
            // linha em 2026-08-30. Ver [`ph2d_field::Joint::reach`].
            //
            // ⭐⭐ **E a caixa cresce SÓ no eixo em que a matriz anda** — é a primeira lei deste
            // ficheiro que a esfera não sabia exprimir.
            let h = b.half();
            let j = joint.reach();
            Ball::of(
                [b.center[0] + span * 0.5, b.center[1], b.center[2]],
                b.radius + span * 0.5 + j,
                [h[0] + span * 0.5 + j, h[1] + j, h[2] + j],
            )
        }
        // ⭐ **A torção varre em torno do Z local, e a bola dela é a MESMA da matriz radial** — cada
        // fatia de `z` é uma rotação em torno da origem, logo `‖(x,y)‖` e `z` são preservados. Uma
        // bola já centrada no eixo fica **inalterada ao bit**; uma descentrada varre o círculo que o
        // centro descreve. *Não se escreve lei nova: aponta-se para a que existe.*
        Unary::Radial { joint, .. } => {
            // ⭐ Pela razão da matriz acima — a costura entre as cópias enche o vinco.
            //
            // ⭐⭐ **O varrimento é no plano XY; o Z não se mexe** — e é isso que a caixa diz e a
            // esfera não dizia.
            let h = b.half();
            let j = joint.reach();
            let raio_xy = axis_distance(b, 2) + j;
            let meia_z = h[2] + j;
            Ball::of(
                [0.0, 0.0, b.center[2]],
                raio_xy.hypot(meia_z),
                [raio_xy, raio_xy, meia_z],
            )
        }
        Unary::Twist { .. } => {
            // ⭐⭐ Como a radial: o giro é no plano XY e o Z é preservado **ao bit**.
            let h = b.half();
            let raio_xy = axis_distance(b, 2);
            Ball::of(
                [0.0, 0.0, b.center[2]],
                raio_xy.hypot(h[2]),
                [raio_xy, raio_xy, h[2]],
            )
        }
        // A secção cresce `slope` por unidade de altura, e a altura é no máximo o próprio raio.
        // ⛔⛔ **ELA IGNORAVA O CENTRO DA BOLA, e a peça saía da caixa do mundo** (auditoria de
        // 2026-08-30, defeito **pré-existente** desde a W18).
        //
        // A secção escala por `k(y) = 1 + s·y` com o `y` **absoluto**, e a conta antiga (`r·(1+|s|r)`)
        // só está certa para uma bola centrada na origem. Medido: uma caixa `half = 0,2` em `x = 3`
        // com `Taper 1,0` dava bordo até `x = 3,4664`, e a peça chega a **`3,8400`** — *«um bordo
        // menor CORTA a peça e não diz nada»*, que é o modo de falha que o topo deste ficheiro diz
        // que nunca pode acontecer.
        //
        // ⚠️ E desde a torção isto ficou pior do que um corte na exportação: o `axis_reach` bebe
        // daqui, logo um `R` pequeno de menos dá um divisor pequeno de menos e o campo **fura**.
        //
        // A lei: o maior factor na bola é `k_max = 1 + |s|·(|c_y| + r)`; o raio cresce por ele e o
        // centro **afasta-se** por `(k_max − 1)` vezes a distância dele ao eixo `Y`. Conservadora nos
        // dois termos, que é a assimetria declarada deste ficheiro.
        Unary::Taper { slope, .. } => {
            if slope == 0.0 || !slope.is_finite() {
                return b;
            }
            let s = slope.abs();
            // ⭐ **A altura é a do EIXO Y, e não o raio** (2026-09-01) — `k(y) = 1 + slope·y` só
            // pergunta pelo `y`, e a esfera responde pelo maior dos três eixos. Numa chapa larga e
            // baixa isso inflava o factor de secção inteiro.
            let k_max = s.mul_add(b.center[1].abs() + b.half()[1], 1.0);
            let fora_do_eixo = b.center[0].hypot(b.center[2]);
            // ⭐⭐ **A secção escala em X e Z; o Y não se mexe** — a inclinação é o exemplo mais
            // claro de uma lei que a esfera obrigava a arredondar para cima nos três eixos.
            let h = b.half();
            let cresce = |e: usize| (k_max - 1.0).mul_add(fora_do_eixo, h[e] * k_max);
            Ball::of(
                b.center,
                (k_max - 1.0).mul_add(fora_do_eixo, b.radius * k_max),
                [cresce(0), h[1], cresce(2)],
            )
        }
        // ⭐⭐ **A DOBRA move a peça, e é o único modificador que o faz de forma não-linear.**
        //
        // ⚠️ **A conta INGÉNUA — «cabe numa bola centrada no centro do arco, de raio `ρ + R`» —
        // EXPLODE quando `κ → 0`**: com `κ = 0,001` ela dá `1000`. E o doc do topo deste ficheiro
        // diz o que isso custa: *«um bordo maior do que a peça custa RESOLUÇÃO»* — a grade da malha
        // passaria a gastar tudo em vazio.
        //
        // ⇒ fica-se com a **MENOR** de duas cercas, e cada uma é boa num regime:
        //  · a do centro do arco, apertada com `κ` grande;
        //  · a da própria peça mais a **corda** que o arco descreve, apertada com `κ` pequeno — e
        //    ela tende para `R` quando `κ → 0`, que é a identidade.
        //
        // ⚠️ **Conservadora de propósito nos dois casos** (a assimetria do ficheiro: um bordo a mais
        // custa resolução, um bordo a menos **corta a peça e não diz nada**).
        Unary::Bend {
            turns,
            lower,
            upper,
            falloff,
            ..
        } => {
            // ⛔⛔ **A CURVATURA EFECTIVA, e não a pedida** (2026-09-01). A árvore dobra por
            // `bend_curvature`, que **satura** contra a parede da peça — e uma caixa calculada
            // sobre a curvatura crua descreve um arco que ninguém desenhou. *Um bordo tem de
            // responder pela geometria que a árvore produz, não pela que o slider pede.*
            let k = crate::stack_bend::bend_curvature(turns, b);
            if k == 0.0 || !k.is_finite() {
                return b;
            }
            let rho = (1.0 / k).abs();
            let h = b.half();
            let alcance = f64::from(b.center[0].abs() + h[0]);
            // ⭐⭐⭐ **A meia-altura é a do EIXO DOBRADO, e não o raio** (2026-08-31). Ela entra num
            // `sin` multiplicado por `2(ρ + alcance)`, então o exagero da esfera compõe-se: numa
            // caixa `0,35 × 0,35 × 0,30` o raio é `0,579` contra os `0,30` do eixo — **1,93×** — e a
            // bola saía **3,6× maior do que a peça**. Isso ia direito ao `piso` da dobra e o divisor
            // dela ficava preso no tecto (`10`).
            //
            // ⭐ Medido: com a extensão axial, `[Bend]` passa de `72,2` para `27,0` passos por raio,
            // `[Bend, Twist]` de `233,1` para `68,4` e `[Bend, Twist, Taper]` de `1 543,6` para
            // `717,3` — e as **cinco** imagens contra a marcha honesta ficam idênticas.
            let meia_altura = f64::from(b.center[2].abs() + h[2]);
            // A corda máxima que um ponto descreve ao varrer o ângulo da peça inteira.
            let corda = 2.0 * (rho + alcance) * (k.abs() * meia_altura * 0.5).sin().abs();
            let pela_corda = f64::from(b.radius) + corda;
            let pelo_centro = rho + alcance;
            #[allow(clippy::cast_possible_truncation)]
            let raio = (pela_corda.min(pelo_centro) as f32).max(b.radius);
            // ⚠️ O centro fica onde estava: a bola é grande o suficiente para conter o arco, e
            // mover o centro exigiria a imagem dele — que é a conta que se está a evitar.
            //
            // ⭐⭐⭐ **E a CAIXA é a do arco, não o cubo do raio** (2026-09-01) — ver
            // [`crate::bounds_bend`]. Ela era `[raio, h₁, raio]`, e numa caixa `0,35³` a `0,12`
            // voltas isso é `0,957` contra os `0,376` que a peça dobrada de facto ocupa: **`2,5×`
            // em dois eixos**, herdados por toda a pilha a jusante.
            //
            // ⚠️ **A lei antiga fica como tecto**: a nova nunca é aceite se for mais folgada, e
            // quando o arco é pequeno demais para ser distinguível da identidade ela devolve a
            // pergunta. *Uma cerca nova entra por `min` com a que já defendia o sítio.*
            let largo = |e: usize| if e == 1 { h[1] } else { raio };
            let meia = crate::bounds_bend::bent_extent(
                b.center,
                h,
                k,
                f64::from(lower),
                f64::from(upper),
                f64::from(falloff),
            )
            .map_or([largo(0), largo(1), largo(2)], |(lo, hi)| {
                std::array::from_fn(|e| {
                    let c = f64::from(b.center[e]);
                    #[allow(clippy::cast_possible_truncation)]
                    let m = (hi[e] - c).abs().max((lo[e] - c).abs()) as f32;
                    m.min(largo(e))
                })
            });
            Ball::of(b.center, raio, meia)
        }
    }
}
