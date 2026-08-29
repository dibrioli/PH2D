//! Gates da **tartaruga** — o que a cadeia vira quando é desenhada, e a invariante do rig
//! que faz disto uma árvore em vez de uma nuvem.

use super::*;
use crate::derive::axiom_modules;

fn nop(_: &str) -> f32 {
    0.0
}

fn setup() -> Setup {
    Setup {
        angle: 90.0,
        step: 1.0,
        width: 1.0,
        width_scale: 0.5,
        length_scale: 0.5,
        root_angle: 90.0,
        tropism: 0.0,
        tropism_angle: -90.0,
        youngest: (0, 1.0),
        // ⚠️ **LOCAL na fixtura de base**, de propósito: os gates da invariante do rig medem o
        // contrato do `rig.*`, e é ele que exige o ângulo local. O modo de MUNDO (o default do
        // produto) tem gates próprios.
        orient_world: false,
    }
}

fn draw(src: &str, set: &Setup) -> Stream {
    let p: &dyn Fn(&str) -> f32 = &nop;
    walk(&axiom_modules(src, p), set)
}

fn scal(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("coluna escalar {name}"),
    }
}

fn vec2(s: &Stream, name: &str) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("coluna vec2 {name}"),
    }
}

/// Três passos a direito: a raiz mais três elementos, empilhados.
#[test]
fn a_straight_stem_stacks_element_on_element_out_of_the_root() {
    let s = draw("FFF", &setup());
    assert_eq!(s.count(), 4, "a raiz mais tres");
    let p = vec2(&s, "P");
    for (i, q) in p.iter().enumerate() {
        assert!(q[0].abs() < 1e-4, "elemento {i} fora do eixo: {q:?}");
        assert!((q[1] - i as f32).abs() < 1e-4, "elemento {i} em y = {i}");
    }
    assert_eq!(scal(&s, "parent"), vec![-1.0, 0.0, 1.0, 2.0]);
}

/// ⭐ **Um ramo pendura-se onde foi aberto** — e ao fechar, a haste continua do MESMO sítio.
///
/// Em `F[+F]F`, o segundo e o terceiro elementos desenhados penduram-se os dois no primeiro.
/// É a assinatura de um galho; uma cadeia única não a produz.
#[test]
fn a_branch_and_the_stem_after_it_hang_off_the_same_element() {
    let s = draw("F[+F]F", &setup());
    let parent = scal(&s, "parent");
    assert_eq!(s.count(), 4, "raiz + tres F");
    assert_eq!(parent[2], 1.0, "o F do ramo pendura no F da haste");
    assert_eq!(parent[3], 1.0, "e o F depois do `]` tambem");
}

/// ⭐⭐ **A INVARIANTE DO RIG: um osso nunca estica.**
///
/// `‖P[i] − P[pai]‖ == len[i]` para todo elemento, sobre uma gramática que usa ramos, saltos,
/// espessura, passo variável e marcas. É o que torna a saída deste nó legítima no contrato
/// `rig.*` — e o que se perderia em silêncio se a posição fosse calculada de outra maneira
/// que não a do `rig.fk`.
#[test]
fn every_bone_measures_exactly_its_own_length() {
    let mut set = setup();
    set.angle = 33.0;
    let s = draw("F[+F!F]-F\"F f F[-FJ]F", &set);
    let (p, parent, len) = (vec2(&s, "P"), scal(&s, "parent"), scal(&s, "len"));
    assert!(
        s.count() > 6,
        "a fixtura tem de ter elementos: {}",
        s.count()
    );
    for i in 0..s.count() {
        let par = parent[i];
        if par < 0.0 {
            continue;
        }
        let j = par as usize;
        let d = (p[i][0] - p[j][0]).hypot(p[i][1] - p[j][1]);
        assert!(
            (d - len[i]).abs() < 1e-4,
            "elemento {i}: len {} mas mede {d}",
            len[i]
        );
    }
}

/// ⚠️ **Um salto (`f`) faz nascer uma RAIZ nova, e é por isso que a invariante aguenta.**
///
/// Se o elemento depois do salto se pendurasse no anterior, `‖P − P[pai]‖` deixaria de ser
/// `len` — e o contrato do rig passaria a ser falso só nos documentos que usam `f`.
#[test]
fn a_jump_starts_a_new_root_instead_of_a_stretched_bone() {
    let s = draw("F f F", &setup());
    let parent = scal(&s, "parent");
    let roots = parent.iter().filter(|p| **p < 0.0).count();
    assert_eq!(roots, 2, "a raiz da planta e a que nasce depois do salto");
    // E o CONTROLE: sem o salto há uma raiz só.
    let one = draw("F F", &setup());
    assert_eq!(scal(&one, "parent").iter().filter(|p| **p < 0.0).count(), 1);
}

/// ⚠️ **Sem um `!` na gramática e com `Width = 1`, a coluna `size` é EXACTAMENTE a
/// identidade** — o nó não pode redimensionar a cena por existir.
#[test]
fn without_a_width_command_the_size_column_is_exactly_the_identity() {
    let s = draw("F[+F]F", &setup());
    for (i, sz) in vec2(&s, "size").iter().enumerate() {
        assert_eq!(
            sz.map(f32::to_bits),
            ph2d_nodegraph::attr::SIZE_IDENTITY.map(f32::to_bits),
            "elemento {i}: {sz:?}"
        );
    }
    // E o `!` de facto afina — senão o gate acima passaria com o comando morto.
    let thin = draw("F!F", &setup());
    let sz = vec2(&thin, "size");
    assert!(
        (sz[2][0] - 0.5).abs() < 1e-6,
        "o `!` multiplica pelo Width Scale, deu {}",
        sz[2][0]
    );
}

