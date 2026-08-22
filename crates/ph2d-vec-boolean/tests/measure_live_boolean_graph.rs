//! **Quanto custa uma REDE de booleanas vivas, por quadro** — o número que decide se o
//! *Live Boolean Graph* (Enio, 2026-08-21) tem teto útil ou não.
//!
//! # A pergunta que esta sonda existe para responder
//!
//! Enio: *"operações booleanas vivas são apenas aparentes e não modificam o path de fato. Ainda
//! assim o custo é alto?"*
//!
//! ⚠️ **Ser aparente não torna o cálculo barato — torna-o REPETIDO.** O que "não destrutivo"
//! significa é que a FONTE é preservada; a geometria resultante continua a ter de ser computada
//! para ser desenhada, e o `bool_live` é um produtor de `LiveGeometry` que corre **a cada
//! quadro**. O destrutivo paga uma vez e guarda; o vivo paga sempre e não guarda. A pergunta certa
//! não é *"é caro?"* mas *"quantas arestas cabem em 16,7 ms?"*.
//!
//! # O que se mede
//!
//! Uma CADEIA de N formas — que é o pior caso de um grafo, porque nenhuma aresta é independente da
//! anterior: cada operação consome o resultado da anterior, exatamente como um grupo booleano
//! aninhado já faz hoje (`bool_live`: *"o mais FUNDO primeiro"*). Um grafo em LEQUE (várias
//! arestas partindo da mesma forma) é mais barato, porque os ramos partem de geometria já pronta.
//!
//! ⚠️ **Formas com vértices a sério** (estrelas de 5 a 12 pontas), não retângulos: o custo do
//! motor é função do número de cruzamentos, e um par de caixas mediria o caso que não acontece.
//!
//! Rodar: `cargo test -p ph2d-vec-boolean --test measure_live_boolean_graph --release -- --ignored --nocapture`
//!
//! ⚠️ **A máquina tem de estar CALMA** — o `CLAUDE.md` mede que a mesma passagem deu 11,36 e
//! 5,50 ms sob carga. Um número colhido durante um `cargo build` não vale nada.

use std::time::Instant;

use ph2d_vec_boolean::{BoolOp, apply};
use ph2d_vec_scene::{ShapeKind, VecPath, cook};

/// Uma estrela deslocada — vértices a sério, e sobreposição garantida com as vizinhas.
fn star(points: f64, dx: f64) -> VecPath {
    let mut v = [0.0; ph2d_vec_scene::MAX_SHAPE_FIELDS];
    v[0] = points;
    v[1] = 0.45;
    cook(ShapeKind::Star, [-2.0 + dx, -2.0], [2.0 + dx, 2.0], &v[..])
}

/// A cadeia: `n` formas, `n - 1` arestas, cada uma consumindo o resultado da anterior.
fn chain(n: usize, op: BoolOp) -> (usize, f64) {
    let shapes: Vec<VecPath> = (0..n)
        .map(|i| star(5.0 + (i % 8) as f64, i as f64 * 0.9))
        .collect();
    let t0 = Instant::now();
    let mut acc = vec![shapes[0].clone()];
    for s in &shapes[1..] {
        let Some(base) = acc.first() else { break };
        acc = apply(base, s, op);
        if acc.is_empty() {
            break;
        }
    }
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    (acc.len(), ms)
}

/// **A TABELA.** Não afirma nada — imprime, e quem lê decide onde fica o teto.
#[test]
#[ignore = "sonda de relógio: rode com --release, --ignored e a máquina calma"]
fn how_many_live_boolean_edges_fit_in_a_frame() {
    const FRAME_MS: f64 = 16.7;
    println!("\n  n formas   arestas      Union      Subtract    Intersect   % de um frame (pior)");
    println!("  ────────   ───────   ─────────   ─────────   ─────────   ────────────────────");
    for n in [2usize, 3, 5, 8, 12, 20] {
        // Aquece: a 1ª corrida de cada tamanho paga o alocador.
        let _ = chain(n, BoolOp::Union);
        let u = chain(n, BoolOp::Union).1;
        let s = chain(n, BoolOp::Subtract).1;
        let i = chain(n, BoolOp::Intersect).1;
        let worst = u.max(s).max(i);
        println!(
            "  {n:>8}   {:>7}   {u:>7.3} ms   {s:>7.3} ms   {i:>7.3} ms   {:>5.1} %",
            n - 1,
            worst / FRAME_MS * 100.0
        );
    }
    println!(
        "\n  Orçamento: um quadro de 60 fps = {FRAME_MS} ms, e a rede NAO e' a unica coisa nele.\n"
    );
}
