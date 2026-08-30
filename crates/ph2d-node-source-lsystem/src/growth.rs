//! **A LEI DO CRESCIMENTO** — a remapagem do `Growth` e a razão de expansão que a ancora.
//!
//! Separado do [`super`] pelo teto de LOC (HR-18, 700 na workspace), no corte que a
//! pergunta desenha: lá fica *o que o nó desenha*, aqui *quanto dele já cresceu*.

//! # § 2026-08-30 — a régua mudou, e o que ela mede é toda a diferença
//!
//! Report do Enio: *"em dragon enquanto cresce (aumentando Generations) parece piscar"*.
//!
//! A lei desta casa põe o TAMANHO da figura numa rampa recta. Ela estava certa; a **grandeza**
//! é que não. Até 30/08 o tamanho era `max(w, h)` da caixa alinhada aos eixos, que **não é
//! invariante à rotação** — e a curva do dragão roda `45°` por geração por construção. Quando a
//! caixa trocava de lado longo, a normalização passava a fixar a OUTRA dimensão e o tamanho
//! verdadeiro **estagnava e depois arrancava**: menor passo do arrasto a `4,5 %` do passo
//! médio, contra `55,2 %` hoje.
//!
//! ⚠️ **O defeito não era da lei nem dos moldes**, e é por isso que a cura é uma linha noutro
//! ficheiro ([`turtle::mean_width`], que traz as duas réguas invariantes **rejeitadas por
//! medição** e a hipótese do deslocamento, também refutada). Os quatro moldes que crescem pela
//! PONTA não passam por aqui e saíram **byte-idênticos**.
//!
//! ⛔ **Os quatro que REFINAM mudaram TODOS de bytes** — a 1.ª redacção desta nota dizia *"só os
//! dois que RODAM se mexeram"* e a auditoria refutou-a: o que só os dois que rodam mudaram foi
//! de forma VISÍVEL. Medido em geração fraccionária: Bush `−0,006 %`, Koch `+0,22 %`, Weed
//! `−0,51 %`, **Dragon `+7,9 %`**. *Escrever "byte-idênticos" numa metade da frase faz a outra
//! metade ler-se como bytes, e como bytes é `4/4`.*
//!
//! ⚠️ Os números do parágrafo do [`crate::build`] que descrevem o report ANTERIOR (`3,05`/`3,00`,
//! `1,62`/`1,51`) foram medidos com a régua velha e descrevem outra grandeza — a conclusão que
//! eles suportam (um slider linear em `g` dá tamanho `r^g`, logo crescimento exponencial) é
//! identidade algébrica e não depende de régua nenhuma.

use super::*;

/// ⭐⭐⭐ **QUANTAS GERAÇÕES O `Growth` PEDE** — a remapagem que faz o arrasto crescer por
/// igual, e o report do Enio de 2026-08-29 (*"ainda não linear"*).
///
/// # O mecanismo
///
/// Uma gramática de refinamento multiplica a figura por uma razão `r` a cada geração — medido,
/// **`3,000` no Bush e `3,003` na Koch** (⛔ a 1.ª redacção dizia *«exactamente 3,00»*, com a
/// régua antiga — ver `crate::build`), em toda geração. Logo `tamanho ≈ r^g`, e um slider
/// linear em `g` dá um crescimento **exponencial**: medido no arrasto inteiro, o Bush anda
/// `+0,017` no início e `+0,157` no fim (`9×`), e o Dragon `+0,007` contra `+0,335` (`48×`).
/// ⚠️ **Esses quatro números são da régua de antes de 2026-08-30 e não foram re-medidos** — a
/// conclusão que eles suportam (`tamanho = r^g` ⇒ um slider linear em `g` cresce
/// exponencialmente) é identidade algébrica e não depende de régua nenhuma.
/// *Eu tinha medido uma travessia; ele arrasta o slider todo.*
///
/// ⇒ Para o TAMANHO ser linear em `t`, resolve-se `r^g = r + t·(r^G − r)`, ou seja
/// `g = log_r(r + t·(r^G − r))`. É isto.
///
/// ⚠️ **`t = 1` devolve `G` EXACTAMENTE**, e é isso que torna o param aditivo: no default nada
/// nesta casa se mexe. ⚠️ E com `r ≈ 1` (as que crescem pela PONTA, cuja razão medida está
/// hoje entre **`1,053`** e **`1,154`** — ⛔ não o `1,06` que aqui estava, que descrevia uma
/// delas) o logaritmo degenera — aí a rampa linear já é a certa, e é ela que se usa.
///
/// ⚠️ O `r` é MEDIDO por quem chama (duas derivações baratas), nunca contado da gramática:
/// ver [`turtle::walk`] para as duas vezes em que contá-lo deu `5` onde a resposta era `3`.
pub(crate) fn growth_generations(g_max: f32, t: f32, r: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t >= 1.0 || g_max <= 1.0 {
        return g_max;
    }
    // ⚠️ A barra `1,05` é o joelho: abaixo dela `log(r)` fica pequeno o bastante para o
    // quociente amplificar o ruído de `f32`, e a rampa linear já é indistinguível da certa.
    if !(r.is_finite() && r > 1.05) {
        return 1.0 + (g_max - 1.0) * t;
    }
    let ln_r = r.ln();
    let alvo = r + t * ((g_max * ln_r).exp() - r);
    (alvo.max(r).ln() / ln_r).clamp(1.0, g_max)
}

