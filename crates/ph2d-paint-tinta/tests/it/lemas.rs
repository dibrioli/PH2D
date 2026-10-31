//! F2 §3.3 — Lemas 1–3 e o teorema como propriedades sobre uma GRELHA determinística (sem acaso),
//! a 3 e a 8 bandas; e a conservação das transferências (eq. 4).

use ph2d_paint_tinta::{
    Bandas, Optica, Pigmento, Volumes, camada, fecha, mistura, optica_celula, pick_up, r_infinito,
    s_fator, transfere,
};

/// Folga de `f32` sobre grandezas em `[0,1]` (~16 ulp perto de 1).
const EPS: f32 = 2.0e-6;
const RINF: [f32; 6] = [0.001, 0.03, 0.2, 0.5, 0.8, 0.999];
const SS: [f32; 3] = [0.02, 0.4, 2.0];
const VOLS: [f32; 6] = [0.0, 0.01, 0.3, 4.0, 40.0, 400.0];
const FUNDOS: [f32; 5] = [0.0, 0.05, 0.4, 0.85, 1.0];
const MEIOS: [(f32, bool); 4] = [(0.0, true), (0.0, false), (0.6, false), (1.0, true)];

/// Pigmentos de teste por banda: R∞ e S rodam pela grelha com a banda (padrão, não calibração).
fn pigmentos<const N: usize>() -> Vec<Pigmento<N>> {
    let mut out = Vec::new();
    for (i, _) in RINF.iter().enumerate() {
        for (j, _) in SS.iter().enumerate() {
            let r = Bandas(core::array::from_fn(|b| RINF[(i + b) % RINF.len()]));
            let s = Bandas(core::array::from_fn(|b| SS[(j + b) % SS.len()]));
            out.push(Pigmento::de_rinf(r, s));
        }
    }
    out
}

fn r_inf_mix<const N: usize>(o: &Optica<N>) -> Bandas<N> {
    Bandas(core::array::from_fn(|b| r_infinito(o.k.0[b] / o.s.0[b])))
}

fn escala<const N: usize>(o: &Optica<N>, a: f32) -> Optica<N> {
    Optica {
        k: o.k.mapa(|_, x| x * a),
        s: o.s.mapa(|_, x| x * a),
    }
}

fn varre<const N: usize>() -> usize {
    let ps = pigmentos::<N>();
    let mut n = 0;
    for (ia, pa) in ps.iter().enumerate() {
        for pb in ps.iter().skip(ia).step_by(5) {
            for &(m, molhado) in &MEIOS {
                let sf = s_fator(m, molhado);
                let (ma, mb) = (pa.masstone(sf), pb.masstone(sf));
                for &va in &VOLS {
                    for &vb in &VOLS {
                        if va + vb == 0.0 {
                            continue;
                        }
                        let o = mistura(&[(pa, va), (pb, vb)], sf);
                        let rinf = r_inf_mix(&o);
                        for b in 0..N {
                            let (lo, hi) = (ma.0[b].min(mb.0[b]), ma.0[b].max(mb.0[b]));
                            // Lema 1: o masstone da mistura fica no envelope dos ingredientes.
                            assert!(
                                rinf.0[b] >= lo - EPS && rinf.0[b] <= hi + EPS,
                                "Lema 1: banda {b} R∞ {} ∉ [{lo},{hi}]",
                                rinf.0[b]
                            );
                        }
                        for &g in &FUNDOS {
                            let rg = Bandas::uniforme(g);
                            let r = camada(&o, &rg);
                            let r_meio = camada(&escala(&o, 0.5), &rg);
                            let r_dobro = camada(&escala(&o, 2.0), &rg);
                            for b in 0..N {
                                let (lo, hi) = (g.min(rinf.0[b]), g.max(rinf.0[b]));
                                // Lema 2: R ∈ [min(R_g,R∞), max(R_g,R∞)] e monótona na espessura.
                                assert!(
                                    r.0[b] >= lo - EPS && r.0[b] <= hi + EPS,
                                    "Lema 2: {} ∉ [{lo},{hi}]",
                                    r.0[b]
                                );
                                assert!(
                                    (r_dobro.0[b] - rinf.0[b]).abs()
                                        <= (r.0[b] - rinf.0[b]).abs() + EPS,
                                    "Lema 2: engrossar afastou de R∞ (banda {b})"
                                );
                                // Lema 3: diluir (todos os v × ½) aproxima do fundo.
                                assert!(
                                    (r_meio.0[b] - g).abs() <= (r.0[b] - g).abs() + EPS,
                                    "Lema 3: diluir afastou do fundo (banda {b})"
                                );
                                // Teorema: R ≥ min(R_g, min_i R∞_i).
                                let piso = g.min(ma.0[b]).min(mb.0[b]);
                                assert!(r.0[b] >= piso - EPS, "Teorema: {} < {piso}", r.0[b]);
                                assert!(r.0[b].is_finite());
                            }
                            n += 1;
                        }
                    }
                }
            }
        }
    }
    n
}

#[test]
fn lemas_e_teorema_a_3_bandas() {
    let n = varre::<3>();
    println!("lemas a 3 bandas: {n} casos");
    assert!(n > 10_000);
}

#[test]
fn lemas_e_teorema_a_8_bandas() {
    let n = varre::<8>();
    println!("lemas a 8 bandas: {n} casos");
    assert!(n > 10_000);
}

#[test]
fn transferencias_conservam_e_assentar_nao_muda_o_que_se_ve() {
    let pal = [crate::azul(), crate::amarelo()];
    let sf = s_fator(0.85, true);
    let mut pincel = Volumes([12.0, 30.0]);
    let mut filme = Volumes([0.0, 0.0]);
    let mut aberto = Volumes([20.0, 0.5]);
    let total = |a: &Volumes<2>, b: &Volumes<2>, c: &Volumes<2>| -> [f32; 2] {
        [a.0[0] + b.0[0] + c.0[0], a.0[1] + b.0[1] + c.0[1]]
    };
    let antes = total(&pincel, &filme, &aberto);
    let _ = transfere(&mut pincel, &mut filme, 0.37); // depósito
    let r1 = camada(&optica_celula(&pal, &filme, &aberto, sf), &crate::PAPEL);
    let _ = transfere(&mut filme, &mut aberto, 0.61); // assentar
    let r2 = camada(&optica_celula(&pal, &filme, &aberto, sf), &crate::PAPEL);
    let _ = pick_up(&mut aberto, &mut pincel, 0.5, 0.8, 0.9);
    let depois = total(&pincel, &filme, &aberto);
    for i in 0..2 {
        assert!(
            (depois[i] - antes[i]).abs() <= 8.0 * f32::EPSILON * antes[i],
            "pigmento {i}"
        );
    }
    for b in 0..3 {
        assert!(
            (r1.0[b] - r2.0[b]).abs() <= EPS,
            "assentar mudou a cor: {r1:?} {r2:?}"
        );
    }
    // Fecho: o fundo novo é exactamente o que se via (aberto sobre o fundo), e o aberto zera.
    let mut rg = crate::PAPEL;
    let visto = camada(&optica_celula(&pal, &Volumes::default(), &aberto, sf), &rg);
    fecha(&mut rg, &mut aberto, &pal, sf);
    assert_eq!(rg, visto);
    assert_eq!(aberto, Volumes::default());
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
