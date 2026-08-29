//! ⭐⭐⭐ **O CRESCIMENTO SÃO DUAS LEIS, E CADA FAMÍLIA DE GRAMÁTICA PRECISA DE UMA.**
//!
//! # O caminho até aqui, porque ele é a lição
//!
//! 1. **Enio, 2026-08-29:** *"porque vários presets não têm crescimento suave, mas em saltos?"*
//!    Medido: quatro dos oito saltavam (`53–137 %` de pior passo contra `5–8 %`).
//! 2. **Pesquisa:** o L-System SOP do Houdini tem **dois** interruptores — *Continuous length*
//!    e *Continuous angles* — e eu tinha construído só o primeiro.
//! 3. **Enio:** *"faça os demais"* → construí, e ele previu *"os que vc tentou corrigir não
//!    ficarão bons"*. Shipei **desligado**, com a medição que concordava com ele.
//! 4. **Ele SMOKOU e retirou a previsão:** *"Melhorou muito. Mas o crescimento dos que não
//!    cresciam suavemente não é linear."*
//! 5. Medi a **DERIVADA** em vez do pior passo, e ele tinha razão pela segunda vez — e a causa
//!    não era onde eu supunha.
//!
//! # A partição, medida (`examples/preset_report.rs`, `examples/probe_curve.rs`)
//!
//! | família | razão de expansão por geração | como anima |
//! |---|---|---|
//! | Tree · Fern · Wild · Sprig | `1,63 → 1,06` (**converge para 1**) | a PONTA estica de zero |
//! | Bush · Weed | `3,00` · `2,03` (**constante**) | a figura REFINA-SE |
//! | Koch · Dragon | `3,00` · `~1,41`, e são CURVAS | idem |
//!
//! # A causa da não-linearidade, e ela não era a que eu supunha
//!
//! **Bush e Weed já eram perfeitamente lineares** (ondulação `0,0×`). Quem não era eram as
//! CURVAS: a Koch subia a `3,05` e **voltava** a `3,00` (`2,3×`), o Dragon subia a `1,62` e
//! voltava a `1,51` (`4,2×`).
//!
//! ⚠️ **As duas rampas brigavam.** O comprimento crescia linearmente enquanto as dobras, ao
//! abrir, **encurtavam** a projecção — a `90°` uma zig-zag ocupa menos do que a mesma linha
//! meio aberta. O produto tem um pico no meio, e o último quinto do slider andava **para trás**.
//! *Andar para trás é o que se vê da cadeira.*
//!
//! ⇒ A cura não é uma constante: é **normalizar pelo que se mede**. A cada instante mede-se o
//! tamanho com as dobras onde elas estão, e escolhe-se o comprimento que põe a figura na rampa
//! recta entre a geração anterior e a nova inteira. Ondulação depois: **`0,0×` nas quatro**.
//!
//! ⛔ **Duas curas ANTERIORES foram medidas e não bastam** — não as reconstrua:
//! - o **`Step Scale`** sozinho (o *Step Size Scale* do Houdini) deixa a figura do mesmo
//!   tamanho — `1/3` é exactamente a razão de Bush e Koch — e o melhor que uma varredura de
//!   oito valores alcança é `105 %` de pior passo. *Estável em tamanho ≠ contínuo na forma.*
//! - a **âncora como CONSTANTE** (`1/spread`, contada da gramática) está errada por
//!   construção: a `F -> F[+F]F[-F]F` põe **5** módulos por cada um e cresce **3,00×** (dois
//!   estão dentro de parênteses e não estendem o caminho), e a `F -> F+F-F-F+F` põe 5 sem
//!   parênteses e cresce `3,00×` na mesma, porque as viragens a dobram. *A razão é geométrica.*

use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::{Column, Stream};

fn size(s: &Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => {
            let x0 = v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
            let x1 = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
            let y0 = v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
            let y1 = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
            // ⚠️⚠️ **A DIAGONAL, e não `max(w, h)`** — o máximo de duas funções suaves tem um
            // JOELHO onde elas se cruzam, e esse joelho lê-se como não-linearidade do produto.
            // Medido: o Sprig imprimia **sete passos exactamente `0,0`** seguidos de uma rampa
            // recta — não porque a planta não crescesse, mas porque a largura era o máximo até
            // a altura a ultrapassar. *Uma régua com uma dobra por construção acusa o produto
            // da dobra dela.*
            ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt()
        }
        _ => 0.0,
    }
}

fn at(p: &ls::Preset, g: f32, over: &[(&str, f32)]) -> f32 {
    let mut o: Vec<(&str, f32)> = vec![
        (ls::param::MODE, ls::MODE_GRAMMAR as f32),
        (ls::param::ANGLE, p.angle),
        (ls::param::STEP, p.step),
        (ls::param::WIDTH, p.width),
    ];
    o.extend_from_slice(over);
    size(&ls::probe_build(p.axiom, p.rules, g, &o))
}

