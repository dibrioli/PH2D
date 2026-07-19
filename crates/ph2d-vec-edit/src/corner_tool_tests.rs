//! Gates das ferramentas de QUINA (Fillet / Chamfer) — o gesto de clicar-e-arrastar de
//! [`PenTool::on_press_corner`] + o arrasto de raio (`Part::Radius`): arredonda, chanfra, e
//! "transforma em quina PRIMEIRO" um ponto suave. Irmão de `corner_handle_tests` (a alça do
//! Node, que estas ferramentas consolidam).

use super::*;
use ph2d_vec_scene::{VecPath, VecPathId, VecVertex, VertexKind};

const PTW: f64 = 0.01; // world-units por pixel (câmera fictícia)

fn nosnap(p: [f64; 2]) -> [f64; 2] {
    p
}

/// Um quadrado fechado de 4 quinas RETAS (90°) em (0,0)-(4,0)-(4,4)-(0,4).
fn square(scene: &mut VecScene) -> VecPathId {
    scene.push_path(VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([4.0, 0.0]),
            VecVertex::corner([4.0, 4.0]),
            VecVertex::corner([0.0, 4.0]),
        ],
        closed: true,
        ..VecPath::default()
    })
}

/// O `corner_radius` do vértice `i` do path `id`.
fn radius(scene: &VecScene, id: VecPathId, i: usize) -> f64 {
    scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .vert(i)
        .unwrap()
        .corner_radius
}

/// **Fillet arredonda** — pressiona a quina (4,0) e arrasta para DENTRO (pela bissetriz): o
/// raio fica POSITIVO (arco). O dedo dita a MAGNITUDE.
#[test]
fn the_fillet_tool_rounds_a_corner() {
    let mut scene = VecScene::new();
    let id = square(&mut scene);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    assert!(pen.on_press_corner(&mut scene, [4.0, 0.0], PTW, false));
    assert!(pen.on_drag(&mut scene, [3.0, 1.0], &mut nosnap));
    pen.on_release();
    let r = radius(&scene, id, 1);
    assert!(r > 0.0, "Fillet arredonda: corner_radius > 0, veio {r}");
}

/// **Chamfer é o SINAL negativo, mesmo de uma quina AFIADA** — o gate do zero-com-sinal. A
/// quina nasce com magnitude 0 (`±0`), e `-0.0 < 0.0` é `false`; se o arrasto só chamasse
/// `set_corner_size` (preservando o sinal), o chanfro sumiria e pintaria ARREDONDADO. O
/// estilo é FORÇADO pelo grab (`chamfer: Some(true)`), então sai negativo.
#[test]
fn the_chamfer_tool_is_negative_even_from_a_sharp_corner() {
    let mut scene = VecScene::new();
    let id = square(&mut scene);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    assert!(pen.on_press_corner(&mut scene, [4.0, 0.0], PTW, true));
    assert!(pen.on_drag(&mut scene, [3.0, 1.0], &mut nosnap));
    pen.on_release();
    let r = radius(&scene, id, 1);
    assert!(r < 0.0, "Chamfer é o SINAL negativo, veio {r}");
}

/// **A ferramenta decide o ESTILO, não o preserva** — o Fillet sobre uma quina que já é
/// chanfro a torna arredondada (o oposto da alça do Node, que preservava o estilo).
#[test]
fn the_fillet_tool_overrides_an_existing_chamfer() {
    let mut scene = VecScene::new();
    let id = square(&mut scene);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    // Primeiro chanfra a quina 1.
    pen.on_press_corner(&mut scene, [4.0, 0.0], PTW, true);
    pen.on_drag(&mut scene, [3.0, 1.0], &mut nosnap);
    pen.on_release();
    assert!(radius(&scene, id, 1) < 0.0, "chanfrou primeiro");
    // Agora o Fillet na MESMA quina a arredonda.
    pen.on_press_corner(&mut scene, [4.0, 0.0], PTW, false);
    pen.on_drag(&mut scene, [3.0, 1.0], &mut nosnap);
    pen.on_release();
    assert!(
        radius(&scene, id, 1) > 0.0,
        "Fillet força arredondado sobre o chanfro"
    );
}

/// **Primeiro transforma em quina** — o algoritmo avançado que o Enio pediu. Um vértice
/// SUAVE (handles colineares) não tem ângulo, então `frame_at_flat` é `None` — não há o que
/// arredondar. O press o AFIA (`make_sharp_corner`: recolhe os handles, vira `Corner`) e só
/// então o arrasto o arredonda.
#[test]
fn a_smooth_vertex_is_turned_into_a_corner_first() {
    let mut scene = VecScene::new();
    let id = square(&mut scene);
    // Vira o vértice 1 num ponto SUAVE — deixa de ser quina (auto-smooth pelos vizinhos).
    {
        let path = scene.path_mut(id).unwrap();
        assert!(ph2d_vec_scene::retype_vertex(path, 1, VertexKind::Smooth));
    }
    // Pré-condição: agora NÃO há quina arredondável ali (tangentes colineares).
    {
        let path = scene.paths().iter().find(|p| p.id == id).unwrap();
        assert!(
            crate::corner_handle::frame_at_flat(path, 1).is_none(),
            "suave = sem quina a arredondar"
        );
    }
    let mut pen = PenTool::new();
    pen.select(Some(id));
    assert!(pen.on_press_corner(&mut scene, [4.0, 0.0], PTW, false));
    // A ferramenta o transformou em quina AFIADA (kind Corner, handles recolhidos na âncora).
    {
        let v = scene
            .paths()
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .vert(1)
            .unwrap();
        assert_eq!(v.kind, VertexKind::Corner, "transformou em quina");
        assert_eq!(v.in_handle, v.anchor, "handles recolhidos na âncora");
        assert_eq!(v.out_handle, v.anchor);
    }
    assert!(pen.on_drag(&mut scene, [3.0, 1.0], &mut nosnap));
    pen.on_release();
    assert!(radius(&scene, id, 1) > 0.0, "e o arrasto arredondou");
}

/// **Arrastar para FORA afia de volta** (raio → 0) — a mesma disciplina relativa da alça:
/// o recuo é a projeção na bissetriz, e uma projeção negativa clampa em zero.
#[test]
fn dragging_outward_keeps_the_corner_sharp() {
    let mut scene = VecScene::new();
    let id = square(&mut scene);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    assert!(pen.on_press_corner(&mut scene, [4.0, 0.0], PTW, false));
    // (5,-1) é para FORA da forma (projeção na bissetriz interna negativa).
    pen.on_drag(&mut scene, [5.0, -1.0], &mut nosnap);
    pen.on_release();
    assert_eq!(radius(&scene, id, 1), 0.0, "arrasto para fora = quina afiada");
}
