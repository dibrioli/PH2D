use super::*;

#[test]
fn mix_full_coverage_replaces() {
    // Mix, a=1, opaque bg ⟹ exactly the source colour, opaque. (Source-over identity.)
    let out = blend_over(BrushBlend::Mix, [0.2, 0.4, 0.6, 1.0], [0.9, 0.1, 0.5], 1.0);
    assert!((out[0] - 0.9).abs() < 1e-6);
    assert!((out[1] - 0.1).abs() < 1e-6);
    assert!((out[2] - 0.5).abs() < 1e-6);
    assert!((out[3] - 1.0).abs() < 1e-6);
}

#[test]
fn mix_zero_coverage_is_noop() {
    let dst = [0.2, 0.4, 0.6, 0.8];
    let out = blend_over(BrushBlend::Mix, dst, [0.9, 0.1, 0.5], 0.0);
    for i in 0..4 {
        assert!((out[i] - dst[i]).abs() < 1e-6, "channel {i}");
    }
}

#[test]
fn mix_over_transparent_takes_source() {
    // Painting onto fully transparent ⟹ source colour at coverage alpha.
    let out = blend_over(BrushBlend::Mix, [0.0, 0.0, 0.0, 0.0], [0.8, 0.2, 0.4], 0.5);
    assert!((out[3] - 0.5).abs() < 1e-6);
    assert!((out[0] - 0.8).abs() < 1e-6);
}

#[test]
fn multiply_halves() {
    let out = blend_over(
        BrushBlend::Multiply,
        [0.5, 0.5, 0.5, 1.0],
        [0.5, 0.5, 0.5],
        1.0,
    );
    assert!((out[0] - 0.25).abs() < 1e-6);
    assert!((out[3] - 1.0).abs() < 1e-6, "colour mode keeps bg alpha");
}

#[test]
fn screen_is_inverse_multiply() {
    let out = blend_over(
        BrushBlend::Screen,
        [0.5, 0.5, 0.5, 1.0],
        [0.5, 0.5, 0.5],
        1.0,
    );
    assert!((out[0] - 0.75).abs() < 1e-6);
}

#[test]
fn erase_and_add_alpha_move_only_alpha() {
    let e = blend_over(
        BrushBlend::EraseAlpha,
        [0.3, 0.3, 0.3, 0.8],
        [1.0, 1.0, 1.0],
        0.5,
    );
    assert!((e[3] - 0.3).abs() < 1e-6);
    assert!((e[0] - 0.3).abs() < 1e-6, "erase keeps colour");
    let a = blend_over(
        BrushBlend::AddAlpha,
        [0.3, 0.3, 0.3, 0.4],
        [1.0, 1.0, 1.0],
        0.5,
    );
    assert!((a[3] - 0.9).abs() < 1e-6);
}

#[test]
fn overlay_is_hardlight_with_swapped_roles() {
    let b = [0.3, 0.6, 0.2];
    let s = [0.7, 0.4, 0.9];
    let ov = blend_rgb(BrushBlend::Overlay, b, s);
    let hl = blend_rgb(BrushBlend::HardLight, s, b);
    for i in 0..3 {
        assert!((ov[i] - hl[i]).abs() < 1e-6, "channel {i}");
    }
}

#[test]
fn luminosity_matches_source_luma_keeps_bg_chroma() {
    // Luminosity: take bg chroma, source luma ⟹ output luma == source luma.
    let out = blend_rgb(BrushBlend::Luminosity, [0.2, 0.5, 0.8], [0.6, 0.6, 0.6]);
    assert!((lum(out) - lum([0.6, 0.6, 0.6])).abs() < 1e-5);
}

#[test]
fn color_mode_outputs_stay_in_range() {
    // HSL modes can push out of [0,1]; clip_color must keep results valid.
    for &mode in &[BrushBlend::Color, BrushBlend::Hue, BrushBlend::Saturation] {
        let out = blend_rgb(mode, [0.05, 0.9, 0.5], [0.95, 0.1, 0.2]);
        for (i, v) in out.iter().enumerate() {
            assert!(
                (-1e-4..=1.0001).contains(v),
                "{mode:?} channel {i} = {v} out of range"
            );
        }
    }
}

#[test]
fn wire_discriminants_round_trip() {
    assert_eq!(BrushBlend::ALL.len(), MAX_BRUSH_BLEND_MODES as usize);
    for (i, &mode) in BrushBlend::ALL.iter().enumerate() {
        // The array index IS the wire discriminant.
        assert_eq!(mode.to_u8() as usize, i, "{mode:?} discriminant != index");
        assert_eq!(BrushBlend::from_u8(i as u8), mode, "from_u8 != ALL[{i}]");
        assert!(!mode.name_key().is_empty(), "{mode:?} has no display name");
    }
    // Out-of-range falls back to Mix (never panics).
    assert_eq!(BrushBlend::from_u8(MAX_BRUSH_BLEND_MODES), BrushBlend::Mix);
    assert_eq!(BrushBlend::from_u8(255), BrushBlend::Mix);
}

#[test]
fn display_names_are_unique() {
    let mut names: Vec<&str> = BrushBlend::ALL.iter().map(|m| m.name_key()).collect();
    names.sort_unstable();
    let n = names.len();
    names.dedup();
    assert_eq!(names.len(), n, "duplicate brush blend display name");
}

