//! A8 — uma janela TAPADA mais curta que `1/32` de um segmento longo passa entre duas amostras?
//!
//! ⚠️ A régua é a própria pergunta da lei ([`Posada::tapado`]) amostrada DENSA (`2 048` por
//! segmento): o que se mede é o erro da AMOSTRAGEM do recorte, não a lei.

use super::super::{Posada, a_vista, avalia, cubica};
use ph2d_vec_scene::VecPath;

/// As janelas tapadas que a amostragem do recorte deixa passar em `fonte` (riscas abertas): o
/// comprimento POSADO de cada uma.
fn janelas_perdidas(fonte: &VecPath, f: &Posada<'_>) -> Vec<f64> {
    const DENSO: usize = 2048;
    let mut out = Vec::new();
    for c in 0..fonte.contour_count() {
        let Some((v, false)) = fonte.contour(c) else {
            continue;
        };
        if v.len() < 2 {
            continue;
        }
        let vis = a_vista(v, f).unwrap_or_else(|| {
            #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
            let fim = (v.len() - 1) as f64;
            vec![(0.0, fim)]
        });
        for k in 0..v.len() - 1 {
            let cb = cubica(v, k);
            let mut inicio: Option<usize> = None;
            for i in 0..=DENSO {
                #[expect(clippy::cast_precision_loss, reason = "amostra")]
                let t = i as f64 / DENSO as f64;
                let tap = f.tapado(avalia(&cb, t));
                match (tap, inicio) {
                    (true, None) => inicio = Some(i),
                    (false, Some(a)) => {
                        inicio = None;
                        #[expect(clippy::cast_precision_loss, reason = "índice")]
                        let (ua, ub) = (k as f64 + a as f64 / DENSO as f64, k as f64 + t);
                        let meio = 0.5 * (ua + ub);
                        if vis.iter().any(|&(x, y)| meio > x && meio < y) {
                            out.push(f.comprimento(v, ua, ub));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    out
}

/// ⭐ **SONDA — A8:** de `60°` a `170°` de `0,5` em `0,5°`, nas fixturas com riscas, as janelas
/// tapadas que o recorte deixa passar e o maior comprimento posado delas.
#[test]
#[ignore = "sonda: imprime"]
fn diag_as_janelas_que_passam_entre_amostras() {
    for (nome, fixtura) in [
        (
            "riscas",
            super::rapida::riscas_dobradas as fn(f32) -> super::rapida::Fixtura,
        ),
        ("barra em S", super::barra_em_s),
    ] {
        let (mut n, mut maior, mut onde) = (0, 0.0_f64, 0.0_f32);
        let mut graus = 60f32;
        while graus <= 170.0 {
            let (fonte, _, campo, pele, prof) = fixtura(graus);
            let f = Posada::nova(&campo, None, &pele, &[], true, &prof)
                .expect("posada")
                .com_a_arte(&fonte);
            for l in janelas_perdidas(&fonte, &f) {
                n += 1;
                if l > maior {
                    (maior, onde) = (l, graus);
                }
            }
            graus += 0.5;
        }
        let largura = fixtura(0.0).0.stroke.as_ref().map_or(0.0, |s| s.width);
        println!(
            "  {nome}: {n} janelas perdidas; a maior {maior:.4} (largura do traço {largura}) a {onde}°"
        );
    }
}
