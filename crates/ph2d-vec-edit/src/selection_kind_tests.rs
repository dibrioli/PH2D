//! Gates do **que a seleção de vértices tem em comum** — arquivo irmão de `selection.rs`.
//!
//! O defeito (auditoria do plano 25, item 5): a leitura devolvia o tipo do vértice **PRIMÁRIO**, e o
//! painel acendia o chip correspondente. Com dois nós de tipos diferentes selecionados, um dos três
//! chips afirmava descrever a seleção INTEIRA — e o botão ao lado dele já agia sobre TODOS
//! (`set_selected_vertex_kind` sempre retipou o conjunto). Era só a LEITURA que mentia.

use crate::{PenTool, SelectedKind};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, VertexKind};

/// Um triângulo de quinas, selecionado, com `verts` escolhidos pelo teste.
fn scene_with(kinds: [VertexKind; 3], selected: &[usize]) -> (VecScene, PenTool, VecPathId) {
    let mut scene = VecScene::new();
    let mut verts: Vec<VecVertex> = [[0.0, 0.0], [4.0, 0.0], [2.0, 3.0]]
        .map(VecVertex::corner)
        .to_vec();
    for (v, k) in verts.iter_mut().zip(kinds) {
        v.kind = k;
    }
    let id = scene.push_path(VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    });
    let mut pen = PenTool::default();
    pen.select(Some(id));
    // ⚠️ A seleção é montada pelo GESTO real (Shift+clique na âncora, `toggle_vert_at`) e não por um
    // setter de teste: um setter seria uma segunda porta para o `selected_verts`, e o que se quer
    // provar é o que o artista de fato consegue selecionar.
    for &i in selected {
        let anchor = scene
            .path(id)
            .and_then(|p| p.vert(i).map(|v| v.anchor))
            .expect("a fixture seleciona vertices que existem");
        pen.toggle_vert_at(&scene, anchor, 0.1);
    }
    (scene, pen, id)
}

/// **Um vértice só: o tipo dele.** O caso comum, e o que o painel sempre mostrou certo.
#[test]
fn one_selected_vertex_reports_its_own_kind() {
    let (scene, pen, _) = scene_with(
        [VertexKind::Corner, VertexKind::Smooth, VertexKind::Corner],
        &[1],
    );
    assert_eq!(
        pen.selected_vertex_kind(&scene),
        Some(SelectedKind::Uniform(VertexKind::Smooth))
    );
}

/// **Vários do MESMO tipo: uniforme.** É o que faz o chip continuar a acender numa multi-seleção
/// legítima — a cura não pode custar isto.
#[test]
fn many_vertices_of_the_same_kind_are_uniform() {
    let (scene, pen, _) = scene_with(
        [VertexKind::Smooth, VertexKind::Smooth, VertexKind::Smooth],
        &[0, 1, 2],
    );
    assert_eq!(
        pen.selected_vertex_kind(&scene),
        Some(SelectedKind::Uniform(VertexKind::Smooth))
    );
}

/// **Tipos DIFERENTES: misto** — e nenhum chip descreve o todo.
///
/// ⚠️ Mutação que tem de sangrar: devolver o tipo do primário (**o código que shipava**). Aqui ela
/// devolveria `Uniform(Corner)` — o painel afirmando "Corner" sobre uma seleção que tem Smooth
/// dentro.
#[test]
fn a_mixed_selection_is_mixed() {
    let (scene, pen, _) = scene_with(
        [VertexKind::Smooth, VertexKind::Corner, VertexKind::Corner],
        &[0, 1],
    );
    assert_eq!(pen.selected_vertex_kind(&scene), Some(SelectedKind::Mixed));
}

/// **A ORDEM não muda a resposta.** O primário é o ÚLTIMO da lista, então uma leitura que ainda o
/// privilegiasse daria respostas diferentes para as duas ordens da MESMA seleção.
#[test]
fn the_answer_does_not_depend_on_which_vertex_was_touched_last() {
    let a = scene_with(
        [VertexKind::Smooth, VertexKind::Corner, VertexKind::Corner],
        &[0, 1],
    );
    let b = scene_with(
        [VertexKind::Smooth, VertexKind::Corner, VertexKind::Corner],
        &[1, 0],
    );
    assert_eq!(
        a.1.selected_vertex_kind(&a.0),
        b.1.selected_vertex_kind(&b.0),
        "a mesma selecao, em outra ordem, deu outra resposta — alguem voltou a ler o primario"
    );
}

/// **Sem vértice selecionado: `None`** (o painel ESCONDE a seção). É o estado de um caminho inteiro
/// selecionado por uma booleana.
#[test]
fn no_selected_vertex_is_none() {
    let (scene, pen, _) = scene_with([VertexKind::Corner; 3], &[]);
    assert_eq!(pen.selected_vertex_kind(&scene), None);
}

/// **Um índice que não existe mais é IGNORADO, não conta como misto.**
///
/// A seleção sobrevive a um delete de vértice (o índice fica pendurado), e um índice morto não
/// descreve vértice nenhum — tratá-lo como um tipo a mais apagaria o chip de uma seleção que é
/// uniforme de verdade.
#[test]
fn a_stale_index_is_ignored_not_mixed() {
    // Seleciona os três e ENCURTA a forma por baixo: os índices 1 e 2 ficam pendurados, que é o
    // estado real depois de um delete de vértice.
    let (mut scene, pen, id) = scene_with([VertexKind::Smooth; 3], &[0, 1, 2]);
    scene
        .path_mut(id)
        .expect("a forma existe")
        .verts
        .truncate(1);
    assert_eq!(
        pen.selected_vertex_kind(&scene),
        Some(SelectedKind::Uniform(VertexKind::Smooth)),
        "um indice que nao existe mais nao descreve vertice nenhum — conta'-lo como um tipo a mais \
         apagaria o chip de uma selecao uniforme de verdade"
    );
}
