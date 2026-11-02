use super::*;
use StackItem::{BaseFill, BaseStroke, Entry};

/// As linhas do topo para o chão: uma camada de traço, o traço de base, o preenchimento de base.
const ROWS: [StackItem; 3] = [Entry(0), BaseStroke, BaseFill];

/// ⭐ Largar na ranhura do fundo põe a linha logo ABAIXO da que fica por cima dela; noutra ranhura,
/// logo ACIMA da que fica por baixo. *Mutação: trocar Above/Below ⇒ o preenchimento cai no lugar
/// errado.*
#[test]
fn the_landing_is_relative_to_the_neighbour_left_in_place() {
    // o preenchimento (fundo, k = 2) para o topo (t = 0): por cima da camada
    assert_eq!(
        landing(&ROWS, 2, 0),
        Some((BaseFill, Place::Above(Entry(0))))
    );
    // a camada (topo) para o fundo: por baixo do preenchimento
    assert_eq!(
        landing(&ROWS, 0, 2),
        Some((Entry(0), Place::Below(BaseFill)))
    );
    // o meio para o topo: por cima da camada
    assert_eq!(
        landing(&ROWS, 1, 0),
        Some((BaseStroke, Place::Above(Entry(0))))
    );
}

/// Na própria ranhura (ou fora da lista) não há queda.
#[test]
fn its_own_slot_is_no_move() {
    assert_eq!(landing(&ROWS, 1, 1), None);
    assert_eq!(landing(&ROWS, 5, 0), None);
}

/// ⭐ A queda aplicada ao documento dá a ordem que o vão mostrou (a pilha do chão ao topo).
#[test]
fn the_landing_applied_to_a_shape_gives_the_previewed_order() {
    let mut p = ph2d_vec_scene::VecPath {
        fill: Some(ph2d_vec_scene::Paint::solid(ph2d_vec_scene::Rgba8::new(
            1, 2, 3, 255,
        ))),
        stroke: Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(4, 5, 6, 255),
            2.0,
        )),
        paints: vec![ph2d_vec_scene::PaintEntry::stroke(
            ph2d_vec_scene::StrokeSpec::new(ph2d_vec_scene::Rgba8::new(7, 8, 9, 255), 8.0),
        )],
        ..ph2d_vec_scene::VecPath::default()
    };
    let (it, onde) = landing(&ROWS, 2, 0).expect("move");
    assert!(p.move_paint(it, onde));
    let topo_primeiro: Vec<_> = p
        .draw_order()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    assert_eq!(topo_primeiro, [BaseFill, Entry(0), BaseStroke]);
}
