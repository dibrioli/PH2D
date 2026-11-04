//! A cor do seletor na tinta de base de UMA forma, pela lei da pilha (plano 32).

use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, rectangle};

fn forma() -> VecPath {
    let mut p = rectangle([0.0, 0.0], [2.0, 2.0]);
    p.fill = Some(Paint::Solid(Rgba8::new(10, 20, 30, 128)));
    p.stroke = Some(StrokeSpec::new(Rgba8::new(40, 50, 60, 200), 1.0));
    p
}

#[test]
fn the_picker_keeps_the_alpha_it_did_not_touch_and_an_open_picker_writes_nothing() {
    let mut p = forma();
    let igual = Rgba8::new(10, 20, 30, 128);
    assert!(!crate::vector_bridge::pick_base(&mut p, true, igual, false));
    let nova = Rgba8::new(1, 2, 3, 128);
    assert!(crate::vector_bridge::pick_base(&mut p, true, nova, true));
    assert_eq!(p.fill, Some(Paint::Solid(nova)));
    assert_eq!(
        p.base_stroke().map(StrokeSpec::color),
        Some(Rgba8::new(40, 50, 60, 200))
    );
    let traco = Rgba8::new(7, 8, 9, 200);
    assert!(crate::vector_bridge::pick_base(&mut p, false, traco, true));
    assert_eq!(p.base_stroke().map(StrokeSpec::color), Some(traco));
    assert_eq!(
        p.fill,
        Some(Paint::Solid(nova)),
        "o traço não mexe no preenchimento"
    );
}

#[test]
fn the_picker_on_a_hidden_base_edits_it_hidden() {
    let mut p = forma();
    assert!(p.set_fill_hidden(true));
    let nova = Rgba8::new(1, 2, 3, 128);
    assert!(crate::vector_bridge::pick_base(&mut p, true, nova, true));
    assert!(p.fill_is_hidden(), "a escolha não destapou a tinta");
    assert_eq!(p.base_fill(), Some(&Paint::Solid(nova)));
}
