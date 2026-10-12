//! O resto dos gates da F9 (as voltas da espiral do Inkscape, as alças de ângulo dentro do aro) —
//! irmão de `shape_handles_tests.rs` pelo tecto de 700 LOC (HR-18).

use super::tests::{HALF, PX, attr, cooker, fixture, pointer, spin};
use super::*;

/// ⭐ AS VOLTAS da espiral: o Inkscape soma `Δθ/360` contínuo, e nós também (F12 — as nossas eram
/// inteiras e a ponta só saltava a meia volta: «o gizmo da espiral não funciona», o dono); a
/// distância da mão não conta, nas duas.
#[test]
fn the_spiral_turns_follow_the_winding_like_inkscape() {
    let f = fixture("spiral");
    let mut n = 0;
    for e in f["experiencias"].as_array().unwrap() {
        let nome = e["nome"].as_str().unwrap();
        // O Shift dele escala e roda (não é uma contagem); e o arrasto de 180° do roteiro é uma
        // recta PELO centro — o sentido de uma volta assim não existe.
        if !nome.starts_with("outer_") || e["mod"] == "shift" || nome.starts_with("outer_a+180") {
            continue;
        }
        let rev = attr(e, "sodipodi:revolution");
        let a0 = pointer(&f, e, "alca_tela", 0.0);
        let b0 = pointer(&f, e, "alvo_tela", 0.0);
        let ang = |p: [f64; 2]| (300.0 - p[1]).atan2(p[0] - 400.0).to_degrees();
        // O y virado inverte o sentido dos dois (o dele enrola para baixo, o nosso para cima).
        let turn = -wrap180(ang(b0) - ang(a0));
        let mut v = ShapeKind::Spiral.defaults();
        v[0] = 3.0;
        // O Ctrl dele prende o ângulo a 15° (+50° → 3,125): é o nosso Shift.
        let mods = Mods {
            snap: e["mod"] == "ctrl",
            ..Mods::default()
        };
        let got = spin(ShapeKind::Spiral, 0, v, turn, mods)[0];
        assert!(
            (got - rev).abs() < 1e-3,
            "{nome}: {got} contra o oráculo {rev}"
        );
        n += 1;
    }
    assert_eq!(n, 6, "as experiências da ponta de fora");
}

/// ⭐ As alças de ÂNGULO de um arco vivem `park_at` por FORA do aro, numa haste (o *pie widget* do
/// Illustrator, F11 — antes recolhiam-se para dentro): nunca em cima das alças do meio das arestas
/// da caixa — e o arrasto delas continua a ler só o ângulo.
#[test]
fn an_angle_handle_lives_outside_the_rim_off_the_box_handles() {
    let k = ShapeKind::Pie;
    let ck = cooker(k, HALF);
    let v = k.defaults(); // início a 0° (3h): no aro, é o meio da aresta direita da caixa
    let st = handles_of(k)
        .iter()
        .position(|h| matches!(h.law, Law::ArcStart { .. }))
        .unwrap();
    let h = &handles_of(k)[st];
    let truth = handle_point(h, &v, HALF, &ck).unwrap();
    assert!((truth[0] - HALF[0]).abs() < 1e-9 && truth[1].abs() < 1e-9);
    let g = handle_grip(k, h, &v, HALF, PX, 0.25, &ck).unwrap();
    assert!(
        (g[0] - (HALF[0] + 0.25)).abs() < 1e-9 && g[1].abs() < 1e-9,
        "{g:?}"
    );
    // Num raio pequeno também fica fora (a haste não depende do tamanho).
    let tiny = handle_grip(k, h, &v, [0.3, 0.3], PX, 0.25, &cooker(k, [0.3, 0.3])).unwrap();
    assert!((tiny[0] - 0.55).abs() < 1e-9, "{tiny:?}");
    // Agarrada no sítio DESENHADO (a pega mede-se a ele) e levada a 40° de volta perto do centro
    // (meio raio): o ângulo da mão é o que conta.
    let mut d = HandleDrag::start(st, v, g, g, HALF);
    let mut r = v;
    for s in 1..=8 {
        let a = (40.0 * f64::from(s) / 8.0).to_radians();
        r = drag(
            k,
            &mut d,
            [HALF[0] * 0.5 * a.cos(), HALF[1] * 0.5 * a.sin()],
            Mods::default(),
            HALF,
            PX,
            &ck,
        );
    }
    assert!((r[1] - 40.0).abs() < 1e-9, "{r:?}");
}
