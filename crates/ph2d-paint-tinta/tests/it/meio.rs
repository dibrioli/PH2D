//! A lei do MEIO (coordenador 31/10, F4_1 §meio): a cor escolhida é o masstone SECO no próprio meio.
//! Seca em qualquer `m` a camada espessa devolve a cor; molhada a aquarela escurece; o óleo não muda.

use ph2d_paint_tinta::oklab::linear_para_srgb;
use ph2d_paint_tinta::{Bandas, R_INF_MIN, Soma, Tinta, camada, fecha, optica, r_infinito};

const MEIOS: [f32; 3] = [0.0, 0.5, 1.0];
const TRANSP: [f32; 3] = [0.0, 0.35, 1.0];
/// Espessura «infinita»: `v·S ≥ 1e5·0,02 = 2000` mesmo na tinta mais transparente (`x ≳ 44` satura).
const V_INF: f32 = 1.0e5;

/// O masstone esperado em sRGB `[0,1]`: a cor do seletor, com o piso `R_INF_MIN` da lei.
fn esperado(srgb: [f32; 3], t: f32) -> [f32; 3] {
    Tinta::de_srgb(srgb, t)
        .linear()
        .map(|r| linear_para_srgb(r.clamp(R_INF_MIN, 1.0)))
}

#[test]
fn masstone_seco_e_a_cor_escolhida_em_todo_meio() {
    let (mut pior_q, mut pior_camada) = (0.0f32, 0.0f32);
    for &m in &MEIOS {
        for &t in &TRANSP {
            for srgb in crate::CORES {
                let tinta = Tinta::de_srgb(srgb, t);
                let o = optica(&Soma::de(tinta, V_INF), m, false);
                let alvo = esperado(srgb, t);
                for fundo in [0.0f32, 1.0] {
                    let r = camada(&o, &Bandas::uniforme(fundo));
                    for b in 0..3 {
                        let q = linear_para_srgb(r_infinito(o.k.0[b] / o.s.0[b]));
                        pior_q = pior_q.max((q - alvo[b]).abs());
                        let e = (linear_para_srgb(r.0[b]) - alvo[b]).abs();
                        pior_camada = pior_camada.max(e);
                    }
                }
            }
        }
    }
    println!(
        "masstone seco: max |sRGB(R∞(𝒦/𝒮)) − cor| = {:.3} níveis · camada espessa {:.3} níveis",
        pior_q * 255.0,
        pior_camada * 255.0
    );
    // «Ao arredondamento»: abaixo de meio nível de 8 bits — o píxel gravado é a cor escolhida.
    assert!(
        pior_q * 255.0 < 0.5,
        "o masstone seco não é a cor escolhida"
    );
    assert!(
        pior_camada * 255.0 < 0.5,
        "a camada espessa seca não é a cor escolhida"
    );
}

#[test]
fn a_aquarela_molhada_escurece_e_seca_para_a_cor() {
    // As cores da célula E9 (§5.1): o azul `40,90,220` e o amarelo `245,205,25`.
    for srgb in [[40.0f32, 90.0, 220.0], [245.0, 205.0, 25.0]].map(|c| c.map(|x| x / 255.0)) {
        let t = 0.0; // opaca
        let tinta = Tinta::de_srgb(srgb, t);
        let alvo = esperado(srgb, t);
        for v in [0.05f32, 0.5, 3.0, V_INF] {
            let s = Soma::de(tinta, v);
            let (molhada, seca) = (optica(&s, 0.0, true), optica(&s, 0.0, false));
            let papel = Bandas([0.85, 0.85, 0.82]);
            let (rm, rs) = (camada(&molhada, &papel), camada(&seca, &papel));
            for b in 0..3 {
                assert!(
                    rm.0[b] < rs.0[b],
                    "v={v} banda {b}: molhada {} não escurece a seca {}",
                    rm.0[b],
                    rs.0[b]
                );
            }
            // O óleo (m = 1): molhado = seco AO BIT (`s(1, molhado) = 1`).
            assert_eq!(optica(&s, 1.0, true), optica(&s, 1.0, false), "óleo v={v}");
            // Secar (o fecho: o aberto cura SECO sobre o fundo) dá o que se vê seco.
            let (mut rg, mut aberto) = (papel, s);
            fecha(&mut rg, &mut aberto, 0.0);
            assert_eq!(rg, rs, "o fecho não é a camada seca");
        }
        // Espessa, a seca é a cor escolhida e a molhada é visivelmente mais escura.
        let s = Soma::de(tinta, V_INF);
        let rm = camada(&optica(&s, 0.0, true), &Bandas::uniforme(1.0));
        let rs = camada(&optica(&s, 0.0, false), &Bandas::uniforme(1.0));
        let nivel = |r: f32| linear_para_srgb(r) * 255.0;
        for b in 0..3 {
            assert!((nivel(rs.0[b]) - alvo[b] * 255.0).abs() < 0.5, "seca ≠ cor");
        }
        println!(
            "aquarela {:?}: seca {:?} · molhada {:?} (sRGB 0..255)",
            srgb.map(|x| (x * 255.0).round()),
            rs.0.map(|r| nivel(r).round()),
            rm.0.map(|r| nivel(r).round())
        );
    }
}
