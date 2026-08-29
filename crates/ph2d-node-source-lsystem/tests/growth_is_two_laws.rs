//! ⭐⭐⭐ **O CRESCIMENTO SÃO DUAS LEIS, E CADA FAMÍLIA DE GRAMÁTICA PRECISA DE UMA** —
//! pesquisa e medição de 2026-08-29, a pedido do Enio (*"acho que o ideal é o crescimento
//! suave. como fazem os grandes apps?"*).
//!
//! O L-System SOP do Houdini tem **dois** interruptores — *Continuous length* e *Continuous
//! angles* — e escala, com a fracção da geração, ou os comprimentos ou os ângulos das
//! operações de tartaruga da última substituição. Eu tinha construído a metade do comprimento.
//!
//! # A partição, medida (`examples/preset_report.rs`)
//!
//! | família | razão de expansão por geração | anima por |
//! |---|---|---|
//! | Tree · Fern · Wild · Sprig | `1,63 → 1,06` (**converge para 1**) | `Generations` — já é suave (3–5 %) |
//! | Koch · Dragon | `3,00` / `~1,41` (**constante**), e são CURVAS | **revelar o traçado** (8–10 %) |
//! | Bush · Weed | `3,00` / `2,03`, e **ramificam** | nenhuma das duas — ver a recusa abaixo |
//!
//! ⛔ **RECUSA MEDIDA — o `Grow Angle` MOVE a figura e NÃO se prova que a alise.**
//!
//! ⚠️⚠️ **A 1.ª redacção desta nota afirmava que ele PIORAVA, citando `69 % → 138 %` — era
//! falso, e o gate apanhou-o.** Aqueles dois números vêm de **duas réguas diferentes** (uma
//! normalizada pela subida total, outra pela média do tamanho); com a MESMA régua, Bush dá
//! `138 %` ligado e `138 %` desligado. *Comparar duas medições feitas com denominadores
//! diferentes é inventar um efeito.*
//!
//! O que está medido:
//! - ele **move** uma gramática de refinamento (Bush, desvio máximo `0,163` a `frac = 0,25`,
//!   a cair para `0,055` em `0,75` — a assinatura certa: o efeito some quando a geração fecha);
//! - ele é **byte-inerte** numa que cresce pela ponta, e a razão é estrutural: ali a viragem
//!   nova é seguida de um não-terminal que **ainda não desenha nada**, então não há geometria
//!   atrás dela para abrir;
//! - e **nenhuma régua que eu tenha consegue dizer se ele ALISA**. A caixa envolvente é grossa
//!   demais, o emparelhamento ponto-a-ponto é indefinido (a contagem salta de 626 para 3 126 na
//!   travessia) e uma grelha de ocupação normalizada mede a mudança de CONTAGEM, não de forma.
//!   ⇒ *shipa desligado, e o que falta é a RÉGUA, não o código.*
//!
//! ⛔ **E o `Step Scale` sozinho também não.** Ele torna a figura estável em tamanho (é o
//! *Step Size Scale* do Houdini, e `1/3` é exactamente a razão de Bush e Koch), mas o melhor
//! que a varredura de oito valores alcança é `105 %` no Bush e `144 %` no Koch — contra `3 %`
//! do Tree. *Estável em tamanho não é contínuo na forma.*
//!
//! ⭐ **A cura que resta para Bush/Weed está NOMEADA e não construída**: interpolar entre
//! **duas cadeias derivadas** (geração `n` e `n+1`), com cada segmento da `n` a morfar nos
//! sub-segmentos da `n+1`. Custa uma segunda derivação por quadro e muda o que `Generations`
//! quer dizer — decisão de produto.

use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::{Column, Stream};

/// A maior dimensão da caixa.
fn size(s: &Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => {
            let x0 = v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
            let x1 = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
            let y0 = v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
            let y1 = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
            (x1 - x0).max(y1 - y0)
        }
        _ => 0.0,
    }
}

