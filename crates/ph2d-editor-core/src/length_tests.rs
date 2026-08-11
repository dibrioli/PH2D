//! Gates da porta única de comprimento (plano 25 §9, a W6 — o rótulo de distância).
//!
//! O gate que carrega a wave é o primeiro: ele não conhece a implementação de
//! nenhuma das duas superfícies, só exige que elas digam **o mesmo número para a
//! mesma distância**.

use super::*;
use crate::project::{DEFAULT_PIXELS_PER_METER, DisplayUnit, ProjectSettings};
use crate::ruler::label_text;

/// A régua do artista por default: 100 px por metro, lendo em PIXELS.
fn shipping() -> LengthDisplay {
    LengthDisplay::of(&ProjectSettings::default())
}

/// **As duas superfícies dizem o mesmo número.**
///
/// O painel de Grid Snap converte por `display_unit.from_meters` (é literalmente
/// o que o `NumberInput` do passo mostra) e a régua imprime `label_text`. Antes
/// desta wave, para a MESMA distância de mundo, o painel dizia **150** e a régua
/// **2** — não por arredondamento, mas porque `paint_rulers` não recebia as
/// settings e portanto não *conseguia* converter.
///
/// Mutação que tem de sangrar: `label_text` voltar a formatar o valor CRU.
#[test]
fn the_ruler_and_the_panel_say_the_same_number_for_the_same_distance() {
    let d = shipping();
    assert_eq!(d.unit, DisplayUnit::Pixels, "o default do projeto");
    for world in [0.5_f64, 1.0, 1.5, 12.0] {
        let panel = f64::from(d.unit.from_meters(world as f32, DEFAULT_PIXELS_PER_METER));
        // O passo em MUNDO que a régua escolheria num zoom de 1 px por unidade
        // de display; o que importa aqui é o VALOR, e ele não depende do passo.
        let printed: f64 = label_text(world, 1.0, d)
            .parse()
            .expect("a régua imprime um número");
        assert!(
            (printed - panel).abs() < 0.5,
            "world {world}: régua {printed} contra painel {panel} — as duas \
             superfícies estão a descrever a mesma distância"
        );
    }
}

/// **Um projeto em METROS é byte-idêntico ao que já shipava** — o CONTROLE.
///
/// `from_meters` é a identidade nessa unidade, então a conversão não pode mover
/// um caractere. Sem este gate, a wave inteira poderia estar a mudar o mundo
/// para todo mundo em vez de só para quem escolheu pixels.
#[test]
fn reading_in_metres_prints_exactly_what_the_old_ruler_printed() {
    let d = LengthDisplay {
        unit: DisplayUnit::Meters,
        pixels_per_meter: DEFAULT_PIXELS_PER_METER,
    };
    // Os mesmos pares que o `ruler_tests` pina desde a W6.2.
    assert_eq!(label_text(0.2, 0.2, d), "0.2");
    assert_eq!(label_text(1.0, 1.0, d), "1");
    assert_eq!(label_text(0.05, 0.05, d), "0.05");
    assert_eq!(
        label_text(-0.0, 1.0, d),
        "0",
        "o zero negativo lê como erro"
    );
}

/// **As casas vêm do passo CONVERTIDO, não do passo de mundo.**
///
/// Um passo de meio metro é `0,5` em metros (uma casa) e `50` em pixels
/// (nenhuma). Converter só o valor imprimiria `150.0` — uma casa decimal que o
/// número não tem resolução para honrar.
///
/// Mutação que tem de sangrar: `decimals_for(step_world)` em vez do convertido.
#[test]
fn the_decimals_come_from_the_step_the_artist_reads() {
    let px = shipping();
    assert_eq!(px.text(1.5, 0.5), "150", "meio metro = 50 px: sem casas");
    let m = LengthDisplay {
        unit: DisplayUnit::Meters,
        pixels_per_meter: DEFAULT_PIXELS_PER_METER,
    };
    assert_eq!(m.text(1.5, 0.5), "1.5", "meio metro em metros: uma casa");
}

/// **A precisão de um rótulo flutuante é a mesma da régua, no mesmo zoom.**
///
/// As duas perguntam `label_step`, então não há como uma mostrar `12` e a outra
/// `12,34` para a mesma distância no mesmo instante.
#[test]
fn the_floating_label_borrows_the_rulers_cadence() {
    let d = shipping();
    for px_per_world in [1.0_f64, 10.0, 100.0, 1000.0] {
        let step = crate::ruler::label_step(px_per_world);
        assert_eq!(
            d.text_at_zoom(1.234_567, px_per_world),
            d.text(1.234_567, step),
            "px_per_world {px_per_world}"
        );
    }
}

/// **A conversão é feita em `f64`.**
///
/// Uma coordenada de régua longe da origem, em pixels, passa da resolução do
/// `f32`: `1e6 m × 100 = 1e8`, e o `f32` só carrega ~7 dígitos, então o rótulo
/// imprimiria um número redondo que não é o do traço.
///
/// Mutação que tem de sangrar: `from_meters_f64` delegar à versão `f32`.
#[test]
fn a_far_coordinate_survives_the_conversion() {
    let d = shipping();
    assert_eq!(d.text(1_000_000.0, 1.0), "100000000");
    assert_eq!(d.text(1_000_001.0, 1.0), "100000100");
}

/// **As duas larguras são a MESMA regra** — a estreita delega.
#[test]
fn the_narrow_conversion_agrees_with_the_wide_one() {
    for unit in [DisplayUnit::Meters, DisplayUnit::Pixels] {
        for m in [0.0_f32, 1.0, -2.5, 37.75] {
            let narrow = unit.from_meters(m, DEFAULT_PIXELS_PER_METER);
            let wide = unit.from_meters_f64(f64::from(m), DEFAULT_PIXELS_PER_METER);
            assert!(
                (f64::from(narrow) - wide).abs() < 1e-6,
                "{unit:?} {m}: {narrow} contra {wide}"
            );
        }
    }
}

/// **O default da régua é o default do projeto** — para que uma fixture que não
/// fala de unidade meça o que o app mede ao abrir.
#[test]
fn the_default_ruler_is_the_projects_ruler() {
    assert_eq!(LengthDisplay::default(), shipping());
}
