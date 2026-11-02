use super::*;
use crate::{Paint, PaintEntry, PaintRef, Rgba8, StrokeSpec};
use StackItem::{BaseFill, BaseStroke, Entry};

const C: Rgba8 = Rgba8 {
    r: 9,
    g: 9,
    b: 9,
    a: 255,
};

fn forma(camadas: usize) -> VecPath {
    VecPath {
        fill: Some(Paint::Solid(C)),
        stroke: Some(StrokeSpec::new(C, 2.0)),
        paints: (0..camadas)
            .map(|k| PaintEntry::stroke(StrokeSpec::new(C, 10.0 + k as f64)))
            .collect(),
        ..VecPath::default()
    }
}

/// O neutro é a ordem de antes da F45: base fill, base stroke, camadas. *Mutação: `Default` derivado
/// (`[0, 0]`) ⇒ as duas bases no mesmo sítio.*
#[test]
fn the_neutral_order_is_the_pre_f45_order() {
    let p = forma(2);
    assert_eq!(
        p.draw_order().collect::<Vec<_>>(),
        [BaseFill, BaseStroke, Entry(0), Entry(1)]
    );
}

/// ⭐ O traço de base pode subir por cima de uma camada, e o preenchimento por cima do traço — e a
/// porta `paint_stack` desenha nessa ordem.
#[test]
fn a_base_fill_can_cover_the_base_stroke_and_the_stack_follows() {
    let mut p = forma(1);
    assert!(p.move_paint(BaseFill, Place::Above(Entry(0))));
    assert_eq!(
        p.draw_order().collect::<Vec<_>>(),
        [BaseStroke, Entry(0), BaseFill]
    );
    let tipos: Vec<_> = p
        .paint_stack()
        .map(|d| matches!(d.paint, PaintRef::Fill(_)))
        .collect();
    assert_eq!(tipos, [false, false, true]);
}

/// Mover uma camada para baixo da base renumera `paints` e a base acompanha.
#[test]
fn moving_a_layer_below_the_base_reindexes_the_stack() {
    let mut p = forma(2);
    assert!(p.move_paint(Entry(1), Place::Below(BaseFill)));
    assert_eq!(
        p.draw_order().collect::<Vec<_>>(),
        [Entry(0), BaseFill, BaseStroke, Entry(1)]
    );
    // a antiga camada 1 (largura 11) é agora a 0
    assert_eq!(p.paints[0].width(), Some(11.0));
}

/// Apagar uma camada deixa a base entre as mesmas vizinhas.
#[test]
fn removing_a_layer_keeps_the_base_between_its_neighbours() {
    let mut p = forma(3);
    assert!(p.move_paint(BaseStroke, Place::Above(Entry(1))));
    // F, E0, E1, S, E2
    assert!(p.remove_paint(0));
    assert_eq!(
        p.draw_order().collect::<Vec<_>>(),
        [BaseFill, Entry(0), BaseStroke, Entry(1)]
    );
}

/// Uma posição que não cabe na pilha (a lista trocada inteira noutro sítio) prende-se ao topo.
#[test]
fn a_stale_base_position_clamps_and_never_panics() {
    let mut p = forma(0);
    p.base_order = BaseOrder {
        fill: 40,
        stroke: 40,
    };
    assert_eq!(p.draw_order().collect::<Vec<_>>(), [BaseStroke, BaseFill]);
}

/// Uma lista que não é permutação é recusada.
#[test]
fn a_non_permutation_is_refused() {
    let mut p = forma(1);
    assert!(!p.set_draw_order(&[BaseFill, BaseFill, Entry(0)]));
    assert!(!p.set_draw_order(&[BaseFill, BaseStroke]));
}
