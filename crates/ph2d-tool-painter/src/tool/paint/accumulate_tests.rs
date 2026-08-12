//! Gates do **Accumulate** — a metade que AFIRMA, ao lado da sonda que MEDE
//! ([`super::accumulate_probe`], que é dona da fixture e dos helpers: um oráculo só, para o gate e a
//! medição que o motivou nunca discordarem sobre o que "o traço" significa).
//!
//! O estudo com os números e o mecanismo é [`docs/Painter/35_accumulate_vs_blender.md`].

use super::accumulate_probe::{alpha, one_stroke, soft_tool};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{PanelEvent, Tool};

/// A razão entre o espaçamento mais denso e o mais esparso, num mesmo CAMINHO.
fn spacing_ratio(accumulate: bool, atten: bool) -> f32 {
    let (mut lo, mut hi) = (f32::MAX, 0.0f32);
    for &sp in &[0.05f32, 0.10, 0.20, 0.40] {
        let mut t = soft_tool(0.5, accumulate);
        if atten {
            t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SPACE_ATTEN));
        }
        t.handle_panel_event(PanelEvent::SetValue(
            core_ids::PAINTER_BRUSH_SPACING,
            f64::from(sp),
        ));
        one_stroke(&mut t, 1);
        let a = alpha(&t);
        lo = lo.min(a);
        hi = hi.max(a);
    }
    hi / lo
}

/// **"Adjust Strength for Spacing" não pode PIORAR o que o nome promete.**
///
/// O knob é o compensador da lei que empilha (Accumulate ON). Sob o teto por texel nada empilha, e
/// ele entrava em `coverage`, que É o teto — então atenuar só abaixava o teto, por um número função
/// do espaçamento: medido **8,17×** onde a lei capada sozinha dá **1,02×** (doc 35 §3.3).
///
/// **Mutação que tem de sangrar:** tirar o `&& self.accumulate` de
/// [`ph2d_painter_brush::BrushSpec::space_overlap_factor`] ⇒ esta razão volta a ~8×.
#[test]
fn the_spacing_knob_never_makes_a_capped_stroke_spacing_dependent() {
    let plain = spacing_ratio(false, false);
    let attenuated = spacing_ratio(false, true);
    assert!(
        plain < 1.15,
        "controle: a lei capada JA e independente do espacamento, e a fixture tem de mostrar isso \
         (razao {plain:.2}x)"
    );
    assert!(
        attenuated <= plain + 0.05,
        "o knob de espacamento PIOROU um traco capado: {attenuated:.2}x contra {plain:.2}x sem ele"
    );
}

/// E o CONTROLE do outro lado: no modo que o knob de fato serve, ele continua servindo — sem esta
/// metade, "desligar a atenuação em todo lugar" passaria no gate acima e mataria a feature.
#[test]
fn the_spacing_knob_still_flattens_the_law_that_piles_up() {
    let raw = spacing_ratio(true, false);
    let fixed = spacing_ratio(true, true);
    assert!(
        raw > 2.0,
        "a lei que empilha DEPENDE do espacamento — a fixture precisa conter isso (razao {raw:.2}x)"
    );
    assert!(
        fixed < raw * 0.6,
        "com a atenuacao a lei que empilha tem de ficar bem mais plana: {fixed:.2}x contra {raw:.2}x"
    );
}

/// ⚠️ **O que a medição REFUTOU, pinado para ninguém "consertar" de volta.**
///
/// O doc 35 recomendava, na 1ª escrita, tirar a cláusula `strength < 1.0` de
/// `stroke_cover_wanted` — a teoria era que o teto passaria a existir em força máxima e o ombro
/// pararia de endurecer. **Construí a cura e medi: byte-idêntica.** A álgebra diz por quê: com
/// `cap = 1` o passo do teto é `m += w·(1 − m)` e o chamador compõe em `a = add/(1 − m) = w`, que é
/// **exatamente** source-over por dab — a lei do Accumulate ON. As duas leis COINCIDEM em
/// `strength = 1`, então a cláusula é uma **otimização** (pular o buffer onde ele provadamente não
/// faz nada), não um defeito.
///
/// Este gate afirma a coincidência, que é o fato durável.
#[test]
fn at_full_strength_the_two_laws_are_the_same_law() {
    for &n in &[1usize, 15] {
        let mut off = soft_tool(1.0, false);
        let mut on = soft_tool(1.0, true);
        one_stroke(&mut off, n);
        one_stroke(&mut on, n);
        assert_eq!(
            off.canvas_rgba, on.canvas_rgba,
            "em strength 1.0 o teto vale 1 ⇒ o cap reduz a source-over por dab; n={n}"
        );
    }
}
