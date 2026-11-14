//! Gate da aritmética do cap por-traço ([`super`]): ela tem de ser, termo a termo, a que shipou.

use super::{Area, cover_add};

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
                        cover_add(m, w, g, cov, 1.0, Area::Nao),
                        want,
                        "non-AA branch drifted at m={m} w={w} g={g} cov={cov}"
                    );
                    let want_aa = |area: Area| {
                        // B29.3: sob a película o grão entra no tecto (com g = 1 é ao bit o que shipou).
                        let cap = if area == Area::Filme {
                            w * g * cov
                        } else {
                            w * cov
                        }
                        .min(1.0);
                        if m >= cap {
                            None
                        } else {
                            Some((w * g * cov) * (1.0 - m / cap.max(1e-4)))
                        }
                    };
                    for area in [Area::Orla, Area::Filme] {
                        assert_eq!(
                            cover_add(m, w, g, cov, 1.0, area),
                            want_aa(area),
                            "AA branch drifted at m={m} w={w} g={g} cov={cov} {area:?}"
                        );
                    }
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
        if let Some(add) = cover_add(m, 0.4, 1.0, 0.5, 1.0, Area::Nao) {
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
        let primeiro = cover_add(m, 1.0, 1.0, 0.6, 0.1, Area::de(film, false)).unwrap();
        assert!(
            primeiro < 0.07,
            "um dab de Flow 0,1 é um passo pequeno (film={film}): {primeiro}"
        );
        for _ in 0..200 {
            if let Some(add) = cover_add(m, 1.0, 1.0, 0.6, 0.1, Area::de(film, false)) {
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

#[test]
fn com_o_filme_o_grao_e_tecto_ao_repassar() {
    // B29.3 (DF-68): no Impasto (filme AA ligado) repassar o pincel apagava o dente do papel — o grão era só
    // TAXA e o texel do vale subia ao tecto do pico. O grão é o TECTO nas duas ramas.
    for film in [false, true] {
        for &g in &[0.2_f32, 0.5, 0.8] {
            let mut m = 0.0_f32;
            for _ in 0..12 {
                if let Some(add) = cover_add(m, 1.0, g, 1.0, 1.0, Area::de(film, false)) {
                    m += add;
                }
            }
            assert!(
                m <= g + 1e-6,
                "film={film} g={g}: o vale passou do grão ({m})"
            );
        }
    }
}

/// ⛔ A porta do meio do B29.3 (BUGS_painter #115; dono 07/10 «só o Wet Paint evolui»): só o Wet Paint faz do grão
/// tecto sob a película; no Impasto a película lê-se como a orla (`w·tecto`, a lei do main 4f46a3573c).
#[test]
fn so_no_wet_paint_o_grao_e_tecto_sob_a_pelicula() {
    let mut impasto = crate::BrushSpec {
        impasto: true,
        fora_do_digital: true,
        ..Default::default()
    };
    assert_eq!(Area::do_pincel(&impasto, true, false), Area::Orla);
    impasto.impasto = false; // o Wet Paint: fora do Digital, sem os flags da aguada e do relevo
    assert_eq!(Area::do_pincel(&impasto, true, false), Area::Filme);
    let digital = crate::BrushSpec::default();
    assert_eq!(Area::do_pincel(&digital, true, false), Area::Orla);
    assert_eq!(Area::do_pincel(&digital, false, true), Area::Orla);
    assert_eq!(Area::do_pincel(&digital, false, false), Area::Nao);
}
