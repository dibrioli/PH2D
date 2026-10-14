//! ⭐ W5q — a `c2` do High por ETAPA da holográfica (traça · sobe · desce · soma · saída, por nível):
//! o fim do [`crate::medida_luz2d_w5o::medida_luz2d_w5o_lastro`]. Com `PH2D_W5Q_FASES=1`, depois da
//! `c2` inteira, o mesmo solver passa ao instrumento ([`ph2d_light2d_gpu::Solver::medir_fases_da_hrc`],
//! um passe por despacho) e imprime o mínimo e a mediana de cada (etapa, nível) em 12 quadros.

use ph2d_light2d_ecs::CenaDeLuz;

use crate::medida_luz2d_w5o::{Par, mediana, minimo};

const QUADROS: usize = 12;

pub(crate) fn a_c2_e_as_etapas(
    gpu: &ph2d_gpu::GpuContext,
    mut p: Par,
    cena: &CenaDeLuz,
    rotulo: &str,
) {
    let mut c2 = Vec::new();
    for _ in 0..QUADROS {
        p.quadro(gpu, cena);
        c2.push(p.tempos(gpu).0.c2_total());
    }
    eprintln!(
        "C {rotulo} · c2 High mín {:.3} · med {:.3} ms",
        minimo(&c2),
        mediana(&c2)
    );
    if std::env::var("PH2D_W5Q_FASES").as_deref() != Ok("1") {
        return;
    }
    assert!(p.vista.medir_fases_da_hrc(gpu), "TIMESTAMP_QUERY");
    let mut por: Vec<(&'static str, u32, Vec<f64>, usize)> = Vec::new();
    for _ in 0..QUADROS {
        p.quadro(gpu, cena);
        for (etapa, nivel, ms, n) in p.vista.fases_da_hrc(gpu).expect("fases") {
            match por.iter_mut().find(|o| o.0 == etapa && o.1 == nivel) {
                Some(o) => o.2.push(ms),
                None => por.push((etapa, nivel, vec![ms], n)),
            }
        }
    }
    let (mut soma_min, mut soma_med) = (0.0, 0.0);
    for (etapa, nivel, ms, n) in &por {
        soma_min += minimo(ms);
        soma_med += mediana(ms);
        eprintln!(
            "F {rotulo} · {etapa} n{nivel} · mín {:.3} · med {:.3} ms · {n} despachos",
            minimo(ms),
            mediana(ms)
        );
    }
    eprintln!("S {rotulo} · soma das etapas mín {soma_min:.3} · med {soma_med:.3} ms");
}
