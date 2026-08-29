//! **A LEI DO CRESCIMENTO** — a remapagem do `Growth` e a razão de expansão que a ancora.
//!
//! Separado do [`super`] pelo teto de LOC (HR-18, 700 na workspace), no corte que a
//! pergunta desenha: lá fica *o que o nó desenha*, aqui *quanto dele já cresceu*.

use super::*;

/// ⭐⭐⭐ **QUANTAS GERAÇÕES O `Growth` PEDE** — a remapagem que faz o arrasto crescer por
/// igual, e o report do Enio de 2026-08-29 (*"ainda não linear"*).
///
/// # O mecanismo
///
/// Uma gramática de refinamento multiplica a figura por uma razão `r` a cada geração — medido,
/// **exactamente `3,00`** no Bush e na Koch, em toda geração. Logo `tamanho ≈ r^g`, e um slider
/// linear em `g` dá um crescimento **exponencial**: medido no arrasto inteiro, o Bush anda
/// `+0,017` no início e `+0,157` no fim (`9×`), e o Dragon `+0,007` contra `+0,335` (`48×`).
/// *Eu tinha medido uma travessia; ele arrasta o slider todo.*
///
/// ⇒ Para o TAMANHO ser linear em `t`, resolve-se `r^g = r + t·(r^G − r)`, ou seja
/// `g = log_r(r + t·(r^G − r))`. É isto.
///
/// ⚠️ **`t = 1` devolve `G` EXACTAMENTE**, e é isso que torna o param aditivo: no default nada
/// nesta casa se mexe. ⚠️ E com `r ≈ 1` (as que crescem pela PONTA, cuja razão converge para
/// `1,06`) o logaritmo degenera — aí a rampa linear já é a certa, e é ela que se usa.
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

/// **A razão de expansão por geração, MEDIDA** — duas derivações baratas.
///
/// ⚠️ Ela é medida em `g` BAIXO de propósito: para uma gramática auto-semelhante a razão é
/// constante (`3,00` em toda geração, medido), e derivar no topo custaria o orçamento inteiro
/// só para saber um número que já se sabe a `4`.
pub(crate) fn measure_ratio(axiom_src: &str, rules_src: &str, p: &Params) -> f32 {
    let params = |n: &str| p.by_name(n);
    let axiom = derive::axiom_modules(axiom_src, &params);
    let rules = grammar::parse_rules(rules_src);
    let span_at = |g: u16| {
        let d = derive::derive(&axiom, &rules, g, 1, MAX_MODULES, &params);
        turtle::span(
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
        )
    };
    // ⚠️⚠️ **O modelo exponencial só vale para uma gramática que ainda MULTIPLICA.** As que
    // crescem pela PONTA têm uma razão que converge para `1` (`1,63 → 1,30 → 1,15 → 1,06`), e
    // aplicar-lhes o logaritmo PIORA-AS — medido: o Tree foi de `0,5×` para `0,8×` e o Wild de
    // `1,8×` para `2,2×` quando o limiar era `1,05` e as apanhava.
    //
    // ⚠️⚠️ **E a MÉDIA GEOMÉTRICA sobre duas gerações, não uma razão só**: a razão do Dragon
    // OSCILA (`2,00 · 1,50 · 1,67 · 1,40 · 1,57`) porque a curva alterna de orientação, e uma
    // amostra única fazia um discriminador de «a razão encolheu?» classificá-lo como
    // convergente — exactamente a peça que ele precisa. *Uma série que oscila não se lê em
    // dois pontos.*
    //
    // ⇒ o limiar tem um significado, e não é um número escolhido: *a figura ainda multiplica
    // por pelo menos `1,25` por geração nesta profundidade?* Medido, ele parte o corpus em
    // duas: `3,00` (Bush, Koch) · `2,03` (Weed) · `1,48` (Dragon) de um lado; `1,15` (Fern) ·
    // `1,10` (Sprig) · `1,08` (Tree) do outro.
    const STILL_MULTIPLIES: f32 = 1.25;
    let (a, b, c) = (span_at(4), span_at(5), span_at(6));
    if !(a > 1e-6 && b > 1e-6 && c > 1e-6) {
        return 1.0;
    }
    let r = (c / a).sqrt();
    if r >= STILL_MULTIPLIES { r } else { 1.0 }
}
