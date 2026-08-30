//! Gates da **decomposição em ramos** — a lei que o report do Enio de 2026-08-30 cobrou.
//!
//! ⚠️ **Duas famílias de propósito.** As primeiras constroem o esqueleto à mão (uma lei de cada
//! vez, sem cozinhar nada); a última corre a **tartaruga a sério** sobre uma gramática que
//! bifurca — porque *a fixtura tem de conter o fenómeno*, e o fenómeno aqui é uma árvore, não
//! um array.

use super::*;

/// `sym` de um osso que desenha.
const F: f32 = b'F' as f32;
/// `sym` de uma âncora de instância (folha).
const J: f32 = b'J' as f32;

fn w(n: usize, v: f32) -> Vec<[f32; 2]> {
    vec![[v, v]; n]
}

/// ⭐⭐⭐ **O REPORT, literalmente.** Uma haste a direito é **UMA** fita, com um ponto por
/// elemento — não `n` retângulos.
///
/// ⚠️ A afirmação é sobre a CONTAGEM e sobre os PONTOS: um gate que só contasse as fitas
/// passaria com uma fita de dois pontos que ignorasse o resto da haste.
#[test]
fn a_straight_stem_is_one_ribbon_and_not_a_pile_of_rectangles() {
    let p = [[0.0, 0.0], [0.0, 1.0], [0.0, 2.0], [0.0, 3.0]];
    let parent = [-1.0, 0.0, 1.0, 2.0];
    let b = branches(&p, &parent, &w(4, 1.0), &[F; 4]);
    assert_eq!(b.len(), 1, "uma haste a direito tem de dar UMA fita: {b:?}");
    assert_eq!(
        b[0].points,
        p.to_vec(),
        "a fita tem de conter todos os pontos da haste, na ordem"
    );
    assert_eq!(b[0].widths.len(), b[0].points.len());
}

/// ⭐⭐ **Uma bifurcação NÃO deixa buraco** — a lei do cpfg (*o filho liga-se ao último ponto de
/// controlo antes do ramo*).
///
/// Tronco `0→1`, e em `1` nascem dois filhos. Saem três fitas, e as duas dos filhos **começam
/// exactamente no ponto do pai**.
#[test]
fn a_fork_gives_three_ribbons_and_the_children_start_at_the_parents_point() {
    //      3   4
    //       \ /
    //        1
    //        |
    //        0
    let p = [
        [0.0, 0.0],
        [0.0, 1.0],
        [0.0, 1.0], // nunca usado (não desenha)
        [-1.0, 2.0],
        [1.0, 2.0],
    ];
    let parent = [-1.0, 0.0, -1.0, 1.0, 1.0];
    let sym = [F, F, J, F, F];
    let b = branches(&p, &parent, &w(5, 1.0), &sym);
    assert_eq!(b.len(), 3, "tronco + dois filhos: {b:?}");
    // O tronco pára na bifurcação.
    assert_eq!(b[0].points, vec![[0.0, 0.0], [0.0, 1.0]]);
    // ⭐ Os dois filhos ABREM no ponto do pai — sem isto há um buraco visível na forquilha.
    for child in &b[1..] {
        assert_eq!(
            child.points[0],
            [0.0, 1.0],
            "o filho tem de começar no ponto do pai, senão a forquilha tem buraco: {child:?}"
        );
        assert_eq!(child.points.len(), 2);
    }
}

/// ⭐ **O filho nunca é mais grosso que o pai onde nasce** — a lei do SpeedTree.
///
/// ⚠️ E vale nos DOIS sentidos: um filho fino não engrossa a junção (senão o `min` seria um
/// `max` disfarçado), e um filho grosso não a faz exceder o pai.
#[test]
fn the_child_is_never_thicker_than_its_parent_where_it_is_born() {
    let p = [[0.0, 0.0], [0.0, 1.0], [-1.0, 2.0], [1.0, 2.0]];
    let parent = [-1.0, 0.0, 1.0, 1.0];
    // O pai (índice 1) tem largura 4; um filho fino (1) e um filho grosso (9).
    let size = vec![[4.0, 4.0], [4.0, 4.0], [1.0, 1.0], [9.0, 9.0]];
    let b = branches(&p, &parent, &size, &[F; 4]);
    let junction: Vec<f32> = b[1..].iter().map(|x| x.widths[0]).collect();
    assert_eq!(
        junction,
        vec![1.0, 4.0],
        "na junção vale o MENOR entre pai e filho — o fino fica fino, o grosso é aparado no pai"
    );
}

