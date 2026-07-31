//! Gates da **ESCALA DA SELEÇÃO** de nós (plano 25 §6, W3b) — filho de `selection.rs`.
//!
//! A queixa que a wave fecha é operacional: *"trabalhar uma forma de 40 nós é clique-a-clique"*.
//! Os oráculos são de ALCANCE (quantos nós um gesto apanha, e quais), nunca da fórmula.

use crate::PenTool;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, VecScene, VecVertex, VertexKind};

/// Um quadrado de lado `2 s` centrado em `c`.
fn square(c: [f64; 2], s: f64) -> VecPath {
    VecPath {
        verts: [
            [c[0] - s, c[1] - s],
            [c[0] + s, c[1] - s],
            [c[0] + s, c[1] + s],
            [c[0] - s, c[1] + s],
        ]
        .map(VecVertex::corner)
        .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(200, 120, 40, 255))),
        ..VecPath::default()
    }
}

/// DUAS formas bem separadas, nenhuma selecionada.
fn two_squares() -> (VecScene, PenTool, VecPathId, VecPathId) {
    let mut scene = VecScene::new();
    let a = scene.push_path(square([0.0, 0.0], 1.0));
    let b = scene.push_path(square([10.0, 0.0], 1.0));
    (scene, PenTool::default(), a, b)
}

/// Caixa que envolve o quadrado centrado em `c`.
fn around(c: [f64; 2]) -> ([f64; 2], [f64; 2]) {
    ([c[0] - 2.0, c[1] - 2.0], [c[0] + 2.0, c[1] + 2.0])
}

/// **O retângulo apanha a forma que ele de facto cobre** — não a que estava selecionada.
///
/// ⚠️ A preferência pelo caminho selecionado era **incondicional**: com a forma A selecionada,
/// arrastar o retângulo sobre B mirava A, apanhava zero nós e devolvia seleção VAZIA. O artista
/// via a caixa passar por cima dos nós e nada acender. É metade do *"o marquee vê um path só"*
/// (a outra — nós de VÁRIAS formas ao mesmo tempo — é ausência por construção e fica nomeada).
#[test]
fn the_marquee_catches_the_shape_it_covers_not_the_selected_one() {
    let (scene, mut pen, a, b) = two_squares();
    pen.select(Some(a));
    let (min, max) = around([10.0, 0.0]);
    pen.box_select(&scene, min, max);
    assert_eq!(pen.selected(), Some(b), "o retangulo nao mudou de forma");
    assert_eq!(
        pen.selected_verts().len(),
        4,
        "o retangulo sobre a outra forma apanhou {} nos",
        pen.selected_verts().len()
    );
}

/// **Shift SOMA; sem Shift SUBSTITUI.** Sem a soma não há como construir uma seleção de nós em
/// duas passadas, que é o gesto normal numa forma grande.
#[test]
fn shift_marquee_adds_and_a_plain_one_replaces() {
    let mut scene = VecScene::new();
    // Um caminho com dois GRUPOS de nós afastados: o retângulo apanha um grupo de cada vez.
    let id = scene.push_path(VecPath {
        verts: [
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [10.0, 0.0],
            [11.0, 0.0],
        ]
        .map(VecVertex::corner)
        .to_vec(),
        closed: false,
        ..VecPath::default()
    });
    let mut pen = PenTool::default();
    pen.select(Some(id));
    pen.box_select_with(&scene, [-1.0, -1.0], [2.0, 2.0], false);
    assert_eq!(pen.selected_verts().len(), 4, "o 1o retangulo falhou");

    pen.box_select_with(&scene, [9.0, -1.0], [12.0, 1.0], true);
    assert_eq!(
        pen.selected_verts().len(),
        6,
        "o Shift+retangulo nao SOMOU: {:?}",
        pen.selected_verts()
    );

    pen.box_select_with(&scene, [9.0, -1.0], [12.0, 1.0], false);
    assert_eq!(
        pen.selected_verts().len(),
        2,
        "o retangulo sem Shift nao SUBSTITUIU: {:?}",
        pen.selected_verts()
    );
}

