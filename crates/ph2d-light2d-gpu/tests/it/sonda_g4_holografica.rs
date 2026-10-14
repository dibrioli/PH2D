//! **Sonda do G4 na holográfica** (à mão): de onde vem a variação do custo entre 1 e 10 000 luzes.
//! Três réguas no MESMO processo, intercaladas, vale o mínimo (`docs/DevOps/MEDIR_VELOCIDADE.md`):
//!
//! - `cinco solvers` — a do gate: um solver por contagem, cada um com os seus buffers;
//! - `um solver` — as cinco cenas no MESMO solver (os mesmos buffers): só o número de luzes muda;
//! - `cinco, uma vez` — um solver por contagem com `hrc_nasce_duas_vezes = false`.

use ph2d_light2d_gpu::{Config, HOLOGRAFICA_PADRAO, Solver, TemposMs};

use crate::cenas::{DEGRAUS, enxame_de_luzes};
use crate::comum::{gpu, grade};
use crate::holografica::com;
use crate::medicao::loadavg;

const CONTAGENS: [u32; 5] = [1, 10, 100, 1_000, 10_000];
const RODADAS: usize = 15;

fn variacao(v: &[f64]) -> f64 {
    let (min, max) = v
        .iter()
        .fold((f64::INFINITY, 0.0_f64), |(a, b), &x| (a.min(x), b.max(x)));
    100.0 * (max - min) / min
}

fn minimo(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::INFINITY, f64::min)
}

#[test]
#[ignore = "sonda: imprime"]
fn sonda_g4_holografica_de_onde_vem_a_variacao() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[0];
    let cfg = |duas: bool| Config {
        hrc_nasce_duas_vezes: duas,
        ..com(Config::padrao(grade(lado, texel)), Some(HOLOGRAFICA_PADRAO))
    };
    let cenas: Vec<_> = CONTAGENS.iter().map(|&n| enxame_de_luzes(n)).collect();
    let mut cinco: Vec<Solver> = CONTAGENS
        .iter()
        .map(|_| Solver::novo(&gpu, cfg(true)).expect("solver"))
        .collect();
    let mut um = Solver::novo(&gpu, cfg(true)).expect("solver");
    let mut uma_vez: Vec<Solver> = CONTAGENS
        .iter()
        .map(|_| Solver::novo(&gpu, cfg(false)).expect("solver"))
        .collect();
    let mede = |s: &mut Solver, k: usize| -> Option<TemposMs> {
        // Duas voltas: a primeira depois de trocar de cena aquece o que a cena muda.
        s.quadro(&gpu, &cenas[k]);
        s.tempos(&gpu)?;
        s.quadro(&gpu, &cenas[k]);
        s.tempos(&gpu)
    };
    let n = CONTAGENS.len();
    let mut a: Vec<Vec<f64>> = vec![Vec::new(); n];
    let mut b: Vec<Vec<f64>> = vec![Vec::new(); n];
    let mut c: Vec<Vec<f64>> = vec![Vec::new(); n];
    let carga = loadavg();
    for r in 0..=RODADAS {
        for o in 0..n {
            let k = (o + r) % n;
            let (Some(ta), Some(tb), Some(tc)) = (
                mede(&mut cinco[k], k),
                mede(&mut um, k),
                mede(&mut uma_vez[k], k),
            ) else {
                eprintln!("sem TIMESTAMP_QUERY");
                return;
            };
            // A rodada 0 é o aquecimento.
            if r > 0 {
                a[k].push(ta.rc());
                b[k].push(tb.rc());
                c[k].push(tc.rc());
            }
        }
    }
    for (nome, v) in [
        ("cinco solvers", a),
        ("um solver", b),
        ("cinco, uma vez", c),
    ] {
        let m: Vec<f64> = v.iter().map(|x| minimo(x)).collect();
        eprintln!(
            "SONDA G4 · {nome} · loadavg {carga} · {m:.4?} ms · variação {:.2} %",
            variacao(&m)
        );
    }
}
