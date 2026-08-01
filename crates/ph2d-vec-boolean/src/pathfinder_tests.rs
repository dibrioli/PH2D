//! Gates das quatro operações novas do Pathfinder (plano 25 §8, W5).
//!
//! O oráculo é sempre **ÁREA**, e não contagem de peças: uma receita errada devolve com frequência
//! o número certo de formas — é a contagem que engana, não a medida.

use super::{PathfinderOp, pathfinder};
use crate::area;
use kurbo::Shape;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecVertex};

/// Um retângulo `[x0, x1] × [y0, y1]`, fechado, com o `fill` dado.
fn rect(x0: f64, y0: f64, x1: f64, y1: f64, fill: u8) -> VecPath {
    VecPath {
        verts: [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(fill, fill, fill, 255))),
        ..VecPath::default()
    }
}

/// Um quadrado `[x, x+s] × [y, y+s]`.
fn square(x: f64, y: f64, s: f64, fill: u8) -> VecPath {
    rect(x, y, x + s, y + s, fill)
}

/// Três quadrados de lado 4 que se sobrepõem **em escada**: cada um cobre um quarto do anterior.
/// A sobreposição de cada par vizinho é `2×4 = 8`; o primeiro e o último **não** se tocam.
fn staircase() -> Vec<VecPath> {
    vec![
        square(0.0, 0.0, 4.0, 10),
        square(2.0, 0.0, 4.0, 20),
        square(4.0, 0.0, 4.0, 30),
    ]
}

fn refs(v: &[VecPath]) -> Vec<&VecPath> {
    v.iter().collect()
}

/// **Minus Back: a FRENTE menos tudo o que está atrás.**
///
/// Na escada, a da frente (`[4,8]`) perde os `2×4 = 8` que a do meio cobre ⇒ `16 − 8 = 8`.
/// ⚠️ O oráculo distingue Minus Back de `Subtract`: o `Subtract` normal daria a de TRÁS menos as
/// outras (`16 − 8 = 8` também!) — e é por isso que o gate mede também **ONDE** a peça está.
#[test]
fn minus_back_keeps_the_front_and_eats_what_is_behind() {
    let src = staircase();
    let out = pathfinder(&refs(&src), PathfinderOp::MinusBack).expect("motor ok");
    assert_eq!(out.len(), 1, "uma peça");
    let a = area(&out[0]);
    assert!((a - 8.0).abs() < 1e-6, "área {a}, esperada 8");
    // A peça é a metade DIREITA da forma da frente: x ∈ [6, 8].
    let bb = crate::to_bez(&out[0]).bounding_box();
    assert!(
        (bb.min_x() - 6.0).abs() < 1e-6 && (bb.max_x() - 8.0).abs() < 1e-6,
        "a peça está em x ∈ [{}, {}] -- Minus Back guardou a forma errada",
        bb.min_x(),
        bb.max_x()
    );
}

/// **Trim: todas sobrevivem, e nenhuma sobreposição fica.**
///
/// A soma das áreas passa a ser a da UNIÃO (`4×4 + 2×4 + 2×4 = 32`), porque cada par vizinho
/// perdeu a sua sobreposição exactamente uma vez.
#[test]
fn trim_keeps_every_shape_and_removes_all_overlap() {
    let src = staircase();
    let out = pathfinder(&refs(&src), PathfinderOp::Trim).expect("motor ok");
    assert_eq!(out.len(), 3, "as três sobrevivem");
    let sum: f64 = out.iter().map(area).sum();
    let union = area(&crate::apply_many(&refs(&src), crate::BoolOp::Union)[0]);
    assert!(
        (sum - union).abs() < 1e-6,
        "as peças somam {sum} e a união mede {union} -- sobrou sobreposição"
    );
    assert!(
        (union - 32.0).abs() < 1e-6,
        "a fixture mudou: união {union}"
    );
}

/// **Trim não toca a de cima** — não há nada acima dela.
#[test]
fn trim_leaves_the_top_one_alone() {
    let src = staircase();
    let out = pathfinder(&refs(&src), PathfinderOp::Trim).expect("motor ok");
    let top = out.last().expect("três peças");
    assert!(
        (area(top) - 16.0).abs() < 1e-6,
        "a de cima foi recortada: {}",
        area(top)
    );
}

