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

/// ⭐ (F14b, o dono: «não fixou afastado um do outro. a posição está variando») **A fila de ⇕ tem um
/// lugar FIXO na caixa visível**: o 1.º `RAIL_OUT` passos à direita da borda, a `h/4` do topo; os
/// seguintes mais um passo cada, à mesma altura — em toda forma, em TODO valor de cada campo (o
/// arrasto de outra alça nunca a move). Antes da 2.ª redacção a fila saltava um passo quando o ⊙
/// do *Radius* do balão passava perto.
#[test]
fn the_arrow_row_has_a_fixed_place_beside_the_visible_box() {
    let pitch = RAIL_PITCH * PARK;
    for &k in ALL_SHAPES {
        let ck = cooker(k, HALF);
        for v in super::f14b_tests::sweep(k) {
            let rails: Vec<[f64; 2]> = handles_of(k)
                .iter()
                .filter(|h| h.spot == Spot::Rail)
                .map(|h| handle_grip(k, h, &v, HALF, 1.0, PARK, &ck).expect("grip"))
                .collect();
            if rails.is_empty() {
                break;
            }
            let (lo, hi) = spots::visible_bbox(&ck(&v)).expect("caixa");
            let y = hi[1] - (hi[1] - lo[1]) * 0.25;
            for (n, p) in rails.iter().enumerate() {
                #[expect(clippy::cast_precision_loss, reason = "n pequeno")]
                let x = hi[0] + (spots::RAIL_OUT + n as f64) * pitch;
                assert!(
                    (p[0] - x).abs() < 1e-9 && (p[1] - y).abs() < 1e-9,
                    "{k:?} {v:?}: o ⇕ {n} em {p:?}, o lugar é {:?}",
                    [x, y]
                );
            }
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
    // A envergadura da ESTRELA (campo 5): a do polígono vive no ponto da aresta (F13).
    let k = ShapeKind::Star;
    let i = rail_of(k, 5);
    let v = k.defaults();
    assert_eq!(v[5], 0.0);
    for (rise, want) in [
        (0.25, 0.5),
        (-0.25, -0.5),
        (0.0004, 0.0),
        (0.123, 0.25),
        (3.0, 1.0),
    ] {
        let out = drag_rail(k, i, v, rise);
        assert!(
            (out[5] - want).abs() < 1e-9,
            "subir {rise}: {} ≠ {want}",
            out[5]
        );
        let others = (0..8).filter(|&j| j != 5).all(|j| out[j] == v[j]);
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
/// baixo dela). Controlo: as duas moradas (a `h/4` e, na caixa baixa, à meia altura) são visitadas.
#[test]
fn no_arrow_touches_a_box_handle_or_the_action_bar_from_20_to_400_px() {
    const HIT: f64 = 12.0;
    const SQ: f64 = 6.0;
    const GAP: f64 = 24.0;
    let park = SQ + HIT;
    let (mut a_um_quarto, mut a_meio) = (0, 0);
    for &k in ALL_SHAPES {
        if !handles_of(k).iter().any(|h| h.spot == Spot::Rail) {
            continue;
        }
        for alto in (20..=400).step_by(4) {
            let half = [100.0, f64::from(alto) * 0.5];
            let ck = cooker(k, half);
            let v = k.defaults();
            let Some((lo, hi)) = spots::visible_bbox(&ck(&v)) else {
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
                if (p[1] - (hi[1] - (hi[1] - lo[1]) * 0.25)).abs() < 1e-9 {
                    a_um_quarto += 1;
                } else if (p[1] - mid[1]).abs() < 1e-9 {
                    a_meio += 1;
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
        a_um_quarto > 0 && a_meio > 0,
        "{a_um_quarto} a h/4, {a_meio} à meia altura"
    );
}

/// ⭐ (F14b, o dono: «A seta e o ponto não devem ficar na mesma posição») **Nenhum ⇕ cai sobre o
/// grip de outra alça**: em toda forma com fila, cada campo varrido na faixa inteira, em três caixas
/// (a do smoke, 100 × 95 px; larga; alta), cada ⇕ fica a pelo menos um passo da fila de cada grip
/// das outras alças. Antes: 300 casos (o *Speech* a 5,8 px do ⊙ do *Radius*; o cubo isométrico a 0).
/// Controlo: o par mais perto existe e é medido.
#[test]
fn no_arrow_lands_on_another_handle() {
    let (mut hits, mut checked, mut nearest) = (Vec::new(), 0, f64::INFINITY);
    for &k in ALL_SHAPES {
        let hs = handles_of(k);
        if !hs.iter().any(|h| h.spot == Spot::Rail) {
            continue;
        }
        for half in [[50.0, 47.7], [100.0, 50.0], [100.0, 200.0]] {
            let ck = cooker(k, half);
            for i in shapes::shown_fields(k) {
                let f = &shapes::desc(k).fields[i];
                for s in 0..=8 {
                    let mut v = k.defaults();
                    v[i] = f.min + (f.max - f.min) * f64::from(s) / 8.0;
                    let g = |h: &HandleDesc| handle_grip(k, h, &v, half, 1.0, 18.0, &ck);
                    let rails: Vec<_> = hs
                        .iter()
                        .filter(|h| h.spot == Spot::Rail)
                        .filter_map(g)
                        .collect();
                    for o in hs.iter().filter(|h| h.spot != Spot::Rail).filter_map(g) {
                        for r in &rails {
                            checked += 1;
                            let d = (r[0] - o[0]).hypot(r[1] - o[1]);
                            nearest = nearest.min(d);
                            if d < RAIL_PITCH * 18.0 - 1e-9 {
                                hits.push(format!("{k:?} {half:?} f{i}={:.3}: {d:.1} px", v[i]));
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(
        hits.is_empty(),
        "{} ⇕ sobre um grip: {:?}",
        hits.len(),
        &hits[..hits.len().min(8)]
    );
    assert!(
        checked > 2000 && nearest.is_finite(),
        "controlo: {checked} pares"
    );
    eprintln!("[fila] {checked} pares, o mais perto a {nearest:.1} px");
}

/// ⭐ (F13, o dono: «tem dois controles para a mesma coisa: curve») **Um campo, um controlo**: o ⇕
/// é o idioma do campo SEM ponto no contorno — nenhum campo tem ⇕ e outra alça ao mesmo tempo.
#[test]
fn no_field_has_an_arrow_and_another_handle() {
    let mut dupes = Vec::new();
    for &k in ALL_SHAPES {
        let hs = handles_of(k);
        for a in hs.iter().filter(|h| h.spot == Spot::Rail) {
            for &f in a.fields {
                if hs
                    .iter()
                    .any(|h| h.spot != Spot::Rail && h.fields.contains(&f))
                {
                    dupes.push(format!("{k:?} campo {f}"));
                }
            }
        }
    }
    assert!(
        dupes.is_empty(),
        "dois controlos para o mesmo campo: {dupes:?}"
    );
}
