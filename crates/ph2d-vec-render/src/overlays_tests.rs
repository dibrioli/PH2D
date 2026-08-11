//! **O que o overlay ACENDE** — irmão de `overlays.rs`, o sujeito.
//!
//! O passe desenha três aparências de âncora: **ciano e maior** (nó escolhido), **laranja** (forma
//! na seleção de objeto) e cinza (o resto). A pergunta que este arquivo pina é a primeira, e ela
//! tinha o defeito escrito no próprio comentário do produto:
//!
//! ```text
//! // A vertex in the multi-selection (selected path only) is drawn bigger
//! let picked = is_sel && selected_verts.contains(&i);
//! ```
//!
//! Um índice sem dono só podia falar da forma PRIMÁRIA, então os nós escolhidos das outras eram
//! desenhados como não-escolhidos: o artista via metade da sua seleção apagar-se ao tocar a segunda
//! forma, com o motor a mover as duas corretamente. Com o dono no par (`(VecPathId, usize)`) a
//! pergunta é feita direta.

use super::*;
use ph2d_vec_scene::{VecScene, VecViewState, VecXforms, rectangle};

/// Roda o passe e devolve quantas âncoras saíram ACESAS.
fn picked_drawn(
    scene: &VecScene,
    selected: Option<VecPathId>,
    selected_paths: &[VecPathId],
    selected_verts: &[(VecPathId, usize)],
) -> usize {
    PICKED_DRAWN.with(|c| c.set(0));
    let mut target = VectorScene::default();
    draw_overlays(
        scene,
        &VecViewState::default(),
        selected,
        selected_paths,
        selected_verts,
        &VecXforms::new(),
        Affine::IDENTITY,
        &mut target,
    );
    PICKED_DRAWN.with(|c| c.get())
}

/// **Um nó escolhido de uma forma NÃO-PRIMÁRIA acende.**
///
/// ⚠️ O CONTROLE é a metade que torna o gate honesto: o mesmo passe com o mesmo par de formas,
/// mudando só de QUEM é o nó escolhido. Sem ele, um overlay que acendesse tudo passaria.
#[test]
fn the_overlay_lights_a_picked_node_of_a_non_primary_shape() {
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));

    // CONTROLE: um nó da forma primária — o caso que sempre funcionou.
    assert_eq!(
        picked_drawn(&scene, Some(a), &[a, b], &[(a, 0)]),
        1,
        "o no' da forma primaria sempre acendeu"
    );
    // E o caso que NÃO funcionava: o primário é A, e o nó escolhido é de B.
    assert_eq!(
        picked_drawn(&scene, Some(a), &[a, b], &[(b, 0)]),
        1,
        "o no' escolhido de uma forma nao-primaria ficou APAGADO"
    );
    // Os dois juntos — a seleção que atravessa formas, que é a wave inteira.
    assert_eq!(
        picked_drawn(&scene, Some(a), &[a, b], &[(a, 0), (a, 2), (b, 1)]),
        3,
        "a selecao inteira tem de acender, nao a metade do primario"
    );
}

/// **E acende o nó CERTO, não *um* nó.** O par `(forma, índice)` tem de casar nos DOIS campos: um
/// overlay que comparasse só o índice acenderia o nó 0 de **todas** as formas.
#[test]
fn the_overlay_matches_the_owner_not_just_the_index() {
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
    assert_eq!(
        picked_drawn(&scene, Some(a), &[a, b], &[(a, 0)]),
        1,
        "comparar so' o indice acenderia tambem o no' 0 de B"
    );
}
