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
        ..VecPath::default()
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

/// ⭐⭐⭐ **O REPORT (Enio, 2026-08-31, com foto):** *"os pontos do círculo cortado estão sobre o
/// outro círculo, mas este outro não reconhece os pontos … e desse modo não me permite deletar o
/// segmento entre os pontos."*
///
/// ⚠️⚠️ **Uma ponta que TERMINA sobre a outra curva não é um CRUZAMENTO.** Um cruzamento é a
/// travessia de duas cordas que continuam para os dois lados; um arco aparado **acaba** ali.
/// Medido: depois de aparar um dos círculos, o outro ficava com **UMA** fronteira onde tinha duas,
/// e a ponta órfã estava a `0,0323` da poligonal dele (a flecha de amostragem dela é `0,12`).
///
/// A fixtura tem as DUAS metades: o controle (antes do corte há duas travessias) e a cura.
#[test]
fn an_endpoint_that_lands_on_a_curve_becomes_a_boundary_of_it() {
    let mut scene = VecScene::new();
    let a = scene.push_path(ph2d_vec_scene::ellipse([0.0, 0.0], 100.0, 100.0));
    let b = scene.push_path(ph2d_vec_scene::ellipse([120.0, 0.0], 100.0, 100.0));
    let xf = VecXforms::new();

    let fronteiras_de = |scene: &VecScene, alvo, outro| {
        let p = scene.path(alvo).expect("o alvo");
        let o = scene.path(outro).expect("o outro");
        ph2d_vec_scene::trim_tool::crossings_against(
            &p.verts,
            p.closed,
            &[(o.verts.clone(), o.closed)],
            400.0,
        )
    };

    // CONTROLE: dois círculos que se sobrepõem cruzam-se DUAS vezes.
    assert_eq!(
        fronteiras_de(&scene, b, a).len(),
        2,
        "a fixtura tem de comecar com duas travessias, senao ela nao contem o fenomeno"
    );

    // O artista apara o pedaço de A que está dentro de B.
    let h = hit(&scene, &xf, a, [95.0, 30.0], 8.0).expect("sobre A");
    assert!(apply(&mut scene, &h));
    let arco = scene.path(a).expect("A sobrevive");
    assert!(!arco.closed, "o circulo aparado ficou ABERTO");

    // A CURA: B continua a ver DUAS fronteiras — a travessia que sobrou e o TOQUE da ponta.
    let depois = fronteiras_de(&scene, b, a);
    assert_eq!(
        depois.len(),
        2,
        "B perdeu a fronteira onde a ponta de A pousa: {depois:?}"
    );

    // …e ela está no sítio certo: sobre a ponta de A.
    let ponta = arco.verts[0].anchor;
    let pb = scene.path(b).expect("B");
    let (frac_da_ponta, dist) =
        ph2d_vec_scene::trim_tool::nearest_fraction(&pb.verts, pb.closed, ponta)
            .expect("a ponta tem projeccao em B");
    assert!(dist < 0.2, "a ponta esta' sobre B (dist = {dist})");
    assert!(
        depois.iter().any(|f| (f - frac_da_ponta).abs() < 1e-6),
        "a fronteira nova nao caiu sobre a ponta: {depois:?} contra {frac_da_ponta}"
    );
}

/// ⚠️ **O cursor EXACTAMENTE sobre uma fronteira ainda escolhe um pedaço.** Com uma folga simétrica
/// os dois lados achavam a MESMA fronteira e o pedaço nascia de largura ZERO — o realce não
/// aparecia e o clique não fazia nada, que lê como *"a ferramenta não pega aqui"*.
///
/// Achado pela sonda do report acima: o primeiro ponto que se aponta num círculo é um dos quatro
/// nós dele.
#[test]
fn a_cursor_exactly_on_a_node_still_gets_a_piece() {
    let mut scene = VecScene::new();
    let c = scene.push_path(ph2d_vec_scene::ellipse([0.0, 0.0], 100.0, 100.0));
    let xf = VecXforms::new();
    // `[-100, 0]` é um dos quatro nós do círculo (fracção `0,5`).
    let h = hit(&scene, &xf, c, [-100.0, 0.0], 5.0).expect("sobre o circulo");
    assert!(
        (h.ate - h.de).abs() > 1e-6,
        "o pedaco nasceu de largura ZERO: ({}, {})",
        h.de,
        h.ate
    );
    assert!(apply(&mut scene, &h), "e o corte tem de acontecer");
    assert!(!scene.path(c).expect("sobrevive").closed);
}