/// ⭐ **Uma folha não parte o tronco em dois.**
///
/// Um `J` pendurado no meio do tronco é um filho — mas não desenha osso. Contá-lo como
/// bifurcação partiria a fita exactamente onde o artista pôs uma folha, que é o defeito mais
/// difícil de diagnosticar desta família (*a planta parece certa e o tronco tem uma emenda*).
#[test]
fn a_leaf_hanging_off_the_trunk_does_not_cut_it_in_two() {
    let p = [[0.0, 0.0], [0.0, 1.0], [0.0, 1.0], [0.0, 2.0]];
    let parent = [-1.0, 0.0, 1.0, 1.0];
    let sym = [F, F, J, F];
    let b = branches(&p, &parent, &w(4, 1.0), &sym);
    assert_eq!(b.len(), 1, "a folha não é uma bifurcação: {b:?}");
    assert_eq!(b[0].points.len(), 3, "o tronco continua inteiro");
}

/// ⭐ **O perfil mede-se por ARCO, não por índice** — a lei que o `WidthStop::pos` desta casa
/// já declara.
///
/// Um L-System encurta o passo a cada geração (o `"`), então os pontos de um ramo são
/// desigualmente espaçados. Indexar por posição na lista poria o meio do perfil no sítio
/// errado — e a mesma planta desenhada com mais gerações mudaria de aparência.
#[test]
fn the_profile_is_measured_along_the_arc_and_not_by_index() {
    // Passos de 3 e 1: o ponto do meio está a 75 % do arco, não a 50 %.
    let p = [[0.0, 0.0], [3.0, 0.0], [4.0, 0.0]];
    let parent = [-1.0, 0.0, 1.0];
    let b = branches(&p, &parent, &w(3, 1.0), &[F; 3]);
    let f = b[0].arc_fractions();
    assert_eq!(f[0], 0.0);
    assert!(
        (f[1] - 0.75).abs() < 1e-6,
        "o ponto do meio está a 75 % do ARCO, não a 50 % da lista: {f:?}"
    );
    assert_eq!(f[2], 1.0);
}

/// **Um ramo de um ponto não é uma fita** — e o caso é real: uma raiz cuja única continuação é
/// uma folha fica sem filho que desenhe.
#[test]
fn a_single_point_branch_is_not_emitted() {
    let p = [[0.0, 0.0], [0.0, 0.0]];
    let parent = [-1.0, 0.0];
    let b = branches(&p, &parent, &w(2, 1.0), &[F, J]);
    assert!(b.is_empty(), "um ponto só não faz fita: {b:?}");
}

/// ⭐⭐⭐ **A ÁRVORE A SÉRIO** — a tartaruga corre uma gramática que bifurca, e o que sai é um
/// número pequeno de fitas, não um elemento por retângulo.
///
/// ⚠️ **É esta que contém o fenómeno.** As de cima medem uma lei de cada vez sobre um array
/// escrito à mão; esta mede o que o Enio vê. A barra é estrutural e não um número escolhido:
/// **toda fita tem ≥ 2 pontos** (senão não é fita) e **a soma dos pontos das fitas excede a
/// contagem de elementos** — porque cada junção repete o ponto do pai, que é exactamente a
/// costura que faz a forquilha não ter buraco.
#[test]
fn a_real_branching_plant_becomes_few_ribbons_not_one_per_segment() {
    let set = crate::turtle::Setup {
        angle: 25.0,
        step: 1.0,
        width: 1.0,
        width_scale: 0.7,
        length_scale: 1.0,
        root_angle: 90.0,
        tropism: 0.0,
        tropism_angle: -90.0,
        youngest: (0, 1.0),
        angle_frac: 1.0,
        orient_world: true,
    };
    let nop: &dyn Fn(&str) -> f32 = &|_| 0.0;
    let chain = crate::derive::axiom_modules("F[+F]F[-F]F", nop);
    let s = crate::turtle::walk(&chain, &set);
    let get2 = |n: &str| match s.get(n) {
        Some(ph2d_nodegraph::attr::Column::Vec2(v)) => v.clone(),
        _ => panic!("{n}"),
    };
    let get1 = |n: &str| match s.get(n) {
        Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.clone(),
        _ => panic!("{n}"),
    };
    let (p, parent, size, sym) = (get2("P"), get1("parent"), get2("size"), get1("sym"));
    let elements = p.len();
    let b = branches(&p, &parent, &size, &sym);

    assert!(!b.is_empty(), "uma planta que bifurca tem de dar fitas");
    assert!(
        b.len() < elements,
        "há {} fitas para {elements} elementos — isso é uma fita por retângulo, \
         que é exactamente o defeito",
        b.len()
    );
    for x in &b {
        assert!(x.points.len() >= 2, "fita degenerada: {x:?}");
        assert_eq!(x.widths.len(), x.points.len());
    }
    let seam: usize = b.iter().map(|x| x.points.len()).sum();
    assert!(
        seam > elements,
        "a soma dos pontos ({seam}) tem de exceder os elementos ({elements}): \
         é o ponto do pai repetido em cada junção que fecha a forquilha"
    );
}
