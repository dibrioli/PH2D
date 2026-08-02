//! **M2 — quanto custa um passe de layout, e a árvore precisa de memo?**
//!
//! A medição que o plano de UI/UX exige ANTES da wave (§9): é ela que decide o teto de nós de uma
//! moldura, e se reconstruir a árvore do motor a cada frame é aceitável ou se ela tem de sobreviver
//! entre frames.
//!
//! Rodar: `cargo test -p ph2d-vec-layout --release -- --ignored --nocapture`
//!
//! ⚠️ `--release` não é preferência: o `dev` deste workspace é `opt-level = 0` e mediria o PERFIL do
//! build, não o motor. O número que decide produto sai do perfil em que o produto roda.

use ph2d_vec_layout::*;
use std::time::Instant;

/// Uma linha achatada com `n` filhos — a barra de ferramentas, no limite.
fn flat(n: usize) -> Vec<Node> {
    let mut v = Vec::with_capacity(n + 1);
    v.push(Node {
        parent: None,
        frame: Some(FrameStyle {
            dir: Dir::RowWrap,
            gap: [4.0, 4.0],
            ..FrameStyle::default()
        }),
        item: ItemStyle::default(),
        size: [1000.0, 1000.0],
    });
    for _ in 0..n {
        v.push(Node {
            parent: Some(0),
            frame: None,
            item: ItemStyle::default(),
            size: [20.0, 10.0],
        });
    }
    v
}

/// Molduras ANINHADAS `depth` níveis, cada uma com 4 filhos — o caso que exercita a recursão do
/// motor em vez da largura.
fn nested(depth: usize) -> Vec<Node> {
    let mut v = vec![Node {
        parent: None,
        frame: Some(FrameStyle {
            dir: Dir::Column,
            gap: [2.0, 2.0],
            ..FrameStyle::default()
        }),
        item: ItemStyle::default(),
        size: [1000.0, 1000.0],
    }];
    let mut cur = 0usize;
    for _ in 0..depth {
        for _ in 0..3 {
            v.push(Node {
                parent: Some(cur),
                frame: None,
                item: ItemStyle::default(),
                size: [20.0, 10.0],
            });
        }
        v.push(Node {
            parent: Some(cur),
            frame: Some(FrameStyle {
                dir: Dir::Row,
                gap: [2.0, 2.0],
                ..FrameStyle::default()
            }),
            item: ItemStyle {
                grow: 1.0,
                ..ItemStyle::default()
            },
            size: [100.0, 100.0],
        });
        cur = v.len() - 1;
    }
    v
}

fn bench(label: &str, nodes: &[Node]) {
    // Aquecimento (o primeiro passe paga alocação de arena que os seguintes reusam).
    for _ in 0..5 {
        let _ = solve(nodes).expect("resolve");
    }
    const N: u32 = 200;
    let t = Instant::now();
    for _ in 0..N {
        let _ = solve(nodes).expect("resolve");
    }
    let per = t.elapsed().as_secs_f64() * 1000.0 / f64::from(N);
    println!("{label:<34} {:>5} nos   {per:>8.4} ms/passe", nodes.len());
}

/// A tabela que decide o teto — e ela é impressa, não afirmada.
#[test]
#[ignore]
fn measure_the_layout_pass() {
    println!("\n== M2: o passe (arvore RECONSTRUIDA a cada chamada) ==");
    for n in [10, 100, 1000] {
        bench(&format!("linha achatada, {n} filhos"), &flat(n));
    }
    for d in [4, 16, 64] {
        bench(&format!("molduras aninhadas, prof. {d}"), &nested(d));
    }
}

/// **De que é feito o passe:** montar a árvore do motor × calcular o layout.
///
/// É esta divisão que responde à pergunta do plano (*"reconstruída × memoizada"*): se montar for a
/// metade grande, memoizar a árvore paga-se; se for o cálculo, memoizar não compra nada e a API
/// simples fica.
#[test]
#[ignore]
fn measure_build_versus_compute() {
    println!("\n== M2: montar a arvore x calcular ==");
    for n in [10, 100, 1000] {
        let nodes = flat(n);
        // O `solve` inteiro.
        for _ in 0..5 {
            let _ = solve(&nodes).expect("resolve");
        }
        const N: u32 = 200;
        let t = Instant::now();
        for _ in 0..N {
            let _ = solve(&nodes).expect("resolve");
        }
        let whole = t.elapsed().as_secs_f64() * 1000.0 / f64::from(N);

        // Só a MONTAGEM: a mesma árvore, sem o cálculo. `solve_build_only` não existe na API de
        // propósito (seria uma porta que não serve ninguém), então a sonda mede o custo pela
        // diferença entre o passe inteiro e um passe com a árvore de UM nó — que monta quase nada e
        // calcula quase nada.
        let trivial = flat(0);
        for _ in 0..5 {
            let _ = solve(&trivial).expect("resolve");
        }
        let t = Instant::now();
        for _ in 0..N {
            let _ = solve(&trivial).expect("resolve");
        }
        let floor = t.elapsed().as_secs_f64() * 1000.0 / f64::from(N);
        println!(
            "{n:>5} filhos: passe {whole:>8.4} ms   piso (1 no) {floor:>8.4} ms   \
             por filho {:>8.6} ms",
            (whole - floor) / f64::from(u32::try_from(n).unwrap_or(1))
        );
    }
}
