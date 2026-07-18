//! Gates da **seleção que sobrevive ao undo** — a política que o `apply_project` aplica.
//!
//! O `apply_project` exige `gfx` (janela + GPU) e não é alcançável headless; por isso a política
//! mora numa função pura. Estes gates são sobre ela, e o irmão de ARQUITETURA
//! (`tests/the_undo_preserves_the_vector_selection.rs`) prova que o `apply_project` a CHAMA — sem
//! ele, alguém pode reintroduzir o `vec_pen.clear()` sozinho e estes ficam todos verdes.

use super::surviving_selection;
use ph2d_vec_scene::{VecScene, rectangle};

/// **O QUE AINDA EXISTE CONTINUA SELECIONADO.** É o que devolve o overlay do envelope depois do
/// Ctrl+Z: a gaiola/os pinos são desenhados a partir da seleção, e sem ela a ferramenta fica
/// funcionando e **invisível** (foi assim que o Enio o descreveu).
#[test]
fn a_shape_that_survives_the_restore_stays_selected() {
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
    assert_eq!(surviving_selection(&[a, b], &scene), vec![a, b]);
}

/// **O QUE O UNDO FEZ DESAPARECER SAI DA SELEÇÃO.** Desfazer a criação de uma forma tem de a tirar
/// da seleção — mantê-la deixaria um id pendurado que o mapa nunca resolve, e o gizmo apontaria
/// para nada.
#[test]
fn a_shape_the_restore_removed_is_dropped() {
    let mut before = VecScene::new();
    let a = before.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = before.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));

    // O estado restaurado é ANTERIOR à criação de `b`.
    let mut restored = VecScene::new();
    let a2 = restored.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    assert_eq!(
        a, a2,
        "os ids da cena são determinísticos — premissa do fixture"
    );

    assert_eq!(
        surviving_selection(&[a, b], &restored),
        vec![a],
        "a forma que o undo desfez continuou selecionada"
    );
}

/// **Seleção vazia continua vazia** — e uma cena vazia não ressuscita nada.
#[test]
fn nothing_selected_stays_nothing() {
    assert!(surviving_selection(&[], &VecScene::new()).is_empty());
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    assert!(surviving_selection(&[], &scene).is_empty());
    assert!(surviving_selection(&[a], &VecScene::new()).is_empty());
}
