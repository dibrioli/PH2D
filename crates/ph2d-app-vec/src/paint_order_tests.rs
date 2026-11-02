//! Gates da ORDEM da pilha do lado da shell (F45): a base tem lugar entre as camadas, e os verbos
//! (↑/↓, apagar, mover pelo arrasto do ⚙) andam na pilha INTEIRA.

use super::{StackVerb, apply, layers_differ, move_item, published, remove, shift, visible_order};
use StackItem::{BaseFill, BaseStroke, Entry};
use ph2d_vec_scene::{
    Paint, PaintEntry, Place, Rgba8, StackItem, StrokeSpec, VecPath, VecScene, VecVertex,
};

/// Duas formas com preenchimento e traço de base e uma camada de traço por cima.
fn cena() -> (VecScene, Vec<u64>) {
    let mut scene = VecScene::default();
    let ids = (0..2)
        .map(|_| {
            scene.push_path(VecPath {
                verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
                    .map(VecVertex::corner)
                    .to_vec(),
                closed: true,
                fill: Some(Paint::solid(Rgba8::new(1, 2, 3, 255))),
                stroke: Some(StrokeSpec::new(Rgba8::new(4, 5, 6, 255), 2.0)),
                paints: vec![PaintEntry::stroke(StrokeSpec::new(
                    Rgba8::new(7, 8, 9, 255),
                    6.0,
                ))],
                ..VecPath::default()
            })
        })
        .collect();
    (scene, ids)
}

fn ordem(scene: &VecScene, id: u64) -> Vec<StackItem> {
    scene.path(id).expect("forma").draw_order().collect()
}

/// ⭐ **↓ numa camada passa por baixo do traço de BASE** (a vizinha à vista), e o índice dela no
/// documento não muda. *Mutação: o `swap` de antes (só entre camadas) ⇒ nada se move.*
#[test]
fn down_passes_a_layer_below_the_base_stroke() {
    let (mut scene, ids) = cena();
    assert!(shift(&mut scene, &ids, 0, false));
    assert_eq!(ordem(&scene, ids[0]), [BaseFill, Entry(0), BaseStroke]);
    // e ↑ devolve-a por cima dele
    assert!(shift(&mut scene, &ids, 0, true));
    assert_eq!(ordem(&scene, ids[0]), [BaseFill, BaseStroke, Entry(0)]);
}

/// ⭐ **O arrasto do ⚙ move em TODA a seleção**, e o verbo `Move` lê a queda guardada pelo painel.
#[test]
fn the_move_verb_applies_the_panels_landing_to_every_selected_shape() {
    let (mut scene, ids) = cena();
    ph2d_panel_vector::state::set_paint_move(Some((BaseFill, Place::Above(Entry(0)))));
    assert!(apply(&mut scene, &ids, StackVerb::Move));
    for id in &ids {
        assert_eq!(ordem(&scene, *id), [BaseStroke, Entry(0), BaseFill]);
    }
    // a queda lê-se uma vez
    assert!(!apply(&mut scene, &ids, StackVerb::Move));
}

/// Apagar uma camada deixa a base entre as mesmas vizinhas.
#[test]
fn removing_keeps_the_base_in_place() {
    let (mut scene, ids) = cena();
    assert!(move_item(
        &mut scene,
        &ids,
        BaseFill,
        Place::Above(Entry(0))
    ));
    assert!(remove(&mut scene, &ids, 0));
    assert_eq!(ordem(&scene, ids[0]), [BaseStroke, BaseFill]);
}

/// ⭐ **A ordem é estrutura**: duas formas com a base em lugares diferentes são «camadas mistas»,
/// e o painel recebe a ordem do primário. *Mutação: comparar só a espécie das camadas ⇒ verde.*
#[test]
fn a_different_base_order_is_a_mixed_stack_and_the_view_carries_the_order() {
    let (mut scene, ids) = cena();
    assert!(!layers_differ(&scene, &ids));
    assert!(move_item(
        &mut scene,
        &ids[..1],
        BaseFill,
        Place::Above(BaseStroke)
    ));
    assert!(layers_differ(&scene, &ids));
    let vivo = ph2d_vec_scene::VecViewState::default();
    let doc = ph2d_editor_core::DocPx::new(1.0);
    let a = published(&scene, &ids[..1], &vivo, doc).expect("publica");
    assert_eq!(a.order, [BaseStroke, BaseFill, Entry(0)]);
}

/// A base sem tinta não é uma linha à vista.
#[test]
fn an_absent_base_is_not_a_visible_row() {
    let (mut scene, ids) = cena();
    scene.path_mut(ids[0]).expect("forma").fill = None;
    assert_eq!(
        visible_order(scene.path(ids[0]).expect("forma")),
        [BaseStroke, Entry(0)]
    );
}

/// ⭐ **Presenças de base diferentes são «camadas mistas»** (auditoria do fecho): o painel conta a
/// pilha À VISTA para o ↑/↓, e a app também — uma forma sem traço e outra com ele dariam contas
/// diferentes. *Mutação: comparar o `draw_order` (bases ausentes incluídas) ⇒ verde.*
#[test]
fn a_different_base_presence_is_a_mixed_stack_and_the_view_has_only_present_bases() {
    let (mut scene, ids) = cena();
    scene.path_mut(ids[0]).expect("forma").stroke = None;
    assert!(layers_differ(&scene, &ids));
    let vivo = ph2d_vec_scene::VecViewState::default();
    let doc = ph2d_editor_core::DocPx::new(1.0);
    let a = published(&scene, &ids[..1], &vivo, doc).expect("publica");
    assert_eq!(a.order, [BaseFill, Entry(0)]);
}
