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
    // torna o Smooth o default.
    //
    // ⚠️ **Este comentário dizia "e nenhum dos outros a tem", e isso deixou de
    // ser verdade** no dia em que a curva da referência entrou na família: a
    // `Plateau` também pousa plana. A frase virou a asserção abaixo, que exige
    // as DUAS — porque uma nota que enumera quem tem uma propriedade é a que
    // envelhece calada.
    let h = 1e-3;
    for f in [Falloff::Smooth, Falloff::Plateau] {
        let slope = (f.weight(1.0) - f.weight(1.0 - h)) / h;
        assert!(
            slope.abs() < 1e-2,
            "{}: inclinação na borda = {slope}",
            f.label()
        );
    }
    // O contraste que dá sentido ao gate: a esfera chega com tangente vertical.
    let sphere = (Falloff::Sphere.weight(1.0) - Falloff::Sphere.weight(1.0 - h)) / h;
    assert!(sphere.abs() > 1.0, "a Sphere devia ser abrupta: {sphere}");
}

/// **A CURVA DA REFERÊNCIA entrou na família, e ela é MAIS CHEIA** — o número
/// que o artista vai ver antes de saber o nome dele.
///
/// ⚠️ A `Plateau` não é uma sexta opção de gosto: ela é a quártica que as dez
/// tools de geometria do SculptGL usam, e sem ela a paridade bit-a-bit não tem
/// como ser pedida ao produto — todo verbo sairia com a forma certa e a
/// *silhueta* errada.
#[test]
fn the_reference_curve_is_fuller_than_the_smooth_and_by_how_much() {
    // A meio raio: `3/16 − 4/8 + 1` contra `(1 − 1/4)²`.
    let (plateau, smooth) = (Falloff::Plateau.weight(0.5), Falloff::Smooth.weight(0.5));
    assert!((plateau - 0.687_5).abs() < 1e-6, "meio raio = {plateau}");
    assert!((smooth - 0.562_5).abs() < 1e-6, "meio raio = {smooth}");
    assert!(
        (plateau / smooth - 1.222_2).abs() < 1e-3,
        "a razão a meio raio é o número da tabela: {}",
        plateau / smooth
    );
    // E ela é mais cheia em TODO o interior, não só no ponto que eu escolhi.
    for k in 1..100 {
        let t = k as f32 / 100.0;
        assert!(
            Falloff::Plateau.weight(t) > Falloff::Smooth.weight(t),
            "t = {t}: {} vs {}",
            Falloff::Plateau.weight(t),
            Falloff::Smooth.weight(t)
        );
    }
}

/// **NENHUMA CURVA DA FAMÍLIA É CÓPIA DE OUTRA.**
///
/// ⚠️ O gate existe porque a `Plateau` chegou de FORA (ela é a da referência, e
/// não um desenho nosso), e a maneira barata de a próxima curva entrar é alguém
/// escrever à mão uma quártica que *parece* com ela. Duas entradas com o mesmo
/// número são dois botões que fazem a mesma coisa — e o painel os pinta lado a
/// lado.
#[test]
fn no_two_falloffs_in_the_family_are_the_same_curve() {
    for (i, a) in Falloff::ALL.into_iter().enumerate() {
        for b in Falloff::ALL.into_iter().skip(i + 1) {
            let apart = (0..=64).any(|k| {
                let t = k as f32 / 64.0;
                (a.weight(t) - b.weight(t)).abs() > 1e-4
            });
            assert!(apart, "{} e {} são a mesma curva", a.label(), b.label());
        }
    }
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

/// ⚠️ **A máscara nasce em força CHEIA, e a razão MUDOU sob os pés desta nota**
/// (2026-08-12). Ela dizia que a força de geometria fazia o traço *parar* em
/// 0,5000 por mais que se esfregasse — verdade sob o ENVELOPE, e o defeito que
/// a lei aditiva removeu: hoje 0,5 satura em duas esfregadas. O que sustenta o
/// default é outra coisa, e é a referência: o `Masking` do original ship
/// **`_intensity = 1.0`** (`Masking.js:15`), enquanto as tools de geometria
/// nascem em 0,5 e 0,75. *Um default defendido por um defeito fica órfão no dia
/// em que o defeito é corrigido.*
#[test]
fn the_mask_is_born_at_full_strength_and_geometry_is_not() {
    assert_eq!(Verb::Mask.default_strength(), 1.0);
    for v in Verb::ALL.iter().filter(|v| !v.paints_mask()) {
        assert_eq!(
            v.default_strength(),
            0.5,
            "{} não é máscara: a força é *quão longe ao longo do trajeto*",
            v.label()
        );
    }
    // E o default do `Brush` continua sendo o de geometria — quem arma o do
    // verbo é quem ESCOLHE o verbo.
    assert_eq!(Brush::default().strength, 0.5);
}

/// A entrega: **um traço protege de fato**, e é o envelope que o faz saturar em
/// vez de uma acumulação que dependeria do espaçamento.
#[test]
fn one_full_strength_stroke_protects_completely() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
    let brush = Brush {
        verb: Verb::Mask,
        radius: 0.5,
        strength: Verb::Mask.default_strength(),
        ..Brush::default()
    };
    let mut stroke = crate::SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &brush,
        &crate::Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
    let peak = mesh
        .masks()
        .expect("mascarou")
        .iter()
        .fold(0.0f32, |a, &b| a.max(b));
    assert!(
        peak > 0.999,
        "um traço tem de chegar ao teto, e chegou a {peak}"
    );

    // ⚠️ **E o par que torna isto uma afirmação sobre ACUMULAÇÃO, não sobre
    // força.** Este bloco media o DEFEITO — *"oito dabs a meia força param em
    // 0,5, porque o envelope é um `max`"* —, e o afirmava. Sob a lei aditiva
    // ele diz o oposto, e é a diferença entre um canal que se constrói e um que
    // satura no primeiro toque: **um** dab a meia força pinta meio, e **oito**
    // chegam ao teto.
    let rub = |n: usize| {
        let mut m = ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
        let mut st = crate::SculptStroke::default();
        st.begin(&m);
        for _ in 0..n {
            st.dab(
                &mut m,
                &Brush {
                    strength: 0.5,
                    ..brush.clone()
                },
                &crate::Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
        }
        m.masks()
            .expect("mascarou")
            .iter()
            .fold(0.0f32, |a, &b| a.max(b))
    };
    let one = rub(1);
    assert!(
        (one - 0.5).abs() < 1e-6,
        "um dab a meia força pinta meio, e pintou {one}"
    );
    let many = rub(8);
    assert!(
        many > 0.999,
        "esfregar a meia força tem de chegar ao teto, e parou em {many}"
    );
}
