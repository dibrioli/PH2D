//! **A CADEIA COM UM VERBO POR PASSO** — os gates de `apply_chain_checked`.
//!
//! O motor sempre dobrou à esquerda (`((a op b) op c)`); o que estava fixo era o **verbo**. Esta
//! porta deixa cada dobra trazer o seu, que é o que torna exprimível *"somo com esta, subtraio
//! aquela"* sem grafo nenhum — o compound shape vivo do Illustrator, e a pilha de modificadores
//! booleanos do Blender.
//!
//! ⚠️ O oráculo é a **ÁREA que o rasterizador de fato preenche** (winding NonZero ≠ 0), amostrada
//! numa **janela COMUM** a todas as formas comparadas. Medir cada forma na própria bbox daria
//! resoluções diferentes a cada uma, e a diferença da RÉGUA entraria na conta como se fosse
//! diferença do método.
//!
//! ⚠️ A fixture é escolhida para que os quatro resultados que ela compara tenham áreas
//! **claramente** distintas (500 · 300 · 600), e não separadas por ruído de amostragem: um gate de
//! ordem que passasse por 0,3% de diferença estaria a medir a grade.

use kurbo::{BezPath, Point, Shape};
use ph2d_vec_boolean::{BoolOp, apply_chain_checked, apply_many_checked};
use ph2d_vec_scene::{VecPath, rectangle};

/// A janela comum, e a grade. Cobre com folga as três formas da fixture.
const WIN: ([f64; 2], [f64; 2]) = ([-5.0, -5.0], [35.0, 25.0]);
const GRID: usize = 800;

/// A folga da RÉGUA, **derivada dela** e não escolhida: a célula mede `40/800 × 30/800` ≈
/// `0,05 × 0,0375`, e a borda mais longa que estes gates medem tem ~140 de perímetro (o anel de
/// fora mais o do furo). Meia célula ao longo dessa borda dá ~3 — logo **4**.
///
/// ⚠️ Ela é duas ordens de grandeza menor que o que os gates separam (500 · 300 · 600): a folga
/// absorve a grade sem chegar perto de absorver uma troca de verbo ou de ordem.
const TOL: f64 = 4.0;

/// A área preenchida, amostrada na janela COMUM — a mesma régua para todas as comparações.
///
/// ⚠️ **Por CONTORNO, nunca pelos `verts` corridos.** O resultado de um `Subtract` é um caminho
/// **composto**: o anel de fora e o do furo, com sentidos opostos, e é a oposição que faz o
/// winding dar zero dentro do buraco. Varrer `verts` como um anel só costura os dois num traçado
/// que não existe — e a primeira versão deste gate reportou **600,75 onde a resposta é 500**,
/// acusando o motor de ignorar o verbo quando quem ignorava o furo era a régua.
fn area(paths: &[VecPath]) -> f64 {
    let mut bez = BezPath::new();
    for path in paths {
        for k in 0..path.contour_count() {
            let Some((verts, _closed)) = path.contour(k) else {
                continue;
            };
            let n = verts.len();
            if n < 3 {
                continue;
            }
            bez.move_to(Point::new(verts[0].anchor[0], verts[0].anchor[1]));
            for i in 0..n {
                let a = &verts[i];
                let b = &verts[(i + 1) % n];
                bez.curve_to(
                    Point::new(a.out_handle[0], a.out_handle[1]),
                    Point::new(b.in_handle[0], b.in_handle[1]),
                    Point::new(b.anchor[0], b.anchor[1]),
                );
            }
            bez.close_path();
        }
    }
    if bez.elements().len() < 3 {
        return 0.0;
    }
    let (lo, hi) = WIN;
    let n = GRID as f64;
    let inside = (0..GRID)
        .flat_map(|iy| (0..GRID).map(move |ix| (ix, iy)))
        .filter(|&(ix, iy)| {
            let x = lo[0] + (hi[0] - lo[0]) * (ix as f64 + 0.5) / n;
            let y = lo[1] + (hi[1] - lo[1]) * (iy as f64 + 0.5) / n;
            bez.winding(Point::new(x, y)) != 0
        })
        .count() as f64;
    inside * (hi[0] - lo[0]) * (hi[1] - lo[1]) / (n * n)
}

/// `A` (a base) · `B` (à direita, encosta e passa) · `C` (pequeno, no meio de B e a morder A).
fn rig() -> (VecPath, VecPath, VecPath) {
    (
        rectangle([0.0, 0.0], [20.0, 20.0]),
        rectangle([10.0, 0.0], [30.0, 20.0]),
        rectangle([15.0, 5.0], [25.0, 15.0]),
    )
}

