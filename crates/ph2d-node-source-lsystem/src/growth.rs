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

/// ⛔ **A LEI DE 2026-08-29 (`g = log_r(r + t·(r^G − r))`) MORREU AQUI — ver o bloco abaixo.**
/// Ela supunha `tamanho = r^g`; o produto entrega a corda entre gerações inteiras, e para quem
/// cresce pela ponta ela nem corria. O `r` medido continua a existir ([`measure_ratio`]) como
/// porta de sonda da FAMÍLIA, e já não manda no arrasto.
///
/// # ⛔⛔⛔ § 2026-08-31 — o modelo `r^g` era o do REMAP, não o do PRODUTO
///
/// Report do Enio, depois da lei do recém-nascido: *"está mais suave mas não é perfeitamente
/// linear"*. **Suave e linear são coisas diferentes** — a 1.ª é a ausência de degraus, a 2.ª é a
/// derivada ser constante —, e a medição deu-lhe razão com **duas** causas distintas, uma por
/// família (`examples/probe_pops.rs --linear`):
///
/// | família | desvio da recta **na fronteira** | no MEIO da geração |
/// |---|---|---|
/// | refinam (`Koch`, `Bush`, `Weed`, `Dragon`) | **`−0,3 %`** ⇒ acerta | **`+9,5 %`** |
/// | ponta (`Tree`, `Fern`, `Wild`, `Sprig`) | **`+7,2 %` · `+10,9 %` · `+8,3 %`** (o `Wild` **`+21,2 %`**) | idem |
///
/// ⭐⭐ **Quem refina erra DENTRO da geração e acerta nas inteiras**, e a causa é a composição de
/// duas leis: este remap resolvia `g` supondo `tamanho = r^g`, e o [`crate::build`] entrega a
/// **CORDA** entre `r^k` e `r^{k+1}` (a normalização dele é linear em `frac`). Uma corda está
/// sempre ACIMA da exponencial que une os mesmos dois pontos — a `r = 3` o excesso a meio é
/// `(1 + 2)/√3 = 1,155`, **`+15,5 %`**. *Duas leis compostas, cada uma certa sozinha.*
///
/// ⭐⭐ **Quem cresce pela PONTA erra nas PRÓPRIAS gerações inteiras**, e a causa é que ninguém
/// as lineariza: a `measure_ratio` devolvia `1,0` para eles e o remap virava a rampa linear em
/// `g`, enquanto o tamanho deles é **côncavo** (a razão converge para `1`, cada geração
/// acrescenta menos). ⚠️⚠️ **E a nota que justificava essa isenção mediu a régua ERRADA:** ela
/// dizia *«o modelo exponencial piora-os — o Tree foi de `0,5×` para `0,8×` de ondulação»*, e
/// ondulação é **suavidade**. *A recusa respondeu a outra pergunta* — exactamente a distinção
/// que o report acabou de nomear.
///
/// ⇒ **A cura é UMA lei para as duas famílias: inverter o que o produto de facto ENTREGA.** O
/// tamanho em função de `g` é linear por troços entre os tamanhos das gerações inteiras, então
/// mede-se essa escada ([`size_ladder`]) e inverte-se. ⛔ Nenhum modelo: nem exponencial, nem
/// convergente, nem limiar.
///
/// **QUANTAS GERAÇÕES O `Growth` PEDE** — o `g` cujo tamanho cai na rampa recta em `t`.
///
/// ⚠️ **`t = 1` devolve `g_max` EXACTAMENTE**, e é isso que torna o param aditivo: no default
/// nada nesta casa se mexe.
///
/// ⚠️ **Uma escada que não SOBE não se pode inverter** (um `Step Scale` que encolha a figura, ou
/// uma gramática que não cresça): aí a rampa linear em `g` é a resposta, e é o que se devolve.
/// *Não é um fallback de conforto — é o caso em que a pergunta «que tamanho?» não tem resposta
/// única.*
pub(crate) fn growth_generations(g_max: f32, t: f32, ladder: &[(f32, f32)]) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t >= 1.0 || g_max <= 1.0 {
        return g_max;
    }
    let plain = 1.0 + (g_max - 1.0) * t;
    let n = ladder.len();
    if n < 2 {
        return plain;
    }
    // O tamanho em `g`, lido da escada (linear por troços, como o produto entrega).
    let at = |g: f32| -> f32 {
        if g <= ladder[0].0 {
            return ladder[0].1;
        }
        if g >= ladder[n - 1].0 {
            return ladder[n - 1].1;
        }
        let i = ladder.partition_point(|(x, _)| *x <= g).max(1);
        let (g0, s0) = ladder[i - 1];
        let (g1, s1) = ladder[i];
        if g1 > g0 {
            s0 + (s1 - s0) * (g - g0) / (g1 - g0)
        } else {
            s0
        }
    };
    let (s0, s1) = (ladder[0].1, at(g_max));
    if s1 <= s0 || !s1.is_finite() {
        return plain;
    }
    let alvo = s0 + t * (s1 - s0);
    // A escada é estritamente crescente por construção ([`size_ladder`] filtra), logo o degrau
    // que contém `alvo` é único.
    let i = ladder
        .partition_point(|(_, s)| *s < alvo)
        .clamp(1, n.saturating_sub(1));
    let (g0, a) = ladder[i - 1];
    let (g1, b) = ladder[i];
    let frac = if b > a { (alvo - a) / (b - a) } else { 0.0 };
    (g0 + (g1 - g0) * frac).clamp(1.0, g_max)
}

