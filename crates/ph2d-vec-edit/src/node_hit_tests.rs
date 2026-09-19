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

/// ⭐⭐⭐ **A CANETA REPORTA ONDE INSERIU** — `(caminho, segmento, t)`, uma vez só.
///
/// ⛔⛔ **Ele nasceu de uma MUTAÇÃO SOBREVIVENTE** (2026-09-19): pôr `ultima_insercao = None` no sítio
/// da escrita deixava `10` testes da shell verdes. O gate de costura de lá lê o TEXTO do despacho —
/// ele afirma que a shell *drena*, nunca que a caneta *grava*. *As duas pontas de um fio precisam
/// cada uma do seu gate.*
///
/// ⚠️ **As três metades são três defeitos diferentes:** não reportar nada (o ponto evapora-se numa
/// forma presa); reportar o `t` errado (o ponto nasce noutro sítio da mesma curva, e o gesto
/// desobedece ao dedo); e reportar duas vezes (dois pontos por um clique).
#[test]
fn a_caneta_reporta_onde_inseriu_uma_vez_so() {
    let (mut scene, mut pen, id) = selected_square();
    assert_eq!(
        pen.take_insercao(),
        None,
        "a caneta reporta uma insercao que nao aconteceu"
    );

    // ⛔⛔ **FORA DO MEIO, de propósito — e foi uma MUTAÇÃO que o exigiu.** A 1.ª redacção carregava
    // no MEIO da aresta, onde o `t` verdadeiro **é** `0,5`: cravar `0,5` no sítio da escrita passava
    // este gate. *Um corpus no ponto neutro de um valor não testa esse valor* — a mesma família que
    // este repo já pagou noutras linhas. Aqui o dedo fica a `30 %` da aresta de baixo.
    const T_DO_DEDO: f64 = 0.3;
    let x = -HALF + T_DO_DEDO * 2.0 * HALF;
    assert_eq!(
        pen.on_press(&mut scene, [x, -HALF], 1.0, false, &mut |p| p),
        crate::PenClick::Inserted,
        "a fixtura deixou de inserir: o resto deste gate passa a ser vacuo"
    );

    let (pid, _seg, t) = pen
        .take_insercao()
        .expect("a caneta inseriu e nao reportou onde: numa forma PRESA o ponto evapora-se");
    assert_eq!(pid, id, "reportou a insercao no caminho ERRADO");
    // ⛔ O `t` é do DEDO: o cursor está no meio da aresta, logo tem de sair perto de `0,5`. Um `t`
    // reconstruído do outro lado (com `0,5` fixo, por exemplo) passaria aqui — e é por isso que a
    // outra metade deste gate é ele ser **lido** e não inventado; ver `take_insercao`.
    // ⛔⛔⛔ **O `t` É O PARÂMETRO DA CURVA, e NÃO a fracção ao longo da corda.** A 1.ª redacção
    // esperava `0,3` (a fracção onde o dedo estava) e leu **`0,375`**: numa quina os dois pontos de
    // controlo interiores colapsam nas âncoras, logo a cúbica é `P0,P0,P1,P1` e a posição avança com
    // `3t² − 2t³` — o *smoothstep*. Resolvendo `3t² − 2t³ = 0,3` dá exactamente `0,375`. *A régua
    // estava errada e o código certo*, e é este o `t` que o `split_segment` precisa.
    //
    // ⭐ ⇒ a régua passa a medir o **PRODUTO**: onde o ponto NASCEU. É a pergunta do artista (*«ele
    // aparece onde eu carreguei?»*), ela não depende da parameterização, e mata na mesma o `t`
    // cravado — com o dedo fora do meio, um `0,5` põe o ponto noutro sítio da aresta.
    assert!(
        (t - 0.5).abs() > 0.05,
        "o `t` reportado foi {t}: o dedo esta' fora do meio, entao um `0,5` aqui e' um valor \
         CRAVADO, e o gate abaixo deixa de discriminar"
    );
    let nascido = scene
        .path(id)
        .expect("o caminho")
        .verts_all()
        .map(|v| v.anchor)
        .min_by(|a, b| {
            (a[0] - x)
                .hypot(a[1] + HALF)
                .total_cmp(&(b[0] - x).hypot(b[1] + HALF))
        })
        .expect("ha' vertices");
    // ⚠️ **A barra é MEDIDA e a fracção que sobra não é desta wave:** o ponto nasce a `1,3125` de um
    // segmento de `2·HALF`, que é **`1,6 %`** dele. O resíduo é a granularidade da busca do ponto
    // mais próximo da caneta (`insert_hit`), **pré-existente** — ela devolve o parâmetro de uma
    // amostragem, não o mínimo exacto. *Nomeado, e deixado onde está: curá-lo é outra wave, e a
    // pergunta desta é se o `t` do dedo VIAJA.*
    let erro = (nascido[0] - x).hypot(nascido[1] + HALF);
    assert!(
        erro < 2.0 * HALF * 0.03,
        "o ponto nasceu a {erro} do dedo (em {nascido:?}, o dedo em [{x}, {}]) — um `t` inventado \
         faz o ponto nascer NOUTRO sitio da mesma curva, e o gesto desobedece ao dedo",
        -HALF
    );
    assert_eq!(
        pen.take_insercao(),
        None,
        "a insercao foi reportada DUAS vezes: um clique daria dois pontos"
    );
}