/// **Cada peça do Trim veste o estilo da SUA fonte** — não o do topo da pilha.
///
/// Sem a re-estampagem, o `apply_many` doaria o estilo do último argumento (a forma de cima), e
/// as três sairiam da mesma cor: uma operação de limpeza a repintar a arte.
#[test]
fn each_trimmed_piece_wears_its_own_style() {
    let src = staircase();
    let out = pathfinder(&refs(&src), PathfinderOp::Trim).expect("motor ok");
    for (piece, source) in out.iter().zip(&src) {
        assert_eq!(
            piece.fill, source.fill,
            "a peça vestiu a cor de outra forma"
        );
    }
}

/// **Crop: o que fica é o que está DENTRO da forma do topo, e o topo some.**
///
/// A do topo (`[4,8]`) recorta as outras duas: da de trás (`[0,4]`) não sobra nada (elas só se
/// tocam na aresta), e da do meio (`[2,6]`) sobra `[4,6] = 2×4 = 8`.
#[test]
fn crop_keeps_what_is_inside_the_top_and_discards_the_frame() {
    let src = staircase();
    let out = pathfinder(&refs(&src), PathfinderOp::Crop).expect("motor ok");
    let sum: f64 = out.iter().map(area).sum();
    assert!((sum - 8.0).abs() < 1e-6, "área total {sum}, esperada 8");
    // A moldura não volta: nenhuma peça pode ter a área cheia da forma do topo.
    assert!(
        out.iter().all(|p| (area(p) - 16.0).abs() > 1e-6),
        "a moldura sobreviveu ao Crop -- ela é a ferramenta, não o conteúdo"
    );
}

/// **Merge: as de MESMA cor que se TOCAM viram uma; a de outra cor fica.**
///
/// Dois quadrados encostados (`[0,4]` e `[4,8]`, mesma cor) com uma faixa de outra cor por cima
/// mordendo o topo dos dois. Depois do Trim os dois viram Ls que continuam a encostar-se em
/// `x = 4` ⇒ soldam num caminho só. Áreas: `13 + 13` para a cor 10, `12` para a 99.
#[test]
fn merge_welds_the_same_fill_that_touch_and_leaves_the_others() {
    let src = vec![
        square(0.0, 0.0, 4.0, 10),
        square(4.0, 0.0, 4.0, 10),
        rect(1.0, 3.0, 7.0, 5.0, 99),
    ];
    let out = pathfinder(&refs(&src), PathfinderOp::Merge).expect("motor ok");
    assert_eq!(
        out.len(),
        2,
        "duas classes de cor, duas formas: {}",
        out.len()
    );
    let sum: f64 = out.iter().map(area).sum();
    assert!((sum - 38.0).abs() < 1e-6, "área total {sum}, esperada 38");
    // A classe da cor 10 virou UMA forma de área 26 (os dois Ls soldados).
    let welded = out
        .iter()
        .find(|p| (area(p) - 26.0).abs() < 1e-6)
        .expect("os dois Ls da mesma cor deviam ter soldado numa forma de area 26");
    assert_eq!(welded.fill, src[0].fill, "a solda vestiu a cor errada");
}

/// **Duas da mesma cor que NÃO se tocam continuam duas** — e isto é semântica, não limitação.
///
/// ⚠️ Eu esperava o contrário e o produto me corrigiu: o motor agrupa por CONTENÇÃO, então
/// componentes desconexas saem como caminhos separados. É também o que o Illustrator faz — o
/// Merge dele solda *adjacent or overlapping*, e duas ilhas não são nenhum dos dois.
#[test]
fn merge_does_not_weld_islands_that_never_touch() {
    let src = vec![
        square(0.0, 0.0, 4.0, 10),
        square(2.0, 0.0, 4.0, 99),
        square(4.0, 0.0, 4.0, 10),
    ];
    let out = pathfinder(&refs(&src), PathfinderOp::Merge).expect("motor ok");
    assert_eq!(
        out.len(),
        3,
        "as duas ilhas da cor 10 nao se tocam: {}",
        out.len()
    );
    let sum: f64 = out.iter().map(area).sum();
    assert!((sum - 32.0).abs() < 1e-6, "área total {sum}, esperada 32");
}

