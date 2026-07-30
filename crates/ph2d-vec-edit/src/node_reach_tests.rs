//! Gates do **ALCANCE DO NÓ no GESTO** (plano 25 §6) — filho de `node_hit.rs`.
//!
//! Os motores estão gateados na `ph2d-vec-scene` (`tests/node_reach.rs`, com os números medidos);
//! aqui o que se prova é a **costura**: que press faz o quê, e que o Delete do editor passa pela
//! porta que preserva a forma. As duas metades falham independentemente — um motor certo atrás de
//! um press que ainda insere é a feature inteira ausente, com a suíte do motor verde.

use crate::{PenClick, PenTool};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, VecScene, VecVertex, VertexKind};

use super::NODE_HIT_PX;

/// Meio-lado do quadrado, em MUNDO — a mesma régua do irmão `node_hit_tests`: o centro só está
/// *longe de tudo* se o meio-lado passar do raio de captura.
const HALF: f64 = 4.0 * NODE_HIT_PX;

/// Um quadrado preenchido, já SELECIONADO (o estado em que as âncoras aparecem).
fn selected_square() -> (VecScene, PenTool, VecPathId) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[-HALF, -HALF], [HALF, -HALF], [HALF, HALF], [-HALF, HALF]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(200, 120, 40, 255))),
        ..VecPath::default()
    });
    let mut pen = PenTool::default();
    pen.select(Some(id));
    (scene, pen, id)
}

/// O meio da aresta de baixo — sobre a CURVA, longe de qualquer âncora.
fn mid_bottom_edge() -> [f64; 2] {
    [0.0, -HALF]
}

/// **Pressionar sobre a curva REFORMA o segmento — não insere um nó.**
///
/// ⚠️ Era a ausência do plano: *"não se pode reformar uma curva sem alterar a topologia dela"*.
/// O press inseria um vértice e agarrava o vértice novo, então mudar a forma de um trecho custava
/// sempre um nó a mais — e o artista que só queria a curva ficava com a topologia mexida.
#[test]
fn pressing_on_the_curve_reshapes_the_segment_instead_of_inserting() {
    let (mut scene, mut pen, id) = selected_square();
    let n0 = scene.path(id).expect("existe").verts.len();
    let click = pen.on_press_node(&mut scene, mid_bottom_edge(), 1.0, false);
    assert_eq!(
        click,
        PenClick::Grabbed,
        "o press sobre a curva devia AGARRAR o segmento"
    );
    assert_ne!(
        click,
        PenClick::Inserted,
        "o press sobre a curva ainda INSERE um no'"
    );
    assert_eq!(
        scene.path(id).expect("existe").verts.len(),
        n0,
        "a contagem de vertices mudou num gesto que nao pode mexer na topologia"
    );
}

/// **E o arrasto move a curva, deixando as âncoras onde estão.** É a outra metade: agarrar o
/// segmento e não o mover seria um gesto que não faz nada.
#[test]
fn dragging_the_grabbed_segment_bends_the_curve_and_leaves_the_anchors() {
    let (mut scene, mut pen, id) = selected_square();
    let at = mid_bottom_edge();
    let anchors: Vec<[f64; 2]> = scene
        .path(id)
        .expect("existe")
        .verts
        .iter()
        .map(|v| v.anchor)
        .collect();
    assert_eq!(
        pen.on_press_node(&mut scene, at, 1.0, false),
        PenClick::Grabbed
    );

    let to = [at[0], at[1] - HALF * 0.5];
    assert!(
        pen.on_drag(&mut scene, to, &mut |p| p),
        "o arrasto nao foi consumido"
    );

    let path = scene.path(id).expect("existe");
    assert_eq!(
        path.verts.len(),
        anchors.len(),
        "nasceu ou morreu um vertice"
    );
    for (v, a) in path.verts.iter().zip(&anchors) {
        assert!(
            (v.anchor[0] - a[0]).abs() < 1e-9 && (v.anchor[1] - a[1]).abs() < 1e-9,
            "uma ancora andou no arrasto de SEGMENTO: {a:?} -> {:?}",
            v.anchor
        );
    }
    // A curva de facto se moveu para onde o dedo foi.
    let now = ph2d_vec_scene::point_on_segment(path, 0, 0.5).expect("ponto");
    assert!(
        (now[0] - to[0]).hypot(now[1] - to[1]) < 1e-6,
        "o ponto agarrado nao seguiu o dedo: {now:?} contra {to:?}"
    );
}