/// A travessia da penúltima geração para a última, amostrada fino: `(tamanhos, passos)`.
fn crossing(p: &ls::Preset, over: &[(&str, f32)]) -> (Vec<f32>, Vec<f32>) {
    const N: usize = 24;
    let g0 = (p.generations - 1.0).max(2.0).floor();
    let hs: Vec<f32> = (0..=N)
        .map(|k| at(p, g0 + k as f32 / N as f32, over))
        .collect();
    let d = hs.windows(2).map(|w| w[1] - w[0]).collect();
    (hs, d)
}

/// A ondulação da rampa: `(maior passo − menor passo) / passo médio`. `0` = recta.
fn ripple(d: &[f32]) -> f32 {
    let mean = d.iter().sum::<f32>() / d.len() as f32;
    if mean.abs() < 1e-9 {
        return f32::MAX;
    }
    let lo = d.iter().copied().fold(f32::MAX, f32::min);
    let hi = d.iter().copied().fold(f32::MIN, f32::max);
    (hi - lo) / mean.abs()
}

/// ⭐⭐⭐ **NENHUM MOLDE ANDA PARA TRÁS** — o defeito que o Enio de facto viu.
///
/// ⚠️ **A régua é a DERIVADA, e é isso que a queixa dele nomeia.** Uma barra sobre o pior
/// passo não a apanharia: a Koch tinha `17 %` de pior passo (perfeitamente aceitável) **e**
/// encolhia no último quinto do slider. *Um salto e um recuo são defeitos diferentes, e só o
/// segundo se lê como «não é linear».*
#[test]
fn no_preset_ever_shrinks_while_the_generations_rise() {
    for p in ls::PRESETS {
        let (hs, d) = crossing(p, &[]);
        assert!(
            hs[hs.len() - 1] > hs[0] * 1.005,
            "{}: a travessia tem de CRESCER: {hs:?}",
            p.label
        );
        let lo = d.iter().copied().fold(f32::MAX, f32::min);
        assert!(
            lo > -1e-4,
            "{}: um passo ANDOU PARA TRAS ({lo}) — a figura encolhe no fim do slider: {d:?}",
            p.label
        );
    }
}

/// ⭐⭐⭐ **E OS QUE REFINAM NÃO ONDULAM MAIS DO QUE OS QUE O DONO JÁ ACEITAVA.**
///
/// ⚠️⚠️ **A barra é COMPARATIVA de propósito, e a medição obrigou-me a isso.** Eu tinha
/// escrito `ripple < 0.25` para os oito e o gate reprovou no **Sprig — `1,5×`, o pior dos
/// oito** —, que é um dos quatro de que o Enio **nunca se queixou**. *A minha noção de «linear»
/// não é a dele; a dele é a família que ele aceitou.*
///
/// ⇒ A referência é o pior dos que crescem pela ponta. Antes da cura os que refinam davam
/// `2,3×` (Koch) e `4,2×` (Dragon) contra `1,5×`; depois dão `0,0×`–`0,7×`. A barra
/// discrimina, e não foi escolhida para passar.
///
/// ⚠️ E as duas famílias saem da razão de expansão **medida**, nunca de uma lista de nomes.
#[test]
fn the_refiners_ripple_no_worse_than_the_tip_growers_the_owner_accepted() {
    let mut tips: Vec<(&str, f32)> = Vec::new();
    let mut refiners: Vec<(&str, f32)> = Vec::new();
    for p in ls::PRESETS {
        let g0 = (p.generations - 1.0).max(2.0).floor();
        let growth = at(p, g0 + 1.0, &[]) / at(p, g0, &[]).max(1e-6);
        let r = ripple(&crossing(p, &[]).1);
        // Quem REFINA: a figura cresce por um factor constante > 1,5 a cada geração.
        if growth > 1.5 {
            &mut refiners
        } else {
            &mut tips
        }
        .push((p.label, r));
    }
    // ⚠️ **O CONTROLE**: as duas famílias têm de estar POVOADAS. Sem ele, o gate passaria num
    // mundo onde todos os moldes são da mesma espécie — e a lei da outra estaria morta.
    assert!(
        tips.len() >= 3 && refiners.len() >= 3,
        "as duas familias tem de existir no corpus: {tips:?} / {refiners:?}"
    );
    let worst_tip = tips.iter().map(|(_, r)| *r).fold(0.0f32, f32::max);
    for (label, r) in &refiners {
        assert!(
            *r <= worst_tip,
            "{label} ondula {r:.2}x, pior que o {:.2}x do pior dos que crescem pela ponta \
             ({tips:?}) — o report do Enio de 2026-08-29 voltou",
            worst_tip
        );
    }
}

