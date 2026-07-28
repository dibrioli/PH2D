//! **Quanto custa capturar o "antes" por REGIÃO** — o número que decide se o S3 vale a wave inteira
//! (doc 28 §7 · §5.20 item 3).
//!
//! O S3 troca **uma cópia do plano inteiro** (o fork do pen-down: 11,7 ms medidos a 4096²) por **uma
//! cópia dos tiles que o passo de fato escreve**. A troca só compensa se a segunda for muito menor — e
//! *"muito menor"* é uma afirmação sobre um número, não sobre uma arquitetura.
//!
//! ⚠️ **Isto mede o PRIMITIVO, não o produto, e a distinção é honesta porque ainda não há produto:** o
//! journal é rede de verificação (debug-only) e o commit continua derivando o `before` de dois
//! snapshots. O que a sonda responde é a pergunta que precede a construção — *se eu trocar, troco por
//! quanto?* — com a geometria REAL de um traço em vez de um palpite.
//!
//! Rodar:
//!
//! ```text
//! cargo test -p ph2d-tool-painter --release what_a_region_journal_costs -- --ignored --nocapture
//! ```

use crate::undo::journal::TileJournal;
use std::sync::Arc;
use std::time::Instant;

fn best(mut f: impl FnMut() -> f64) -> f64 {
    (0..5).map(|_| f()).fold(f64::MAX, f64::min)
}

/// A pegada de um dab em ELEMENTOS do plano RGBA — `(x0, y0, x1, y1)`, meio-aberta.
fn dab_area(px: f32, py: f32, r: f32, side: usize) -> (usize, usize, usize, usize) {
    let lo = |v: f32| v.max(0.0) as usize;
    let x0 = lo(px - r).min(side) * 4;
    let x1 = ((px + r).max(0.0) as usize).min(side) * 4;
    let y0 = lo(py - r).min(side);
    let y1 = ((py + r).max(0.0) as usize).min(side);
    (x0, y0, x1, y1)
}

/// **A cópia por REGIÃO contra a cópia por PLANO** — os dois números lado a lado, na mesma tela.
///
/// Três geometrias, porque a resposta depende do gesto e um número só esconderia isso:
///
/// - **traço curto** (o caso comum: um risco de ~600 px),
/// - **traço que atravessa a tela** (o pior caso honesto de um gesto),
/// - **`None`** (o sítio que não sabe onde escreve) — que TEM de custar o que o fork custa, senão a
///   política *"quem não sabe passa o plano inteiro"* seria uma regressão em vez de um fallback.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_a_region_journal_costs_against_the_fork_it_replaces() {
    eprintln!("\n[journal] captura por REGIAO x fork do PLANO (o que o S3 troca por que)");
    for side in [2048usize, 4096] {
        let n = side * side * 4;
        let canvas: Arc<Vec<u8>> = Arc::new(vec![200u8; n]);
        let mb = n as f64 / (1024.0 * 1024.0);

        // A linha de base: o fork que o S3 remove. Um segundo dono força a cópia, exatamente como o
        // `cursor` do histórico faz em repouso (medido, §5.20: canvas com DOIS donos).
        let src = Arc::clone(&canvas);
        let fork = best(|| {
            let mut a = Arc::clone(&src);
            let _keep = Arc::clone(&src);
            let t0 = Instant::now();
            let v = crate::plane_copy::par_clone(&a);
            a = Arc::new(v);
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            std::hint::black_box(a.len());
            dt
        });

        for (name, from, to, r, events) in [
            ("traco curto  ", 200.0f32, 800.0f32, 100.0f32, 20usize),
            ("traco na tela", 100.0, (side - 100) as f32, 100.0, 60),
        ] {
            let (ms, bytes) = {
                let mut ms = f64::MAX;
                let mut bytes = 0usize;
                for _ in 0..5 {
                    let mut j: TileJournal<u8> = TileJournal::default();
                    let t0 = Instant::now();
                    for k in 0..=events {
                        let f = k as f32 / events as f32;
                        let p = from + (to - from) * f;
                        j.capture(&canvas, side * 4, Some(dab_area(p, p, r, side)));
                    }
                    let dt = t0.elapsed().as_secs_f64() * 1000.0;
                    bytes = j.heap_bytes();
                    ms = ms.min(dt);
                }
                (ms, bytes)
            };
            eprintln!(
                "  {side}²  {name}  captura {ms:>7.2} ms · retem {:>6.1} MB   (fork {fork:>7.2} ms · \
                 plano {mb:.0} MB)  ⇒ {:.1}× mais barato",
                bytes as f64 / (1024.0 * 1024.0),
                fork / ms.max(1e-9),
            );
        }

        // O fallback: quem não sabe onde escreve captura o plano inteiro. Ele NÃO pode ser mais caro que
        // o fork — se for, a política de falha do S1 (*lento, nunca errado*) deixa de valer.
        let whole = best(|| {
            let mut j: TileJournal<u8> = TileJournal::default();
            let t0 = Instant::now();
            j.capture(&canvas, side * 4, None);
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            std::hint::black_box(j.heap_bytes());
            dt
        });
        eprintln!(
            "  {side}²  None (nao sei)   captura {whole:>7.2} ms · retem {mb:>6.1} MB   (fork \
             {fork:>7.2} ms)  ⇒ {:.2}× do fork",
            whole / fork,
        );
    }
    eprintln!();
}
