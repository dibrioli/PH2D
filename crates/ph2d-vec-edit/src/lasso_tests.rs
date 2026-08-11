//! **Gates do LAÇO** — irmão de `selection.rs`, o sujeito.
//!
//! O plano 25 §9 nomeava o laço como *"wave dela"*, e a wave anterior (o dono no par de seleção)
//! era o pré-requisito: um laço que varre os nós de duas formas não significa nada enquanto a
//! seleção só souber guardar os de uma.
//!
//! ⚠️ **O gate que carrega esta wave é o da EQUIVALÊNCIA** — um laço cujo polígono É um retângulo
//! tem de apanhar **exatamente** o que a caixa apanha. Ele não conhece a implementação (nem sabe
//! que existe um corpo partilhado), e é o que impede as duas metades de divergirem no dia em que
//! uma delas ganhar um caso especial: se alguém der ao laço uma cópia do corpo, o gate continua
//! verde no dia da cópia e fica vermelho no primeiro refino de uma só.
//!
//! ⚠️ **E o discriminador é o laço CÔNCAVO.** Um laço implementado como a bounding box do próprio
//! caminho passa em todo gate de contagem simples — é a forma mais provável de a feature nascer
//! errada, porque *funciona* em toda fixture convexa.

use crate::PenTool;
use ph2d_vec_scene::{VecPathId, VecScene, VecViewState, rectangle};

/// A em `x ∈ [0,1]`, B em `x ∈ [2,3]` — o mesmo par do irmão `multi_path_tests`.
fn two_squares() -> (VecScene, PenTool, VecPathId, VecPathId) {
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
    (scene, PenTool::default(), a, b)
}

/// O polígono de um retângulo `[min, max]` — quatro cantos, anti-horário.
fn rect_poly(min: [f64; 2], max: [f64; 2]) -> Vec<[f64; 2]> {
    vec![
        [min[0], min[1]],
        [max[0], min[1]],
        [max[0], max[1]],
        [min[0], max[1]],
    ]
}

// ── O ALCANCE ───────────────────────────────────────────────────────────────

/// **O laço apanha os nós dentro dele, e só esses.**
#[test]
fn a_lasso_picks_only_the_nodes_inside_it() {
    let (scene, mut pen, a, _b) = two_squares();
    // Um losango que cobre A inteira e nem chega a B.
    let poly = vec![[0.5, -1.0], [1.8, 0.5], [0.5, 2.0], [-0.8, 0.5]];
    pen.lasso_select_with(&scene, &poly, false);
    assert_eq!(pen.selected_verts().len(), 4, "os quatro cantos de A");
    assert_eq!(pen.selected_paths(), [a], "B ficou de fora");
}

/// **O laço CÔNCAVO exclui o que ele contorna** — o discriminador.
///
/// A caixa envolvente deste polígono cobre as DUAS formas; o polígono cobre só a da esquerda. Um
/// laço implementado como o bbox do próprio caminho apanha 8 e passa em todo gate de contagem.
///
/// Mutação que tem de sangrar: trocar `point_in_polygon` pelo teste da caixa envolvente do
/// polígono ⇒ 8 nós.
#[test]
fn a_concave_lasso_excludes_what_it_curls_around() {
    let (scene, mut pen, a, b) = two_squares();
    // Um "C" deitado: sobe pela esquerda, engole A, volta por baixo — a boca do C fica virada para
    // B, que está DENTRO da caixa envolvente e FORA do polígono.
    let poly = vec![
        [-0.5, -0.5],
        [3.5, -0.5],
        [3.5, -0.2],
        [-0.2, -0.2],
        [-0.2, 1.2],
        [3.5, 1.2],
        [3.5, 1.5],
        [-0.5, 1.5],
    ];
    pen.lasso_select_with(&scene, &poly, false);
    assert_eq!(
        pen.selected_verts().len(),
        0,
        "o C so' cerca o VAO entre as formas: nenhum no' cai dentro dele — se apanhou, o laco esta' \
         a usar a CAIXA do proprio caminho, e nao o caminho"
    );
    // O controle: a caixa envolvente DESSE mesmo caminho apanha as duas formas inteiras.
    let mut ctl = PenTool::default();
    ctl.box_select(&scene, [-0.5, -0.5], [3.5, 1.5]);
    assert_eq!(ctl.selected_verts().len(), 8);
    assert_eq!(ctl.selected_paths(), [a, b]);
}

