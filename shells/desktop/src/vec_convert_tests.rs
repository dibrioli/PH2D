//! Seam de **"Convert to Curves"** — prova, headless e do jeito que o shell o percorre, que o
//! converter ASSA a pilha de efeitos (a metade que faltava; Enio 2026-07-19: *"Convert to
//! Curves não funciona para isso"*). Sem este teste a fiação ficava verde nos unit tests do
//! motor e MORTA no produto.

use super::*;
use crate::vec_entities::VecEntityMap;
use ph2d_vec_scene::{VecPath, VecVertex};

/// Um quadrado sincronizado numa entidade, com um Zig Zag ATIVO adicionado pelo MESMO caminho
/// do produto (`fx_bridge`) — não um `PathEffect` fabricado à mão.
fn scene_with_effect_path() -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    // Um Zig Zag levado ao máximo do 1º parâmetro — ativo, muda a geometria de facto.
    crate::fx_bridge::add(&mut scene, id, 1);
    crate::fx_bridge::set_param(&mut scene, id, 0, 0, 1.0);
    (sim, scene, map, id)
}

/// **Convert to Curves ASSA a pilha de efeitos** de um caminho SEM forma viva (`VecShape`) — o
/// caso que não funcionava, porque o botão só olhava para o shape. Depois de converter, a pilha
/// está vazia e a geometria autorada é a cozida.
#[test]
fn convert_to_curves_bakes_the_effect_stack() {
    let (mut sim, mut scene, mut map, id) = scene_with_effect_path();
    // A aparência (cozida) e a fonte (autorada) DIFEREM — o efeito está mesmo ativo.
    let cooked = scene.path(id).unwrap().cooked().into_owned();
    assert_ne!(
        cooked.verts,
        scene.path(id).unwrap().verts,
        "pré-condição: o Zig Zag tem de mudar a geometria, senão o bake não prova nada"
    );

    let new_sel = crate::vec_convert::to_curves(&mut sim, &mut scene, &mut map, &[id]);

    assert!(
        new_sel.contains(&id),
        "um caminho sem forma viva sobrevive à conversão (fica intacto, só perde os efeitos)"
    );
    let p = scene.path(id).unwrap();
    assert!(
        p.effects.is_empty(),
        "a pilha de efeitos tem de sair VAZIA — é o que Convert to Curves passou a fazer"
    );
    assert_eq!(
        p.verts, cooked.verts,
        "a geometria autorada virou a cozida (Expand Appearance)"
    );
}

/// Convert to Curves num caminho SEM efeitos é um no-op para a pilha (não há o que assar) e não
/// derruba o caminho — a conversão continua a servir os outros casos (texto/forma) sem estragar
/// um caminho cru.
#[test]
fn convert_to_curves_leaves_a_plain_path_alone() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let before = scene.path(id).unwrap().clone();

    let new_sel = crate::vec_convert::to_curves(&mut sim, &mut scene, &mut map, &[id]);

    assert!(new_sel.contains(&id));
    assert_eq!(
        &before,
        scene.path(id).unwrap(),
        "sem forma viva nem efeitos, converter não pode mexer no caminho"
    );
}
