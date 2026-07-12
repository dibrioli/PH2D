//! Testes de `shapes.rs` — arquivo irmao (teto de 700 LOC por arquivo, HR-18).
//!
//! Ligado por `#[path]` no modulo pai, entao `use super::*` continua valendo.

use super::*;
use ph2d_vec_scene::{ALL_SHAPES, MAX_SHAPE_FIELDS};

/// **Gate anti-forma-sem-UI:** toda forma que o `cook` desenha tem descritor aqui —
/// senão ela seria desenhável e invisível no painel (ou vice-versa: um descritor sem
/// forma). E nenhuma passa do teto de campos.
#[test]
fn every_cookable_shape_has_a_ui_descriptor_and_fits_the_field_cap() {
    for &k in ALL_SHAPES {
        let d = SHAPES.iter().find(|d| d.kind == k);
        assert!(d.is_some(), "{k:?} cozinha mas não tem descritor de UI");
        let d = d.unwrap();
        assert!(
            d.fields.len() <= MAX_SHAPE_FIELDS,
            "{k:?}: {} campos > teto {MAX_SHAPE_FIELDS}",
            d.fields.len()
        );
    }
    assert_eq!(
        SHAPES.len(),
        ALL_SHAPES.len(),
        "descritor órfão no catálogo"
    );
}

/// O default de toda forma CABE na faixa declarada dos campos dela — senão a forma
/// nasceria já clampada, e o número do painel discordaria da geometria no 1º frame.
#[test]
fn the_geometry_defaults_sit_inside_the_ui_ranges() {
    for &k in ALL_SHAPES {
        let d = desc(k);
        let defs = k.defaults();
        for (i, f) in d.fields.iter().enumerate() {
            // O raio de canto é o caso especial: o default é MUNDO, a faixa é PX.
            if f.unit == FieldUnit::Px {
                continue;
            }
            assert!(
                defs[i] >= f.min && defs[i] <= f.max,
                "{k:?}.{}: default {} fora de [{}, {}]",
                f.label,
                defs[i],
                f.min,
                f.max
            );
        }
    }
}

/// A fronteira de unidade fecha: px → mundo → px devolve o mesmo número. É o que
/// impede o raio de saltar de escala a cada clique.
#[test]
fn the_px_fields_round_trip_across_the_unit_boundary() {
    const PTW: f64 = 0.01;
    let mut ui: ShapeValues = [0.0; MAX_SHAPE_FIELDS];
    ui[0] = 5.0; // Sides (Count — não viaja)
    ui[1] = 30.0; // Radius (Px — viaja)
    let world = to_world(ShapeKind::Polygon, &ui, PTW);
    assert!((world[0] - 5.0).abs() < 1e-9, "contagem não vira mundo");
    assert!((world[1] - 0.3).abs() < 1e-9, "30 px x 0.01 = 0.3 de mundo");
    let back = to_ui(ShapeKind::Polygon, &world, PTW);
    assert!((back[1] - 30.0).abs() < 1e-9, "voltou a 30 px");
}

/// O clamp respeita a faixa e arredonda as contagens (um polígono de 4.7 lados não
/// existe).
#[test]
fn clamp_bounds_the_fields_and_rounds_the_counts() {
    let mut v: ShapeValues = [0.0; MAX_SHAPE_FIELDS];
    v[0] = 4.7; // Sides
    v[1] = 9_999.0; // Radius (px)
    clamp(ShapeKind::Polygon, &mut v);
    assert!((v[0] - 5.0).abs() < 1e-9, "lados arredondam");
    assert!((v[1] - 500.0).abs() < 1e-9, "raio clampa no teto");
}
