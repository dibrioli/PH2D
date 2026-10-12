//! Gates da F12 — a FILA de ⇕ na lateral (o dono, 12/10: «setas lado a lado … os modais
//! retirados de todas as formas»). Irmão pelo tecto de LOC.

use super::tests::cooker;
use super::*;
use ph2d_vec_scene::ALL_SHAPES;

const HALF: [f64; 2] = [2.0, 1.0];
const PARK: f64 = 0.18;

fn rail_of(kind: ShapeKind, field: usize) -> usize {
    handles_of(kind)
        .iter()
        .position(|h| h.spot == Spot::Rail && h.fields == [field])
        .unwrap_or_else(|| panic!("{kind:?}: sem ⇕ para o campo {field}"))
}

/// ⭐ Os ⇕ ficam UM AO LADO DO OUTRO: o 1.º na borda direita da caixa visível, os seguintes para
/// FORA, à mesma altura, a `RAIL_PITCH · park_at` — em todas as formas.
#[test]
fn the_arrows_stand_side_by_side_outwards() {
    for &k in ALL_SHAPES {
        let ck = cooker(k, HALF);
        let v = k.defaults();
        let rails: Vec<[f64; 2]> = handles_of(k)
            .iter()
            .filter(|h| h.spot == Spot::Rail)
            .map(|h| handle_grip(k, h, &v, HALF, 1.0, PARK, &ck).expect("grip"))
            .collect();
        let Some(first) = rails.first() else { continue };
        let (_, hi) = spots::curve_bbox(&ck(&v)).expect("caixa");
        assert!((first[0] - hi[0]).abs() < 1e-9, "{k:?}: o 1.º na borda");
        for (n, p) in rails.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "n pequeno")]
            let want = first[0] + n as f64 * RAIL_PITCH * PARK;
            assert!((p[0] - want).abs() < 1e-9, "{k:?}: o ⇕ {n} em x {p:?}");
            assert!(
                (p[1] - first[1]).abs() < 1e-12,
                "{k:?}: o ⇕ {n} fora da fila"
            );
        }
    }
}

fn drag_rail(kind: ShapeKind, i: usize, v: ShapeValues, rise: f64) -> ShapeValues {
    let ck = cooker(kind, HALF);
    let q = handle_grip(kind, &handles_of(kind)[i], &v, HALF, 1.0, PARK, &ck).expect("grip");
    let mut d = HandleDrag::start(i, v, q, q, HALF);
    drag(
        kind,
        &mut d,
        [q[0] + 0.3, q[1] + rise * 2.0 * HALF[1]],
        Mods::default(),
        HALF,
        1.0,
        &ck,
    )
}

/// ⭐ Um campo CONTÍNUO no ⇕ anda a faixa inteira na altura da caixa, no passo do campo; o
/// arrasto horizontal não conta.
#[test]
fn a_continuous_arrow_runs_the_whole_range_in_the_box_height() {
    let k = ShapeKind::Polygon;
    let i = rail_of(k, 3);
    let v = k.defaults();
    assert_eq!(v[3], 0.0);
    for (rise, want) in [
        (0.25, 0.5),
        (-0.25, -0.5),
        (0.0004, 0.0),
        (0.123, 0.25),
        (3.0, 1.0),
    ] {
        let out = drag_rail(k, i, v, rise);
        assert!(
            (out[3] - want).abs() < 1e-9,
            "subir {rise}: {} ≠ {want}",
            out[3]
        );
        let others = (0..8).filter(|&j| j != 3).all(|j| out[j] == v[j]);
        assert!(others, "só o campo do ⇕ muda");
    }
}

/// Uma ESCOLHA no ⇕ anda opção a opção (um degrau por 1/24 da caixa) e satura nas pontas.
#[test]
fn a_choice_arrow_steps_through_the_options() {
    let k = ShapeKind::RoundRect;
    let i = rail_of(k, 5);
    let v = k.defaults();
    for (steps, want) in [(1.0, 1.0), (2.0, 2.0), (10.0, 3.0), (-1.0, 0.0)] {
        let out = drag_rail(k, i, v, steps / 24.0);
        assert_eq!(out[5], want, "{steps} degraus");
    }
}
