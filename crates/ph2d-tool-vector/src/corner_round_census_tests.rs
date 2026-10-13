//! ⭐ (F14 B) **O censo das quinas**: toda forma do catálogo que mostra uma quina viva — no
//! default, em qualquer valor de qualquer campo, em qualquer proporção da caixa — tem quem a
//! arredonde: o «Corner round» (a porta única, `ph2d_vec_scene::round_shape_corners`) ou um
//! arredondador próprio que a apaga; as que ficam de fora estão em [`OUTSIDE`], com a medida.
//! Doc: `docs/UI_New_and_Simple/vetor_nova_ui/24_arredondar_as_quinas.md`.

use crate::shapes::{FieldUnit, SHAPES, ShapeDesc};
use ph2d_vec_scene::corner_live::corner_at;
use ph2d_vec_scene::{ShapeKind, ShapeValues, VecPath, cook};

const BOX: f64 = 4.0;
const PX: f64 = 0.01;

/// As formas com arredondador PRÓPRIO: o campo (índice) e o valor (unidade da UI) que, sozinho,
/// apaga toda quina viva do default.
const OWN: &[(ShapeKind, &[usize], f64, &str)] = &[
    (
        ShapeKind::RoundRect,
        &[0],
        40.0,
        "o Radius (os quatro cantos)",
    ),
    (ShapeKind::Polygon, &[1], 40.0, "o Radius do canto"),
    (
        ShapeKind::Star,
        &[2, 3],
        20.0,
        "o Tip round e o Notch round (8/8 campos)",
    ),
    (ShapeKind::Cloud, &[2], 0.03, "o Smooth (o filete do vale)"),
];

/// As formas que ficam DE FORA — forma · porquê · a medida (13/10). O censo abaixo prova que
/// nenhuma mostra quina visível em variante nenhuma.
const OUTSIDE: &[(ShapeKind, &str)] = &[
    (ShapeKind::Line, "aberta, dois pontos: não há quina"),
    (
        ShapeKind::Arc,
        "aberto e liso (os vértices do arco: seno 0)",
    ),
    (
        ShapeKind::Spiral,
        "aberta e lisa; com Expansion ≥ 4,17 o centro tem troços de ~1e-9 (recuo máximo 2e-9): ruído",
    ),
    (
        ShapeKind::Pill,
        "as pontas são semicírculos tangentes às rectas",
    ),
    (
        ShapeKind::Cylinder,
        "as bordas são elipses tangentes às paredes (seno ≤ 2e-15)",
    ),
    (
        ShapeKind::Junction,
        "círculo inteiro; a cruz é desenho sobre a face",
    ),
    (
        ShapeKind::Brace,
        "aberta; o bico do meio é uma cúspide de ângulo zero (seno 0, cosseno 1), o resto liso",
    ),
    (
        ShapeKind::Moon,
        "os chifres são cúspides de ângulo zero (seno 0, cosseno 1 em toda a Phase): entre duas curvas tangentes não cabe arco",
    ),
];

/// Toda forma está numa classe só: o «Corner round», um arredondador próprio, ou de fora.
#[test]
fn every_shape_is_in_exactly_one_class() {
    for d in SHAPES {
        let n = usize::from(d.kind.corner_round_field().is_some())
            + OWN.iter().filter(|o| o.0 == d.kind).count()
            + OUTSIDE.iter().filter(|o| o.0 == d.kind).count();
        assert_eq!(n, 1, "{}", d.label);
    }
}

/// Uma quina conta quando um raio a pode mexer de forma visível: o recuo máximo dela passa de
/// `1e-6` da caixa (`4e-4` px numa caixa de 400 px). Medido: a espiral com *Expansion* ≥ 4,17
/// encolhe as voltas de dentro a troços de `~1e-9` (recuo máximo `2e-9`) cujas alças, abaixo do
/// `EPS` do motor, caem no ponto vizinho — ruído nanométrico, não uma quina que se veja.
const VISIBLE: f64 = 1e-6 * BOX;

fn live_corners(p: &VecPath) -> usize {
    let mut c = vec![(p.verts.as_slice(), p.closed)];
    c.extend(
        p.subpaths
            .iter()
            .filter(|s| s.closed)
            .map(|s| (s.verts.as_slice(), true)),
    );
    c.into_iter()
        .map(|(v, closed)| {
            (0..v.len())
                .filter(|&i| corner_at(v, closed, i).is_some_and(|f| f.max_setback > VISIBLE))
                .count()
        })
        .sum()
}

