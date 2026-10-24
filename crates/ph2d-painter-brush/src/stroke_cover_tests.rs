//! Gate da aritmética do cap por-traço ([`super`]): ela tem de ser, termo a termo, a que shipou.

use super::cover_add;

#[test]
fn the_cap_is_the_arithmetic_that_shipped() {
    // Com Flow 1 o `cover_add` reproduz as duas ramas que estavam inline no `bands.rs` EXATAMENTE (mesmas
    // operações, mesma ordem, mesmos guards 1e-4) — a byte-identidade de todo pincel de fábrica com
    // Accumulate off repousa nisto (nenhum tem Flow < 1; censo 24/10).
    for &m in &[0.0_f32, 0.1, 0.5, 0.9, 1.0] {
        for &w in &[0.0_f32, 0.25, 0.75, 1.0] {
            for &g in &[0.3_f32, 1.0] {
                for &cov in &[0.4_f32, 1.0] {
                    let want = {
                        let cap = (g * cov).min(1.0);
                        if m >= cap { None } else { Some(w * (cap - m)) }
                    };
                    assert_eq!(
                        cover_add(m, w, g, cov, 1.0, false),
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
                        cover_add(m, w, g, cov, 1.0, true),
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
        if let Some(add) = cover_add(m, 0.4, 1.0, 0.5, 1.0, false) {
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

#[test]
fn o_flow_e_a_taxa_e_a_strength_o_tecto() {
    // Ordem do Enio (24/10), a lei do Photoshop/Krita: com Accumulate off, Strength 0,6 e Flow 0,1, um dab
    // deposita só `w·0,1` do caminho, e muitos dabs ACUMULAM até 0,6 — nunca até 0,06, nunca acima de 0,6.
    for film in [false, true] {
        let mut m = 0.0_f32;
        let primeiro = cover_add(m, 1.0, 1.0, 0.6, 0.1, film).unwrap();
        assert!(
            primeiro < 0.07,
            "um dab de Flow 0,1 é um passo pequeno (film={film}): {primeiro}"
        );
        for _ in 0..200 {
            if let Some(add) = cover_add(m, 1.0, 1.0, 0.6, 0.1, film) {
                m += add;
            }
        }
        assert!(
            (m - 0.6).abs() < 0.01,
            "converge à Strength, não a Strength×Flow (film={film}): {m}"
        );
        assert!(m <= 0.6 + 1e-6, "e nunca passa do tecto (film={film}): {m}");
    }
}