/// **`%` corta o resto do ramo**, e o que vem depois do `]` continua.
#[test]
fn the_cut_drops_the_rest_of_its_branch_and_only_that() {
    let full = draw("F[+FFF]F", &setup());
    let cut = draw("F[+F%FF]F", &setup());
    assert_eq!(full.count(), 6, "raiz + 5 F");
    assert_eq!(
        cut.count(),
        4,
        "raiz + o F da haste + 1 no ramo + o de depois"
    );
    // A haste depois do `]` sobreviveu: o último elemento pendura no primeiro F.
    let parent = scal(&cut, "parent");
    assert_eq!(*parent.last().unwrap(), 1.0);
}

/// **O tropismo curva o passo para a direcção declarada** — e a `0` nada se move.
///
/// A régua é o ângulo de mundo do último elemento: com a tartaruga a subir (90°) e o
/// tropismo a puxar para baixo (−90°), o produto vectorial é máximo e cada passo desvia.
#[test]
fn tropism_bends_the_walk_and_zero_leaves_it_straight() {
    let mut set = setup();
    set.root_angle = 0.0; // a andar para +x
    let straight = draw("FFFF", &set);
    let w0 = *scal(&straight, "wrot").last().unwrap();
    assert!(w0.abs() < 1e-4, "sem tropismo o rumo nao muda: {w0}");

    set.tropism = 10.0;
    set.tropism_angle = -90.0; // para baixo
    let bent = draw("FFFF", &set);
    let w1 = *scal(&bent, "wrot").last().unwrap();
    assert!(
        w1 < -10.0,
        "o rumo tem de cair na direccao do tropismo, deu {w1}"
    );
    let py = vec2(&bent, "P").last().unwrap()[1];
    assert!(py < 0.0, "e a ponta tem de estar abaixo do eixo, deu {py}");
}

/// **`J` pousa um elemento SEM segmento** — a marca de folha/flor, na posição do pai e com
/// um `sym` próprio, que é o que a torna seleccionável a jusante.
#[test]
fn a_leaf_mark_lands_on_its_parent_with_no_bone_and_its_own_symbol() {
    let s = draw("FJ", &setup());
    assert_eq!(s.count(), 3);
    let (p, len, sym) = (vec2(&s, "P"), scal(&s, "len"), scal(&s, "sym"));
    assert_eq!(len[2], 0.0, "uma marca nao tem osso");
    assert_eq!(p[2], p[1], "e fica onde o pai esta");
    assert_eq!(sym[2], f32::from(b'J'));
    assert_eq!(sym[1], f32::from(b'F'), "e o tronco diz que e' um F");
}

/// A profundidade de ramo é uma coluna, e conta os colchetes ABERTOS.
#[test]
fn the_depth_column_counts_open_brackets() {
    let s = draw("F[+F[+F]]F", &setup());
    let d = scal(&s, "depth");
    assert_eq!(d[1], 0.0, "a haste esta ao nivel do chao");
    assert_eq!(d[2], 1.0, "o primeiro ramo");
    assert_eq!(d[3], 2.0, "o ramo dentro do ramo");
    assert_eq!(d[4], 0.0, "e depois dos dois `]` voltamos ao chao");
}

/// Toda letra que não é comando é **muda**: estrutura a reescrita e não desenha nada.
#[test]
fn an_unknown_letter_draws_nothing() {
    let with = draw("FXYZF", &setup());
    let without = draw("FF", &setup());
    assert_eq!(with.count(), without.count());
    assert_eq!(vec2(&with, "P"), vec2(&without, "P"));
}

/// ⭐⭐ **A FORMA APONTA PARA ONDE O RAMO CRESCE** — o report do Enio de 2026-08-28.
///
/// O lowering desenha cada instância com o ângulo da coluna **`rot`**, e o contrato do `rig.*`
/// diz que `rot` é o ângulo LOCAL. Num galho a direito o local é ≈ `0` ⇒ a forma carimbada saía
/// sempre em pé, qualquer que fosse a direcção do ramo.
///
/// ⚠️ **O CONTROLE é o modo LOCAL**: ali o `rot` de uma haste a direito TEM de ser ≈ `0`, e é
/// isso que prova que os dois modos são de facto dois — e que o local continua a servir o rig.
#[test]
fn in_growth_mode_the_shape_faces_along_its_branch() {
    let mut world = setup();
    world.orient_world = true;
    world.angle = 40.0;
    // Uma haste a subir, depois um ramo a 40° para cada lado.
    let w = draw("FF[+FF][-FF]", &world);
    let (rot, wrot) = (scal(&w, "rot"), scal(&w, "wrot"));
    for (i, (r, wr)) in rot.iter().zip(&wrot).enumerate() {
        assert!(
            (r - wr).abs() < 1e-4,
            "no modo de crescimento o elemento {i} tem de apontar para o MUNDO: {r} vs {wr}"
        );
    }
    // E os ramos apontam de facto para lados diferentes — senão o gate acima seria vacuo.
    let mut sorted = rot.clone();
    sorted.sort_by(f32::total_cmp);
    assert!(
        sorted.last().unwrap() - sorted.first().unwrap() > 70.0,
        "os dois ramos abrem 80 graus entre si: {rot:?}"
    );

    // ⚠️ O CONTROLE: em LOCAL a mesma haste sai toda a zero (nada virou em relação ao pai).
    let mut local = world;
    local.orient_world = false;
    let l = scal(&draw("FF[+FF][-FF]", &local), "rot");
    let straight = l.iter().filter(|r| r.abs() < 1e-4).count();
    assert!(
        straight >= 4,
        "em local uma haste a direito tem de ter `rot` zero: {l:?}"
    );
}