fn ui(d: &ShapeDesc, i: usize, x: f64) -> f64 {
    if d.fields[i].unit == FieldUnit::Px {
        x * PX
    } else {
        x
    }
}

/// Toda variante do censo: o default, cada campo na faixa dele, e a proporção da caixa.
fn variants(d: &ShapeDesc) -> Vec<(String, ShapeValues, f64)> {
    let base = d.kind.defaults();
    let mut out = vec![("default".to_string(), base, 1.0)];
    for i in 0..d.fields.len() {
        for x in super::stroke_census_tests::sweep(d, i) {
            let mut v = base;
            v[i] = ui(d, i, x);
            out.push((format!("{} = {x}", d.fields[i].label), v, 1.0));
        }
    }
    for s in 0..=16 {
        let aspect = 0.25 * 16f64.powf(f64::from(s) / 16.0);
        out.push((format!("aspecto {aspect:.3}"), base, aspect));
    }
    out
}

fn cooked(k: ShapeKind, v: &ShapeValues, aspect: f64) -> VecPath {
    cook(k, [0.0, 0.0], [BOX, BOX * aspect], v)
}

/// O campo «Corner round» é o ÚLTIMO de cada forma que o tem, em `Px`, neutro em zero — e a
/// tabela da cena (`ShapeKind::corner_round_field`) aponta-o.
#[test]
fn the_corner_round_field_is_the_last_and_neutral() {
    for d in SHAPES {
        let at = d.fields.iter().position(|f| f.label == "Corner round");
        assert_eq!(at, d.kind.corner_round_field(), "{}", d.label);
        if let Some(i) = at {
            assert_eq!(i + 1, d.fields.len(), "{}: o último", d.label);
            assert_eq!(d.fields[i].unit, FieldUnit::Px, "{}", d.label);
            assert_eq!(d.kind.defaults()[i], 0.0, "{}: neutro", d.label);
        }
    }
}

/// ⭐ Toda quina viva tem quem a arredonde: com o «Corner round» a 40 px não sobra quina em
/// variante nenhuma; os arredondadores próprios apagam as do default; e as de [`OUTSIDE`] nunca
/// mostram quina.
#[test]
fn every_live_corner_has_a_rounder() {
    let mut bad = Vec::new();
    for d in SHAPES {
        let own = OWN.iter().find(|o| o.0 == d.kind);
        for (what, v, aspect) in variants(d) {
            let n = live_corners(&cooked(d.kind, &v, aspect));
            if n == 0 {
                continue;
            }
            match (d.kind.corner_round_field(), own) {
                (Some(i), _) => {
                    let mut r = v;
                    r[i] = ui(d, i, 40.0);
                    let left = live_corners(&cooked(d.kind, &r, aspect));
                    if left > 0 {
                        bad.push(format!(
                            "{} {what}: {left} quina(s) com o Corner round",
                            d.label
                        ));
                    }
                }
                (None, Some(_)) => {}
                (None, None) => {
                    bad.push(format!("{} {what}: {n} quina(s) sem arredondador", d.label))
                }
            }
        }
        if let Some((k, fields, x, why)) = own {
            let mut v = k.defaults();
            for &i in *fields {
                v[i] = ui(d, i, *x);
            }
            let left = live_corners(&cooked(*k, &v, 1.0));
            if left > 0 {
                bad.push(format!("{}: {why} deixa {left} quina(s)", d.label));
            }
        }
    }
    for (k, why) in OUTSIDE {
        assert!(!why.is_empty(), "{k:?}: a razão escrita");
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// ⭐ (C) O raio nasce e NENHUMA outra alça sai do sítio: cada uma lê-se na forma com o «Corner
/// round» a zero (a quina que o raio parte em duas mudava os índices dos vértices). Só o ⊙/⇕ do
/// próprio campo e a fila de ⇕ (que segue a caixa visível) podem andar.
#[test]
fn the_corner_round_moves_no_other_handle() {
    use crate::shape_handles::{Spot, handle_point, handles_of};
    let half = [1.0, 1.0];
    for d in SHAPES {
        let Some(f) = d.kind.corner_round_field() else {
            continue;
        };
        let ck = |v: &ShapeValues| cook(d.kind, [-1.0, -1.0], [1.0, 1.0], v);
        for (k, h) in handles_of(d.kind).iter().enumerate() {
            if h.fields.contains(&f) || h.spot == Spot::Rail {
                continue;
            }
            let base = d.kind.defaults();
            let mut round = base;
            round[f] = 0.3;
            assert_eq!(
                handle_point(h, &base, half, &ck),
                handle_point(h, &round, half, &ck),
                "{} alça {k}",
                d.label
            );
        }
    }
}
