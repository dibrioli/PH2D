//! ⏱ **A RÉGUA da equipa assíncrona e da saída** (W8, `docs/MiroClone/06_equipa_e_saida.md` §4),
//! antes de qualquer tecto:
//!
//! - **guardar uma versão** de um quadro de 10 mil / 100 mil elementos (notas; fluxograma com setas):
//!   o documento clonado em memória, os bytes postcard, e os bytes comprimidos (deflate rápido e por
//!   omissão) — bytes e ms, e o caminho de volta (restaurar = ler os bytes).
//!
//! Variantes no MESMO processo, rodadas intercaladas com a ordem rodada, vale o MÍNIMO (mediana ao
//! lado). Corre à mão:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --release --test it measure_team -- --ignored --nocapture
//! ```

use super::measure_encode_cost::{flow, notes};
use super::measure_gpu_raster_cost::min_med;
use ph2d_board_model::{BoardDoc, BoardSet};
use std::io::Write as _;
use std::time::Instant;

const ROUNDS: usize = 7;

#[derive(Clone, Copy, Debug)]
enum Keep {
    /// O documento clonado (a versão em memória, sem bytes).
    Clone,
    /// Os bytes postcard do quadro.
    Bytes,
    /// Postcard + deflate rápido (nível 1).
    Fast,
    /// Postcard + deflate por omissão (nível 6).
    Deflate,
}

const KEEPS: [Keep; 4] = [Keep::Clone, Keep::Bytes, Keep::Fast, Keep::Deflate];

fn deflate(bytes: &[u8], level: u32) -> Vec<u8> {
    let mut z = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::new(level));
    z.write_all(bytes).unwrap();
    z.finish().unwrap()
}

fn inflate(bytes: &[u8]) -> Vec<u8> {
    let mut z = flate2::write::DeflateDecoder::new(Vec::new());
    z.write_all(bytes).unwrap();
    z.finish().unwrap()
}

/// Guarda uma versão por `keep`; devolve (ms a guardar, bytes, ms a restaurar).
fn keep_once(set: &BoardSet, keep: Keep) -> (f64, usize, f64) {
    let t = Instant::now();
    let (size, payload): (usize, Option<Vec<u8>>) = match keep {
        Keep::Clone => {
            let doc: BoardDoc = set.boards()[0].doc.clone();
            std::hint::black_box(&doc);
            (0, None)
        }
        Keep::Bytes => {
            let b = set.to_bytes().unwrap();
            (b.len(), Some(b))
        }
        Keep::Fast | Keep::Deflate => {
            let level = if matches!(keep, Keep::Fast) { 1 } else { 6 };
            let b = deflate(&set.to_bytes().unwrap(), level);
            (b.len(), Some(b))
        }
    };
    let save = t.elapsed().as_secs_f64() * 1e3;
    let t = Instant::now();
    if let Some(p) = payload {
        let raw = if matches!(keep, Keep::Bytes) {
            p
        } else {
            inflate(&p)
        };
        std::hint::black_box(BoardSet::from_bytes(&raw).unwrap());
    }
    (save, size, t.elapsed().as_secs_f64() * 1e3)
}

#[test]
#[ignore = "régua: corre-se à mão em --release (ver o cabeçalho)"]
fn measure_team_cost() {
    let boards: [(&str, BoardSet); 4] = [
        ("notas 10k", notes(10_000, false)),
        ("notas 100k", notes(100_000, false)),
        ("fluxo 10k", flow(10_000)),
        ("fluxo 100k", flow(100_000)),
    ];
    println!("\n== guardar uma versão (ms a guardar · bytes · ms a restaurar; mínimo [mediana]) ==");
    for (name, set) in &boards {
        let mut save = vec![Vec::new(); KEEPS.len()];
        let mut back = vec![Vec::new(); KEEPS.len()];
        let mut size = vec![0usize; KEEPS.len()];
        for r in 0..ROUNDS {
            for k in 0..KEEPS.len() {
                let i = (k + r) % KEEPS.len();
                let (s, n, b) = keep_once(set, KEEPS[i]);
                save[i].push(s);
                back[i].push(b);
                size[i] = n;
            }
        }
        for (i, keep) in KEEPS.iter().enumerate() {
            let (sm, sd) = min_med(&save[i]);
            let (bm, bd) = min_med(&back[i]);
            println!(
                "{name:>11} {keep:>8?}: {sm:8.2} [{sd:8.2}] ms · {:>11} B · {bm:8.2} [{bd:8.2}] ms",
                size[i]
            );
        }
    }
}
