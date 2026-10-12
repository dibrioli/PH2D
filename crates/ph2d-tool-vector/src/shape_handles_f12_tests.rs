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

/// ⭐ (F13 C) **O ⇕ do *Viewed* dos isométricos: subir = «From above»** (o índice 0) — só ele
/// inverte; o resto da fila sobe o valor (controlo: o tipo do canto, acima).
#[test]
fn the_viewed_arrow_goes_up_to_from_above() {
    for (k, f) in [
        (ShapeKind::IsoCube, 2),
        (ShapeKind::IsoCone, 1),
        (ShapeKind::IsoPyramid, 2),
    ] {
        let i = rail_of(k, f);
        let mut below = k.defaults();
        below[f] = 1.0;
        assert_eq!(drag_rail(k, i, below, 1.0 / 24.0)[f], 0.0, "{k:?}: subir");
        let mut above = k.defaults();
        above[f] = 0.0;
        assert_eq!(drag_rail(k, i, above, -1.0 / 24.0)[f], 1.0, "{k:?}: descer");
        assert_eq!(drag_rail(k, i, above, 1.0 / 24.0)[f], 0.0, "{k:?}: satura");
        let downs = handles_of(k).iter().filter(|h| h.down).count();
        assert_eq!(downs, 1, "{k:?}: só o Viewed inverte");
    }
    for &k in ALL_SHAPES {
        if !matches!(
            k,
            ShapeKind::IsoCube | ShapeKind::IsoCone | ShapeKind::IsoPyramid
        ) {
            assert!(handles_of(k).iter().all(|h| !h.down), "{k:?} inverteu");
        }
    }
}

/// ⭐ (F13 D) **Nenhum ⇕ toca uma alça da caixa nem a ④, de 20 a 400 px de altura**: em toda forma
/// com fila, o disco de acerto de cada ⇕ (`HIT` = 12 px) fica fora dos 8 quadrados de escala (lado
/// 12 px) e dentro da banda vertical da caixa ± a folga da ④ (24 px, que flutua por cima ou por
/// baixo dela). Controlo: as duas moradas (na borda a `h/4` e por fora à meia altura) são visitadas.
#[test]
fn no_arrow_touches_a_box_handle_or_the_action_bar_from_20_to_400_px() {
    const HIT: f64 = 12.0;
    const SQ: f64 = 6.0;
    const GAP: f64 = 24.0;
    let park = SQ + HIT;
    let (mut na_borda, mut por_fora) = (0, 0);
    for &k in ALL_SHAPES {
        if !handles_of(k).iter().any(|h| h.spot == Spot::Rail) {
            continue;
        }
        for alto in (20..=400).step_by(4) {
            let half = [100.0, f64::from(alto) * 0.5];
            let ck = cooker(k, half);
            let v = k.defaults();
            let Some((lo, hi)) = spots::curve_bbox(&ck(&v)) else {
                continue;
            };
            let mid = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
            let alcas: Vec<[f64; 2]> = [lo[0], mid[0], hi[0]]
                .iter()
                .flat_map(|&x| [lo[1], mid[1], hi[1]].map(|y| [x, y]))
                .filter(|p| *p != mid)
                .collect();
            for h in handles_of(k).iter().filter(|h| h.spot == Spot::Rail) {
                let p = handle_grip(k, h, &v, half, 1.0, park, &ck).expect("grip");
                if (p[0] - hi[0]).abs() < 1e-9 {
                    na_borda += 1;
                } else if (p[1] - mid[1]).abs() < 1e-9 {
                    por_fora += 1;
                }
                for a in &alcas {
                    let dx = ((p[0] - a[0]).abs() - SQ).max(0.0);
                    let dy = ((p[1] - a[1]).abs() - SQ).max(0.0);
                    assert!(
                        dx.hypot(dy) >= HIT - 1e-9,
                        "{k:?} a {alto} px: o ⇕ em {p:?} toca a alça em {a:?}"
                    );
                }
                assert!(
                    p[1] - HIT >= lo[1] - GAP && p[1] + HIT <= hi[1] + GAP,
                    "{k:?} a {alto} px: o ⇕ em {p:?} entra na banda da ④"
                );
            }
        }
    }
    assert!(
        na_borda > 0 && por_fora > 0,
        "{na_borda} na borda, {por_fora} por fora"
    );
}