/// **Um laço RETANGULAR apanha exatamente o que a caixa apanha** — a equivalência.
///
/// O oráculo não conhece a implementação: compara duas seleções produzidas por duas portas
/// públicas. É ele que prova que o filtro de escondido/travado, o modo aditivo, o primário e o
/// `selected_paths` são os MESMOS nos dois — sem afirmar nada sobre como.
#[test]
fn a_rectangular_lasso_picks_exactly_what_the_box_picks() {
    for (min, max) in [
        ([-0.5, -0.5], [3.5, 1.5]), // as duas
        ([-0.5, -0.5], [1.5, 1.5]), // só A
        ([1.5, -0.5], [3.5, 1.5]),  // só B
        ([0.4, 0.4], [0.6, 0.6]),   // nenhuma
        ([-0.5, -0.5], [3.5, 0.5]), // a fileira de baixo das duas
    ] {
        let (scene, mut boxed, _, _) = two_squares();
        let (_, mut lassoed, _, _) = two_squares();
        boxed.box_select(&scene, min, max);
        lassoed.lasso_select_with(&scene, &rect_poly(min, max), false);
        assert_eq!(
            boxed.selected_verts(),
            lassoed.selected_verts(),
            "a caixa e um laco retangular discordaram em {min:?}..{max:?}"
        );
        assert_eq!(boxed.selected_paths(), lassoed.selected_paths());
        assert_eq!(boxed.selected(), lassoed.selected());
    }
}

/// **O laço atravessa formas** — a wave anterior a pagar-se.
#[test]
fn the_lasso_spans_shapes_like_the_box_does() {
    let (scene, mut pen, a, b) = two_squares();
    let poly = rect_poly([-0.5, -0.5], [3.5, 1.5]);
    pen.lasso_select_with(&scene, &poly, false);
    assert_eq!(pen.selected_verts().len(), 8);
    assert_eq!(pen.selected_paths(), [a, b]);
}

/// **Shift SOMA, e sem ele SUBSTITUI** — o mesmo do retângulo.
#[test]
fn the_lasso_adds_when_additive_and_replaces_when_not() {
    let (scene, mut pen, a, b) = two_squares();
    pen.lasso_select_with(&scene, &rect_poly([-0.5, -0.5], [1.5, 1.5]), false);
    assert_eq!(pen.selected_verts().len(), 4);
    pen.lasso_select_with(&scene, &rect_poly([1.5, -0.5], [3.5, 1.5]), true);
    assert_eq!(pen.selected_verts().len(), 8, "somou B a A");
    assert_eq!(pen.selected_paths(), [a, b]);
    // E sem aditivo, o segundo laço fica sozinho.
    pen.lasso_select_with(&scene, &rect_poly([1.5, -0.5], [3.5, 1.5]), false);
    assert_eq!(pen.selected_verts().len(), 4);
    assert_eq!(pen.selected_paths(), [b]);
}

/// **O laço respeita ESCONDIDO e TRAVADO** — a exigência que a wave do dono criou, herdada pelo
/// corpo partilhado.
///
/// Mutação que tem de sangrar: tirar o `is_pickable` do corpo ⇒ 8.
#[test]
fn the_lasso_leaves_hidden_and_locked_shapes_alone() {
    let (scene, mut pen, a, b) = two_squares();
    pen.set_view(VecViewState {
        hidden: vec![b],
        ..Default::default()
    });
    pen.lasso_select_with(&scene, &rect_poly([-0.5, -0.5], [3.5, 1.5]), false);
    assert_eq!(pen.selected_verts().len(), 4, "B esta' escondida");
    assert_eq!(pen.selected_paths(), [a]);

    let (scene, mut pen, a, b) = two_squares();
    pen.set_view(VecViewState {
        locked: vec![b],
        ..Default::default()
    });
    pen.lasso_select_with(&scene, &rect_poly([-0.5, -0.5], [3.5, 1.5]), false);
    assert_eq!((pen.verts_in(a).count(), pen.verts_in(b).count()), (4, 0));
}

/// **Um laço DEGENERADO não apanha nada** — menos de três pontos não delimitam área.
///
/// É o gêmeo do retângulo de tamanho zero: um gesto que falhou. E não deixa a seleção de OBJETO em
/// ruínas — a região vazia não desmancha o que já estava escolhido (a mesma lei do irmão).
#[test]
fn a_degenerate_lasso_selects_nothing() {
    let (scene, mut pen, a, _b) = two_squares();
    pen.lasso_select_with(&scene, &rect_poly([-0.5, -0.5], [1.5, 1.5]), false);
    assert_eq!(pen.selected_paths(), [a]);
    for poly in [vec![], vec![[0.5, 0.5]], vec![[0.0, 0.0], [1.0, 1.0]]] {
        pen.lasso_select_with(&scene, &poly, false);
        assert_eq!(
            pen.selected_verts().len(),
            0,
            "poly de {} pontos",
            poly.len()
        );
        assert_eq!(
            pen.selected_paths(),
            [a],
            "a selecao de OBJETO sobreviveu ao gesto que falhou"
        );
    }
}
