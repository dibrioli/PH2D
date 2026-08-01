//! Gates do pincel: os falloffs, os predicados que a UI vai perguntar, e a
//! expansão da simetria.

use super::*;

#[test]
fn every_falloff_is_one_at_the_centre_zero_at_the_rim_and_never_climbs() {
    for f in Falloff::ALL {
        assert!(
            (f.weight(0.0) - 1.0).abs() < 1e-6,
            "{}: centro = {}",
            f.label(),
            f.weight(0.0)
        );
        assert_eq!(f.weight(1.0), 0.0, "{}: borda", f.label());
        assert_eq!(f.weight(1.5), 0.0, "{}: além da borda", f.label());
        let mut prev = f32::INFINITY;
        for k in 0..=100 {
            let t = k as f32 / 100.0;
            let w = f.weight(t);
            assert!(
                w <= prev + 1e-6,
                "{} sobe entre {} e {t}",
                f.label(),
                t - 0.01
            );
            assert!(
                (0.0..=1.0).contains(&w),
                "{} = {w} fora de [0,1]",
                f.label()
            );
            prev = w;
        }
    }
}

#[test]
fn the_smooth_falloff_lands_on_the_rim_with_zero_slope() {
    // C¹ na borda: sem isto o traço deixa um DEGRAU na fronteira do pincel, que
    // é o artefato que se vê antes de se saber o nome dele. É a propriedade que
    // torna o Smooth o default, e nenhum dos outros a tem.
    let h = 1e-3;
    let slope = (Falloff::Smooth.weight(1.0) - Falloff::Smooth.weight(1.0 - h)) / h;
    assert!(slope.abs() < 1e-2, "inclinação na borda = {slope}");
    // O contraste que dá sentido ao gate: a esfera chega com tangente vertical.
    let sphere = (Falloff::Sphere.weight(1.0) - Falloff::Sphere.weight(1.0 - h)) / h;
    assert!(sphere.abs() > 1.0, "a Sphere devia ser abrupta: {sphere}");
}

#[test]
fn a_nan_distance_weighs_nothing_instead_of_poisoning_a_vertex() {
    for f in Falloff::ALL {
        assert_eq!(f.weight(f32::NAN), 0.0, "{}", f.label());
    }
}

#[test]
fn the_mirrors_expand_to_powers_of_two_and_the_first_copy_is_the_original() {
    for (sym, want) in [
        (Symmetry::default(), 1),
        (Symmetry::MIRROR_X, 2),
        (
            Symmetry {
                x: true,
                y: true,
                z: false,
            },
            4,
        ),
        (
            Symmetry {
                x: true,
                y: true,
                z: true,
            },
            8,
        ),
    ] {
        let (signs, n) = sym.signs();
        assert_eq!(n, want, "{sym:?}");
        assert_eq!(signs[0], [1.0, 1.0, 1.0], "a 1ª cópia é o dab original");
        // Sem duplicatas: um dab espelhado duas vezes no mesmo eixo é o próprio,
        // e aplicá-lo duas vezes seria trabalho dobrado sem efeito visível — o
        // tipo de desperdício que ninguém percebe até a malha grande.
        for i in 0..n {
            for j in (i + 1)..n {
                assert_ne!(signs[i], signs[j], "cópias {i} e {j} iguais em {sym:?}");
            }
        }
    }
}

// ⚠️ **`invert_flips_only_the_verbs_that_have_a_sign` foi REMOVIDO, não movido.**
// O oráculo dele era `Brush::reach`, que **é** `honours_invert` composto com um
// sinal ⇒ a asserção era verdadeira por construção e não podia falhar — nem
// antes nem depois da cura da whitelist. Mantê-lo ao lado do substituto seria
// guardar um gate verde que já provou não ver o defeito que nomeia. Quem afirma
// a propriedade agora é
// `verb_tests::invert_changes_the_result_of_exactly_the_verbs_that_have_an_opposite`,
// e o oráculo dele é o estado da MALHA.

#[test]
fn the_reach_is_a_fraction_of_the_radius_never_an_absolute_distance() {
    // A lição que o impasto do Painter pagou: com distância absoluta, um pincel
    // pequeno e um grande picam no MESMO valor, e o grande vira uma poça chata
    // porque a razão altura÷largura despenca. Amarrado ao raio, o domo tem a
    // mesma razão de aspecto em toda escala.
    let b = Brush::default();
    let (small, big) = (b.reach(0.01), b.reach(1.0));
    assert!((big / small - 100.0).abs() < 1e-3, "{small} vs {big}");
}

#[test]
fn the_families_that_the_ui_asks_about_agree_with_the_verb_list() {
    let plane: Vec<_> = Verb::ALL
        .into_iter()
        .filter(|v| v.uses_plane())
        .map(Verb::label)
        .collect();
    assert_eq!(plane, ["Flatten", "Fill", "Scrape", "Clay"]);
    let ring: Vec<_> = Verb::ALL
        .into_iter()
        .filter(|v| v.uses_neighbours())
        .map(Verb::label)
        .collect();
    assert_eq!(ring, ["Smooth", "Sharpen"]);
    let mask: Vec<_> = Verb::ALL
        .into_iter()
        .filter(|v| v.paints_mask())
        .map(Verb::label)
        .collect();
    assert_eq!(mask, ["Mask"]);
    // Nomes únicos: dois verbos com o mesmo rótulo é um seletor em que o artista
    // não consegue escolher o que quer.
    let mut labels: Vec<_> = Verb::ALL.into_iter().map(Verb::label).collect();
    labels.sort_unstable();
    let n = labels.len();
    labels.dedup();
    assert_eq!(labels.len(), n);
}