/// ⭐⭐⭐ **A ESCADA — quanto a figura MEDE em cada geração inteira, `1..=gens`.**
///
/// É o observável que o remap inverte, e ele é **medido**, nunca modelado: a razão de uma
/// gramática de refinamento é constante e a de uma da ponta converge, e um modelo que sirva as
/// duas não existe. ⚠️ A medição custa **uma travessia por geração** sobre uma derivação só —
/// ver [`derive::derive_recording`], que é a MESMA derivação que o cook usa.
/// ⭐⭐ **Sub-degraus por geração para quem cresce pela PONTA.**
///
/// Quem REFINA tem o tamanho **normalizado** pelo [`crate::build`] para cair na corda entre as
/// duas gerações inteiras ⇒ um degrau por geração descreve a curva **exactamente** (medido:
/// `±0,01 %` no Bush/Weed/Koch). Quem cresce pela ponta não tem normalização nenhuma, e a curva
/// dele curva-se dentro da geração.
///
/// ⚠️ **`3` é MEDIDO, e é o último valor que serve** (bancada `probe_pops.rs --ideal`, tecto por
/// força bruta do desvio da recta):
///
/// | `M` | `Tree` | `Fern` | `Wild` | `Sprig` |
/// |---|---|---|---|---|
/// | 1 | `−0,22 %` | `−0,07 %` | `−0,44 %` | **`−11,51 %`** |
/// | 2 | `−0,11 %` | `−0,03 %` | `−0,14 %` | `−3,33 %` |
/// | **3** | `−0,07 %` | `−0,02 %` | `−0,06 %` | **`−1,40 %`** |
/// | 4 | `−0,05 %` | `−0,01 %` | `−0,07 %` | ⛔ **patamar: não invertível** |
///
/// ⛔ Acima de `3` o `Sprig` mostra um **patamar** — o tamanho dele fica exactamente parado no
/// primeiro quarto de cada geração, porque a ponta mais alta da planta é um galho **lateral** e
/// o rebento novo tem de o ultrapassar antes de a silhueta crescer. Um intervalo plano não tem
/// inversa única.
const TIP_SUBSTEPS: u16 = 3;

