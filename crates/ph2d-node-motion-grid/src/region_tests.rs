//! Os gates da **FORMA** desta grade (doc 89, folha 01).
//!
//! ⚠️ **Aqui a forma RECORTA e a contagem CAI** — ao contrário do `motion.scatter`, que
//! empacota. Uma rede não se dobra para caber num círculo, e um gate que exigisse a
//! contagem intacta estaria a pedir a lei do outro nó.

use super::*;
use ph2d_motion_region::{Region, SHAPE_CIRCLE, SHAPE_RING};

fn region(shape: f32, inner: f32) -> Region {
    grid_region(9, 9, 1.0, 1.0, shape, inner)
}

fn built() -> Vec<[f32; 2]> {
    build_grid(9, 9, 1.0, 1.0, RECOMMENDED_MAX_ELEMENTS)
}

/// ⭐ **O RETÂNGULO NÃO PERDE UM PONTO NEM MOVE UM BIT** — o default é a grade de
/// sempre, e o `carve` sai por ramo antes de varrer o que quer que seja.
#[test]
fn the_default_shape_keeps_the_whole_lattice_bit_for_bit() {
    let raw = built();
    let cut = carve(built(), &region(0.0, 0.9));
    assert_eq!(cut.len(), 81, "a grade inteira");
    for (i, (p, q)) in cut.iter().zip(&raw).enumerate() {
        assert_eq!(p.map(f32::to_bits), q.map(f32::to_bits), "ponto {i}");
    }
}

/// ⭐ **O CÍRCULO RECORTA, e a fração guardada CONVERGE para `π/4`** à medida que a
/// rede afina.
///
/// ⚠️ **A primeira versão deste gate exigia `π/4` numa rede de 9×9 e mediu `0,605`** —
/// e o produto estava certo. `π/4` é a razão de ÁREAS, e uma rede pequena é um conjunto
/// DISCRETO: contar pontos de rede dentro de um disco é o problema do círculo de Gauss,
/// `N(r) = πr² + O(r)`, e a `81` o termo de bordo ainda é um quinto do total. *Uma
/// régua do contínuo aplicada a um conjunto discreto acusa código correcto* — a
/// afirmação que se pode fazer é sobre a TAXA, e é essa que se mede.
#[test]
fn the_kept_fraction_converges_to_the_area_ratio_as_the_lattice_thins() {
    let kept = |side: usize| -> f32 {
        let gap = 8.0 / (side as f32 - 1.0);
        let pts = build_grid(side, side, gap, gap, RECOMMENDED_MAX_ELEMENTS);
        let total = pts.len() as f32;
        carve(
            pts,
            &grid_region(side, side, gap, gap, SHAPE_CIRCLE as f32, 0.0),
        )
        .len() as f32
            / total
    };
    let ladder: Vec<(usize, f32)> = [9, 21, 51, 101].map(|s| (s, kept(s))).into();
    for (s, f) in &ladder {
        println!("rede {s}x{s}: guardou {f:.4}");
    }
    let pi4 = std::f32::consts::FRAC_PI_4;
    // O erro TEM de encolher — é isso que «converge» quer dizer.
    let err: Vec<f32> = ladder.iter().map(|(_, f)| (f - pi4).abs()).collect();
    assert!(
        err[0] > err[3] * 3.0,
        "o erro tinha de encolher pelo menos 3x de 9x9 a 101x101: {err:?}"
    );
    assert!(
        err[3] < 0.02,
        "a 101x101 ja' tinha de estar colado em pi/4: {:.4}",
        ladder[3].1
    );
}

/// E o que sobra está mesmo dentro — com os quatro cantos fora, que é o que
/// «recortar» quer dizer.
#[test]
fn the_circle_keeps_only_what_is_inside_it() {
    let cut = carve(built(), &region(SHAPE_CIRCLE as f32, 0.0));
    assert!(cut.len() < 81 && !cut.is_empty(), "cortou: {}", cut.len());
    let circle = region(SHAPE_CIRCLE as f32, 0.0);
    for p in &cut {
        assert!(circle.contains(*p), "sobrou um de fora: {p:?}");
    }
    assert!(
        !cut.iter().any(|p| p[0].abs() > 3.9 && p[1].abs() > 3.9),
        "um canto sobreviveu ao disco"
    );
}

