//! Gates do Shape Builder — o gesto inteiro, sem gfx.
//!
//! Estes não testam a booleana (ela tem os dela) nem o arranjo (idem). Testam **o que o
//! dedo do artista produz**: passar o cursor por regiões e soltar. É o seam.

use super::*;
use ph2d_vec_scene::{VecVertex, contains_point};

/// Um quadrado fechado, em MUNDO (a sessão já recebe tudo assado).
fn square(x0: f64, y0: f64, x1: f64, y1: f64) -> VecPath {
    VecPath {
        verts: [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// Duas caixas sobrepostas: A = [0,10]×[0,10], B = [6,16]×[0,10].
/// Três faces: só-A (x<6) · a sobreposição (6..10) · só-B (x>10).
fn session() -> BuildSession {
    BuildSession {
        arr: Arrangement::new(vec![
            square(0.0, 0.0, 10.0, 10.0),
            square(6.0, 0.0, 16.0, 10.0),
        ]),
        sources: vec![1, 2],
        marked: Vec::new(),
        hover: None,
        subtract: false,
        dragging: false,
    }
}

const ONLY_A: [f64; 2] = [3.0, 5.0];
const OVERLAP: [f64; 2] = [8.0, 5.0];
const ONLY_B: [f64; 2] = [13.0, 5.0];

/// Passar o cursor SEM apertar realça, mas não pinta. (Sem isto, o realce de hover viraria
/// uma marcação — e o artista destruiria a arte só de mover o mouse por cima dela.)
#[test]
fn hovering_highlights_but_does_not_mark() {
    let mut s = session();
    s.touch(ONLY_A);
    assert!(s.hover.is_some(), "a face sob o cursor é realçada");
    assert!(s.marked.is_empty(), "…e NÃO é pintada sem o botão");
}

/// **O gesto central:** arrastar por cima das três regiões e soltar dá UMA forma — a união.
#[test]
fn dragging_across_every_region_merges_them_into_one_shape() {
    let mut s = session();
    s.dragging = true;
    for p in [ONLY_A, OVERLAP, ONLY_B] {
        s.touch(p);
    }
    assert_eq!(s.marked.len(), 3, "as três faces foram pintadas");

    let out = s.resolve();
    assert_eq!(out.len(), 1, "uma forma só (as três faces se tocam)");
    // E ela é a UNIÃO: contém os três pontos, e nada fora das duas caixas.
    for p in [ONLY_A, OVERLAP, ONLY_B] {
        assert!(contains_point(&out[0], p), "a união contém {p:?}");
    }
    assert!(!contains_point(&out[0], [20.0, 5.0]), "e nada fora delas");
}

/// **A LUA CRESCENTE** — o gesto que justifica a feature existir.
///
/// Alt + clicar na sobreposição de duas formas: ela some, e sobram as duas fatias. No
/// Pathfinder isso exige entender qual op subtrai qual operando; aqui é apontar a parte que
/// sobra e apagá-la.
#[test]
fn alt_clicking_the_overlap_deletes_it_and_leaves_the_two_slices() {
    let mut s = session();
    s.subtract = true;
    s.dragging = true;
    s.touch(OVERLAP);
    assert_eq!(s.marked.len(), 1);

    let out = s.resolve();
    assert_eq!(
        out.len(),
        2,
        "as duas fatias, DESCONEXAS (a ponte foi removida)"
    );
    // Uma fatia de cada lado, e nenhuma contém a sobreposição.
    let has = |p: [f64; 2]| out.iter().any(|o| contains_point(o, p));
    assert!(has(ONLY_A), "a fatia da esquerda ficou");
    assert!(has(ONLY_B), "a fatia da direita ficou");
    assert!(!has(OVERLAP), "e a sobreposição SUMIU — é a lua crescente");
}

/// Pintar UMA face e unir: ela vira a sua própria forma, e a sobra continua existindo. É o
/// "extrair esta região" — o mesmo gesto, sem o Alt.
#[test]
fn clicking_one_region_extracts_it_and_keeps_the_rest() {
    let mut s = session();
    s.dragging = true;
    s.touch(OVERLAP);

    let out = s.resolve();
    assert_eq!(out.len(), 3, "a região extraída + as duas sobras");
    assert!(
        out.iter().any(|o| contains_point(o, OVERLAP)),
        "a região pintada saiu como forma"
    );
    assert!(out.iter().any(|o| contains_point(o, ONLY_A)));
    assert!(out.iter().any(|o| contains_point(o, ONLY_B)));
}

/// **Unir e subtrair são COMPLEMENTARES**, e é por isso que os dois saem da mesma conta:
/// o que uma entrega, a outra joga fora. Se divergirem, existe uma sequência de gestos que
/// perde ou duplica geometria — e o artista descobre isso tarde.
#[test]
fn merge_and_subtract_are_exact_complements_of_the_same_gesture() {
    let paint = |subtract: bool| {
        let mut s = session();
        s.subtract = subtract;
        s.dragging = true;
        s.touch(OVERLAP);
        s.resolve()
    };
    let merged = paint(false);
    let subtracted = paint(true);

    // O que a subtração devolve é EXATAMENTE a sobra que o merge também devolve.
    assert_eq!(subtracted.len(), 2);
    assert_eq!(merged.len(), 3);
    for p in [ONLY_A, ONLY_B] {
        assert!(subtracted.iter().any(|o| contains_point(o, p)));
        assert!(merged.iter().any(|o| contains_point(o, p)));
    }
    // E a face pintada só aparece no merge.
    assert!(merged.iter().any(|o| contains_point(o, OVERLAP)));
    assert!(!subtracted.iter().any(|o| contains_point(o, OVERLAP)));
}

/// Pintar tudo e SUBTRAIR apaga tudo. (É o extremo do complemento acima, e o gate de que a
/// sobra vazia não vira uma forma degenerada.)
#[test]
fn subtracting_every_region_leaves_nothing() {
    let mut s = session();
    s.subtract = true;
    s.dragging = true;
    for p in [ONLY_A, OVERLAP, ONLY_B] {
        s.touch(p);
    }
    assert!(s.resolve().is_empty(), "não sobrou nada, e nada é nada");
}

/// Soltar sem ter pintado nada é um NO-OP. Sem esta guarda, um clique perdido no vazio
/// destruiria as formas de origem e devolveria a união delas.
#[test]
fn releasing_without_painting_anything_is_a_no_op() {
    let mut s = session();
    s.dragging = true;
    s.touch([50.0, 50.0]); // fora de tudo
    assert!(s.marked.is_empty());
    assert!(s.resolve().is_empty(), "nada pintado, nada feito");
}

/// A mesma face pintada duas vezes (o cursor volta por cima dela) conta UMA vez — senão a
/// união receberia a mesma geometria repetida.
#[test]
fn painting_the_same_face_twice_marks_it_once() {
    let mut s = session();
    s.dragging = true;
    s.touch(OVERLAP);
    s.touch([8.5, 5.5]); // ainda na sobreposição
    s.touch(OVERLAP);
    assert_eq!(s.marked.len(), 1, "uma face, uma marca");
}

/// Uma sessão precisa de 2+ formas FECHADAS. Com menos, não há região para pintar — e abrir
/// a sessão mesmo assim faria o modo capturar o canvas e não fazer nada (o pior estado de
/// UI que existe: parece quebrado).
#[test]
fn a_session_needs_at_least_two_closed_shapes() {
    let mut scene = VecScene::new();
    let a = scene.push_path(square(0.0, 0.0, 10.0, 10.0));
    let xf = VecXforms::default();

    assert!(
        BuildSession::open(&scene, &xf, &[a]).is_none(),
        "uma forma só não tem região"
    );

    // Um contorno ABERTO não é região (a booleana precisa de fechado).
    let open = scene.push_path(VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([5.0, 5.0])],
        closed: false,
        ..VecPath::default()
    });
    assert!(
        BuildSession::open(&scene, &xf, &[a, open]).is_none(),
        "uma fechada + uma aberta ainda é UMA região"
    );

    let b = scene.push_path(square(6.0, 0.0, 16.0, 10.0));
    assert!(BuildSession::open(&scene, &xf, &[a, b]).is_some());
}
