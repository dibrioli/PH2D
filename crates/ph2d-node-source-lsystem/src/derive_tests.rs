//! Gates da **reescrita** — as três dimensões do ABOP, cada uma com o controlo que a
//! separa de uma que não a tem.

use super::*;
use crate::grammar::parse_rules;

fn nop(_: &str) -> f32 {
    0.0
}

fn run(axiom: &str, rules: &str, gens: u16) -> Derived {
    let p: &dyn Fn(&str) -> f32 = &nop;
    let a = axiom_modules(axiom, p);
    derive(&a, &parse_rules(rules), gens, 1, crate::MAX_MODULES, p)
}

fn text(d: &Derived) -> String {
    d.chain.iter().map(|m| m.sym as char).collect()
}

/// A produção canónica do ABOP (fig. 1.24), letra a letra.
#[test]
fn the_canonical_bush_rewrites_exactly() {
    assert_eq!(text(&run("F", "F -> F[+F]F[-F]F", 1)), "F[+F]F[-F]F");
    // E a segunda geração é a primeira com cada `F` substituído: os **cinco** `F` viram 11
    // símbolos cada, e os **seis** de estrutura (`[ + ] [ - ]`) ficam.
    let g2 = run("F", "F -> F[+F]F[-F]F", 2);
    assert_eq!(g2.chain.len(), 5 * 11 + 6, "{}", text(&g2));
    assert_eq!(g2.generations, 2);
}

/// ⭐ **A produção IDENTIDADE preserva a geração.**
///
/// Um símbolo sem regra passa intacto — e mantém a geração em que nasceu. Sem isso, a planta
/// inteira rejuvenesceria a cada passo e o crescimento fraccionário faria TUDO encolher em
/// vez de só a ponta.
#[test]
fn a_symbol_with_no_rule_keeps_the_generation_it_was_born_in() {
    let d = run("FX", "X -> XX", 3);
    let f = d.chain.iter().find(|m| m.sym == b'F').expect("o F ficou");
    assert_eq!(f.born, 0, "o F veio do axioma e continua da geracao 0");
    let x = d.chain.iter().find(|m| m.sym == b'X').expect("ha X");
    assert_eq!(x.born, 3, "os X sao da geracao mais nova");
}

/// **Paramétrica**: o argumento é recalculado a cada geração.
#[test]
fn a_parametric_argument_is_recomputed_every_generation() {
    let d = run("A(1)", "A(x) -> A(x*0.5)", 3);
    assert_eq!(d.chain.len(), 1);
    assert!(
        (d.chain[0].args[0] - 0.125).abs() < 1e-6,
        "1 · 0.5³ = 0.125, deu {}",
        d.chain[0].args[0]
    );
}

/// **Condicional**: a reescrita PÁRA quando a condição deixa de valer — e o controlo é a
/// mesma gramática sem a condição, que não pára.
#[test]
fn a_condition_stops_the_rewriting_and_the_control_does_not() {
    let with = run("A(1)", "A(x) : x > 0.3 -> A(x*0.5)", 6);
    assert!(
        (with.chain[0].args[0] - 0.25).abs() < 1e-6,
        "1 → .5 → .25 e para (0.25 nao e > 0.3), deu {}",
        with.chain[0].args[0]
    );
    let without = run("A(1)", "A(x) -> A(x*0.5)", 6);
    assert!(
        without.chain[0].args[0] < 0.02,
        "sem condicao ela desce as seis vezes, deu {}",
        without.chain[0].args[0]
    );
}

/// ⭐⭐ **O contexto da ESQUERDA atravessa um ramo completo e sobe ao pai.**
///
/// Em `A[+B]C`, o vizinho esquerdo de `C` é o `A` — não o `]`, não o `B` (que vive noutro
/// ramo), não o `+`. É a metade que separa um L-System de um `replace()` sobre texto, e a
/// que faz um sinal subir por uma planta em vez de por uma fita.
#[test]
fn the_left_context_skips_a_whole_branch_and_climbs_to_the_parent() {
    let p: &dyn Fn(&str) -> f32 = &nop;
    let chain = axiom_modules("A[+B]C", p);
    let c = chain.iter().position(|m| m.sym == b'C').unwrap();
    let l = left_neighbour(&chain, c).expect("ha vizinho a esquerda");
    assert_eq!(chain[l].sym, b'A', "deu {:?}", chain[l].sym as char);

    // E o de DENTRO do ramo: o vizinho esquerdo de `B` é o `A` (sobe-se pelo `[`, e o `+` é
    // pontuação que a procura atravessa).
    let b = chain.iter().position(|m| m.sym == b'B').unwrap();
    let lb = left_neighbour(&chain, b).expect("ha vizinho");
    assert_eq!(chain[lb].sym, b'A');
}

