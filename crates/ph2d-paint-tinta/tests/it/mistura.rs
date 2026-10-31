//! A lei de MISTURA (Oklab pesado pelo volume): associatividade, N passadas = uma, deriva 0 em 20
//! vidrados, Lema 1 em L Oklab (nunca abaixo da mais escura), ganho de ruído ±1/255.

use ph2d_paint_tinta::oklab::{linear_para_oklab, oklab_para_srgb};
use ph2d_paint_tinta::{Bandas, R_INF_MIN, Soma, Tinta};

use crate::CORES;

const VOLS: [f32; 5] = [0.01, 0.3, 4.0, 40.0, 400.0];

fn tintas() -> Vec<Tinta> {
    CORES
        .iter()
        .enumerate()
        .map(|(i, &c)| Tinta::de_srgb(c, i as f32 / 9.0))
        .collect()
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn cor(s: Soma) -> Tinta {
    s.cor().expect("volume > 0")
}

fn u8s(lab: [f32; 3]) -> [u8; 3] {
    oklab_para_srgb(lab).map(|c| (c * 255.0).round() as u8)
}

#[test]
fn a_mistura_e_associativa_e_comutativa() {
    let ts = tintas();
    let mut pior = 0.0f32;
    let mut n = 0;
    for (i, &a) in ts.iter().enumerate() {
        for (j, &b) in ts.iter().enumerate().skip(i + 1) {
            for &c in ts.iter().skip(j + 1) {
                for &(va, vb, vc) in &[(0.3, 40.0, 4.0), (400.0, 0.01, 4.0), (1.0, 1.0, 1.0)] {
                    let (sa, sb, sc) = (Soma::de(a, va), Soma::de(b, vb), Soma::de(c, vc));
                    let r0 = cor(sa.junta(sb).junta(sc)).lab;
                    for r in [
                        sa.junta(sb.junta(sc)),
                        sc.junta(sb).junta(sa),
                        sb.junta(sa.junta(sc)),
                        sc.junta(sa).junta(sb),
                    ] {
                        pior = pior.max(dist(cor(r).lab, r0));
                    }
                    n += 1;
                }
            }
        }
    }
    println!("associatividade: {n} trios × 5 ordens · max ΔOklab {pior:.2e}");
    // Arredondamento de 2 somas f32 de grandezas ≤ 400·1: ~4 ulp relativos da cor (≤ 1).
    assert!(pior <= 1.0e-6, "{pior}");
}

#[test]
fn n_passadas_sao_uma_e_vinte_vidrados_nao_derivam() {
    let ts = tintas();
    let (mut pior_n, mut pior_d) = (0.0f32, 0.0f32);
    for &a in &ts {
        // Deriva: 20 vidrados de 10 % da MESMA cor — o 8 bits nunca muda.
        let mut s = Soma::de(a, 1.0);
        for _ in 0..20 {
            s = s.junta(Soma::de(a, 0.1));
            assert_eq!(u8s(cor(s).lab), u8s(a.lab), "deriva em 8 bits");
            pior_d = pior_d.max(dist(cor(s).lab, a.lab));
        }
        for &b in &ts {
            // N passadas de v/N de B sobre A = uma passada de v.
            for &v in &VOLS {
                let uma = cor(Soma::de(a, 2.0).junta(Soma::de(b, v))).lab;
                let mut s = Soma::de(a, 2.0);
                for _ in 0..24 {
                    s = s.junta(Soma::de(b, v / 24.0));
                }
                pior_n = pior_n.max(dist(cor(s).lab, uma));
            }
        }
    }
    println!(
        "deriva 20 vidrados: max ΔOklab {pior_d:.2e} (0 em 8 bits) · 24 passadas × 1: {pior_n:.2e}"
    );
    // Cota analítica: N somas f32 no numerador e no denominador, ≤ ½ ulp relativo cada ⇒ ≤ N·ε
    // relativo sobre |Lab| ≤ 1 (24·ε = 2,9e-6; medido 31/10: 2,4e-6). 20 vidrados: 20·ε = 2,4e-6.
    assert!(
        pior_d <= 20.0 * f32::EPSILON && pior_n <= 24.0 * f32::EPSILON,
        "{pior_d} {pior_n}"
    );
}

#[test]
fn lema_1_a_mistura_nunca_fica_abaixo_da_mais_escura_em_l() {
    let ts = tintas();
    let mut n = 0;
    let mut pior = f32::INFINITY;
    for &a in &ts {
        for &b in &ts {
            for &va in &VOLS {
                for &vb in &VOLS {
                    let m = cor(Soma::de(a, va).junta(Soma::de(b, vb)));
                    let (lo, hi) = (a.lab[0].min(b.lab[0]), a.lab[0].max(b.lab[0]));
                    assert!(
                        m.lab[0] >= lo - 1e-6 && m.lab[0] <= hi + 1e-6,
                        "L fora: {m:?}"
                    );
                    pior = pior.min(m.lab[0] - lo);
                    n += 1;
                }
            }
            // Vidrado 20×10 % de B sobre A: L monótona de A para B, nunca abaixo da mais escura.
            let mut s = Soma::de(a, 1.0);
            let mut l_ant = a.lab[0];
            for _ in 0..20 {
                s = s.junta(Soma::de(b, 0.1));
                let l = cor(s).lab[0];
                let sobe = b.lab[0] >= a.lab[0];
                assert!(if sobe {
                    l >= l_ant - 1e-6
                } else {
                    l <= l_ant + 1e-6
                });
                assert!(l >= a.lab[0].min(b.lab[0]) - 1e-6);
                l_ant = l;
            }
        }
    }
    println!("Lema 1 (L Oklab): {n} misturas, min (L − L_mais_escura) = {pior:+.2e}");
}

#[test]
fn ruido_de_um_degrau_de_8_bits_nao_e_amplificado() {
    let base = |c: [f32; 3]| Tinta::de_srgb(c, 0.3);
    // A cor que a CAMADA usa: média Oklab → linear → corte a [R_INF_MIN, 1] → Oklab (para medir).
    let saida = |a: [f32; 3], b: [f32; 3], w: f32| {
        let m = cor(Soma::de(base(a), 1.0 - w).junta(Soma::de(base(b), w)));
        linear_para_oklab(Bandas(m.linear()).mapa(|_, r| r.clamp(R_INF_MIN, 1.0)).0)
    };
    let mut pior = 0.0f32;
    let mut n = 0;
    for &a in &CORES {
        for &b in &CORES {
            for w in [0.1, 0.5, 0.9] {
                let r0 = saida(a, b, w);
                for canal in 0..3 {
                    for d in [-1.0f32 / 255.0, 1.0 / 255.0] {
                        let mut ap = a;
                        ap[canal] = (ap[canal] + d).clamp(0.0, 1.0);
                        let din = dist(base(ap).lab, base(a).lab);
                        if din < 1e-5 {
                            continue;
                        }
                        let ganho = dist(saida(ap, b, w), r0) / din;
                        pior = pior.max(ganho);
                        n += 1;
                    }
                }
            }
        }
    }
    println!("ruído ±1/255: {n} perturbações, ganho máx {pior:.3}× (F2b: K-M 3 bandas 20,2×)");
    assert!(pior <= 1.5, "ganho {pior}");
}