pub(crate) fn size_ladder(
    axiom_src: &str,
    rules_src: &str,
    p: &Params,
    gens: u16,
) -> Vec<(f32, f32)> {
    let params = |n: &str| p.by_name(n);
    let axiom = derive::axiom_modules(axiom_src, &params);
    let rules = grammar::parse_rules(rules_src);
    let mut out: Vec<(f32, f32)> = Vec::with_capacity(gens as usize);
    // ⚠️ `(step, f, g)`: a cadeia é a da geração `step`, com o material dela a `f`, o que o
    // produto entrega em `g = step − 1 + f`. Os três campos são EXACTAMENTE os que o braço da
    // ponta do [`crate::build`] passa (`len_frac = ang_frac = frac`, `lat = 1`) — medir com
    // outros devolveria a escada de uma figura que ninguém desenha.
    let set = |step: u16, f: f32, g: f32| turtle::Setup {
        angle: p.angle,
        // ⚠️ **O `step_scale` entra AQUI**, com o expoente FRACCIONÁRIO: sem ele a escada
        // mediria uma figura que o produto nunca desenha — é o mesmo `powf` do
        // [`crate::build`], no mesmo sítio da fórmula. (Numa gramática PARAMÉTRICA o
        // comprimento viaja no módulo e este campo nem é lido; ali o factor é inerte.)
        step: p.step * p.step_scale.max(1e-4).powf(g),
        width: 1.0,
        width_scale: p.width_scale,
        length_scale: p.length_scale,
        root_angle: p.root_angle,
        tropism: p.tropism,
        tropism_angle: p.tropism_angle,
        angle_frac: f,
        youngest: (step, f),
        newborn: f,
        orient_world: true,
        leaf_first_level: p.leaf_first_level,
        leaf_angle: p.leaf_angle,
        leaf_spread: p.leaf_spread,
        leaf_effects: p.leaf_effects.round() as i32 != 0,
        seed: p.seed,
    };
    derive::derive_recording(
        &axiom,
        &rules,
        gens,
        // ⛔⛔ **A SEMENTE DO ARTISTA, não `1`.** Numa gramática ESTOCÁSTICA a semente escolhe a
        // planta, e uma escada derivada com outra semente mede **outra planta** — o remap
        // colocaria o arrasto na recta de uma figura que ninguém vê. Medido no `Wild` (o único
        // molde com pesos): com `1` fixo o desvio ficava em `−7,69 %`; com a semente certa,
        // `−0,29 %`. *As gramáticas determinísticas são byte-idênticas a qualquer semente, e é
        // por isso que este defeito só tinha UM sujeito no corpus.*
        crate::seed_bits(p.seed),
        MAX_MODULES,
        &params,
        &mut |chain, previous, step| {
            // ⭐ A densidade sai da FAMÍLIA, medida na própria cadeia — ver [`TIP_SUBSTEPS`].
            let refines = derive::refines_at(chain, step);
            // ⛔⛔ **E do `Step Scale`, que é a segunda razão para sub-degraus.** Quem refina
            // tem o tamanho normalizado para a CORDA entre as duas gerações inteiras, e no
            // neutro (`1,0`, o default e o valor dos oito moldes) um degrau por geração
            // descreve-a **exactamente** (medido: `±0,02 %`). Fora do neutro deixa de a
            // descrever: o [`crate::build`] mede as DUAS pontas no factor `ss^g` do instante
            // ACTUAL, enquanto a escada mediria cada geração no factor DELA. Medido na `Koch`
            // com `Step Scale = 0,5`: desvio `0,3737` com a lei de 2026-08-29, `0,1540` com a
            // escada de um degrau, **`0,0244`** com esta — `15×` melhor, e ⏳ ainda `2,4 %`, que
            // fica NOMEADO (o neutro dá `≤1,35 %`). ⚠️ *O preço é pago só por quem mexe no
            // botão*: no neutro `m` é `1` e o caminho é o de sempre, ao bit.
            let neutral_scale = (p.step_scale - 1.0).abs() < 1e-6;
            let m = if refines && neutral_scale {
                1
            } else {
                TIP_SUBSTEPS
            };
            for j in 1..=m {
                let f = f32::from(j) / f32::from(m);
                let g = f32::from(step) - 1.0 + f;
                // A escada começa em `g = 1`: abaixo disso o arrasto não vai.
                if g < 1.0 - 1e-6 {
                    continue;
                }
                let size = if refines && m > 1 {
                    // A MESMA conta do `build`: as duas pontas no factor do instante, e a
                    // corda entre elas. ⚠️ Duas travessias por sub-degrau, e é por isso que ela
                    // só corre fora do neutro.
                    let antes = width::mean_width(previous, &set(step, 1.0, g));
                    let cheia = width::mean_width(chain, &set(step, 1.0, g));
                    antes + (cheia - antes) * f
                } else {
                    width::mean_width(chain, &set(step, f, g))
                };
                out.push((g, size));
            }
        },
    );
    // ⭐⭐ **A REDE DO PATAMAR: um degrau que não SOBE é descartado.** Onde a curva é plana a
    // escada fica localmente mais grossa — que é exactamente o que evita o salto de tinta —, e
    // onde ela sobe fica com a densidade toda. ⛔ Sem isto, uma gramática com um patamar mais
    // largo que `1/M` tornaria a inversão ambígua e a lei inteira cairia para a rampa linear.
    let mut keep: Vec<(f32, f32)> = Vec::with_capacity(out.len());
    for (g, s) in out {
        if s.is_finite() && s > 0.0 && keep.last().is_none_or(|(_, prev)| s > prev * 1.000_01) {
            keep.push((g, s));
        }
    }
    keep
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
        let w = width::mean_width(
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
                // Geração INTEIRA nas duas amostras (`1.0` em toda a parte): a razão de
                // expansão é entre gerações fechadas, e a lei do recém-nascido não a toca.
                newborn: 1.0,
                orient_world: true,
                leaf_first_level: p.leaf_first_level,
                leaf_angle: p.leaf_angle,
                leaf_spread: p.leaf_spread,
                leaf_effects: p.leaf_effects.round() as i32 != 0,
                seed: p.seed,
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
