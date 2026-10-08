//! **G12 (lado Rust)** — `queda` contra a fórmula PUBLICADA em 1 000 pontos.
//!
//! Fonte: B. Karis, «Real Shading in Unreal Engine 4», SIGGRAPH 2013 (Physically Based Shading
//! course), eq. (9), a janela da queda: `saturate(1 − (d/R)⁴)²`. O `1/(d²+1)` da mesma equação é
//! o inverso do quadrado do 3D e NÃO entra: em 2D a lei física é o ângulo subtendido, que o
//! gabarito integra. A gêmea WGSL (W2) é medida contra esta mesma tabela.

use ph2d_light2d_ref::queda::queda;

/// A fórmula, escrita aqui letra a letra, independente da implementação.
fn publicada(d: f64, r: f64) -> f64 {
    if r.is_infinite() {
        return 1.0;
    }
    let x = d / r;
    let s = (1.0 - x * x * x * x).clamp(0.0, 1.0);
    s * s
}

#[test]
fn g12_a_queda_e_a_janela_de_karis_em_mil_pontos() {
    let alcances = [0.5_f64, 1.0, 3.7, 25.0, 1e3, f64::INFINITY];
    let mut n = 0;
    let mut pior = 0.0_f64;
    for (i, &r) in alcances.iter().enumerate() {
        let por_alcance = if i + 1 == alcances.len() {
            1000 - 5 * 166
        } else {
            166
        };
        for k in 0..por_alcance {
            // De 0 a 1,5·R: atravessa o joelho em R e a cauda nula; R infinito varre 0..1000 m.
            let escala = if r.is_finite() { 1.5 * r } else { 1e3 };
            let t = escala * f64::from(k) / f64::from(por_alcance - 1);
            let erro = (queda(t, r) - publicada(t, r)).abs();
            pior = pior.max(erro);
            assert!(
                erro <= 4.0 * f64::EPSILON,
                "t={t} R={r}: {} × {}",
                queda(t, r),
                publicada(t, r)
            );
            n += 1;
        }
    }
    assert_eq!(n, 1000);
    eprintln!("G12: {n} pontos, |queda − publicada| máximo = {pior:e}");
}

#[test]
fn g12_os_pontos_ancora_da_janela() {
    assert_eq!(queda(0.0, 2.0), 1.0);
    assert_eq!(queda(2.0, 2.0), 0.0);
    assert_eq!(queda(3.0, 2.0), 0.0);
    assert_eq!(queda(1.0, 2.0), (1.0 - 1.0 / 16.0) * (1.0 - 1.0 / 16.0));
    assert_eq!(queda(123.0, f64::INFINITY), 1.0);
}
