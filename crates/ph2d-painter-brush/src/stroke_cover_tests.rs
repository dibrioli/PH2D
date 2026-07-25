//! Gate da aritmética do cap por-traço ([`super`]): ela tem de ser, termo a termo, a que shipou.

use super::cover_add;

#[test]
fn the_cap_is_the_arithmetic_that_shipped() {
    // Pure code motion: o `cover_add` extraído reproduz as duas ramas que estavam inline no `bands.rs`
    // EXATAMENTE (mesmas operações, mesma ordem, mesmos guards 1e-4) — a byte-identidade do caminho de
    // pigmento repousa nisto.
    for &m in &[0.0_f32, 0.1, 0.5, 0.9, 1.0] {
        for &w in &[0.0_f32, 0.25, 0.75, 1.0] {
            for &g in &[0.3_f32, 1.0] {
                for &cov in &[0.4_f32, 1.0] {
                    let want = {
                        let cap = (g * cov).min(1.0);
                        if m >= cap { None } else { Some(w * (cap - m)) }
                    };
                    assert_eq!(
                        cover_add(m, w, g, cov, false),
                        want,
                        "non-AA branch drifted at m={m} w={w} g={g} cov={cov}"
                    );
                    let want_aa = {
                        let cap = (w * cov).min(1.0);
                        if m >= cap {
                            None
                        } else {
                            Some((w * g * cov) * (1.0 - m / cap.max(1e-4)))
                        }
                    };
                    assert_eq!(
                        cover_add(m, w, g, cov, true),
                        want_aa,
                        "AA branch drifted at m={m} w={w} g={g} cov={cov}"
                    );
                }
            }
        }
    }
}

#[test]
fn the_cap_builds_toward_the_ceiling_and_never_past_it() {
    // A propriedade que o cap É: passar de novo APROFUNDA (é o que o pigmento quer, e desde 2026-07-25 é
    // também o que a máscara quer — ordem do Enio: ela pinta como o brush), e nunca passa do teto.
    let mut m = 0.0_f32;
    let mut seen = Vec::new();
    for _ in 0..8 {
        if let Some(add) = cover_add(m, 0.4, 1.0, 0.5, false) {
            m += add;
            seen.push(m);
        }
    }
    assert!(
        seen.windows(2).all(|w| w[1] > w[0]),
        "cada passada tem de aprofundar: {seen:?}"
    );
    assert!(
        m <= 0.5 + 1e-6,
        "e nenhuma delas passa do teto (Strength 0.5), got {m}"
    );
}
