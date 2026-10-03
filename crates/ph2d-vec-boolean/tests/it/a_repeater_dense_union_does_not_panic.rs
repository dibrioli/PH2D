//! ⛔ **O `linesweeper` 0.4 PANICA com geometria FINITA** (`curve/mod.rs:364`, um `unwrap`) — medido
//! em 2026-10-03 numa forma presa com um *Repeater* de `39 × 39` cópias que giram, pela união do
//! contacto. A porta única do motor (`engine::binary_grouped_checked`) isola o pânico; este gate
//! dirige a MESMA geometria pela porta pública e exige uma resposta (`Some` ou `None`), nunca um
//! pânico.

use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::fx_repeat::RepeatSpec;
use ph2d_vec_scene::{ShapeKind, cook_tinted};

#[test]
fn a_dense_spinning_repeater_union_answers_instead_of_panicking() {
    let mut barra = cook_tinted(
        ShapeKind::RoundRect,
        [-8.5, 2.0],
        [-1.5, 3.0],
        &[0.5],
        [200, 140, 60],
    );
    barra.effects = vec![FxEntry::new(PathEffect::Repeat(RepeatSpec {
        copies_x: 39.0,
        move_x: -80.0,
        copies_y: 39.0,
        move_y: -80.0,
        spin: -72.0,
        orbit: -72.0,
    }))];
    let cozido = barra.cooked().into_owned();
    let mut so = cozido.clone();
    so.subpaths.retain(|c| c.closed);
    let r = std::panic::catch_unwind(|| ph2d_vec_boolean::resolve_overlap(&so).is_some());
    println!("  {} contornos: {r:?}", so.contour_count());
    assert!(r.is_ok(), "a união PANICOU — o pânico do linesweeper voltou a atravessar a porta");
}