/// ⭐⭐ **UMA CASCA É UM ANEL** — a afirmação que apagou o param `fill` do C4D.
///
/// `inner` a subir encolhe a banda até só sobrar a moldura de fora, e valores
/// intermédios dão espessuras que o par *Solid / Shell* não sabe exprimir.
#[test]
fn the_shell_of_the_c4d_grid_is_this_rings_big_hole() {
    let solid = carve(built(), &region(SHAPE_CIRCLE as f32, 0.0)).len();
    let thick = carve(built(), &region(SHAPE_RING as f32, 0.5)).len();
    let shell = carve(built(), &region(SHAPE_RING as f32, 0.85)).len();
    assert!(
        solid > thick && thick > shell && shell > 0,
        "a banda encolhe monotonamente: {solid} > {thick} > {shell} > 0"
    );
    // A casca é MESMO só a moldura: nada dela fica no miolo.
    let ring = region(SHAPE_RING as f32, 0.85);
    for p in carve(built(), &ring) {
        assert!(
            ring.radial(p) >= 0.85 - 1e-3,
            "a casca vazou para dentro: {p:?}"
        );
    }
}

/// ⚠️ **`Index` e `Count` são RENUMERADOS depois do corte** — eles são a identidade que
/// toda paleta e toda rampa a jusante endereçam, e um `Index` com buracos faria a cor
/// saltar exactamente onde a forma mordeu.
#[test]
fn the_identity_columns_are_renumbered_after_the_cut() {
    let reg = {
        let mut r = NodeRegistry::new();
        register(&mut r).expect("registra");
        r
    };
    let mut g = ph2d_nodegraph::graph::Graph::new();
    let n = g.add_node("motion.grid");
    g.set_param(n, "rows", 9.0);
    g.set_param(n, "cols", 9.0);
    g.set_param(n, ph2d_motion_region::SHAPE, SHAPE_CIRCLE as f32);
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let out = cook.cook(&g, &reg, n, 0.0).expect("coze");
    let s = out[0].as_stream();
    let Some(Column::Scalar(idx)) = s.get("Index") else {
        panic!("Index")
    };
    let Some(Column::Scalar(cnt)) = s.get("Count") else {
        panic!("Count")
    };
    assert!(idx.len() < 81, "CONTROLE: o disco de facto cortou");
    for (i, v) in idx.iter().enumerate() {
        assert_eq!(*v, i as f32, "Index com buraco em {i}");
    }
    for v in cnt {
        assert_eq!(*v, idx.len() as f32, "Count tem de ser o que SOBROU");
    }
}

/// ⛔ **A FRONTEIRA DO DEVICE está nomeada e é verificável**: só o retângulo tem
/// `count_law`, porque contar pontos de uma rede num disco não tem forma fechada.
#[test]
fn only_the_rectangle_reaches_the_device() {
    let applicable = GPU_KERNEL.applicable.expect("a fronteira e' declarada");
    let with = |v: f32| {
        applicable(&move |name: &str| {
            if name == ph2d_motion_region::SHAPE {
                v
            } else {
                0.0
            }
        })
    };
    assert!(with(0.0), "o retangulo continua no device");
    assert!(!with(SHAPE_CIRCLE as f32), "o disco cai para a CPU");
    assert!(!with(SHAPE_RING as f32), "o anel cai para a CPU");
}

/// A extensão da região é a dos PONTOS: o círculo encosta na coluna de fora.
#[test]
fn the_inscribed_circle_touches_the_outer_row() {
    let circle = region(SHAPE_CIRCLE as f32, 0.0);
    // Com 9 colunas e gap 1 a extensão é 8, logo o meio-eixo é 4 — e o ponto (4,0) é
    // o do meio da coluna de fora.
    assert!(circle.contains([4.0, 0.0]), "o meio da coluna de fora caiu");
    assert!(
        circle.contains([0.0, 4.0]),
        "o meio da fileira de fora caiu"
    );
    assert!(!circle.contains([4.0, 4.0]), "mas a esquina nao");
}