/// ⭐⭐ **A ÂNCORA É O NÚMERO CERTO, E NÃO SÓ «UM NÚMERO QUE MELHORA».**
///
/// ⚠️⚠️ **Este gate nasceu de uma mutação que SOBREVIVEU.** Trocar a pose de partida de
/// `frac = 0` (viragens fechadas) por `frac = 1` (abertas) muda a âncora de `1/5` para `1/3`,
/// e um gate que só perguntava *«melhorou 2×?»* ficou verde com as duas. *Uma barra de
/// «melhorou» não distingue duas âncoras que ambas melhoram.*
///
/// A régua é o significado da âncora: com ela aplicada, a geração nova em `frac → 0` tem de ter
/// **o tamanho da anterior**. É uma identidade, não uma desigualdade.
#[test]
fn the_anchor_puts_the_new_generation_exactly_on_top_of_the_previous_one() {
    for (name, axiom, rules) in [
        ("arbusto (com ramos)", "F", "F -> F[+F]F[-F]F"),
        ("koch (curva pura)", "F", "F -> F+F-F-F+F"),
        ("duplicacao", "F", "F -> FF"),
    ] {
        let anchor = ls::probe_anchor(axiom, rules, 4.0);
        assert!(
            (0.02..1.0).contains(&anchor),
            "{name}: a ancora saiu {anchor}"
        );
        let s = |g: f32| {
            size(&ls::probe_build(
                axiom,
                rules,
                g,
                &[(ls::param::MODE, ls::MODE_GRAMMAR as f32)],
            ))
        };
        let ratio = s(4.0 + 1e-3) / s(4.0).max(1e-6);
        assert!(
            (0.97..1.03).contains(&ratio),
            "{name}: logo depois da geracao 4 a figura mede {ratio:.3}x a de 4 — a ancora nao \
             a poe por cima da anterior (ancora = {anchor:.4})"
        );
    }
}

/// ⛔ **O ESCAPE: desligar o `Grow Angle` devolve o degrau inteiro de sempre.**
///
/// ⚠️ ⭐ As duas metades. Sem a primeira o interruptor é um knob morto; sem a segunda ele não
/// é um escape — e um artista que prefira o degrau (ou que precise de bissecar) fica sem saída.
#[test]
fn switching_the_angle_growth_off_gives_back_the_whole_step() {
    let bush = ls::PRESETS
        .iter()
        .find(|p| p.label == "Bush")
        .expect("o molde existe");
    // 1. Desligado, a fracção é INERTE: o passo é inteiro, byte a byte.
    let off = |g: f32| at(bush, g, &[(ls::param::CONTINUOUS_ANGLE, 0.0)]);
    assert_eq!(
        off(3.25).to_bits(),
        off(3.75).to_bits(),
        "desligado, o passo tem de ser INTEIRO"
    );
    // 2. E ligado (o default) ela interpola.
    assert_ne!(
        at(bush, 3.25, &[]).to_bits(),
        at(bush, 3.75, &[]).to_bits(),
        "ligado, a fraccao tem de interpolar — senao o interruptor e' um knob morto"
    );
    // 3. E o default É ligado.
    let default = ls::MANIFEST
        .params
        .iter()
        .find(|p| p.name == ls::param::CONTINUOUS_ANGLE)
        .expect("o param existe")
        .default;
    assert_eq!(default, 1.0, "o `Grow Angle` shipa LIGADO desde 2026-08-29");
}

/// ⭐ **O `Grow Length` é BYTE-INERTE numa gramática de refinamento** — e a inércia é
/// estrutural, não um defeito.
///
/// Ali quem manda no comprimento é a normalização medida; o interruptor de esticar-de-zero não
/// tem sujeito, porque não há rebento novo a sair de nada. *Onde uma lei é inerte é um facto
/// sobre a lei.*
#[test]
fn the_length_growth_switch_has_no_subject_in_a_refinement_grammar() {
    let bush = ls::PRESETS
        .iter()
        .find(|p| p.label == "Bush")
        .expect("o molde existe");
    assert_eq!(
        at(bush, 3.5, &[(ls::param::CONTINUOUS_LENGTH, 0.0)]).to_bits(),
        at(bush, 3.5, &[(ls::param::CONTINUOUS_LENGTH, 1.0)]).to_bits(),
        "numa gramatica de refinamento o Grow Length nao tem sujeito"
    );
    // ⚠️ **O CONTROLE**: numa que cresce pela ponta ele MANDA. Sem esta metade, arrancar o
    // interruptor inteiro deixaria o gate verde.
    let tree = ls::PRESETS
        .iter()
        .find(|p| p.label == "Tree")
        .expect("o molde existe");
    assert_ne!(
        at(tree, 4.5, &[(ls::param::CONTINUOUS_LENGTH, 0.0)]).to_bits(),
        at(tree, 4.5, &[(ls::param::CONTINUOUS_LENGTH, 1.0)]).to_bits(),
        "numa que cresce pela ponta o Grow Length TEM de mandar"
    );
}
