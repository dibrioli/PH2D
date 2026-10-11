//! Gates da cara das alças (F11 — o idioma dos *Live Shapes* do Illustrator, doc 17 §1).

use super::*;
use crate::shape_handles::{fields_without_handle, handles_of};

fn faces(kind: ShapeKind) -> Vec<Glyph> {
    handles_of(kind).iter().map(glyph_of).collect()
}

/// ⭐ Cada alça com a cara do que faz: ⊙ arredonda, ⇕ conta, ⊙ na haste gira, ○ mede um raio.
#[test]
fn each_handle_wears_the_face_of_what_it_does() {
    use Glyph::{Angle, Corner, Count, Radius};
    assert_eq!(faces(ShapeKind::RoundRect), [Corner; 4]);
    assert_eq!(faces(ShapeKind::Polygon), [Count, Corner]);
    assert_eq!(faces(ShapeKind::Star), [Count, Corner, Radius, Corner]);
    assert_eq!(faces(ShapeKind::Ellipse), [Angle, Angle]);
    assert_eq!(faces(ShapeKind::Spiral), [Radius]);
}

/// ⭐ **Simplificar é tirar do CANVAS, não do produto** (o dono na F10: *«não podemos reduzir
/// parâmetros»*): o que o Illustrator não põe no canvas — a envergadura da estrela e do polígono, o
/// furo da pizza, o raio interno da espiral — vai ao CARTÃO (a conta `fields_without_handle`).
#[test]
fn what_left_the_canvas_is_on_the_card() {
    for (kind, field) in [
        (ShapeKind::Star, 5),
        (ShapeKind::Polygon, 3),
        (ShapeKind::Ellipse, 2),
        (ShapeKind::Spiral, 2),
    ] {
        assert!(
            fields_without_handle(kind).any(|i| i == field),
            "{kind:?}: o campo {field} saiu do canvas e não foi ao cartão"
        );
    }
}

/// A etiqueta diz o campo e o número na unidade dele (a ficha «Points: 7» do Illustrator).
#[test]
fn the_readout_names_the_field_and_its_number() {
    let star = handles_of(ShapeKind::Star);
    let mut v = [0.0; 8];
    v[0] = 7.0;
    v[1] = 0.382;
    let name = |s: &'static str| s.to_owned();
    let len = |w: f64| format!("{w} m");
    assert_eq!(
        readout(ShapeKind::Star, &star[0], &v, &name, &len),
        "Points: 7"
    );
    assert_eq!(
        readout(ShapeKind::Star, &star[2], &v, &name, &len),
        "Inner: 0.38"
    );
    v[2] = 0.25;
    assert_eq!(
        readout(ShapeKind::Star, &star[3], &v, &name, &len),
        "Tip round: 0.25 m"
    );
}

/// O círculo tracejado do raio passa pelo ponto agarrado (a elipse da caixa, normalizada); fora
/// da lei `Radial` não há círculo.
#[test]
fn the_radius_ring_runs_through_the_handle() {
    let star = handles_of(ShapeKind::Star);
    let half = [2.0, 1.0];
    let at = [0.6, -0.3];
    let ring = radius_ring(&star[2], at, half, 64);
    let k = (at[0] / half[0]).hypot(at[1] / half[1]);
    assert_eq!(ring.len(), 65);
    for p in &ring {
        assert!(((p[0] / half[0]).hypot(p[1] / half[1]) - k).abs() < 1e-12);
    }
    assert!(radius_ring(&star[0], at, half, 64).is_empty());
}
