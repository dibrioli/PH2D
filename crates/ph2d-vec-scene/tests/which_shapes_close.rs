//! MEDIÇÃO (doc 89 §14, §7.3): quais das 47 formas do `cook()` produzem contorno
//! FECHADO, e portanto podem ser preenchidas. A cerca do `source.shape` afirma que
//! "Arc (wedge) e Spiral são follow-ups (precisam de wedge-close / traço)" — este
//! teste diz o número em vez de o assumir.
use ph2d_vec_scene::{ALL_SHAPES, ShapeKind, cook};

#[test]
#[ignore = "medicao: cargo test -p ph2d-vec-scene --test which_shapes_close -- --ignored --nocapture"]
fn which_of_the_catalogue_can_be_filled() {
    let (a, b) = ([-1.0, -1.0], [1.0, 1.0]);
    let (mut open, mut closed) = (Vec::new(), Vec::new());
    for (i, k) in ALL_SHAPES.iter().enumerate() {
        let v = k.defaults();
        let p = cook(*k, a, b, &v);
        let line = format!(
            "{i:>2} {k:?} verts={} rule={:?}",
            p.verts.len(),
            p.fill_rule
        );
        if p.closed {
            closed.push(line)
        } else {
            open.push(line)
        }
    }
    eprintln!("  FECHADAS ({}):", closed.len());
    for l in &closed {
        eprintln!("    {l}");
    }
    eprintln!("  ABERTAS ({}):", open.len());
    for l in &open {
        eprintln!("    {l}");
    }
}
