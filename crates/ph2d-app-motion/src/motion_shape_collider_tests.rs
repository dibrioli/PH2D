//! Os gates da caixa envolvente de colisão (doc 109 §5) — **medida no contorno que o produto
//! constrói**.
//!
//! ⚠️ A régua passa pelo `build_shape_path` do produto, e não por uma elipse montada aqui: o
//! defeito que estes gates existem para apanhar é o colisor discordar do que se DESENHA.

use super::{ColliderFit, measure};
use ph2d_node_motion_shape::{ShapeKind, ShapeParams};

/// A forma em raio 1, com todos os outros params nos defaults do manifesto e `ajusta` por cima.
fn caixa_de(kind: ShapeKind, ajusta: impl FnOnce(&mut ShapeParams)) -> ColliderFit {
    let (mut p, _) = ShapeParams::read_unit(super::super::manifest_default);
    p.kind = kind;
    ajusta(&mut p);
    measure(&super::super::build_shape_path(&p))
}

const TOL: f32 = 1e-3;

fn perto(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < TOL && (a[1] - b[1]).abs() < TOL
}

/// ⭐ **Um círculo cabe na caixa de meio-lado `1`, centrada na origem.**
#[test]
fn a_circle_fits_the_unit_box_centred_on_its_origin() {
    let f = caixa_de(ShapeKind::Circle, |_| {});
    assert!(
        perto(f.half, [1.0, 1.0]) && perto(f.center, [0.0, 0.0]),
        "{f:?}"
    );
}

/// ⭐⭐ **Um quadrado de meio-lado 1 declara O PRÓPRIO QUADRADO** — e não o círculo `√2` à volta dele,
/// que era o defeito do report.
#[test]
fn a_square_fits_itself_not_the_circle_around_it() {
    let f = caixa_de(ShapeKind::Square, |_| {});
    assert!(
        perto(f.half, [1.0, 1.0]) && perto(f.center, [0.0, 0.0]),
        "{f:?}"
    );
}

/// ⭐⭐ **Um retângulo declara as PROPORÇÕES dele** — a caixa segue o `aspect`.
#[test]
fn a_rectangle_fits_its_own_proportions() {
    let f = caixa_de(ShapeKind::Rectangle, |p| p.aspect = 0.5);
    assert!(perto(f.half, [1.0, 0.5]), "{f:?}");
}

/// ⭐⭐ **Uma estrela de CINCO pontas NÃO está centrada na origem dela** — a ponta de um lado não tem
/// par do outro, e é o centro medido que o solver usa.
///
/// ⚠️ **Cinco, e não o default:** a estrela do manifesto tem SEIS pontas, que é simétrica nos dois
/// eixos e dá o centro na origem (medido: `[1e-16, 0]`) — a fixtura sem o fenómeno leria a porta
/// como inútil.
#[test]
fn a_star_is_centred_between_its_extremes_not_on_its_origin() {
    let f = caixa_de(ShapeKind::Star, |p| p.sides = 5);
    let fora = f.center[0].hypot(f.center[1]);
    assert!(
        fora > 0.05 && fora < 0.15,
        "o meio da caixa sai da origem: {f:?}"
    );
    assert!(
        f.center[0].abs() < TOL || f.center[1].abs() < TOL,
        "e fica sobre o eixo de simetria: {f:?}"
    );
    assert!(
        f.half[0].max(f.half[1]) <= 1.0 + TOL,
        "as pontas estao em raio 1: {f:?}"
    );
}

/// ⭐⭐ **A declaração atravessa a CADEIA DO PRODUTO**: o shell publica, o nó declara, o duplicador
/// carimba — e cada cópia chega com a CAIXA da forma. Desligado, nenhuma cópia a tem.
///
/// ⚠️ **As duas metades num gate só**, porque *«a coluna existe»* passa com o `Collide` ignorado
/// e *«a coluna não existe»* passa com o nó desligado da cadeia.
#[test]
fn the_declared_box_rides_the_duplicator_to_every_copy() {
    use crate::motion_state::MotionState;
    use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, Column};
    use ph2d_nodegraph::graph::Edge;

    let copias = |collide: bool| -> (Option<Vec<[f32; 2]>>, bool) {
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
        let caixa = match s.get(COLLIDER_BOX_COLUMN) {
            Some(Column::Vec2(v)) => Some(v.clone()),
            _ => None,
        };
        (caixa, s.get(COLLIDER_COLUMN).is_some())
    };

    let (ligado, raio) = copias(true);
    let ligado = ligado.expect("com Collide as copias trazem a caixa");
    assert!(!raio, "Box (o default) nao declara raio");
    assert_eq!(ligado.len(), 4);
    for m in &ligado {
        assert!(
            perto(*m, [1.0, 1.0]),
            "o quadrado declara o proprio quadrado: {m:?}"
        );
    }
    assert_eq!(
        copias(false),
        (None, false),
        "sem Collide nenhuma copia declara"
    );
}
