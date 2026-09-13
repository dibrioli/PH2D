//! Os gates da lei de vista. ⚠️ Cada barra aqui vem de uma medição escrita no doc do módulo, e o
//! gate do oráculo traz a METADE JUSTA: a vista errada tem de reprovar sobre a mesma fixture.

use super::*;

/// O oráculo — ver o cabeçalho do próprio ficheiro e o doc do módulo.
const FIXTURE: &str = include_str!("../fixtures/ocio_neutral_nodes.txt");

/// **A barra nos nós do LUT.** Medido em 2026-09-13: resíduo `8,9e-6` nos nós, e fora deles o
/// oráculo erra `1,4e-4` na mediana e `0,030` no pior caso (a interpolação do LUT dele).
const ORACLE_BAR: f32 = 2.0e-5;

/// Quantas linhas a fixture tem de ter: 57 cinzas + 400 cores (ver `fixture_neutral.py`).
const FIXTURE_ROWS: usize = 457;

struct Row {
    scene: [f32; 3],
    oracle_srgb: [f32; 3],
}

fn rows() -> Vec<Row> {
    FIXTURE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let v: Vec<f32> = l
                .split_whitespace()
                .map(|t| t.parse().expect("a fixture tem números"))
                .collect();
            assert_eq!(v.len(), 6, "linha malformada na fixture: {l}");
            Row {
                scene: [v[0], v[1], v[2]],
                oracle_srgb: [v[3], v[4], v[5]],
            }
        })
        .collect()
}

/// O pior erro, em sRGB codificado, de uma vista contra a fixture inteira.
fn worst_against_oracle(view: ViewTransform) -> (f32, [f32; 3]) {
    let mut worst = (0.0_f32, [0.0_f32; 3]);
    for r in rows() {
        let ours = to_display(r.scene, 0.0, view).map(ph2d_color::srgb::linear_to_srgb_unit);
        for (o, oracle) in ours.iter().zip(r.oracle_srgb) {
            let e = (o - oracle).abs();
            if e > worst.0 {
                worst = (e, r.scene);
            }
        }
    }
    worst
}

/// ⭐ **A `Neutral` É a vista do Blender, em todo nó do LUT dele.**
///
/// ⚠️ **As duas metades**: a lei certa passa por baixo da barra, e a `Standard` — a vista errada
/// sobre a MESMA fixture — tem de ficar muito acima dela. Sem a segunda, uma fixture que só tivesse
/// luz abaixo do joelho aprovaria qualquer vista que cortasse em `1`.
#[test]
fn the_neutral_view_is_the_oracles_at_every_lut_node() {
    assert_eq!(
        rows().len(),
        FIXTURE_ROWS,
        "a fixture perdeu ou ganhou linhas — um gate sobre menos amostras aprova mais"
    );
    let (neutral, at) = worst_against_oracle(ViewTransform::Neutral);
    assert!(
        neutral <= ORACLE_BAR,
        "a Neutral afasta-se do oráculo {neutral:e} (barra {ORACLE_BAR:e}) na entrada {at:?}"
    );
    let (standard, _) = worst_against_oracle(ViewTransform::Standard);
    assert!(
        standard > 100.0 * ORACLE_BAR,
        "a Standard ficou a {standard:e} do oráculo da Neutral — a fixture não distingue as duas vistas"
    );
}

/// ⭐ **O olhar padrão não muda nada dentro do branco**, ao bit — e o `Look` é a MESMA lei, não uma
/// segunda redacção dela.
#[test]
fn the_default_look_changes_nothing_inside_white_and_is_the_same_law() {
    for c in [[0.0, 0.18, 1.0], [0.5, 0.25, 0.125], [1.0; 3]] {
        assert_eq!(Look::default().apply(c), c);
    }
    let look = Look {
        exposure_stops: 1.5,
        view: ViewTransform::Neutral,
    };
    let c = [0.3, 0.9, 2.0];
    assert_eq!(look.apply(c), to_display(c, 1.5, ViewTransform::Neutral));
}

/// A `Standard` é o corte, e dentro do branco é a identidade **ao bit**.
#[test]
fn standard_is_the_identity_inside_white_and_the_clamp_outside() {
    for v in [0.0, 0.001, 0.18, 0.5, 1.0] {
        assert_eq!(to_display([v; 3], 0.0, ViewTransform::Standard), [v; 3]);
    }
    assert_eq!(
        to_display([4.0, 0.5, -1.0], 0.0, ViewTransform::Standard),
        [1.0, 0.5, 0.0]
    );
}

/// ⭐ **Um stop dobra a luz da CENA, antes da vista** — e não o pixel depois dela.
#[test]
fn a_stop_doubles_the_scene_light_before_the_view() {
    for view in ViewTransform::ALL {
        for stops in [-3.0, -1.0, 0.0, 1.5, 4.0] {
            for c in [[0.01, 0.2, 0.05], [0.3, 0.3, 0.3], [2.0, 0.7, 0.1]] {
                let one_stop_up = to_display(c, stops + 1.0, view);
                let twice_the_light = to_display(c.map(|v| v * 2.0), stops, view);
                for k in 0..3 {
                    assert!(
                        (one_stop_up[k] - twice_the_light[k]).abs() <= 1.0e-6,
                        "{view:?} a {stops} stops: {one_stop_up:?} contra {twice_the_light:?} para {c:?}"
                    );
                }
            }
        }
    }
}

/// A `Neutral` nunca escurece quando a luz sobe, e nunca chega a passar do branco.
#[test]
fn the_neutral_view_is_monotone_on_grey_and_never_passes_white() {
    let mut previous = -1.0_f32;
    let mut x = 1.0e-4_f32;
    while x < 1.0e6 {
        let y = to_display([x; 3], 0.0, ViewTransform::Neutral)[0];
        assert!(y >= previous, "desceu em {x}: {y} depois de {previous}");
        assert!(y <= 1.0, "passou do branco em {x}: {y}");
        previous = y;
        x *= 1.01;
    }
    assert!(
        previous > 0.999,
        "a luz muito forte não chegou ao branco: {previous}"
    );
}

/// ⭐ **Abaixo do joelho a cor autorada fica**: a vista só tira o desvio, igual nos três canais.
#[test]
fn the_neutral_view_keeps_the_authored_colour_below_the_knee() {
    let c = [0.5, 0.25, 0.1];
    let o = to_display(c, 0.0, ViewTransform::Neutral);
    assert!(((o[0] - o[1]) - (c[0] - c[1])).abs() <= 1.0e-6, "{o:?}");
    assert!(((o[1] - o[2]) - (c[1] - c[2])).abs() <= 1.0e-6, "{o:?}");
}

/// Uma luz que não é número nunca pinta `NaN` nem sai de `0..=1`.
#[test]
fn a_light_that_is_not_a_number_never_paints_outside_the_screen() {
    let nan = f32::NAN;
    let inf = f32::INFINITY;
    for view in ViewTransform::ALL {
        for c in [
            [nan, 0.5, 0.5],
            [inf, 0.2, 0.1],
            [-1.0, -2.0, -3.0],
            [f32::MAX; 3],
        ] {
            for stops in [0.0, 100.0, nan, inf, -inf] {
                let o = to_display(c, stops, view);
                assert!(
                    o.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
                    "{view:?} com {c:?} a {stops} stops pintou {o:?}"
                );
            }
        }
    }
}
