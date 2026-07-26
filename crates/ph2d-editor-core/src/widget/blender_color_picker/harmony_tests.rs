//! Gates de [`super::partners`] — as cores ligadas de uma harmonia.
//!
//! O oráculo é o MATIZ na roda HSV (medido de volta pelo `rgba_to_hsv`), não a regra do offset:
//! um complementar TEM de sentar oposto, uma tríade nos cantos. A base é saturada de propósito —
//! um cinza não tem matiz para girar (o caso degenerado que a nota de `partners` descreve).

use super::{Harmony, partners};
use crate::widget::blender_color_picker::channels::{hsv_to_rgba8, rgba_to_hsv};
use ph2d_tokens::ColorValue;

/// Uma base com matiz LIMPO (30°) e saturação alta, longe da emenda 0/360 onde o wrap confunde.
fn base_at(hue_deg: f32) -> ColorValue {
    let [r, g, b, a] = hsv_to_rgba8(hue_deg / 360.0, 0.9, 0.9, 1.0);
    ColorValue::from_rgba8(r, g, b, a)
}

fn hue_deg(cv: ColorValue) -> f32 {
    rgba_to_hsv(cv.rgba).0 * 360.0
}

/// Menor diferença angular entre dois matizes (em graus, 0..180).
fn circ_diff(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

/// **Off é só a base** — a seção não intromete no picker que todo módulo já usa.
#[test]
fn a_neutral_scheme_is_just_the_base() {
    let base = base_at(30.0);
    let p = partners(base, Harmony::None);
    assert_eq!(p.len(), 1, "Off deve devolver só a base");
    assert_eq!(p[0], base);
}

/// **A base é SEMPRE a 1ª cor**, em todo esquema — é o que ancora "mover a base gira o resto".
#[test]
fn the_base_is_always_first() {
    let base = base_at(30.0);
    for scheme in Harmony::ALL {
        assert_eq!(
            partners(base, scheme)[0],
            base,
            "{scheme:?}: a base não é a 1ª"
        );
    }
}

/// **Cada esquema tem a contagem certa** (base + parceiras).
#[test]
fn each_scheme_has_the_right_count() {
    let base = base_at(30.0);
    for (scheme, n) in [
        (Harmony::None, 1),
        (Harmony::Complementary, 2),
        (Harmony::Analogous, 3),
        (Harmony::Triad, 3),
        (Harmony::SplitComplementary, 3),
        (Harmony::Tetrad, 4),
        (Harmony::Monochromatic, 4),
    ] {
        assert_eq!(
            partners(base, scheme).len(),
            n,
            "{scheme:?}: contagem errada"
        );
        assert!(n <= Harmony::MAX_COLORS, "{scheme:?} excede MAX_COLORS");
    }
}

/// **As parceiras sentam nos ângulos certos da roda** — o oráculo é o matiz, não o offset. A
/// mutação que usa `h` em vez de `h + offset` deixa toda parceira na base (diff 0) e sangra aqui.
#[test]
fn the_partners_sit_at_the_right_wheel_angles() {
    let base = base_at(30.0);
    let bh = hue_deg(base);
    // (esquema, offsets esperados das parceiras que NÃO são a base)
    for (scheme, offs) in [
        (Harmony::Complementary, &[180.0][..]),
        (Harmony::Analogous, &[-30.0, 30.0][..]),
        (Harmony::Triad, &[120.0, 240.0][..]),
        (Harmony::SplitComplementary, &[150.0, 210.0][..]),
        (Harmony::Tetrad, &[90.0, 180.0, 270.0][..]),
    ] {
        let p = partners(base, scheme);
        for (k, &off) in offs.iter().enumerate() {
            let got = hue_deg(p[k + 1]); // +1: a base é a [0]
            let want = (bh + off).rem_euclid(360.0);
            assert!(
                circ_diff(got, want) < 3.0,
                "{scheme:?} parceira {k}: matiz {got:.1}, esperado {want:.1}"
            );
        }
    }
}

/// **Monochromatic mantém o matiz e varia o VALOR** — a única que não gira a roda.
#[test]
fn monochromatic_keeps_the_hue_and_varies_value() {
    let base = base_at(30.0);
    let p = partners(base, Harmony::Monochromatic);
    let bh = hue_deg(base);
    let mut values: Vec<f32> = Vec::new();
    for cv in &p {
        let (_, s, v, _) = rgba_to_hsv(cv.rgba);
        if s > 0.05 {
            // parceira lavada (s baixo) perde matiz definido; as saturadas têm de casar a base
            assert!(circ_diff(hue_deg(*cv), bh) < 3.0, "mono mudou o matiz");
        }
        values.push(v);
    }
    let vmin = values.iter().cloned().fold(f32::MAX, f32::min);
    let vmax = values.iter().cloned().fold(f32::MIN, f32::max);
    assert!(
        vmax - vmin > 0.2,
        "mono não variou o valor (spread {:.2})",
        vmax - vmin
    );
}

/// Sonda (não-gate) — imprime as parceiras medidas de uma base laranja saturada
/// (matiz 30°), para a mensagem de smoke carregar números REAIS.
/// `cargo test -p ph2d-editor-core --lib harmony::tests::probe -- --ignored --nocapture`
#[test]
#[ignore = "measurement probe, not a gate"]
fn probe_partner_hues() {
    let base = base_at(30.0);
    println!("base hue {:.1} rgba {:?}", hue_deg(base), base.rgba);
    for scheme in Harmony::ALL {
        let p = partners(base, scheme);
        let hues: Vec<String> = p.iter().map(|c| format!("{:.1}", hue_deg(*c))).collect();
        println!("{scheme:?}: {} colors, hues [{}]", p.len(), hues.join(", "));
    }
}

/// **As parceiras são LIGADAS: girar a base gira TODAS pelo mesmo Δ**, preservando o espaçamento
/// relativo (o "linked" do Corel). É o que faz a harmonia seguir a cor em vez de congelar.
#[test]
fn rotating_the_base_rotates_every_partner_by_the_same_delta() {
    let delta = 40.0_f32;
    let a = partners(base_at(30.0), Harmony::Triad);
    let b = partners(base_at(30.0 + delta), Harmony::Triad);
    assert_eq!(a.len(), b.len());
    for (k, (pa, pb)) in a.iter().zip(&b).enumerate() {
        let moved = circ_diff(hue_deg(*pa), hue_deg(*pb));
        assert!(
            (moved - delta).abs() < 3.0,
            "parceira {k} moveu {moved:.1}, esperado {delta:.1} — o espaçamento não é preservado"
        );
    }
}
