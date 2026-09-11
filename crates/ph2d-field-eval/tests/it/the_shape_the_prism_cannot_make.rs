//! ⭐⭐ **O TRIÂNGULO DE TRÊS VÉRTICES QUAISQUER** (W131) — o escaleno, que o prisma não faz.
//!
//! | gate | o defeito que ele apanha |
//! |---|---|
//! | `the_three_vertices_are_where_the_artist_put_them` | os vértices deixarem de ser os do painel |
//! | `the_winding_does_not_matter` | a peça **desaparecer** quando um vértice cruza o lado oposto |
//! | `the_field_is_one_lipschitz_however_thin_it_gets` | a peça **rasgar** (medido pela DEFINIÇÃO) |
//! | `the_fillet_never_leaves_the_piece` | o filete comer a peça inteira |

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn campo(p: Primitive) -> Field {
    Field::new(
        &FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0),
        )
        .expect("a peça"),
    )
}

fn tri(a: [f32; 2], b: [f32; 2], c: [f32; 2], round: f32) -> Primitive {
    Primitive::Triangle {
        a,
        b,
        c,
        half_height: 0.12,
        round,
        chamfer: 0.0,
    }
}

/// ⭐ **A peça está onde o artista pôs os três pontos** — dentro, na pele e fora.
#[test]
fn the_three_vertices_are_where_the_artist_put_them() {
    let (a, b, c) = ([-0.34_f32, -0.20_f32], [0.36, -0.10], [-0.05, 0.38]);
    let f = campo(tri(a, b, c, 0.0));
    // O baricentro está DENTRO.
    let g = [
        f64::from(a[0] + b[0] + c[0]) / 3.0,
        f64::from(a[1] + b[1] + c[1]) / 3.0,
    ];
    assert!(
        f.at(g[0], g[1], 0.0) < -1.0e-3,
        "o baricentro tinha de estar dentro e leu {:.5}",
        f.at(g[0], g[1], 0.0)
    );
    // Cada VÉRTICE está na pele (a menos do arredondamento do `f32`).
    for v in [a, b, c] {
        let d = f.at(f64::from(v[0]), f64::from(v[1]), 0.0);
        assert!(
            d.abs() < 2.0e-3,
            "o vértice {v:?} leu {d:.5}, e devia estar na pele"
        );
    }
    // ⚠️ **Um ponto do lado de FORA de cada aresta** — é o que separa esta forma de um disco.
    for (p, q) in [(a, b), (b, c), (c, a)] {
        let m = [f64::from(p[0] + q[0]) * 0.5, f64::from(p[1] + q[1]) * 0.5];
        let (dx, dy) = (f64::from(q[0] - p[0]), f64::from(q[1] - p[1]));
        let n = dx.hypot(dy);
        // a normal exterior de `p → q` numa volta anti-horária
        let fora = [m[0] + 0.05 * dy / n, m[1] - 0.05 * dx / n];
        let dentro = [m[0] - 0.05 * dy / n, m[1] + 0.05 * dx / n];
        let (a1, a2) = (f.at(fora[0], fora[1], 0.0), f.at(dentro[0], dentro[1], 0.0));
        assert!(
            a1.max(a2) > 1.0e-3 && a1.min(a2) < -1.0e-3,
            "a aresta {p:?}→{q:?} tinha de separar dentro de fora, e leu {a1:.4}/{a2:.4}"
        );
    }
}

/// ⭐⭐⭐ **A ORDEM DOS VÉRTICES NÃO IMPORTA** — e é o gate que impede a peça de desaparecer.
///
/// ⚠️ **O `half_plane` é negativo à ESQUERDA**, logo os três só apontam para dentro numa volta
/// anti-horária. *Um artista que arrasta um vértice para o outro lado do lado oposto inverte a volta
/// sem saber* — e sem a correcção no construtor a peça some naquele instante.
#[test]
fn the_winding_does_not_matter() {
    let (a, b, c) = ([-0.34_f32, -0.20_f32], [0.36, -0.10], [-0.05, 0.38]);
    let horaria = campo(tri(a, c, b, 0.0));
    let anti = campo(tri(a, b, c, 0.0));
    let mut pior = 0.0_f64;
    for i in 0..30 {
        for j in 0..30 {
            for k in 0..30 {
                let at = |t: usize| -0.6 + 1.2 * (t as f64 + 0.5) / 30.0;
                let (x, y, z) = (at(i), at(j), at(k));
                pior = pior.max((horaria.at(x, y, z) - anti.at(x, y, z)).abs());
            }
        }
    }
    assert!(
        pior < 1.0e-9,
        "as duas voltas tinham de dar o MESMO campo e diferem em {pior:.3e}"
    );
}

