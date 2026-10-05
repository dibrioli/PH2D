//! A pele guarda o ângulo de cada osso e a tabela das juntas (A10, fila do esqueleto §F60): a
//! mistura dá o MESMO ponto que a conta por ponto de antes, e custa menos.

use super::{MisturaDoAngulo, junta};
use crate::Skin;

/// A mistura de antes da cache: a junta e o ângulo de cada osso calculados por ponto.
fn antiga(k: &Skin, p: [f64; 2], w: &[f64]) -> [f64; 2] {
    let (mut num, mut den) = ([0.0_f64, 0.0], 0.0_f64);
    for (i, a) in k.bones.iter().enumerate() {
        let wi = w.get(i).copied().unwrap_or(0.0);
        if wi <= 0.0 {
            continue;
        }
        for (j, b) in k.bones.iter().enumerate().skip(i + 1) {
            let wj = w.get(j).copied().unwrap_or(0.0);
            if wj <= 0.0 {
                continue;
            }
            let q = wi * wj;
            let c = junta(a, b);
            num[0] = q.mul_add(c[0], num[0]);
            num[1] = q.mul_add(c[1], num[1]);
            den += q;
        }
    }
    let centro = (den > 0.0 && num[0].is_finite() && num[1].is_finite())
        .then(|| [num[0] / den, num[1] / den]);
    let Some(c) = centro else {
        return k.blend_linear(p, w);
    };
    let (mut sx, mut sy, mut soma) = (0.0_f64, 0.0_f64, 0.0_f64);
    for (b, &peso) in k.bones.iter().zip(w.iter()) {
        if peso == 0.0 {
            continue;
        }
        let t = b.angulo_da_pose();
        sx = peso.mul_add(t.cos(), sx);
        sy = peso.mul_add(t.sin(), sy);
        soma += peso;
    }
    if soma == 0.0 || (sx == 0.0 && sy == 0.0) {
        return k.blend_linear(p, w);
    }
    let n = sx.hypot(sy);
    let (co, si) = (sx / n, sy / n);
    let base = k.blend_linear(c, w);
    let d = [p[0] - c[0], p[1] - c[1]];
    [
        si.mul_add(-d[1], co.mul_add(d[0], base[0])),
        si.mul_add(d[0], co.mul_add(d[1], base[1])),
    ]
}

/// Pontos e pesos de uma cadeia de três ossos (pesos com dois e três ossos a mandar).
fn amostra() -> (Skin, Vec<([f64; 2], [f64; 3])>) {
    let k = super::centro_tests::cadeia3(10.0, 20.0, 30.0, 0.7, -1.9);
    let mut v = Vec::new();
    for i in 0..400_u32 {
        let x = f64::from(i) * 0.075;
        let a = f64::from(i % 7) / 7.0;
        let b = f64::from(i % 5) / 5.0 * (1.0 - a);
        v.push(([x, f64::from(i % 3) - 1.0], [a, b, 1.0 - a - b]));
    }
    (k, v)
}

/// ⭐⭐ **GATE — a mistura com a cache dá o ponto de antes AO BIT**, nas duas leis do ângulo.
#[test]
fn a_cache_da_pele_da_o_ponto_de_antes_ao_bit() {
    let (k, pts) = amostra();
    assert_eq!(k.mistura(), MisturaDoAngulo::Circulo);
    for (p, w) in &pts {
        let (a, b) = (k.blend(*p, w), antiga(&k, *p, w));
        assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits), "p {p:?} w {w:?}");
    }
    // ⛔ O CONTROLO: a amostra exercita a junta (dois ossos ou mais com peso).
    assert!(
        pts.iter()
            .any(|(_, w)| w.iter().filter(|x| **x > 0.0).count() >= 2)
    );
}

/// ⭐ **SONDA — o preço da mistura**, ns por ponto: a cache contra a conta por ponto, intercaladas
/// no MESMO processo (7 rodadas, ordem rodada, o mínimo; a mediana como controlo).
#[test]
#[ignore = "sonda: imprime"]
fn diag_o_preco_da_mistura() {
    let (k, pts) = amostra();
    let mut tempos: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
    for rodada in 0..7 {
        for v in [rodada % 2, 1 - rodada % 2] {
            let t = std::time::Instant::now();
            let mut acc = 0.0;
            for _ in 0..50 {
                for (p, w) in &pts {
                    let q = if v == 0 {
                        k.blend(*p, w)
                    } else {
                        antiga(&k, *p, w)
                    };
                    acc += std::hint::black_box(q[0]);
                }
            }
            std::hint::black_box(acc);
            #[expect(clippy::cast_precision_loss, reason = "contagem")]
            tempos[v].push(t.elapsed().as_secs_f64() * 1e9 / (50 * pts.len()) as f64);
        }
    }
    for (nome, mut t) in ["com a cache", "por ponto"].into_iter().zip(tempos) {
        t.sort_by(f64::total_cmp);
        println!("  {nome}: mín {:.1} ns · mediana {:.1} ns", t[0], t[3]);
    }
    println!(
        "  loadavg {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
