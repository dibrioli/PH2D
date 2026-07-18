//! Gates da ponte da seção Effects.

use super::*;
use ph2d_vec_scene::{VecPath, VecVertex};

fn scene_with_square() -> (VecScene, VecPathId) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    (scene, id)
}

/// A seção é POR-CAMINHO: com zero ou dois selecionados não há referente.
#[test]
fn the_section_governs_exactly_one_selected_path() {
    assert_eq!(sole_path(&[]), None);
    assert_eq!(sole_path(&[7]), Some(7));
    assert_eq!(sole_path(&[7, 9]), None, "dois: 'o Trim' não tem referente");
}

/// **Pôr o Trim não pode mudar o desenho.**
///
/// Ele nasce no ponto neutro, que é um no-op byte-idêntico — senão o artista veria a forma
/// saltar no instante do clique, antes de tocar em qualquer parâmetro.
#[test]
fn adding_the_trim_does_not_move_a_single_point() {
    let (mut scene, id) = scene_with_square();
    let before = scene.path(id).expect("path").cooked().into_owned();
    toggle_trim(&mut scene, id);
    assert!(trim_of(&scene, id).is_some(), "o Trim entrou");
    let after = scene.path(id).expect("path").cooked().into_owned();
    assert_eq!(before.verts, after.verts, "o clique não pode mover nada");
}

/// O toggle **tira** o que pôs — e o caminho volta a ser byte-idêntico ao original.
#[test]
fn the_toggle_removes_what_it_added() {
    let (mut scene, id) = scene_with_square();
    toggle_trim(&mut scene, id);
    toggle_trim(&mut scene, id);
    assert_eq!(trim_of(&scene, id), None);
    assert!(
        scene.path(id).expect("path").effects.is_empty(),
        "a pilha volta a ficar VAZIA — é ela que faz o cooked() emprestar em vez de alocar"
    );
}

/// Cada parâmetro escreve no seu campo, e **só** no seu.
#[test]
fn each_parameter_writes_its_own_field() {
    let (mut scene, id) = scene_with_square();
    toggle_trim(&mut scene, id);
    set_trim_param(&mut scene, id, TrimParam::Start, 0.2);
    assert_eq!(trim_of(&scene, id), Some((0.2, 1.0, 0.0)));
    set_trim_param(&mut scene, id, TrimParam::End, 0.7);
    assert_eq!(trim_of(&scene, id), Some((0.2, 0.7, 0.0)));
    set_trim_param(&mut scene, id, TrimParam::Offset, 0.4);
    assert_eq!(trim_of(&scene, id), Some((0.2, 0.7, 0.4)));
}

/// Ajustar sem Trim é no-op — a ponte não depende de o painel ter razão sobre o que oferece.
#[test]
fn setting_a_parameter_without_a_trim_is_a_no_op() {
    let (mut scene, id) = scene_with_square();
    set_trim_param(&mut scene, id, TrimParam::End, 0.5);
    assert_eq!(trim_of(&scene, id), None);
    assert!(scene.path(id).expect("path").effects.is_empty());
}

/// **Tirar remove TODOS os Trims**, não só o primeiro: a UI expõe um, mas um documento vindo
/// de código pode ter mais, e deixar órfãos invisíveis seria a pior das saídas.
#[test]
fn removing_clears_every_trim_not_just_the_first() {
    let (mut scene, id) = scene_with_square();
    let p = scene.path_mut(id).expect("path");
    p.effects = vec![
        PathEffect::Trim(TrimSpec::default()),
        PathEffect::Trim(TrimSpec::default()),
    ];
    toggle_trim(&mut scene, id);
    assert!(scene.path(id).expect("path").effects.is_empty());
}
