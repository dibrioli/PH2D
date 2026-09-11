//! **A BANCADA QUE ESCOLHE O TECTO** (`ph2d_node_source_lsystem::MAX_MODULES`).
//!
//! ⚠️ Ela IMPRIME e não escreve nada: quem mexe no número é quem a corre, e põe a tabela ao
//! lado da constante (`CLAUDE.md` §0). Correr:
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && \
//!   cargo test -p ph2d-node-source-lsystem --release --test measure_lsystem_ceiling -- --ignored --nocapture
//! ```
//!
//! # De que RECURSO é o tecto
//!
//! Não é memória: 262 144 módulos × 24 bytes são 6,3 MB, e nenhuma máquina desta casa nota
//! isso. É o **orçamento de um quadro**. Este nó existe para ser animado — `Generations` a
//! subir é como a planta cresce — e como ele é `Effect::Pure`, cada valor novo do slider
//! **re-deriva a cadeia inteira dentro do quadro**. Logo o que se mede é `derivar +
//! interpretar`, que é exactamente o que um quadro paga.
//!
//! ⚠️ **Mede-se em `--release`.** O nó corre em `--release` no produto, e o debug mede o
//! perfil do build em vez do algoritmo (precedente registado no `CLAUDE.md` §5, Flip).
//!
//! ⚠️ **E o teto NÃO pode ser em ITERAÇÕES**, que é a forma tentadora: a taxa de expansão é
//! propriedade da REGRA. As três gramáticas abaixo existem para o mostrar — `F -> FF` dobra,
//! a de arbusto quintuplica, e a paramétrica ramifica em dois. Vinte gerações de uma são
//! triviais e da outra são impossíveis.

use std::time::Instant;

/// Uma medição: constrói a árvore e devolve `(elementos, mediana em ms)`.
fn measure(axiom: &str, rules: &str, gens: f32) -> (usize, f64) {
    // Uma corrida de aquecimento — a primeira paga o parse e o crescimento dos `Vec`.
    let _ = ph2d_node_source_lsystem::probe_build(axiom, rules, gens, &[]);
    let mut ts = Vec::new();
    let mut n = 0;
    for _ in 0..5 {
        let t0 = Instant::now();
        let s = ph2d_node_source_lsystem::probe_build(axiom, rules, gens, &[]);
        ts.push(t0.elapsed().as_secs_f64() * 1000.0);
        n = s.count();
    }
    ts.sort_by(f64::total_cmp);
    (n, ts[ts.len() / 2])
}

/// O tamanho da cadeia derivada, para a coluna «módulos» da tabela.
fn modules(axiom: &str, rules: &str, gens: f32) -> usize {
    // A cadeia não é pública; a contagem de ELEMENTOS é o que se observa, e a razão
    // módulos/elementos é constante por gramática — a tabela declara as duas.
    ph2d_node_source_lsystem::probe_build(axiom, rules, gens, &[]).count()
}