/// **`Ctrl+A` apanha todos os nós** do caminho selecionado. `false` sem caminho — dizer que sim
/// faria o shell tratar como gesto o que não mudou nada.
#[test]
fn select_all_takes_every_node_of_the_selected_path() {
    let (scene, mut pen, a, _) = two_squares();
    assert!(
        !pen.select_all_verts(&scene),
        "sem selecao nao ha o que apanhar"
    );
    pen.select(Some(a));
    assert!(pen.select_all_verts(&scene));
    assert_eq!(pen.selected_verts().len(), 4);
}

/// **`Tab` percorre, e dá a volta.** Sem seleção começa no primeiro; `Shift+Tab` anda para trás e
/// dá a volta pelo fim — é o que faz o gesto ter porta de entrada nos dois sentidos.
#[test]
fn tab_walks_the_nodes_and_wraps() {
    let (scene, mut pen, a, _) = two_squares();
    pen.select(Some(a));
    for want in [0, 1, 2, 3, 0] {
        assert!(pen.step_vert_selection(&scene, true));
        assert_eq!(pen.selected_vert(), Some(want), "o Tab saltou");
    }
    // E para trás a partir do 0 dá a volta para o último.
    assert!(pen.step_vert_selection(&scene, false));
    assert_eq!(pen.selected_vert(), Some(3), "o Shift+Tab nao deu a volta");
    // ⚠️ O Tab SUBSTITUI: percorrer é olhar um de cada vez, e somar ao andar tornaria a tecla um
    // "select all" lento.
    assert_eq!(pen.selected_verts().len(), 1);
}

/// **Select Subpath apanha o CONTORNO inteiro que a seleção toca** — e só ele. Num compound é o
/// que separa *este buraco* de *a forma inteira*, distinção que o `Ctrl+A` não faz.
#[test]
fn select_subpath_takes_the_touched_contour_and_only_it() {
    let mut scene = VecScene::new();
    let mut p = square([0.0, 0.0], 4.0);
    // O furo: um 2º contorno dentro do 1º.
    p.subpaths.push(ph2d_vec_scene::Contour::new_closed(
        [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
    ));
    let id = scene.push_path(p);
    let mut pen = PenTool::default();
    pen.select(Some(id));
    // Um nó do FURO (índices planos 4..8).
    pen.box_select(&scene, [0.5, 0.5], [1.5, 1.5]);
    assert_eq!(
        pen.selected_verts(),
        [6],
        "a fixture nao pegou um no' do furo"
    );
    assert!(pen.select_subpath_verts(&scene));
    assert_eq!(
        pen.selected_verts().len(),
        4,
        "o Select Subpath apanhou {} nos -- devia apanhar so' os 4 do furo",
        pen.selected_verts().len()
    );
    assert!(
        pen.selected_verts().iter().all(|&i| i >= 4),
        "apanhou nos do contorno de FORA: {:?}",
        pen.selected_verts()
    );
}

/// **Select Same apanha todos os nós do TIPO do primário.** É o gesto que transforma *"afiar as
/// quinas desta estrela"* de N cliques em dois.
#[test]
fn select_same_takes_every_node_of_the_primarys_kind() {
    let mut scene = VecScene::new();
    let mut p = square([0.0, 0.0], 1.0);
    // Dois de cada tipo — sem a MISTURA a fixture não contém o fenômeno: com todos iguais,
    // "apanhou os do tipo" e "apanhou todos" dão a mesma resposta.
    p.verts[0].kind = VertexKind::Smooth;
    p.verts[2].kind = VertexKind::Smooth;
    let id = scene.push_path(p);
    let mut pen = PenTool::default();
    pen.select(Some(id));
    pen.box_select(&scene, [-1.5, -1.5], [-0.5, -0.5]); // só o vértice 0 (Smooth)
    assert_eq!(
        pen.selected_verts(),
        [0],
        "a fixture nao pegou um no' Smooth"
    );
    assert!(pen.select_verts_of_same_kind(&scene));
    assert_eq!(
        pen.selected_verts(),
        [0, 2],
        "o Select Same nao apanhou os dois Smooth (e so' eles)"
    );
}
