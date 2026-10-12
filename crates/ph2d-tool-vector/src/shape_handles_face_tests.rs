//! Gates da cara das alças (F11 — o idioma dos *Live Shapes* do Illustrator, doc 17 §1).

use super::*;
use crate::shape_handles::{Law, Spot, fields_without_handle, handles_of};

fn faces(kind: ShapeKind) -> Vec<Glyph> {
    handles_of(kind).iter().map(glyph_of).collect()
}

/// ⭐ Cada alça com a cara do que faz: ⊙ arredonda, ⇕ conta, ⊙ na haste gira, ○ mede um raio.
#[test]
fn each_handle_wears_the_face_of_what_it_does() {
    use Glyph::{Angle, Corner, Count, Radius};
    assert_eq!(
        faces(ShapeKind::RoundRect),
        [Corner, Corner, Corner, Corner, Count]
    );
    assert_eq!(
        faces(ShapeKind::Polygon),
        [Count, Corner, Radius, Count, Count, Count]
    );
    assert_eq!(
        faces(ShapeKind::Star),
        [Count, Corner, Radius, Corner, Count, Count, Count, Count]
    );
    assert_eq!(faces(ShapeKind::Ellipse), [Corner, Radius, Angle, Angle]);
    assert_eq!(faces(ShapeKind::Spiral), [Radius, Count, Count]);
}

/// ⭐ (F12) **O que o cartão segurava é um ⇕ da fila** (o dono, 12/10: «os ajustes que estão nos
/// modais passam para setas lado a lado … e os modais retirados de todas as formas»).
#[test]
fn what_was_on_the_card_is_an_arrow_on_the_side() {
    for (kind, field) in [
        (ShapeKind::Star, 5),
        (ShapeKind::Polygon, 3),
        (ShapeKind::Polygon, 4),
        (ShapeKind::RoundRect, 5),
        (ShapeKind::Spiral, 1),
        (ShapeKind::Spiral, 2),
    ] {
        assert!(
            handles_of(kind)
                .iter()
                .any(|h| h.spot == Spot::Rail && h.law == Law::Steps && h.fields == [field]),
            "{kind:?}: o campo {field} não tem o ⇕ dele"
        );
    }
    for &kind in ph2d_vec_scene::ALL_SHAPES {
        assert_eq!(
            fields_without_handle(kind).count(),
            0,
            "{kind:?} ainda abre o cartão"
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
    // Uma ESCOLHA diz o nome da opção, não o índice.
    v[4] = 2.0;
    let corner = handles_of(ShapeKind::Star)
        .iter()
        .find(|h| h.fields == [4])
        .unwrap();
    assert_eq!(
        readout(ShapeKind::Star, corner, &v, &name, &len),
        "Corner: Concave"
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
