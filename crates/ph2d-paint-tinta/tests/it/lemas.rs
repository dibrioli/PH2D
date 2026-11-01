//! F2 §3.3 — Lemas 2–3 e o teorema da CAMADA sobre uma GRELHA determinística, a 3 e a 8 bandas; e as
//! transferências (eq. 4): conservação de volume, assentar invisível, fecho bit-exacto.

use ph2d_paint_tinta::{
    Bandas, Optica, Soma, camada, fecha, optica_celula, optica_de_rinf, pick_up, r_infinito,
    s_de_transparencia, transfere,
};

/// Folga de `f32` sobre grandezas em `[0,1]` (~16 ulp perto de 1).
const EPS: f32 = 2.0e-6;
const RINF: [f32; 6] = [0.001, 0.03, 0.2, 0.5, 0.8, 0.999];
const TRANSP: [f32; 3] = [0.0, 0.35, 1.0];
const VOLS: [f32; 6] = [0.0, 0.01, 0.3, 4.0, 40.0, 400.0];
const FUNDOS: [f32; 5] = [0.0, 0.05, 0.4, 0.85, 1.0];
const MEIOS: [(f32, bool); 4] = [(0.0, true), (0.0, false), (0.6, false), (1.0, true)];

fn varre<const N: usize>() -> usize {
    let mut n = 0;
    for i in 0..RINF.len() {
        let r = Bandas::<N>(core::array::from_fn(|b| RINF[(i + b) % RINF.len()]));
        for &t in &TRANSP {
            for &(m, molhado) in &MEIOS {
                let o = |v: f32| optica_de_rinf(r, s_de_transparencia(t), v, m, molhado);
                for &v in &VOLS {
                    let (op, meio, dobro) = (o(v), o(0.5 * v), o(2.0 * v));
                    let rinf: Bandas<N> = Bandas(core::array::from_fn(|b| {
                        if v > 0.0 {
                            r_infinito(op.k.0[b] / op.s.0[b])
                        } else {
                            0.0
                        }
                    }));
                    for &g in &FUNDOS {
                        let rg = Bandas::uniforme(g);
                        let (rr, r_meio, r_dobro) =
                            (camada(&op, &rg), camada(&meio, &rg), camada(&dobro, &rg));
                        for b in 0..N {
                            assert!(rr.0[b].is_finite());
                            if v == 0.0 {
                                assert_eq!(rr.0[b], g, "sem pigmento a camada é o fundo");
                                continue;
                            }
                            let (lo, hi) = (g.min(rinf.0[b]), g.max(rinf.0[b]));
                            // Lema 2 / teorema: R ∈ [min(R_g,R∞), max(R_g,R∞)].
                            assert!(
                                rr.0[b] >= lo - EPS && rr.0[b] <= hi + EPS,
                                "Lema 2: {} ∉ [{lo},{hi}]",
                                rr.0[b]
                            );
                            // Lema 2: monótona na espessura (engrossar aproxima de R∞).
                            assert!(
                                (r_dobro.0[b] - rinf.0[b]).abs()
                                    <= (rr.0[b] - rinf.0[b]).abs() + EPS,
                                "Lema 2: engrossar afastou de R∞ (banda {b})"
                            );
                            // Lema 3: diluir (v × ½) só aproxima do fundo.
                            assert!(
                                (r_meio.0[b] - g).abs() <= (rr.0[b] - g).abs() + EPS,
                                "Lema 3: diluir afastou do fundo (banda {b})"
                            );
                        }
                        n += 1;
                    }
                }
            }
        }
    }
    n
}

#[test]
fn lemas_2_3_e_teorema_a_3_bandas() {
    let n = varre::<3>();
    println!("lemas 2–3 a 3 bandas: {n} casos");
    assert!(n >= 2000);
}

#[test]
fn lemas_2_3_e_teorema_a_8_bandas() {
    let n = varre::<8>();
    println!("lemas 2–3 a 8 bandas: {n} casos");
    assert!(n >= 2000);
}

