//! Os gates dos dois raios de colisão (doc 109) — **medidos no contorno que o produto constrói**.
//!
//! ⚠️ A régua passa pelo `build_shape_path` do produto, e não por uma elipse montada aqui: o
//! defeito que estes gates existem para apanhar é o raio discordar do que se DESENHA.

use super::measure;
use ph2d_node_motion_shape::{ShapeKind, ShapeParams};

/// A forma em raio 1, com todos os outros params nos defaults do manifesto.
fn raios(kind: ShapeKind) -> [f32; 2] {
    let (mut p, _) = ShapeParams::read_unit(super::super::manifest_default);
    p.kind = kind;
    measure(&super::super::build_shape_path(&p))
}

const TOL: f32 = 1e-3;

/// ⭐ **Um círculo tem UM raio** — à volta e por dentro são o mesmo número, e é `1`.
#[test]
fn a_circle_is_one_radius_around_and_inside() {
    let [around, inside] = raios(ShapeKind::Circle);
    assert!((around - 1.0).abs() < TOL, "around {around}");
    assert!((inside - 1.0).abs() < TOL, "inside {inside}");
}

/// ⭐ **Um quadrado de meio-lado 1 é `√2` à volta e `1` por dentro** — os dois números que tornam
/// o ajuste uma escolha a sério (a diagonal contra o lado).
#[test]
fn a_square_is_root_two_around_and_one_inside() {
    let [around, inside] = raios(ShapeKind::Square);
    assert!((around - std::f32::consts::SQRT_2).abs() < TOL, "around {around}");
    assert!((inside - 1.0).abs() < TOL, "inside {inside}");
}

/// ⭐⭐ **A declaração atravessa a CADEIA DO PRODUTO**: o shell publica, o nó escolhe, o duplicador
/// carimba — e cada cópia chega com o colisor da forma. Desligado, nenhuma cópia o tem.
///
/// ⚠️ **As duas metades num gate só**, porque *«a coluna existe»* passa com o `Collide` ignorado
/// e *«a coluna não existe»* passa com o nó desligado da cadeia.
#[test]
fn the_declared_collider_rides_the_duplicator_to_every_copy() {
    use crate::motion_state::MotionState;
    use ph2d_nodegraph::attr::{COLLIDER_COLUMN, Column};
    use ph2d_nodegraph::graph::Edge;

    let copias = |collide: bool| -> Option<Vec<f32>> {
        let mut state = MotionState::new();
        let g = &mut state.doc.graph;
        let forma = g.add_node("source.shape");
        g.set_param(forma, "kind", ShapeKind::Square.index() as f32);
        if collide {
            g.set_param(forma, ph2d_node_motion_shape::param::COLLIDE, 1.0);
        }
        let grade = g.add_node("motion.grid");
        g.set_param(grade, "rows", 2.0);
        g.set_param(grade, "cols", 2.0);
        let dup = g.add_node("motion.duplicator");
        for (from, to) in [((forma, 0), (dup, 0)), ((grade, 0), (dup, 1))] {
            g.connect(Edge {
                from,
                to,
                delayed: false,
            })
            .expect("liga");
        }
        super::super::publish(&mut state, 0.0);
        let out = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, dup, 0.0)
            .expect("coze");
        let s = out[0].as_stream();
        assert_eq!(s.count(), 4, "uma forma em quatro pontos");
        match s.get(COLLIDER_COLUMN) {
            Some(Column::Scalar(v)) => Some(v.clone()),
            _ => None,
        }
    };

    let ligado = copias(true).expect("com Collide as copias trazem o colisor");
    assert_eq!(ligado.len(), 4);
    for r in &ligado {
        assert!(
            (r - std::f32::consts::SQRT_2).abs() < TOL,
            "o quadrado declara o raio a volta (Around e' o default): {r}"
        );
    }
    assert!(copias(false).is_none(), "sem Collide nenhuma copia declara");
}

/// ⭐ **Uma estrela reserva as PONTAS à volta e o MIOLO por dentro** — e o miolo é mais que zero.
#[test]
fn a_star_reserves_its_points_around_and_its_core_inside() {
    let [around, inside] = raios(ShapeKind::Star);
    assert!((around - 1.0).abs() < TOL, "as pontas estao em raio 1: {around}");
    assert!(
        inside > 0.1 && inside < around * 0.8,
        "o miolo e' bem mais pequeno que as pontas: inside {inside}, around {around}"
    );
}