/// **A razão de expansão por geração, MEDIDA — e `1,0` quando ela não se aplica.**
///
/// ⚠️ Ela é medida em `g` BAIXO de propósito: para uma gramática auto-semelhante a razão é
/// constante em toda geração, e derivar no topo custaria o orçamento inteiro só para saber um
/// número que já se sabe a `4`.
///
/// ⚠️⚠️ **O modelo exponencial só vale para uma gramática que REFINA.** As que crescem pela
/// PONTA têm uma razão que converge para `1` e aplicar-lhes o logaritmo **piora-as** — medido:
/// o Tree foi de `0,5×` para `0,8×` e o Wild de `1,8×` para `2,2×` de ondulação; e o modo
/// GUIADO dá desvio `2,1 %` tratado como ponta contra `8,8 %` tratado como refinador.
///
/// ⚠️⚠️ **E a MÉDIA GEOMÉTRICA sobre duas gerações, não uma razão só**: a razão do Dragon
/// OSCILA porque a curva alterna de orientação, e uma amostra única fazia um discriminador de
/// «a razão encolheu?» classificá-lo como convergente — exactamente a peça que precisa da lei.
/// *Uma série que oscila não se lê em dois pontos.*
///
/// ⛔⛔ **A FAMÍLIA JÁ NÃO SE DECIDE AQUI.** Até 2026-08-30 isto tinha um limiar
/// (`razão >= 1,25`); ele esgotou-se quando a régua passou a ser invariante à rotação, e a
/// decisão mudou-se para a ESTRUTURA — [`derive::Derived::grows_by_refining`], que é a mesma
/// porta que a lei do comprimento usa. Toda a história, com os números, está lá.
///
/// ⚠️ **O `1,0` continua a significar «não remapear»**, que é o contrato que o
/// [`growth_generations`] e as sondas leem.
pub(crate) fn measure_ratio(axiom_src: &str, rules_src: &str, p: &Params) -> f32 {
    let (r, refines) = raw_ratio_and_family(axiom_src, rules_src, p);
    if refines { r } else { 1.0 }
}

/// **A razão CRUA, sem a decisão de família** — a porta pela qual um gate consegue perguntar
/// *quanto* em vez de só *de que lado*.
pub(crate) fn raw_ratio(axiom_src: &str, rules_src: &str, p: &Params) -> f32 {
    raw_ratio_and_family(axiom_src, rules_src, p).0
}

/// A razão medida **e** a família, das mesmas derivações — `(razão, refina?)`.
pub(crate) fn raw_ratio_and_family(axiom_src: &str, rules_src: &str, p: &Params) -> (f32, bool) {
    let params = |n: &str| p.by_name(n);
    let axiom = derive::axiom_modules(axiom_src, &params);
    let rules = grammar::parse_rules(rules_src);
    let at = |g: u16| {
        let d = derive::derive(&axiom, &rules, g, 1, MAX_MODULES, &params);
        let w = turtle::mean_width(
            &d.chain,
            &turtle::Setup {
                angle: p.angle,
                step: 1.0,
                width: 1.0,
                width_scale: p.width_scale,
                length_scale: p.length_scale,
                root_angle: p.root_angle,
                tropism: p.tropism,
                tropism_angle: p.tropism_angle,
                angle_frac: 1.0,
                youngest: (d.generations, 1.0),
                orient_world: true,
            },
        );
        (w, d)
    };
    // ⚠️ **A geração 5 do meio SAIU** (auditoria de 2026-08-30): o valor dela CANCELAVA na
    // média geométrica `√(c/a)` e só alimentava uma guarda — uma derivação e uma travessia
    // inteiras por um número que ninguém lia, e a régua nova tornou-as mais caras.
    let (a, _) = at(4);
    let (c, six) = at(6);
    // ⭐ A família sai da MESMA derivação, e pela porta que o [`crate::build`] usa.
    let refines = six.grows_by_refining();
    if !(a > 1e-6 && c > 1e-6) {
        return (1.0, refines);
    }
    ((c / a).sqrt(), refines)
}
