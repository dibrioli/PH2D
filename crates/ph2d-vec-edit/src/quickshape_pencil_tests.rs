//! Gates do GATILHO do QuickShape, pelo [`crate::Pencil`] — irmão de `quickshape_tests.rs` (tecto de
//! 700 LOC): segurar, arrastar, `Esc`, o interruptor e o `Shift`.

use super::tests::{hand, sides};
use super::*;

use crate::quickshape_hold::{HOLD_NS, HoldTick};

fn rect_stroke() -> Vec<P2> {
    hand(
        &[
            [0.0, 0.0],
            [400.0, 0.0],
            [400.0, 250.0],
            [0.0, 250.0],
            [0.0, 0.0],
        ],
        1.0,
    )
}

/// Desenha `pts` a 4 ms por amostra; devolve o carimbo da última.
pub(super) fn draw(pencil: &mut crate::Pencil, scene: &mut ph2d_vec_scene::VecScene, pts: &[P2]) -> u128 {
    let dy = |i: usize| crate::pencil_width::PenDynamics {
        pressure: 1.0,
        t_ns: i as u128 * 4_000_000,
    };
    pencil.on_press(scene, pts[0], 1.0, dy(0));
    for (i, &p) in pts.iter().enumerate().skip(1) {
        pencil.on_drag(scene, p, dy(i));
    }
    (pts.len() as u128 - 1) * 4_000_000
}

#[test]
fn holding_still_for_half_a_second_turns_the_stroke_into_the_shape_and_release_commits_it() {
    let mut scene = ph2d_vec_scene::VecScene::default();
    let mut pencil = crate::Pencil::default();
    let t = draw(&mut pencil, &mut scene, &rect_stroke());
    let id = pencil.active_path().unwrap();
    assert_eq!(
        // O meio segundo conta desde que a mão ENTROU na folga (os últimos 10 px, ~8 ms antes da
        // última amostra) — 50 ms antes ainda não é hora.
        pencil.hold_tick(&mut scene, t + HOLD_NS - 50_000_000, false, true),
        HoldTick::Idle,
        "antes da hora"
    );
    assert!(!scene.path(id).unwrap().closed);
    assert_eq!(
        pencil.hold_tick(&mut scene, t + HOLD_NS, false, true),
        HoldTick::Recognised
    );
    assert_eq!(
        pencil.hold_tick(&mut scene, t + 2 * HOLD_NS, false, true),
        HoldTick::Idle,
        "nasce uma vez"
    );
    let p = scene.path(id).unwrap();
    assert!(
        p.closed && p.verts.len() == 4,
        "o caminho vivo É o rectângulo"
    );
    assert!(
        pencil
            .width_stops(crate::pencil_width::WidthSource::Speed)
            .is_uniform(),
        "a forma tem a largura do estilo"
    );
    // Arrastar sem soltar: o fim do traço afasta-se do centro para o DOBRO ⇒ a forma dobra.
    let width = |p: &VecPath| {
        let xs = p.verts.iter().map(|v| v.anchor[0]);
        xs.clone().fold(f64::MIN, f64::max) - xs.fold(f64::MAX, f64::min)
    };
    let (w0, centre) = (
        width(p),
        centroid(&p.verts.iter().map(|v| v.anchor).collect::<Vec<_>>()),
    );
    let end = rect_stroke()[rect_stroke().len() - 1];
    let far = add(centre, mul(sub(end, centre), 2.0));
    pencil.on_drag(
        &mut scene,
        far,
        crate::pencil_width::PenDynamics {
            pressure: 1.0,
            t_ns: t,
        },
    );
    let w = width(scene.path(id).unwrap());
    assert!(
        (w - 2.0 * w0).abs() < 1e-6,
        "escala 2× à volta do centro: {w0} → {w}"
    );
    assert!(pencil.on_release(&mut scene));
    let p = scene.path(id).unwrap();
    assert!(
        p.closed && p.verts.len() == 4,
        "soltar comete a FORMA, não o Hobby"
    );
    assert!(
        p.verts
            .iter()
            .all(|v| v.kind == ph2d_vec_scene::VertexKind::Corner),
        "nós de quina editáveis"
    );
}

#[test]
fn moving_esc_and_the_switch_keep_the_free_stroke() {
    let mut scene = ph2d_vec_scene::VecScene::default();
    let mut pencil = crate::Pencil::default();
    let t = draw(&mut pencil, &mut scene, &rect_stroke());
    // Desligado: nunca.
    assert_eq!(
        pencil.hold_tick(&mut scene, t + 10 * HOLD_NS, false, false),
        HoldTick::Idle
    );
    // A mão andou há pouco (o carimbo da última amostra): ainda não.
    pencil.on_drag(
        &mut scene,
        [10.0, 10.0],
        crate::pencil_width::PenDynamics {
            pressure: 1.0,
            t_ns: t + HOLD_NS,
        },
    );
    assert_eq!(
        pencil.hold_tick(&mut scene, t + HOLD_NS + 1, false, true),
        HoldTick::Idle,
        "mexer reinicia"
    );
    let mut pencil = crate::Pencil::default();
    let t = draw(&mut pencil, &mut scene, &rect_stroke());
    assert_eq!(
        pencil.hold_tick(&mut scene, t + HOLD_NS, false, true),
        HoldTick::Recognised
    );
    let id = pencil.active_path().unwrap();
    assert!(
        pencil.quick_dismiss(&mut scene),
        "o Esc devolve o traço livre"
    );
    assert!(!scene.path(id).unwrap().closed && !pencil.is_quick());
    assert_eq!(
        pencil.hold_tick(&mut scene, t + 5 * HOLD_NS, false, true),
        HoldTick::Idle,
        "e não volta"
    );
    assert!(
        !pencil.quick_dismiss(&mut scene),
        "sem forma o Esc é o de sempre"
    );
}