/// O pior passo de uma varredura fina do `Generations`, contra a MÉDIA do tamanho.
///
/// ⚠️ **A média e não a subida total** — a 1.ª régua dividia pela subida, e como o objectivo do
/// `Step Scale` é precisamente deixar a figura do MESMO tamanho, ela imprimiu `619 050 %`.
/// *Uma régua normalizada pelo que a cura leva a zero mede a cura ao contrário.*
fn worst_step(p: &ls::Preset, over: &[(&str, f32)]) -> f32 {
    const N: usize = 40;
    let hs: Vec<f32> = (0..=N)
        .map(|j| {
            let g = 1.0 + (p.generations - 1.0) * j as f32 / N as f32;
            let mut o: Vec<(&str, f32)> = vec![
                (ls::param::MODE, ls::MODE_GRAMMAR as f32),
                (ls::param::ANGLE, p.angle),
                (ls::param::STEP, p.step),
                (ls::param::WIDTH, p.width),
            ];
            o.extend_from_slice(over);
            size(&ls::probe_build(p.axiom, p.rules, g, &o))
        })
        .collect();
    let mean = hs.iter().sum::<f32>() / hs.len() as f32;
    hs.windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0f32, f32::max)
        / mean.max(1e-6)
}

/// Os quatro que crescem pela PONTA, pelo nome.
const TIP_GROWERS: &[&str] = &["Tree", "Fern", "Wild", "Sprig"];

/// ⭐⭐ **AS QUE CRESCEM PELA PONTA JÁ SÃO SUAVES, e continuam a sê-lo.**
///
/// A barra é `10 %` do tamanho médio — folgada face aos `3–5 %` medidos, apertada face aos
/// `69–138 %` das outras. É a fronteira entre as duas famílias, não um número escolhido.
#[test]
fn the_tip_growers_animate_smoothly_and_the_others_are_a_different_family() {
    for p in ls::PRESETS {
        let w = worst_step(p, &[]);
        if TIP_GROWERS.contains(&p.label) {
            assert!(
                w < 0.10,
                "{} devia crescer suave e o pior passo foi {:.0}% do tamanho",
                p.label,
                w * 100.0
            );
        } else {
            // ⚠️ **O CONTROLE, e é ele que torna a barra acima uma afirmação**: sem ele, uma
            // barra de 10 % passaria com TODOS os moldes suaves — e nós sabemos, por medição,
            // que quatro não são. Se um dia forem, este gate cai e a pessoa vem ler a recusa.
            assert!(
                w > 0.20,
                "{} passou a crescer suave ({:.0}%) — se foi de propósito, a recusa medida no \
                 cabeçalho deste ficheiro tem de ser reconferida",
                p.label,
                w * 100.0
            );
        }
    }
}

