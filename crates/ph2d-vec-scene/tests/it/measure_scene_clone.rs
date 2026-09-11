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

/// ⭐⭐⭐ **O QUE UM PASSO DE DESENHO VECTORIAL DE FACTO MUDA** — a pergunta irmã da F8.3.
///
/// ```text
/// cargo test -p ph2d-vec-scene --release --test measure_scene_clone -- --ignored --nocapture
/// ```
///
/// # Porque ela é feita AQUI, e o que ela decide
///
/// A F8.1 pôs um `Arc` na cena **inteira**: dois passos que não tocam o vector partilham um
/// documento. ⚠️ **E o resíduo é o mesmo que o Flip tinha:** numa sessão de desenho **cada passo
/// muda a cena**, e aí o `Arc` do documento não compra nada.
///
/// No Flip a resposta foi partilhar por **DESENHO** (`99,0 %` de desperdício a 96 quadros, a pilha
/// de `912 MB` para `9,5`). ⚠️ **A pergunta simétrica aqui não tem a mesma resposta de antemão**, e
/// o preço da cura também não: o `FlipObject::drawings` era privado com três acessores e **dez**
/// chamadores externos; o `VecScene::paths` tem **766**, e o `paths_mut() -> &mut [VecPath]` é
/// inexprimível sobre `[Arc<VecPath>]`. ⇒ *o ganho tem de pagar isso, e é por isso que ele se mede
/// antes.*
///
/// # ⛔⛔ RECUSA MEDIDA (2026-09-08) — os DOIS lados, para quem decidir depois
///
/// **O ganho é real e é de MEMÓRIA, não de relógio:**
/// - desperdício de `90,0 %` a 10 formas, `99,0 %` a 100, `99,9 %` a 1 000 e **`100,0 %` a 5 000**;
/// - a pilha de `256` estados iria de **`296,6 MB` para `0,1 MB`** a 5 000 formas;
/// - ⚠️ **o relógio já está bem**: o clone custa `0,100 ms` a 5 000 formas — `0,6 %` de um quadro.
///   *Esta cura não compra velocidade nenhuma; compra residência.*
///
/// **O preço foi MEDIDO, não estimado:**
/// - trocar `paths: Vec<VecPath>` por `Vec<Arc<VecPath>>` dá **43 erros em 7 ficheiros** da própria
///   crate — e ⚠️ **esse número é um piso enganoso**: um `cargo check` que pára na crate de baixo
///   nunca chega às de cima, e há **670** usos de `.paths()` fora dela;
/// - `paths_mut() -> &mut [VecPath]` é **inexprimível** sobre `[Arc<VecPath>]`;
/// - funções de **outras crates** pedem `&[VecPath]` (`ph2d-vec-boolean`, `ph2d-vec-blend`), e
///   passar-lhes a fatia deixa de compilar;
/// - a variante cirúrgica — o `Arc` no campo que de facto pesa (`VecPath::verts`) — tem `1 803`
///   usos, dos quais `~93` mutáveis, e o campo é **`pub`**.
///
/// **⇒ A recusa, e o que a reabre.** O `ph2d-vec-scene` é a espinha do módulo Vector, e o §F4.6c
/// pagou esta lição por escrito: mexer numa espinha que outra linha tem viva **é catástrofe de
/// merge**, e foi isso que bloqueou aquela fase durante dez dias. O caso que dói aqui — uma sessão
/// vetorial longa com milhares de formas — custa `296 MB`, contra os `2,3 GB` que o Flip custava
/// antes da F8.3. ⚠️ **Reabre quando:** a `line/Vector` estiver parada (a janela em que o custo de
/// merge cai a zero), **ou** quando alguém medir uma cena real acima de ~5 000 formas — que é onde
/// esta tabela deixa de ser um exercício.
#[test]
#[ignore = "mede memoria; irmao do de cima"]
fn only_the_touched_path_needs_to_be_new() {
    println!("\n  o que um PASSO de desenho vectorial muda, contra o que ele CLONA hoje");
    println!("  fixtura: quadrados de 4 vertices (o piso do que um artista desenha)");
    println!("  ┌────────┬───────────────┬───────────────┬──────────────┬──────────────────────┐");
    println!("  │ formas │ a cena INTEIRA│ UMA forma     │ desperdicio  │ x256 na pilha        │");
    println!("  ├────────┼───────────────┼───────────────┼──────────────┼──────────────────────┤");
    const UNDO_CAP: usize = 256;
    let vazia = postcard::to_allocvec(&VecScene::new())
        .map(|v| v.len())
        .unwrap_or(0);
    let uma = postcard::to_allocvec(&scene_of(1))
        .map(|v| v.len())
        .unwrap_or(0);
    let forma = uma.saturating_sub(vazia);
    for n in [10usize, 100, 1_000, 5_000] {
        let inteira = postcard::to_allocvec(&scene_of(n))
            .map(|v| v.len())
            .unwrap_or(0);
        let pct = if inteira > 0 {
            100.0 - (forma as f64 / inteira as f64) * 100.0
        } else {
            0.0
        };
        let mb_hoje = (inteira * UNDO_CAP) as f64 / (1024.0 * 1024.0);
        let mb_part = (forma * UNDO_CAP) as f64 / (1024.0 * 1024.0);
        println!(
            "  │ {n:>6} │ {inteira:>13} │ {forma:>13} │ {pct:>11.1} % │ {mb_hoje:>7.1} → {mb_part:>6.1} MB │"
        );
    }
    println!("  └────────┴───────────────┴───────────────┴──────────────┴──────────────────────┘");
    println!("  (o Flip, para comparar: 99,0 % de desperdicio a 96 quadros — F8.3, 08/09)");
    println!("  ⛔ RECUSA MEDIDA (2026-09-08): o ganho e' de MEMORIA, nao de relogio, e o preco");
    println!("     toca uma espinha de outras linhas. Ver o doc deste teste.\n");
}