/// **A CANETA continua a inserir** — a inserção não se perdeu, mudou de ferramenta. É a divisão do
/// Illustrator: a seta branca reforma, a Pen acrescenta âncora. Sem este gate, mover o gesto do
/// Node teria removido a capacidade em silêncio.
#[test]
fn the_pen_still_inserts_on_a_segment() {
    let (mut scene, mut pen, id) = selected_square();
    let n0 = scene.path(id).expect("existe").verts.len();
    let click = pen.on_press(&mut scene, mid_bottom_edge(), 1.0, false, &mut |p| p);
    assert_eq!(
        click,
        PenClick::Inserted,
        "a Caneta deixou de inserir na curva"
    );
    assert_eq!(
        scene.path(id).expect("existe").verts.len(),
        n0 + 1,
        "a Caneta nao acrescentou o vertice"
    );
}

/// **O Delete do editor passa pela porta que PRESERVA a forma.**
///
/// O gate do motor vive na `ph2d-vec-scene`; este prova a costura, e ela falha sozinha: um
/// `verts.remove` de volta aqui deixaria os quatro gates do motor verdes e a feature ausente.
/// O oráculo é o HANDLE do vizinho — um remove cru não o toca, e é isso que faz a curva morrer
/// com o ponto.
#[test]
fn deleting_a_selected_node_refits_its_neighbours() {
    let mut scene = VecScene::new();
    // ⚠️ **A ESCALA é o fenômeno**, a mesma cicatriz que o irmão `node_hit_tests` prega: com
    // `px_to_world = 1` o raio de captura é `NODE_HIT_PX` unidades de MUNDO, e um arco de 2
    // unidades cabe inteiro dentro dele — o press no meio agarrava um handle do vizinho, e o gate
    // nasceu vermelho sobre produto correto. Ampliado, só a âncora do meio está sob o cursor.
    const S: f64 = 10.0 * NODE_HIT_PX;
    let smooth = |a: [f64; 2], i: [f64; 2], o: [f64; 2]| VecVertex {
        anchor: [a[0] * S, a[1] * S],
        in_handle: [i[0] * S, i[1] * S],
        out_handle: [o[0] * S, o[1] * S],
        kind: VertexKind::Smooth,
        corner_radius: 0.0,
    };
    let id = scene.push_path(VecPath {
        verts: vec![
            smooth([0.0, 0.0], [-0.55, -0.55], [0.55, 0.55]),
            smooth([1.0, 1.0], [0.6, 1.0], [1.4, 1.0]),
            smooth([2.0, 0.0], [1.45, 0.55], [2.55, -0.55]),
        ],
        closed: false,
        ..VecPath::default()
    });
    let before_out = scene.path(id).expect("existe").verts[0].out_handle;
    let mut pen = PenTool::default();
    pen.select(Some(id));
    // O nó é selecionado pelo GESTO real (clicar a âncora), não por um setter — é o caminho que
    // o artista percorre, e é ele que põe o índice na lista que o Delete consome.
    assert_eq!(
        pen.on_press_node(&mut scene, [S, S], 1.0, false),
        PenClick::Grabbed,
        "o press na ancora do meio nao a agarrou"
    );
    pen.on_release();
    assert_eq!(
        pen.selected_verts(),
        [1],
        "o clique nao selecionou o no' do meio"
    );
    assert!(pen.delete_selected_vertex(&mut scene));

    let path = scene.path(id).expect("existe");
    assert_eq!(path.verts.len(), 2, "o no' selecionado nao saiu");
    let after_out = path.verts[0].out_handle;
    assert!(
        (after_out[0] - before_out[0]).hypot(after_out[1] - before_out[1]) > 1e-6,
        "o handle do vizinho nao foi re-ajustado ({before_out:?}) -- o Delete voltou a ser um \
         `remove` cru e a curva morre com o ponto"
    );
}