/// ⛔⛔ **A RECUSA É DE PRODUTO, NÃO DE MEDIÇÃO — e a diferença é o assunto deste gate.**
///
/// ⚠️⚠️ **A 1.ª redacção afirmava que o `Grow Angle` NÃO alisava, e ficou falsa no mesmo dia.**
/// Com a âncora medida ele alisa, e por muito: Bush **`69 % → 9 %`**, Weed `53 % → 9 %`, Koch
/// `69 % → 17 %`, Dragon `34 % → 31 %`. O que estava errado era eu ter parado de procurar a
/// âncora — ela não se conta a partir da gramática (a Koch põe 5 módulos e cresce `3,00×`),
/// mede-se percorrendo as duas gerações.
///
/// ⛔ **E mesmo assim ele shipa desligado, por VEREDITO do dono do produto** (2026-08-29:
/// *"os que vc tentou corrigir não ficarão bons"*): os quatro ficam em `9–31 %` contra os
/// `5–8 %` de quem cresce pela ponta, e o gesto que se vê é a figura **desdobrar-se**, que não
/// é crescer.
///
/// ⭐ *Uma recusa MEDIDA e um veredito de PRODUTO são coisas diferentes, e vestir um de outro
/// é o modo de falha que esta linha já cometeu hoje.* Este gate afirma as duas metades pelo
/// que elas são: o número diz que a lei funciona, e o default diz que o dono não a quer ligada.
#[test]
fn the_angle_growth_does_smooth_a_refinement_grammar_and_ships_off_by_product_verdict() {
    let bush = ls::PRESETS
        .iter()
        .find(|p| p.label == "Bush")
        .expect("o molde existe");
    let off = worst_step(bush, &[]);
    let on = worst_step(bush, &[(ls::param::CONTINUOUS_ANGLE, 1.0)]);
    // 1. A lei FUNCIONA — e por uma margem que não é ruído.
    assert!(
        on < off * 0.5,
        "o Grow Angle deixou de alisar o Bush ({:.0}% ligado contra {:.0}% desligado) — a \
         ancora medida partiu-se, e o `previous` da derivacao e' o primeiro sitio a olhar",
        on * 100.0,
        off * 100.0
    );
    // 2. E MESMO ASSIM o produto shipa com ela desligada. ⚠️ É esta metade que morre no dia em
    //    que alguém ligar o default sem falar com o dono.
    let default = ls::MANIFEST
        .params
        .iter()
        .find(|p| p.name == ls::param::CONTINUOUS_ANGLE)
        .expect("o param existe")
        .default;
    assert_eq!(
        default, 0.0,
        "o `Grow Angle` shipa DESLIGADO por veredito do dono do produto (2026-08-29), nao por \
         a lei nao funcionar — ver o doc deste gate antes de mexer"
    );
    // 3. ⚠️ E o CONTROLE da recusa: desligado, a fracção é INERTE numa gramática de
    //    refinamento — o degrau inteiro é o produto, byte a byte.
    let step = |g: f32| {
        worst_step(bush, &[]);
        ls::probe_build(
            bush.axiom,
            bush.rules,
            g,
            &[
                (ls::param::MODE, ls::MODE_GRAMMAR as f32),
                (ls::param::ANGLE, bush.angle),
                (ls::param::STEP, bush.step),
            ],
        )
        .count()
    };
    assert_eq!(step(3.25), step(3.75), "por omissao o passo e' inteiro");
}

/// ⭐ **Ele shipa DESLIGADO, e é BYTE-INERTE numa gramática que cresce pela ponta.**
///
/// ⚠️⚠️ A inércia não é um defeito, é estrutural: em `A(s) -> F(s)![+A(s*0.7)][-A(s*0.7)]` a
/// viragem nova é seguida de um `A`, um não-terminal que **ainda não desenha nada** — não há
/// geometria atrás dela para abrir. A 1.ª redacção deste gate exigia que ele movesse o
/// `DEFAULT_RULES` e reprovava sobre produto correcto. *Onde uma lei é inerte é um facto sobre
/// a lei, não um bug* — e é o gate acima que prova que ela NÃO é inerte onde tem sujeito.
#[test]
fn the_angle_growth_switch_ships_off_and_is_inert_where_it_has_no_subject() {
    let default = ls::MANIFEST
        .params
        .iter()
        .find(|p| p.name == ls::param::CONTINUOUS_ANGLE)
        .expect("o param existe")
        .default;
    assert_eq!(default, 0.0, "ele shipa DESLIGADO — ver a recusa medida");

    let tip = |on: f32| {
        ls::probe_build(
            ls::DEFAULT_AXIOM,
            ls::DEFAULT_RULES,
            4.5,
            &[
                (ls::param::MODE, ls::MODE_GRAMMAR as f32),
                (ls::param::CONTINUOUS_ANGLE, on),
            ],
        )
    };
    assert_eq!(
        size(&tip(0.0)).to_bits(),
        size(&tip(1.0)).to_bits(),
        "o Grow Angle mexeu numa gramatica que cresce pela PONTA — ali a viragem nova nao tem \
         nada desenhado atras dela, entao ou o `born` mudou de sentido ou a lei mudou de sitio"
    );
}