/// ⭐ **O contexto da DIREITA pode estar DENTRO de um ramo** (ABOP fig. 1.31) — e o
/// controlo é um símbolo que não está em ramo nenhum.
#[test]
fn the_right_context_may_live_inside_a_branch() {
    let p: &dyn Fn(&str) -> f32 = &nop;
    let chain = axiom_modules("A[B]C", p);
    let a = chain.iter().position(|m| m.sym == b'A').unwrap();
    assert!(
        right_match(&chain, a, b'B').is_some(),
        "o B esta no ramo que comeca logo a seguir ao A"
    );
    assert!(
        right_match(&chain, a, b'C').is_some(),
        "o C esta a seguir ao ramo, no mesmo nivel"
    );
    assert!(
        right_match(&chain, a, b'Z').is_none(),
        "o CONTROLO: o que nao esta la nao casa"
    );
}

/// **Sensível a contexto, ponta a ponta**: o sinal ANDA um passo por geração.
///
/// `B < A -> B` faz o `B` empurrar-se para a direita ao longo da fita. Com 3 gerações ele
/// avançou exactamente 3 posições — que é a definição de propagação.
#[test]
fn a_signal_walks_one_step_per_generation() {
    let d = run("BAAAAA", "B < A -> B ; B -> A", 3);
    assert_eq!(text(&d), "AAABAA", "deu {}", text(&d));
}

/// **Estocástica**: a mesma semente reproduz, sementes diferentes divergem, e uma gramática
/// DETERMINÍSTICA é byte-idêntica qualquer que seja a semente.
///
/// ⚠️ A terceira metade é a que impede um sorteio gasto onde não há escolha — sem ela,
/// mudar a semente mexeria numa planta que não tem nada de aleatório.
#[test]
fn the_draw_is_reproducible_divergent_and_absent_when_there_is_no_choice() {
    let p: &dyn Fn(&str) -> f32 = &nop;
    let stoch = parse_rules("F -> (0.5) F[+F] ; F -> (0.5) F[-F]");
    let a = axiom_modules("FFFFFFFF", p);
    let s = |seed| {
        derive(&a, &stoch, 3, seed, crate::MAX_MODULES, p)
            .chain
            .iter()
            .map(|m| m.sym as char)
            .collect::<String>()
    };
    assert_eq!(s(7), s(7), "a mesma semente reproduz");
    assert_ne!(s(7), s(8), "sementes diferentes divergem");

    let det = parse_rules("F -> F[+F]F");
    let d = |seed| {
        derive(&a, &det, 3, seed, crate::MAX_MODULES, p)
            .chain
            .iter()
            .map(|m| m.sym as char)
            .collect::<String>()
    };
    assert_eq!(d(1), d(9999), "uma gramatica determinista ignora a semente");
}

/// ⚠️ **O orçamento pára numa geração INTEIRA, e diz quantas correu.**
///
/// Uma cadeia quimera (parte reescrita, parte não) desenharia como uma planta partida, e o
/// artista leria isso como um defeito. O controlo é o mesmo `budget` sobre uma gramática que
/// cabe: ali as gerações pedidas correm todas.
#[test]
fn the_budget_stops_on_a_whole_generation_and_reports_it() {
    let p: &dyn Fn(&str) -> f32 = &nop;
    let a = axiom_modules("F", p);
    let doubling = parse_rules("F -> FF");
    let d = derive(&a, &doubling, 20, 1, 1000, p);
    assert!(d.generations < 20, "20 duplicacoes nao cabem em 1000");
    assert_eq!(
        d.chain.len(),
        1 << d.generations,
        "a cadeia e' exactamente 2^geracoes — nenhuma passagem ficou a meio"
    );
    let fits = derive(&a, &doubling, 5, 1, 1000, p);
    assert_eq!(fits.generations, 5, "o CONTROLE: 32 cabe em 1000");
}

/// Sem regras, o axioma passa intacto e nenhuma geração corre — o nó dropado da paleta com
/// a gramática ainda por escrever não pode custar nada.
#[test]
fn no_rules_means_no_work_and_the_axiom_survives() {
    let d = run("F[+F]F", "", 12);
    assert_eq!(text(&d), "F[+F]F");
    assert_eq!(d.generations, 0);
}
