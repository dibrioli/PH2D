//! A10 — a ponta do traço na ponta do VINCO. ⚠️ **A régua é a CONVERGÊNCIA:** a mesma lei com a
//! malha fina cada vez mais fina; a referência é o lado `32` (o desvio da recta cai `4×` por dobra,
//! a `32` fica `~10⁻³` do de hoje). Mede-se onde acaba cada trecho do traço contra a referência.

use super::super::super::frente::DIV_FIXO;
use super::{LARGURA, desenho_aqui, polilinhas};
use crate::skin_desenho::SkinDesenhado;

/// O desenho numa thread NOVA (o memo do quadro é por thread) com o lado `div` (`None` = a lei).
fn desenho(graus: f32, div: Option<usize>) -> SkinDesenhado {
    std::thread::scope(|s| {
        s.spawn(|| {
            DIV_FIXO.with(|c| c.set(div));
            desenho_aqui(graus)
        })
        .join()
        .expect("thread do desenho")
    })
}

/// As pontas dos trechos ABERTOS da camada do traço (um fechado inteiro não tem ponta).
fn pontas(d: &SkinDesenhado) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    for x in d.values() {
        let Some(t) = &x.traco else { continue };
        let mut abertos = t.clone();
        abertos.subpaths.retain(|c| !c.closed);
        if abertos.closed {
            abertos.verts.clear();
        }
        for l in polilinhas(&abertos, false) {
            if l.len() > 2 {
                out.extend([l[0], l[l.len() - 1]]);
            }
        }
    }
    out
}

/// Para cada ponta de `a`, a distância à ponta mais perto de `b`, em larguras (ordenadas).
fn desvios(a: &[[f64; 2]], b: &[[f64; 2]]) -> Vec<f64> {
    let mut v: Vec<f64> = a
        .iter()
        .map(|p| {
            b.iter()
                .map(|q| (p[0] - q[0]).hypot(p[1] - q[1]))
                .fold(f64::MAX, f64::min)
                / LARGURA
        })
        .collect();
    v.sort_by(f64::total_cmp);
    v
}

/// ⭐ **SONDA — a ponta do vinco converge?** Por pose, o lado fixo `1…16` e a lei contra o `32`:
/// as pontas, o maior desvio em larguras e o tempo do desenho.
#[test]
#[ignore = "sonda: imprime"]
fn diag_a_ponta_do_vinco_converge() {
    for graus in [110f32, 130.0, 150.0, 160.0, 170.0, 175.0] {
        let refe = pontas(&desenho(graus, Some(32)));
        println!("  {graus}°: {} pontas na referência", refe.len());
        for div in [Some(1), Some(2), Some(4), Some(8), Some(16), None] {
            let t0 = std::time::Instant::now();
            let d = desenho(graus, div);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            let p = pontas(&d);
            let (ida, volta) = (desvios(&p, &refe), desvios(&refe, &p));
            let max = ida.iter().chain(&volta).copied().fold(0.0, f64::max);
            println!(
                "    lado {div:?}: {} pontas · maior desvio {max:.3} larg. · {ms:.1} ms · {:.2?}",
                p.len(),
                ida
            );
        }
    }
}