/// ⭐⭐ **O `Step Scale` deixa uma gramática de refinamento do MESMO TAMANHO** — o que ele de
/// facto compra, medido, em vez do que se esperava dele.
///
/// Bush e Koch expandem **exactamente `3,00`** por geração; com `1/3` a figura fica estável.
/// ⛔ E isto **não** a torna contínua — ver a recusa no cabeçalho. As duas coisas são
/// diferentes, e este gate existe para que ninguém as volte a confundir.
#[test]
fn the_step_scale_keeps_a_refinement_grammar_the_same_size_but_not_continuous() {
    let koch = ls::PRESETS
        .iter()
        .find(|p| p.label == "Koch")
        .expect("o molde existe");
    let span = |scale: f32| {
        let sizes: Vec<f32> = (2..=5)
            .map(|g| {
                size(&ls::probe_build(
                    koch.axiom,
                    koch.rules,
                    g as f32,
                    &[
                        (ls::param::MODE, ls::MODE_GRAMMAR as f32),
                        (ls::param::ANGLE, koch.angle),
                        (ls::param::STEP, koch.step),
                        (ls::param::STEP_SCALE, scale),
                    ],
                ))
            })
            .collect();
        sizes.iter().copied().fold(f32::MIN, f32::max)
            / sizes.iter().copied().fold(f32::MAX, f32::min)
    };
    // Sem ele, a figura cresce 3x por geração — 27x ao longo de quatro.
    assert!(
        span(1.0) > 20.0,
        "sem Step Scale a Koch cresce {:.1}x",
        span(1.0)
    );
    // Com `1/3`, estável.
    assert!(
        span(1.0 / 3.0) < 1.05,
        "com Step Scale = 1/3 a Koch tem de ficar do mesmo tamanho, e variou {:.2}x",
        span(1.0 / 3.0)
    );
    // ⛔ E MESMO ASSIM ela não é contínua — a segunda metade da recusa.
    assert!(
        worst_step(koch, &[(ls::param::STEP_SCALE, 1.0 / 3.0)]) > 0.20,
        "a Koch passou a ser continua so' com o Step Scale — reconfira a recusa do cabecalho"
    );
}

/// ⭐⭐⭐ **A ÂNCORA É O NÚMERO CERTO, E NÃO SÓ «UM NÚMERO QUE MELHORA».**
///
/// ⚠️⚠️ **Este gate nasceu de uma mutação que SOBREVIVEU.** Trocar a pose de partida de
/// `frac = 0` (viragens fechadas) por `frac = 1` (abertas) muda a âncora de `1/5` para `1/3` —
/// e o gate irmão, que só perguntava *«melhorou 2×?»*, ficou verde com as duas. *Uma barra de
/// «melhorou» não distingue duas âncoras que ambas melhoram.*
///
/// A régua é o próprio significado da âncora: com ela aplicada, a geração nova em `frac = 0`
/// tem de ter **o tamanho da anterior**. É uma identidade, não uma desigualdade.
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
        // A identidade: `size(n+1, frac->0)` == `size(n, inteira)`.
        let at = |g: f32| {
            let s = ls::probe_build(
                axiom,
                rules,
                g,
                &[
                    (ls::param::MODE, ls::MODE_GRAMMAR as f32),
                    (ls::param::CONTINUOUS_ANGLE, 1.0),
                ],
            );
            match s.get("P") {
                Some(Column::Vec2(v)) if !v.is_empty() => {
                    let x0 = v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
                    let x1 = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
                    let y0 = v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
                    let y1 = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
                    (x1 - x0).max(y1 - y0)
                }
                _ => 0.0,
            }
        };
        let (whole, just_after) = (at(4.0), at(4.0 + 1e-3));
        let ratio = just_after / whole.max(1e-6);
        assert!(
            (0.97..1.03).contains(&ratio),
            "{name}: logo depois da geracao 4 a figura mede {ratio:.3}x a de 4 — a ancora nao \
             a poe por cima da anterior (ancora = {anchor:.4})"
        );
    }
}
