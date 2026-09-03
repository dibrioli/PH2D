//! ⭐ **O que custa CLONAR a cena vetorial** — o número que a F8 nomeia e nunca mediu.
//!
//! ```text
//! cargo test -p ph2d-vec-scene --release --test measure_scene_clone -- --ignored --nocapture
//! ```
//!
//! ⚠️ **`#[ignore]`**: mede um relógio (família de flakes do `CLAUDE.md` §5.0) e imprime o `load`.
//!
//! # Porque este número importa mais do que o do restauro
//!
//! O `ProjectState::capture` do shell corre **em todo quadro com input** e faz `vec.clone()`. O
//! restauro, ao lado, corre **uma vez por Ctrl+Z**. ⇒ um custo `O(documento)` aqui é pago 60×/s
//! enquanto o artista arrasta; lá é pago quando ele carrega numa tecla.
//!
//! ⛔ **A F2 mediu a captura do MUNDO (`0,189 ms`) e não esta.** *Uma fase que mede metade de uma
//! porta conclui sobre a porta inteira* — é a mesma forma do achado de 2026-09-02 sobre a fixtura
//! que já tinha o fenómeno e não lhe fazia a pergunta.
//!
//! # E a RESIDÊNCIA, que é a segunda pergunta
//!
//! A pilha guarda `UNDO_CAP = 256` estados, **cada um com uma cena inteira**. O plano nomeia isto
//! (*«responde ao relógio de codificar, não à memória de guardar»*) e deixa-o por medir; aqui ele
//! é medido pelo tamanho serializado de uma cena, que é o limite inferior honesto do que 256 cópias
//! ocupam.

use ph2d_vec_scene::{VecPath, VecScene, VecVertex};
use std::time::Instant;

const ITERS: usize = 25;

fn load_average() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(f64::NAN)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
    v[v.len() / 2]
}

/// Uma cena de `n` formas, cada uma um **quadrado de quatro vértices** — a forma mais barata que
/// um artista de facto desenha, para que o número seja um **piso** e não um caso escolhido.
fn scene_of(n: usize) -> VecScene {
    let mut scene = VecScene::new();
    for i in 0..n {
        let k = i as f64;
        let p = VecPath {
            verts: vec![
                VecVertex::corner([k, 0.0]),
                VecVertex::corner([k + 1.0, 0.0]),
                VecVertex::corner([k + 1.0, 1.0]),
                VecVertex::corner([k, 1.0]),
            ],
            closed: true,
            ..VecPath::default()
        };
        scene.push_path(p);
    }
    scene
}

#[test]
#[ignore = "mede um relogio; rode com a maquina calma e --release"]
fn cloning_the_scene_costs_the_document_and_this_is_the_number() {
    println!(
        "\n  clone da VecScene (o que a captura faz em TODO quadro com input) — mediana de {ITERS}, load = {:.2}",
        load_average()
    );
    println!("  ┌────────┬──────────┬──────────────────┬───────────────┐");
    println!("  │ formas │  clone   │ % de um quadro   │ bytes/estado  │");
    println!("  ├────────┼──────────┼──────────────────┼───────────────┤");
    for n in [10usize, 100, 1_000, 5_000] {
        let scene = scene_of(n);
        let mut samples = Vec::with_capacity(ITERS);
        for _ in 0..ITERS {
            let t0 = Instant::now();
            let c = scene.clone();
            samples.push(t0.elapsed().as_secs_f64() * 1000.0);
            std::hint::black_box(&c);
        }
        let ms = median(samples);
        let bytes = postcard::to_allocvec(&scene).map(|v| v.len()).unwrap_or(0);
        println!(
            "  │ {n:>6} │ {ms:>6.3} ms │ {:>15.1} % │ {bytes:>13} │",
            ms / 16.7 * 100.0
        );
    }
    println!("  └────────┴──────────┴──────────────────┴───────────────┘");
    println!("  (a captura do MUNDO, para comparar: 0,189 ms parada — F2, 2026-08-25)");
    println!("  (a pilha guarda UNDO_CAP = 256 estados, cada um com uma cena inteira)\n");
}