/// **Merge com todas da mesma cor dá UMA forma** — e a área é a da união.
#[test]
fn merge_of_one_colour_gives_one_shape() {
    let src = staircase()
        .into_iter()
        .map(|mut p| {
            p.fill = Some(Paint::solid(Rgba8::new(7, 7, 7, 255)));
            p
        })
        .collect::<Vec<_>>();
    let out = pathfinder(&refs(&src), PathfinderOp::Merge).expect("motor ok");
    assert_eq!(out.len(), 1, "uma cor, uma forma");
    assert!(
        (area(&out[0]) - 32.0).abs() < 1e-6,
        "área {}",
        area(&out[0])
    );
}

/// As quatro de conjunto continuam a passar pelo motor **verbatim** — a porta nova não pode ter
/// mudado o que já shipava.
#[test]
fn the_four_set_ops_still_go_straight_to_the_engine() {
    let src = staircase();
    for (op, b) in [
        (PathfinderOp::Union, crate::BoolOp::Union),
        (PathfinderOp::Subtract, crate::BoolOp::Subtract),
        (PathfinderOp::Intersect, crate::BoolOp::Intersect),
        (PathfinderOp::Exclude, crate::BoolOp::Exclude),
    ] {
        let via_door = pathfinder(&refs(&src), op).expect("motor ok");
        let direct = crate::apply_many(&refs(&src), b);
        assert_eq!(via_door.len(), direct.len(), "{op:?}: contagem");
        for (a, d) in via_door.iter().zip(&direct) {
            assert!((area(a) - area(d)).abs() < 1e-9, "{op:?}: área divergiu");
        }
    }
}

/// Menos de duas formas não é uma operação de Pathfinder — devolve vazio em vez de inventar.
#[test]
fn fewer_than_two_shapes_is_not_an_operation() {
    let one = square(0.0, 0.0, 1.0, 0);
    for op in [
        PathfinderOp::Trim,
        PathfinderOp::Merge,
        PathfinderOp::Crop,
        PathfinderOp::MinusBack,
    ] {
        assert!(
            pathfinder(&[&one], op).expect("motor ok").is_empty(),
            "{op:?} com uma forma só"
        );
    }
}

/// **`Ok(vazio)` e `Err` são coisas DIFERENTES** — e até esta wave eram indistinguíveis: o
/// `binary_op(...).ok()?` dobrava a falha do motor em vazio, e o artista via o mesmo nada nos dois
/// casos, num crate que se autodeclara *early beta*.
///
/// Duas formas DISJUNTAS interseptadas não têm resposta — e isso é `Ok(vazio)`, não erro.
#[test]
fn an_empty_result_is_not_a_failure() {
    let a = square(0.0, 0.0, 1.0, 0);
    let b = square(50.0, 50.0, 1.0, 0);
    let out = pathfinder(&[&a, &b], PathfinderOp::Intersect).expect("disjuntas nao sao um ERRO");
    assert!(out.is_empty(), "duas disjuntas nao se intersetam");
}

/// **Uma entrada não-finita vira `Err`, com o motivo — e NÃO derruba o app.**
///
/// ⚠️ Medido, e é o achado desta fatia: o `linesweeper` **PANICA** com `NaN` lá dentro
/// (`geom.rs:63`, `assert!(x.is_finite())`) em vez de devolver o `Error::NaN` que ele declara — o
/// `binary_op` dele só examina o BOUNDING BOX, e `min`/`max` com NaN devolve o outro operando,
/// então o NaN atravessa a checagem. A guarda é NOSSA.
///
/// E a entrada é alcançável de verdade: um `Transform` degenerado assado na geometria (ADR-0111)
/// produz exactamente isto. Este gate é a diferença entre um toast e um crash.
#[test]
fn a_refused_input_carries_its_reason() {
    let good = square(0.0, 0.0, 4.0, 0);
    let mut bad = square(1.0, 1.0, 2.0, 0);
    bad.verts[0].anchor[0] = f64::NAN;
    bad.verts[0].in_handle[0] = f64::NAN;
    bad.verts[0].out_handle[0] = f64::NAN;
    let err = pathfinder(&[&good, &bad], PathfinderOp::Union)
        .expect_err("uma coordenada NaN tem de ser RECUSADA, nao dobrada em vazio");
    assert!(
        !err.reason().is_empty(),
        "a recusa chegou sem motivo -- e' o motivo que o artista le^"
    );
}