/// Pigment mix `0` (or a bare-paper destination) is `blend_over` **byte-for-byte** — a non-pigment
/// brush is unchanged.
#[test]
fn pigment_mix_zero_is_byte_identical() {
    let dst = [0.30, 0.50, 0.80, 0.90];
    for &(color, a) in &[
        ([0.9, 0.1, 0.2], 0.4f32),
        ([0.0, 0.0, 0.0], 1.0),
        ([1.0, 1.0, 1.0], 0.7),
    ] {
        assert_eq!(
            blend_over_pigment(BrushBlend::Mix, dst, color, a, 0.0),
            blend_over(BrushBlend::Mix, dst, color, a),
            "pigment mix 0 must equal blend_over exactly"
        );
    }
    // Bare paper (dst alpha 0): the dab just lands, even at full mix.
    let bare = [0.0, 0.0, 0.0, 0.0];
    assert_eq!(
        blend_over_pigment(BrushBlend::Mix, bare, [0.9, 0.8, 0.1], 0.5, 1.0),
        blend_over(BrushBlend::Mix, bare, [0.9, 0.8, 0.1], 0.5),
        "no pigment to mix with on bare paper → plain deposit"
    );
}

/// ⭐ **O papel branco é o substrato** (J3 canetas 2.ª onda): sobre o branco OPACO o pigmento pousa como o over
/// simples, ao bit (o MyPaint `deevad/ballpen` sobre o papel: 0,68 por canal, o over), e sobre uma tinta de cor
/// mistura — mais escuro que o over (o cruzamento azul × vermelho do oráculo, `64,74,121` contra o over
/// `83,94,152`).
#[test]
fn o_papel_branco_e_o_substrato_do_pigmento() {
    let (papel, azul) = ([1.0, 1.0, 1.0, 1.0], [0.0086, 0.13, 0.45]);
    for a in [0.2f32, 0.68, 1.0] {
        assert_eq!(
            blend_over_pigment(BrushBlend::Mix, papel, azul, a, 1.0),
            blend_over(BrushBlend::Mix, papel, azul, a),
            "o papel branco misturou como um pigmento a {a}"
        );
    }
    let vermelho = [0.65, 0.08, 0.11, 1.0];
    let (pig, over) = (
        blend_over_pigment(BrushBlend::Mix, vermelho, azul, 0.68, 1.0),
        blend_over(BrushBlend::Mix, vermelho, azul, 0.68),
    );
    assert!(
        pig[0] + pig[1] + pig[2] < over[0] + over[1] + over[2],
        "sobre a tinta não escureceu: {pig:?} × {over:?}"
    );
}

/// The pigment path mixes a blue canvas + a yellow dab **toward green** — the subtractive hallmark
/// the plain source-over (Krita's smudge) cannot produce.
///
/// ⛔⛔ **A METADE *«o verde fica ACIMA da média plana»* MORREU com a troca de lei** (ordem do dono
/// 2026-09-20: *«trocar as duas para a lei do Wet Paint»*). Ela era verdade do **RYB** e é FALSA do
/// **Kubelka–Munk**, e não por defeito: no K–M de constante única as absorvências SOMAM, logo a
/// mistura de dois pigmentos saturados é mais ESCURA do que a média — medido, `0,275` de verde
/// contra `0,500` da média plana, onde o RYB dava acima. *Uma metade que descrevia a lei antiga
/// não descreve «mistura de pigmento»; descrevia o RYB.*
///
/// ⭐ **O que fica é o que é hallmark das DUAS leis** — o verde DOMINAR, que a mistura de luz não
/// produz —, mais a metade nova que afirma a troca: a mistura é mais escura que a média plana, e
/// no dia em que alguém voltar ao RYB ela reprova com a mudança à vista no diff.
#[test]
fn pigment_mixes_blue_and_yellow_toward_green() {
    let blue = [0.10, 0.20, 0.75, 1.0]; // already-laid wet blue (dst alpha 1)
    let yellow = [0.90, 0.80, 0.10];
    let plain = blend_over(BrushBlend::Mix, blue, yellow, 0.5);
    let pig = blend_over_pigment(BrushBlend::Mix, blue, yellow, 0.5, 1.0);
    assert!(
        pig[1] > pig[0] && pig[1] > pig[2],
        "green is the dominant channel of blue + yellow: {pig:?}"
    );
    assert!(
        pig[1] < plain[1],
        "a lei do Wet Paint SOMA absorvências, logo a mistura é mais ESCURA que a média plana \
             (pig {pig:?} vs plain {plain:?}) — se isto passar a ser falso, a lei voltou ao RYB"
    );
    // E o controlo que separa «mistura de pigmento» de «mistura de luz»: a média plana NÃO tem
    // dominante verde nenhuma (ela é o cinzento que o modelo existe para não produzir).
    assert!(
        plain[1] <= plain[0] + 1e-6,
        "o controlo tem de ser um cinzento sem dominante verde: {plain:?}"
    );
}

/// RYB round-trip keeps a colour close to itself (the mix at `t = 0`/`1` is nearly the endpoint).
#[test]
fn ryb_mix_endpoints_are_near_identity() {
    for c in [
        [0.9, 0.8, 0.1],
        [0.1, 0.2, 0.75],
        [0.5, 0.5, 0.5],
        [0.2, 0.7, 0.3],
    ] {
        let a = ryb_mix(c, [0.0, 0.0, 0.0], 0.0); // t=0 → c
        let b = ryb_mix([0.0, 0.0, 0.0], c, 1.0); // t=1 → c
        for k in 0..3 {
            assert!(
                (a[k] - c[k]).abs() < 0.02,
                "ryb t=0 drifted: {a:?} vs {c:?}"
            );
            assert!(
                (b[k] - c[k]).abs() < 0.02,
                "ryb t=1 drifted: {b:?} vs {c:?}"
            );
        }
    }
}
