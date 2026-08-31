//! ⭐⭐⭐ **UMA REGRA DESCARTADA DIZ PORQUÊ** — o silêncio que estava aberto desde 2026-08-29.
//!
//! # A cerca que este ficheiro defende
//!
//! ⚠️ **A política de erro NÃO muda, e isso é metade da lei:** o que não se entende continua a
//! descartar *aquela* regra e a deixar as outras vivas — recusar a gramática inteira apagaria a
//! planta enquanto o artista escreve a segunda regra, que é o estado normal de quem autora. O
//! que muda é que a razão deixa de se perder.
//!
//! # As cinco razões, e por que são um enum
//!
//! O parser tem exactamente cinco saídas de descarte, e cada uma tem uma cura diferente para
//! quem escreveu o texto. Um `bool` («há erro») mandaria o artista procurar; um `String` pronto
//! obrigaria um gate a casar frases.
//!
//! ⚠️⚠️ **E a queixa nasce no MESMO `return Err` que descarta** — nunca num segundo leitor do
//! texto. Este nó já pagou esse defeito uma vez: o gate dos pesos lia-os com um
//! `str::parse::<f32>()` próprio e ficava verde sobre uma soma que o motor nunca calculava.

use ph2d_node_source_lsystem as ls;

/// Quantos módulos a gramática deriva — a prova de que a regra foi mesmo descartada, e não
/// só acusada.
fn modules(rules: &str) -> usize {
    ls::probe_build(ls::DEFAULT_AXIOM, rules, 5.0, &[]).count()
}

#[test]
fn each_of_the_five_ways_to_lose_a_rule_names_itself() {
    // ⚠️ **A régua de cada linha é a MESMA regra, com um caractere estragado** — assim o que
    // separa as células é a razão, e não duas gramáticas diferentes.
    let casos: &[(&str, ls::RuleProblem)] = &[
        ("A(s) F(s)", ls::RuleProblem::NoArrow),
        // ⚠️ **A cabeça VAZIA, e não um `(` solto** — a 1.ª redacção usava `( -> F(1)` e o
        // produto respondeu `NoArrow`, com razão: um `(` abre parênteses e a busca da seta só
        // olha o nível de topo, logo aquele `->` está *dentro* de uma lista de argumentos por
        // fechar. *A fixtura é que estava errada, e ela acusava o parser.*
        ("-> F(1)", ls::RuleProblem::BadPredecessor),
        ("A(s) : s <= -> F(s)", ls::RuleProblem::BadCondition),
        ("A(s) -> (40%) F(s)", ls::RuleProblem::BadWeight),
        // ⚠️ **Sem `)` NENHUM na cauda.** A 1.ª redacção era `-> (0.5 F(s)`, e o produto
        // respondeu `BadWeight`, com razão: o `)` que fecha o `F(s)` é um `)`, então o peso lido
        // é o texto `0.5 F(s`, que não é um número. *As duas razões só se separam quando não há
        // parêntese nenhum a seguir* — e o artista vê a regra citada nos dois casos.
        ("A(s) -> (0.5 F", ls::RuleProblem::UnclosedWeight),
    ];
    for (src, esperado) in casos {
        let q = ls::grammar_complaints(src);
        assert_eq!(
            q.len(),
            1,
            "`{src}` tem de dar UMA queixa e deu {}: {q:?}",
            q.len()
        );
        assert_eq!(q[0].problem, *esperado, "`{src}`");
        assert_eq!(
            q[0].rule,
            src.trim(),
            "a queixa NOMEIA a regra que o artista escreveu"
        );
        assert!(
            !q[0].problem.say().is_empty(),
            "`{src}` tem de ter uma frase para o artista"
        );
    }
    // ⚠️ **Cinco razões distintas, e não a mesma cinco vezes** — sem isto o teste passaria com
    // um enum de uma variante só.
    let mut vistas: Vec<ls::RuleProblem> = casos.iter().map(|(_, p)| *p).collect();
    vistas.dedup();
    assert_eq!(vistas.len(), 5, "as cinco razões têm de ser distintas");
}

#[test]
fn a_grammar_that_is_right_never_complains() {
    // ⚠️ **Os OITO moldes**, não um exemplo escolhido: um aviso que aparece sobre produto
    // correcto ensina a ignorar o aviso, e depois disso ele não existe.
    for p in ls::PRESETS {
        let q = ls::grammar_complaints(p.rules);
        assert!(
            q.is_empty(),
            "o molde `{}` queixa-se de si próprio: {q:?}",
            p.label
        );
    }
    assert!(ls::grammar_complaints(ls::DEFAULT_RULES).is_empty());
    // ⚠️ Uma gramática que acaba em `;` é pontuação, não erro — acusá-la poria um aviso
    // permanente em toda gramática bem escrita.
    assert!(ls::grammar_complaints("F -> FF ;").is_empty());
    assert!(ls::grammar_complaints("").is_empty());
}

#[test]
fn the_complaint_agrees_with_what_the_parser_actually_did() {
    // ⭐ A metade que impede a queixa de ser decoração: quem se queixa **perdeu** a regra, e
    // quem não se queixa manteve-a. Sem isto, um `Vec::new()` no lugar da porta passaria.
    let bom = "A(s) -> F(s)[+A(s*0.7)][-A(s*0.7)]";
    let mau = "A(s) -> (40%) F(s)[+A(s*0.7)][-A(s*0.7)]";
    assert!(ls::grammar_complaints(bom).is_empty());
    assert_eq!(ls::grammar_complaints(mau).len(), 1);
    let (n_bom, n_mau) = (modules(bom), modules(mau));
    assert!(
        n_bom > n_mau,
        "a regra descartada tem de encolher a planta: {n_bom} contra {n_mau}"
    );
    // ⚠️ E a regra MÁ não pode levar as boas com ela.
    let misto = "A(s) -> F(s)[+A(s*0.7)][-A(s*0.7)] ; B -> (40%) F";
    assert_eq!(ls::grammar_complaints(misto).len(), 1, "só a má se queixa");
    assert_eq!(
        modules(misto),
        n_bom,
        "a regra boa sobrevive ao vizinho malformado, ao módulo"
    );
}
