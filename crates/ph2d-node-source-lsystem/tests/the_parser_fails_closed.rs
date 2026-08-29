//! ⭐⭐⭐ **O QUE O PARSER NÃO ENTENDE, ELE DESCARTA** — e antes de 2026-08-29 ele fazia o
//! contrário em dois dos três sub-campos de uma regra.
//!
//! # O mecanismo, e por que era invisível
//!
//! Uma regra é `[ctxE <] pred [> ctxD] [: cond] -> [(peso)] succ`. O parser tinha **três
//! políticas de erro para três sub-campos da mesma regra**:
//!
//! | sub-campo | o que fazia | direcção |
//! |---|---|---|
//! | predecessor | descartava a regra | fecha ✓ |
//! | condição | `parse(c).ok()` ⇒ `None` ⇒ a jusante lê *«não há condição»* | **ABRE** |
//! | peso | vira o neutro `1,0` **e** a cauda com os parênteses vai desenhar | **ABRE ×2** |
//!
//! ⚠️ *O dono do produto nunca saberá que escreveu mal.* Uma condição com um `<=` (que o
//! `ph2d-expr-parse` não tem) fazia o travão **desaparecer**, e a regra passava a disparar
//! sempre — 512× mais módulos, byte-a-byte iguais a não ter escrito condição nenhuma.
//! Um peso `(40%)` apagava a planta inteira, porque o `%` é o **corte** da tartaruga.
//!
//! ⛔ **A cura NÃO é acrescentar `<=`/`>=` ao `ph2d-expr-parse`** — ele é o parser partilhado
//! do ADR-0144 (a timeline também o usa), e a condição evaporava **por igual** quando estava
//! vazia ou truncada. O defeito era a política de erro, não a lista de operadores.

use ph2d_node_source_lsystem as ls;

fn n_at(rules: &str, gens: f32) -> usize {
    ls::probe_build("A(step)", rules, gens, &[]).count()
}

/// ⭐⭐ **Uma condição que não compila DESCARTA a regra** — nunca «não há condição».
///
/// A régua é a contagem, e ela é honesta aqui de propósito: o defeito **multiplica** os
/// módulos (o travão some), então a contagem é a grandeza que o vê.
#[test]
fn a_condition_that_does_not_compile_discards_the_rule() {
    // O CONTROLO POSITIVO: uma condição que compila trava a derivação.
    let braked = n_at("A(s) : s > 0.05 -> F(s)A(s*0.5)A(s*0.5)", 12.0);
    let free = n_at("A(s) -> F(s)A(s*0.5)A(s*0.5)", 12.0);
    assert!(
        braked < free / 4,
        "o controlo falhou: com travao {braked}, sem travao {free} — sem esta diferenca o \
         resto deste gate nao mede nada"
    );

    // E as três formas de condição malformada: nenhuma pode dar o mesmo que «sem condição».
    for bad in ["s >= 0.05", "", "s > "] {
        let got = n_at(&format!("A(s) : {bad} -> F(s)A(s*0.5)A(s*0.5)"), 12.0);
        assert_ne!(
            got, free,
            "a condicao {bad:?} evaporou — a regra disparou como se nao tivesse travao nenhum"
        );
        // E o que ela de facto faz: a regra some, então só o axioma sobra.
        assert!(
            got <= 2,
            "a condicao {bad:?} devia DESCARTAR a regra e deu {got} modulos"
        );
    }
}

/// ⭐⭐ **Um peso que não é um número DESCARTA a regra** — nunca o neutro `1,0`.
///
/// ⚠️ **Duas coisas de uma vez**, e por isso duas asserções: o peso ilegível ganhava o
/// **maior** de três pesos típicos (a regra mal escrita virava a mais provável), e a cauda
/// seguia com os parênteses lá dentro **para o desenho** — o `%` corta, o `-` e o `+` viram.
#[test]
fn a_malformed_weight_discards_the_rule_instead_of_becoming_the_neutral() {
    let control = n_at("A(s) -> F(s)[+A(s*0.6)]A(s*0.6)", 4.0);
    // O CONTROLO POSITIVO: um peso legal não muda o que a regra desenha.
    assert_eq!(
        n_at("A(s) -> (0.001) F(s)[+A(s*0.6)]A(s*0.6)", 4.0),
        control,
        "um peso legal e' so' um peso — ele nao pode mudar o desenho de uma regra sozinha"
    );
    for bad in ["(40%)", "(-0.5)", "(0.3+0.2)", "(0"] {
        let got = n_at(&format!("A(s) -> {bad} F(s)[+A(s*0.6)]A(s*0.6)"), 4.0);
        assert_ne!(
            got, control,
            "o peso {bad:?} passou como se fosse legal — ou ele desenhou, ou virou o neutro"
        );
        assert!(
            got <= 2,
            "o peso {bad:?} devia DESCARTAR a regra e deu {got} modulos"
        );
    }
}

/// ⭐⭐⭐ **O `Variation` no MÁXIMO do slider não devolve o meio dele.**
///
/// ⚠️ Este é o defeito alcançável com **zero digitação, por um slider de fábrica**. O
/// `shape::rules` escreve os pesos com `{:.3}`; em `Variation = 1,00` o `1 − v` sai como o
/// literal `(0.000)`, que a guarda `v > 0.0` reprova — e com a falha ABERTA aquilo virava
/// peso `1,0`, o **maior** dos três. Medido (largura média, 200 sementes, guiado, gens=3):
///
/// | v | largura |
/// |---|---|
/// | 0,000 | 1,0004 |
/// | 0,500 | 0,9735 |
/// | 0,995 | 0,9368 |
/// | **1,000** | **0,9734** ← INVERTIA, e reproduzia exactamente o `v = 0,500` |
///
/// A régua aqui é o próprio sintoma: o extremo do slider tem de ficar do lado do `0,995`, e
/// **não** voltar para o meio.
#[test]
fn the_variation_slider_does_not_invert_at_its_own_maximum() {
    let width = |v: f32| -> f32 {
        let mut total = 0.0;
        const SEEDS: usize = 120;
        for seed in 0..SEEDS {
            let s = ls::probe_build(
                "",
                "",
                3.0,
                &[
                    (ls::param::MODE, ls::MODE_GUIDED as f32),
                    (ls::param::VARIATION, v),
                    (ls::param::SEED, seed as f32),
                ],
            );
            if let Some(ph2d_nodegraph::attr::Column::Vec2(p)) = s.get("P") {
                let lo = p.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
                let hi = p.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
                total += hi - lo;
            }
        }
        total / SEEDS as f32
    };
    let (w0, w_mid, w_near, w_max) = (width(0.0), width(0.5), width(0.995), width(1.0));
    // O CONTROLO: o slider tem de fazer alguma coisa entre as pontas, senão o resto é vazio.
    assert!(
        (w0 - w_near).abs() > 1e-3,
        "o Variation nao mexe em nada: {w0} contra {w_near}"
    );
    // E a afirmação: o extremo continua a viagem, nunca volta ao meio.
    assert!(
        (w_max - w_near).abs() < (w_max - w_mid).abs(),
        "v = 1,00 ({w_max:.4}) esta' mais perto do MEIO ({w_mid:.4}) do que do vizinho \
         v = 0,995 ({w_near:.4}) — o extremo do slider inverte"
    );
}