/// ⚠️ `#[ignore]`: é uma VARREDURA, não um gate. Ela mede dezenas de derivações e a mais
/// pesada leva dezenas de milissegundos — não tem lugar no portão de fecho, e o número que
/// ela escolhe já está gravado na constante.
#[test]
#[ignore = "varredura de medição — corre-a quem mexer no MAX_MODULES"]
fn sweep_the_frame_cost_of_a_derivation() {
    let budget_ms = 1000.0 / 60.0;
    println!("\n=== o que UM QUADRO paga por uma derivação (orçamento {budget_ms:.2} ms) ===");
    for (name, axiom, rules, gens) in [
        ("arbusto  F -> F[+F]F[-F]F", "F", "F -> F[+F]F[-F]F", 1..=7),
        ("dobra    F -> FF", "F", "F -> FF", 8..=18),
        (
            "param    A(s) -> F(s)![+A][-A]",
            "A(step)",
            "A(s) -> F(s)![+A(s*0.7)][-A(s*0.7)]",
            6..=16,
        ),
        (
            "estocast F -> (.5)F[+F] | (.5)F[-F]",
            "F",
            "F -> (0.5) F[+F]F ; F -> (0.5) F[-F]F",
            1..=8,
        ),
    ] {
        println!("\n--- {name} ---");
        println!(
            "{:>4} | {:>10} | {:>9} | {:>7}",
            "gen", "elementos", "ms", "% quadro"
        );
        for g in gens {
            let (n, ms) = measure(axiom, rules, g as f32);
            println!(
                "{g:>4} | {n:>10} | {ms:>9.3} | {:>6.1}%",
                ms / budget_ms * 100.0
            );
            if ms > budget_ms * 4.0 {
                println!("     (parou: já passa quatro quadros)");
                break;
            }
        }
    }

    println!("\n=== a SATURAÇÃO: o tecto de hoje, visto de cima ===");
    let cap = ph2d_node_source_lsystem::MAX_MODULES;
    println!("MAX_MODULES = {cap}");
    for g in [16.0, 20.0, 24.0, 31.0] {
        let (n, ms) = measure("F", "F -> FF", g);
        println!("  F->FF a {g:>4} gerações: {n:>8} elementos em {ms:>8.3} ms");
    }
    // A prova de que o tecto de fato satura: pedir mais não custa mais.
    let a = measure("F", "F -> FF", 20.0);
    let b = measure("F", "F -> FF", 30.0);
    assert_eq!(a.0, b.0, "saturado, a contagem tem de ser a mesma");
    println!(
        "\n  saturado: 20 e 30 gerações dão os mesmos {} elementos",
        a.0
    );
}

/// **O custo é LINEAR no tamanho da cadeia, não pior** — a afirmação que torna a tabela
/// extrapolável em vez de uma lista de pontos.
///
/// Uma derivação é uma varredura por módulo com uma procura de contexto que, numa gramática
/// sem contexto, não anda. Se a razão custo/elemento subisse com o tamanho, o tecto teria de
/// ser reencontrado a cada gramática nova em vez de derivado da tabela.
///
/// ⚠️ **Gate de RAZÃO ⇒ família de flake sob fan-out** (`CLAUDE.md` §5.0): se ele reprovar
/// numa corrida de milhares de testes, re-corra-o SOZINHO antes de olhar para o commit.
#[test]
#[ignore = "gate de razão — sensível a carga; corre sozinho"]
fn the_cost_grows_with_the_chain_and_not_faster() {
    let (n1, t1) = measure("F", "F -> FF", 14.0);
    let (n2, t2) = measure("F", "F -> FF", 17.0);
    assert!(n2 > n1 * 4, "a fixtura tem de crescer: {n1} -> {n2}");
    let per1 = t1 / n1 as f64;
    let per2 = t2 / n2 as f64;
    assert!(
        per2 < per1 * 2.5,
        "o custo POR ELEMENTO explodiu: {per1:.2e} ms -> {per2:.2e} ms"
    );
}

/// O elo entre a bancada e a constante: as três gramáticas SATURAM no mesmo sítio, e a
/// contagem de elementos nunca passa o tecto de módulos.
#[test]
fn no_grammar_ever_exceeds_the_declared_ceiling() {
    for (axiom, rules) in [
        ("F", "F -> FF"),
        ("F", "F -> F[+F]F[-F]F"),
        ("A(step)", "A(s) -> F(s)![+A(s*0.7)][-A(s*0.7)]"),
    ] {
        // ⚠️ **`+ 1`, e o um é a RAIZ.** A tartaruga planta-a antes do primeiro símbolo (sem
        // ela um axioma que comece por `[` não teria a que se pendurar), então ela não sai da
        // cadeia e não é contada pelo orçamento dela. É a única folga, é exactamente uma
        // linha, e está escrita no doc do tecto.
        let n = modules(axiom, rules, 32.0);
        assert!(
            n <= ph2d_node_source_lsystem::MAX_MODULES + 1,
            "{rules}: {n} elementos passam o tecto de {}",
            ph2d_node_source_lsystem::MAX_MODULES
        );
    }
}
