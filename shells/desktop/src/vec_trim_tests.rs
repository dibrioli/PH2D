//! Gates da COSTURA do Trim — o que a lei pura não alcança: o espaço em que se mede e o que
//! acontece à cena.

use super::*;
use ph2d_vec_scene::{Rgba8, StrokeSpec, VertexKind, Xform};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn reta(scene: &mut VecScene, a: [f64; 2], b: [f64; 2]) -> VecPathId {
    scene.push_path(VecPath {
        id: 0,
        verts: vec![v(a[0], a[1]), v(b[0], b[1])],
        closed: false,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(9, 9, 9, 255), 2.0)),
        subpaths: Vec::new(),
        fill_rule: ph2d_vec_scene::FillRule::NonZero,
        effects: Vec::new(),
    })
}

/// ⭐⭐ **O CASO DO PEDIDO — *"entre linhas sobrepostas"*.** Duas retas em cruz: aparar a ponta de
/// uma tira só o toco, do cruzamento até à ponta.
///
/// ⛔⛔ **E a fixtura é EXACTAMENTE a que mordia.** A travessia cai no 8.º de 16 pontos da
/// poligonal de detecção da vertical — em cima de uma amostra —, e o `seg_cross` recusava
/// travessias que não estivessem ESTRITAMENTE dentro da aresta. Medido: com a ponta deslocada
/// `0,1` o cruzamento aparecia (`0,400`), e exactamente sobre a amostra não (`1,0` = nenhum).
/// ⚠️ *É o caso mais comum que há* — um artista desenha em coordenadas redondas —, e uma fixtura
/// com números "quaisquer" teria aprovado o defeito. **Não mude os números desta.**
#[test]
fn trimming_a_stub_of_a_cross_removes_only_the_stub() {
    let mut scene = VecScene::new();
    let h = reta(&mut scene, [0.0, 0.0], [10.0, 0.0]);
    let _v = reta(&mut scene, [4.0, -5.0], [4.0, 5.0]);
    let xf = VecXforms::new();

    // O cursor no toco da esquerda (antes do cruzamento em x=4).
    let hit = hit(&scene, &xf, h, [1.0, 0.0], 1.0).expect("o cursor esta' sobre a reta");
    assert_eq!(hit.contour, 0);
    assert!(hit.de.abs() < 1e-6, "comeca na ponta: {}", hit.de);
    assert!(
        (hit.ate - 0.4).abs() < 1e-3,
        "tem de parar no CRUZAMENTO (x=4 de 10 -> 0,4), e parou em {}",
        hit.ate
    );

    assert!(apply(&mut scene, &hit), "a cena mudou");
    let sobrou = scene.path(h).expect("a reta sobrevive — so' o toco saiu");
    assert!(
        sobrou.verts[0].anchor[0] > 3.9,
        "o que sobrou tem de comecar no cruzamento: {:?}",
        sobrou.verts[0].anchor
    );
    assert_eq!(
        sobrou.stroke.as_ref().map(|s| s.width),
        Some(2.0),
        "o estilo viaja"
    );
}

/// ⚠️⚠️ **A MEDIÇÃO É NO LOCAL DO ALVO**, e uma pose não-uniforme separa as duas escolhas.
///
/// A vertical é posta em `x = 8` no MUNDO por um `Transform` que a desloca; se os cruzamentos
/// fossem medidos sem a levar ao local do alvo, ela cortaria em `x = 4` e o pedaço sairia com menos
/// de metade do comprimento certo. *Um gate sem pose nenhuma não distingue as duas leis.*
#[test]
fn the_crossing_is_measured_in_the_targets_own_space() {
    let mut scene = VecScene::new();
    let h = reta(&mut scene, [0.0, 0.0], [10.0, 0.0]);
    let vert = reta(&mut scene, [4.0, -5.0], [4.0, 5.0]);
    let mut xf = VecXforms::new();
    // A vertical anda +4 em x: no mundo ela cruza a horizontal em x = 8.
    xf.insert(vert, Xform([1.0, 0.0, 0.0, 1.0, 4.0, 0.0]));

    let hit = hit(&scene, &xf, h, [1.0, 0.0], 1.0).expect("sobre a reta");
    assert!(
        (hit.ate - 0.8).abs() < 1e-3,
        "a travessia esta' em x=8 (0,8 do comprimento) e o corte parou em {}",
        hit.ate
    );
}

/// **Uma reta que não cruza nada é a peça toda ⇒ o caminho SOME da cena.**
#[test]
fn a_lone_line_is_removed_whole() {
    let mut scene = VecScene::new();
    let h = reta(&mut scene, [0.0, 0.0], [10.0, 0.0]);
    let xf = VecXforms::new();
    let hit = hit(&scene, &xf, h, [5.0, 0.0], 1.0).expect("sobre a reta");
    assert!(apply(&mut scene, &hit));
    assert!(
        scene.path(h).is_none(),
        "sem fronteira no meio, o pedaco e' a peca toda"
    );
}

/// **Fora do alcance não há pedaço** — e sem isto o clique no vazio apagaria a última coisa que o
/// cursor tocou.
#[test]
fn a_cursor_far_from_the_path_hits_nothing() {
    let mut scene = VecScene::new();
    let h = reta(&mut scene, [0.0, 0.0], [10.0, 0.0]);
    let xf = VecXforms::new();
    assert!(hit(&scene, &xf, h, [5.0, 50.0], 1.0).is_none());
}