/// ⭐⭐⭐ **O CAMPO É 1-LIPSCHITZ POR MAIS FINA QUE A PEÇA FIQUE** — medido pela DEFINIÇÃO.
///
/// ⛔⛔ **A diferença central não serve aqui, e está medido:** ela lê o *acorde* de um vinco e
/// acusa `1,10`–`1,35` sobre um campo cuja constante é `1,000`. ⚠️ *Uma régua que sobre-lê num vinco
/// convexo transforma toda quina aguda num defeito.*
///
/// ⛔ E foi este gate que apanhou a versão que ship-aria partida: o `intersection_joint_n` com
/// `chamfer == 0` **infla** (a lei da casa), e a peça lia `1,94`.
#[test]
fn the_field_is_one_lipschitz_however_thin_it_gets() {
    let mut sem = 987_654_321_u64;
    let mut rnd = move || {
        sem = sem.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        ((sem >> 33) as f64 / f64::from(u32::MAX)) * 2.4 - 1.2
    };
    let mut pior = 0.0_f64;
    let mut onde = String::new();
    for (nome, a, b, c) in [
        (
            "escaleno",
            [-0.34_f32, -0.20_f32],
            [0.36_f32, -0.10_f32],
            [-0.05_f32, 0.38_f32],
        ),
        ("agudo", [-0.03, -0.35], [0.03, -0.35], [0.0, 0.35]),
        ("obtuso", [-0.35, 0.0], [0.35, 0.0], [0.02, 0.05]),
        ("lasca", [-0.40, 0.0], [0.40, 0.0], [0.10, 0.004]),
    ] {
        for round in [0.0_f32, 0.5, 0.999] {
            let mut p = tri(a, b, c, 0.0);
            let limite = ph2d_field::round_limit(&p).unwrap_or(0.0);
            p = tri(a, b, c, limite * round);
            let f = campo(p);
            let mut aqui = 0.0_f64;
            for _ in 0..60_000 {
                let u = [rnd(), rnd(), rnd()];
                let v = [rnd(), rnd(), rnd()];
                let d =
                    ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt();
                if d < 1.0e-6 {
                    continue;
                }
                aqui = aqui.max((f.at(u[0], u[1], u[2]) - f.at(v[0], v[1], v[2])).abs() / d);
            }
            if aqui > pior {
                pior = aqui;
                onde = format!("{nome} com o filete a {:.0} % do tecto", round * 100.0);
            }
        }
    }
    assert!(
        pior <= 1.02,
        "o campo tem constante {pior:.4} em «{onde}» — a marcha atravessa a superfície"
    );
}

/// ⭐ **O MAIOR filete que o documento aceita ainda deixa peça** — e o tecto é o INRAIO.
#[test]
fn the_fillet_never_leaves_the_piece() {
    let (a, b, c) = ([-0.34_f32, -0.20_f32], [0.36, -0.10], [-0.05, 0.38]);
    let base = tri(a, b, c, 0.0);
    let limite = ph2d_field::round_limit(&base).expect("o triângulo tem parede");
    let esperado = ph2d_field_eval::ops_triangle::inradius(
        [f64::from(a[0]), f64::from(a[1])],
        [f64::from(b[0]), f64::from(b[1])],
        [f64::from(c[0]), f64::from(c[1])],
    )
    .min(0.12);
    assert!(
        (f64::from(limite) - esperado).abs() < 1.0e-6,
        "o tecto do filete tinha de ser o INRAIO ({esperado:.5}) e é {limite:.5}"
    );
    let f = campo(tri(a, b, c, limite * 0.999));
    let g = [
        f64::from(a[0] + b[0] + c[0]) / 3.0,
        f64::from(a[1] + b[1] + c[1]) / 3.0,
    ];
    assert!(
        f.at(g[0], g[1], 0.0) < 0.0,
        "com o maior filete ainda tinha de sobrar peça no baricentro"
    );
}
