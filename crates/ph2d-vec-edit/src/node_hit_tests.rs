//! Gates da porta **`node_edit_hit_at`** — arquivo irmão de `node_hit.rs`.
//!
//! O oráculo é *"este press vai EDITAR geometria?"*, e a razão de a porta existir é que o retorno do
//! `on_press_node` **não** responde isso: ele devolve `Grabbed` tanto ao agarrar um vértice como ao
//! apenas selecionar a forma pelo preenchimento. Quem depende da diferença é a shell, que congela a
//! receita de uma forma VIVA antes do gesto — e congelar num clique que só seleciona expandiria a
//! forma sem o artista pedir.

use super::NODE_HIT_PX;
use crate::PenTool;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, VecScene, VecVertex};

/// Meio-lado do quadrado da fixture, em unidades de MUNDO.
///
/// ⚠️ **A escala é o fenômeno, não decoração.** Com `px_to_world = 1` o raio de captura é
/// `NODE_HIT_PX` unidades de mundo, então o centro de um quadrado só está *longe de tudo* se o
/// meio-lado passar disso — num quadrado de lado 2 a distância do centro à aresta é **1**, e o
/// controle abaixo media um press que de fato inseriria um vértice (foi como ele nasceu: vermelho,
/// sobre produto correto).
const HALF: f64 = 4.0 * NODE_HIT_PX;

/// Um quadrado preenchido centrado na origem, já SELECIONADO (é o estado em que as âncoras
/// aparecem, e o único em que o insert-no-segmento é oferecido).
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

/// **Uma ÂNCORA sob o cursor: o press edita.**
#[test]
fn a_press_on_an_anchor_reports_the_path_it_would_edit() {
    let (scene, pen, id) = selected_square();
    assert_eq!(
        pen.node_edit_hit_at(&scene, [HALF, HALF], 1.0),
        Some(id),
        "o press vai agarrar esta ancora e a porta disse que nao edita nada"
    );
}

/// **Perto de um SEGMENTO: o press edita** (insere um vértice e o agarra).
///
/// O MEIO da aresta de baixo: a `HALF` unidades da âncora mais próxima (bem fora do raio), então
/// quem responde só pode ser a busca do insert.
#[test]
fn a_press_near_a_segment_reports_the_path_it_would_edit() {
    let (scene, pen, id) = selected_square();
    assert_eq!(
        pen.node_edit_hit_at(&scene, [0.0, -HALF], 1.0),
        Some(id),
        "o press vai INSERIR um vertice neste segmento e a porta disse que nao edita nada"
    );
}

/// **CONTROLE: um clique no PREENCHIMENTO não edita nada** — e é este o caso que o retorno do
/// `on_press_node` confunde com o de cima (os dois devolvem `Grabbed`).
///
/// ⚠️ Mutação que tem de sangrar: a shell perguntar ao retorno do press em vez desta porta. Ela
/// congelaria a receita da forma viva num clique de SELEÇÃO — expandir sem ninguém pedir.
#[test]
fn a_press_on_the_fill_edits_nothing() {
    let (scene, pen, _) = selected_square();
    assert_eq!(
        pen.node_edit_hit_at(&scene, [0.0, 0.0], 1.0),
        None,
        "o centro do quadrado esta' longe de toda ancora e de todo segmento: este press apenas \
         SELECIONA, e dizer que ele edita faz a shell congelar a receita de uma forma viva"
    );
}

/// **A porta e o press concordam sobre o RAIO.**
///
/// Não é gosto: se a porta captura mais longe que o press, a shell congela a receita de uma forma
/// viva num clique que o press depois recusa (o artista perde a Live Shape e não ganha edição
/// nenhuma); se captura menos, o nó é editado e o `recook_into` o descarta em silêncio depois. É a
/// mesma lei que o `corner_hit_at` já carrega, no par vizinho.
#[test]
fn the_door_and_the_press_share_one_hit_radius() {
    let (mut scene, mut pen, id) = selected_square();
    // Uma distância que cabe DENTRO do raio (0,9 px de tela) e um `px_to_world` de 1: a porta e o
    // press têm de dar a MESMA resposta nos dois lados da fronteira.
    let inside = [HALF + 0.9 * NODE_HIT_PX, HALF];
    let outside = [HALF + 1.6 * NODE_HIT_PX, HALF];
    assert_eq!(pen.node_edit_hit_at(&scene, inside, 1.0), Some(id));
    assert_eq!(pen.node_edit_hit_at(&scene, outside, 1.0), None);
    // E o press de facto agarra no primeiro e não no segundo.
    assert_eq!(
        pen.on_press_node(&mut scene, inside, 1.0, false),
        crate::PenClick::Grabbed
    );
    pen.on_release();
    let mut fresh = PenTool::default();
    fresh.select(Some(id));
    // Fora do raio o press cai no `path_at` (que também acha o preenchimento) — o que ele NÃO faz é
    // armar um arrasto de vértice, e é isso que a porta prometia.
    fresh.on_press_node(&mut scene, outside, 1.0, false);
    assert!(
        !fresh.is_dragging(),
        "o press armou um arrasto fora do raio que a porta declarou"
    );
}
