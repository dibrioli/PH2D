//! **G12 (lado Rust)** — `queda` contra a fórmula PUBLICADA em 1 000 pontos.
//!
//! Fonte: B. Karis, «Real Shading in Unreal Engine 4», SIGGRAPH 2013 (Physically Based Shading
//! course), eq. (9), a janela da queda: `saturate(1 − (d/R)⁴)²`. O `1/(d²+1)` da mesma equação é
//! o inverso do quadrado do 3D e NÃO entra: em 2D a lei física é o ângulo subtendido, que o
//! gabarito integra. A gêmea WGSL (`ph2d-light2d-gpu`) é medida contra esta mesma tabela,
//! `queda::pontos_g12` — a porta partilhada.

use ph2d_light2d_ref::queda::{pontos_g12, queda};

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
    let pontos = pontos_g12();
    assert_eq!(pontos.len(), 1000);
    // A tabela atravessa o joelho e a cauda nula em cada alcance finito, e o ∞.
    for r in [0.5, 1.0, 3.7, 25.0, 1e3] {
        assert!(pontos.iter().any(|&(t, a)| a == r && t > r));
    }
    assert!(pontos.iter().any(|&(_, a)| a.is_infinite()));
    let mut pior = 0.0_f64;
    for (t, r) in pontos {
        let erro = (queda(t, r) - publicada(t, r)).abs();
        pior = pior.max(erro);
        assert!(
            erro <= 4.0 * f64::EPSILON,
            "t={t} R={r}: {} × {}",
            queda(t, r),
            publicada(t, r)
        );
    }
    eprintln!("G12: 1000 pontos, |queda − publicada| máximo = {pior:e}");
}

#[test]
fn g12_os_pontos_ancora_da_janela() {
    assert_eq!(queda(0.0, 2.0), 1.0);
    assert_eq!(queda(2.0, 2.0), 0.0);
    assert_eq!(queda(3.0, 2.0), 0.0);
    assert_eq!(queda(1.0, 2.0), (1.0 - 1.0 / 16.0) * (1.0 - 1.0 / 16.0));
    assert_eq!(queda(123.0, f64::INFINITY), 1.0);
}