/// **O VERBO UNIFORME É A BOOLEANA DE SEMPRE.** A porta N-ária passou a delegar nesta cadeia, e
/// este é o gate que prova que a delegação não mexeu em nada.
///
/// ⚠️ Sem ele, a refatoração seria uma mudança de comportamento disfarçada de mudança de forma.
#[test]
fn a_uniform_chain_is_the_boolean_that_was_always_there() {
    let (a, b, c) = rig();
    for op in [
        BoolOp::Union,
        BoolOp::Subtract,
        BoolOp::Intersect,
        BoolOp::Exclude,
    ] {
        let n_ary = apply_many_checked(&[&a, &b, &c], op).unwrap();
        let chain = apply_chain_checked(&a, &[(&b, op), (&c, op)]).unwrap();
        assert_eq!(
            chain, n_ary,
            "{op:?}: a cadeia uniforme divergiu da porta N-ária"
        );
    }
}

/// **CADA DOBRA TRAZ O SEU VERBO** — a capacidade inteira, numa afirmação.
///
/// `(A ∪ B) − C` cobre `[0,30]×[0,20]` com um furo de 10×10: **500**. Trocar os dois verbos dá
/// `(A − B) ∪ C` = uma tira de 200 mais um retângulo solto de 100: **300**.
#[test]
fn a_different_verb_per_step_draws_a_different_shape() {
    let (a, b, c) = rig();
    let union_then_cut =
        area(&apply_chain_checked(&a, &[(&b, BoolOp::Union), (&c, BoolOp::Subtract)]).unwrap());
    let cut_then_union =
        area(&apply_chain_checked(&a, &[(&b, BoolOp::Subtract), (&c, BoolOp::Union)]).unwrap());
    assert!(
        (union_then_cut - 500.0).abs() < TOL,
        "(A ∪ B) − C devia dar 500, deu {union_then_cut:.2}"
    );
    assert!(
        (cut_then_union - 300.0).abs() < TOL,
        "(A − B) ∪ C devia dar 300, deu {cut_then_union:.2}"
    );
}

/// **A CADEIA É UMA DOBRA À ESQUERDA** — *"quem vem depois atua sobre o resultante dos
/// anteriores"*, que é a frase inteira do desenho (Enio, 2026-08-22).
///
/// A prova é a equivalência com o ANINHAMENTO: combinar `A` e `B`, e só então subtrair `C` do
/// resultado, tem de dar exatamente o que a cadeia dá — porque é literalmente o que ela faz.
///
/// ⚠️ Este gate é o que impede a cadeia de virar, por acidente, *"cada forma contra a BASE"*
/// (`(A∪B) ∪ (A−C)`), que é a outra leitura possível de "uma operação por forma" e desenha outra
/// coisa.
#[test]
fn each_step_folds_onto_the_accumulated_result_not_onto_the_base() {
    let (a, b, c) = rig();
    let chain = apply_chain_checked(&a, &[(&b, BoolOp::Union), (&c, BoolOp::Subtract)]).unwrap();
    // O aninhamento, feito à mão: primeiro o grupo de dentro, depois o de fora sobre o resultado.
    let inner = apply_many_checked(&[&a, &b], BoolOp::Union).unwrap();
    let inner_refs: Vec<&VecPath> = inner.iter().collect();
    let mut outer_in = inner_refs.clone();
    outer_in.push(&c);
    let nested = apply_many_checked(&outer_in, BoolOp::Subtract).unwrap();
    let (ca, na) = (area(&chain), area(&nested));
    assert!(
        (ca - na).abs() < TOL,
        "a cadeia ({ca:.2}) tem de dar o mesmo que aninhar ({na:.2})"
    );
    assert!(
        (ca - 500.0).abs() < TOL,
        "e as duas têm de dar 500 — se derem 600, dobraram sobre a BASE, não sobre o acumulado"
    );
}

/// **A ORDEM DECIDE, e é por isso que ela precisa de aparecer na hierarquia.** As mesmas três
/// formas e os mesmos dois verbos, trocados de posição: 500 contra 600.
#[test]
fn the_order_of_the_steps_changes_the_result() {
    let (a, b, c) = rig();
    let bc =
        area(&apply_chain_checked(&a, &[(&b, BoolOp::Union), (&c, BoolOp::Subtract)]).unwrap());
    let cb =
        area(&apply_chain_checked(&a, &[(&c, BoolOp::Subtract), (&b, BoolOp::Union)]).unwrap());
    assert!(
        (bc - 500.0).abs() < TOL && (cb - 600.0).abs() < TOL,
        "trocar a ordem tem de mudar o desenho: {bc:.2} e {cb:.2}"
    );
}

/// **A BASE SOZINHA NÃO É UMA OPERAÇÃO.** Sem nenhuma dobra não há nada a combinar, e o vazio é a
/// resposta — a mesma que a porta N-ária sempre deu a uma lista de um só.
///
/// ⚠️ E note o que este gate NÃO consegue testar: *a base com um verbo próprio*. Ela não tem onde
/// o guardar — o verbo vive no par que entra, e um par sem path não existe. É a representação a
/// apagar o caso especial, em vez de um `if` a defendê-lo.
#[test]
fn a_base_with_no_folds_is_not_an_operation() {
    let (a, _b, _c) = rig();
    assert!(apply_chain_checked(&a, &[]).unwrap().is_empty());
}