#[test]
fn the_shift_regularises_live_and_turns_in_fifteen_degree_steps() {
    let mut scene = ph2d_vec_scene::VecScene::default();
    let mut pencil = crate::Pencil::default();
    let t = draw(&mut pencil, &mut scene, &rect_stroke());
    assert_eq!(
        pencil.hold_tick(&mut scene, t + HOLD_NS, false, true),
        HoldTick::Recognised
    );
    assert_eq!(
        pencil.hold_tick(&mut scene, t + HOLD_NS + 1, true, true),
        HoldTick::Reshaped
    );
    let id = pencil.active_path().unwrap();
    let c: Vec<P2> = scene
        .path(id)
        .unwrap()
        .verts
        .iter()
        .map(|v| v.anchor)
        .collect();
    let s = sides(&c);
    assert!(
        s.iter().all(|x| (x - s[0]).abs() < 1e-6),
        "o quadrado: {s:?}"
    );
    // Rodar 20° à volta do centro com o Shift ⇒ 15° (o degrau).
    let centre = centroid(&c);
    let end = sub([0.0, 0.0], centre);
    let p = add(centre, rotate(end, 20f64.to_radians()));
    pencil.on_drag(
        &mut scene,
        p,
        crate::pencil_width::PenDynamics {
            pressure: 1.0,
            t_ns: t,
        },
    );
    let c2: Vec<P2> = scene
        .path(id)
        .unwrap()
        .verts
        .iter()
        .map(|v| v.anchor)
        .collect();
    let turned = wrap_pi(angle(sub(c2[1], c2[0])) - angle(sub(c[1], c[0])));
    assert!(
        (turned.to_degrees() - 15.0).abs() < 1e-6,
        "rodou {}°",
        turned.to_degrees()
    );
}

#[test]
fn after_esc_a_new_pause_does_not_bring_the_shape_back() {
    let mut scene = ph2d_vec_scene::VecScene::default();
    let mut pencil = crate::Pencil::default();
    let t = draw(&mut pencil, &mut scene, &rect_stroke());
    assert_eq!(
        pencil.hold_tick(&mut scene, t + HOLD_NS, false, true),
        HoldTick::Recognised
    );
    assert!(pencil.quick_dismiss(&mut scene));
    // A mão anda um pouco (amostra NOVA, carimbo novo) e pára outra vez: o Esc vale o gesto inteiro.
    let t2 = t + 2 * HOLD_NS;
    pencil.on_drag(
        &mut scene,
        [3.0, 3.0],
        crate::pencil_width::PenDynamics {
            pressure: 1.0,
            t_ns: t2,
        },
    );
    assert_eq!(
        pencil.hold_tick(&mut scene, t2 + HOLD_NS, false, true),
        HoldTick::Idle
    );
    assert!(!pencil.is_quick());
}

#[test]
fn a_hand_that_trembles_at_rest_still_holds_but_one_that_moves_does_not() {
    use crate::pencil_width::PenDynamics;
    let mut scene = ph2d_vec_scene::VecScene::default();
    let mut pencil = crate::Pencil::default();
    let t = draw(&mut pencil, &mut scene, &rect_stroke());
    let end = rect_stroke()[rect_stroke().len() - 1];
    // Em repouso a mão treme ±3 px (passa o passo de 1,5 px: cada tremor é uma amostra NOVA).
    for k in 1..=10u32 {
        let d = if k % 2 == 0 { 3.0 } else { -3.0 };
        let at = PenDynamics {
            pressure: 1.0,
            t_ns: t + u128::from(k) * 40_000_000,
        };
        pencil.on_drag(&mut scene, add(end, [d, 0.0]), at);
    }
    assert_eq!(
        pencil.hold_tick(&mut scene, t + HOLD_NS, false, true),
        HoldTick::Recognised,
        "parada DENTRO da folga desde t: o tremor não reinicia o meio segundo"
    );
    // Controlo: a mesma mão que ANDA 20 px (fora da folga) não está parada.
    let mut pencil = crate::Pencil::default();
    let t = draw(&mut pencil, &mut scene, &rect_stroke());
    let at = PenDynamics {
        pressure: 1.0,
        t_ns: t + 400_000_000,
    };
    pencil.on_drag(&mut scene, add(end, [20.0, 0.0]), at);
    assert_eq!(
        pencil.hold_tick(&mut scene, t + HOLD_NS, false, true),
        HoldTick::Idle
    );
}