#[test]
fn transferencias_conservam_e_assentar_nao_muda_o_que_se_ve() {
    let (m, molhado) = (0.85, true);
    let mut pincel = Soma::de(crate::azul(), 12.0).junta(Soma::de(crate::amarelo(), 30.0));
    let mut filme = Soma::default();
    let mut aberto = Soma::de(crate::azul(), 20.0).junta(Soma::de(crate::amarelo(), 0.5));
    let total = |a: &Soma, b: &Soma, c: &Soma| a.junta(*b).junta(*c);
    let antes = total(&pincel, &filme, &aberto);
    let _ = transfere(&mut pincel, &mut filme, 0.37); // depósito
    let r1 = camada(&optica_celula(&filme, &aberto, m, molhado), &crate::PAPEL);
    let _ = transfere(&mut filme, &mut aberto, 0.61); // assentar
    let r2 = camada(&optica_celula(&filme, &aberto, m, molhado), &crate::PAPEL);
    let _ = pick_up(&mut aberto, &mut pincel, 0.5, 0.8, 0.9);
    let depois = total(&pincel, &filme, &aberto);
    let rel = |x: f32, y: f32| (x - y).abs() / y.abs().max(1.0);
    assert!(rel(depois.v, antes.v) <= 8.0 * f32::EPSILON, "volume");
    assert!(rel(depois.t, antes.t) <= 8.0 * f32::EPSILON, "Σv·t");
    for i in 0..3 {
        assert!(
            rel(depois.lab[i], antes.lab[i]) <= 8.0 * f32::EPSILON,
            "Σv·Lab[{i}]"
        );
    }
    let mut pior = 0.0f32;
    for b in 0..3 {
        pior = pior.max((r1.0[b] - r2.0[b]).abs());
    }
    println!("assentar: max |ΔR| = {pior:.2e}");
    assert!(pior <= EPS, "assentar mudou a cor: {r1:?} {r2:?}");
    // Fecho: o fundo novo é EXACTAMENTE o que se via (aberto SECO sobre o fundo), e o aberto zera.
    let mut rg = crate::PAPEL;
    let visto = camada(&optica_celula(&Soma::default(), &aberto, m, false), &rg);
    fecha(&mut rg, &mut aberto, m);
    assert_eq!(rg, visto);
    assert_eq!(aberto, Soma::default());
}

#[test]
fn s_tende_a_zero_e_beer_lambert() {
    for &k in &[0.0f32, 1e-6, 0.01, 0.5, 3.0, 20.0] {
        for &g in &FUNDOS {
            let r = ph2d_paint_tinta::camada_banda(k, 0.0, g);
            let bl = g * (-2.0 * k).exp();
            assert!((r - bl).abs() <= EPS, "S=0, K={k}, Rg={g}: {r} vs {bl}");
        }
    }
}

#[test]
fn sobre_branco_a_composicao_devolve_r_w() {
    let mut pior = 0.0f32;
    for &v in &VOLS {
        for &g in &FUNDOS {
            let rg = Bandas([g, 0.5 * g + 0.2, 1.0 - 0.5 * g]);
            let o: Optica<3> = optica_celula(
                &Soma::de(crate::azul(), v),
                &Soma::de(crate::amarelo(), 0.3 * v),
                0.4,
                true,
            );
            let rw = camada(&o, &rg);
            let (cor, a) = ph2d_paint_tinta::ecra_sobre_branco(&o, &rg);
            assert!((0.0..=1.0).contains(&a));
            for b in 0..3 {
                assert!((0.0..=1.0).contains(&cor.0[b]));
                // A tela compõe nos valores sRGB: o `over` sobre branco devolve `sRGB(R_w)` (E9 §7).
                let e = ph2d_paint_tinta::oklab::linear_para_srgb(rw.0[b]);
                pior = pior.max((cor.0[b] * a + (1.0 - a) - e).abs());
            }
        }
    }
    println!("sobre branco: max |cor·α + 1 − α − sRGB(R_w)| = {pior:.2e}");
    assert!(pior <= EPS);
}
