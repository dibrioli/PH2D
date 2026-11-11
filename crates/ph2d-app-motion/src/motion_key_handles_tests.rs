//! As alças do arco entre chaves de posição (dono 10/11: «Edit curves on canvas»).

use super::{Alca, alca_hit, alcas, drag_alca};
use crate::motion_key_points::{drag_grab, motion_path_hit_with_keys};
use crate::motion_path_overlay::MotionPathGrab;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Transform, World};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_timeline::{PropKind, TimelineDoc, TimelineState, world_pose_at};

const TOL: f32 = 1e-3;

fn window() -> WindowSize {
    WindowSize {
        width: 800,
        height: 600,
    }
}

fn rt(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn key(doc: &mut TimelineDoc, e: u64, prop: PropKind, t: f64, v: f32, i: Interp) {
    doc.insert_key(e, prop, rt(t), AnimValue::Float(v), i);
}

/// Um objeto (filho de um pai estático rodado/escalado se `pai`), X e Y em 0 s e 1 s, Linear.
fn simples(pai: bool) -> (World, TimelineState, u64) {
    let mut w = World::new();
    let base = Transform::from_translation(Vec2::ZERO);
    let e = if pai {
        let p = w
            .spawn(Transform {
                translation: Vec2::new(2.0, -1.0),
                rotation: 0.6,
                scale: Vec2::new(1.5, 0.75),
                skew_x: 0.0,
                skew_y: 0.0,
            })
            .id();
        w.spawn((base, ChildOf(p))).id()
    } else {
        w.spawn(base).id()
    }
    .to_bits();
    let mut st = TimelineState::default();
    key(
        &mut st.doc,
        e,
        PropKind::TranslationX,
        0.0,
        0.0,
        Interp::Linear,
    );
    key(
        &mut st.doc,
        e,
        PropKind::TranslationX,
        1.0,
        3.0,
        Interp::Linear,
    );
    key(
        &mut st.doc,
        e,
        PropKind::TranslationY,
        0.0,
        0.0,
        Interp::Linear,
    );
    key(
        &mut st.doc,
        e,
        PropKind::TranslationY,
        1.0,
        1.5,
        Interp::Linear,
    );
    (w, st, e)
}

/// A cena do smoke: Y em 0/0,5/1, X só em 0/1 (com o easing `x_interp`).
fn cena(x_interp: Interp) -> (World, TimelineState, u64) {
    let mut w = World::new();
    let e = w
        .spawn(Transform::from_translation(Vec2::ZERO))
        .id()
        .to_bits();
    let mut st = TimelineState::default();
    key(&mut st.doc, e, PropKind::TranslationX, 0.0, 0.0, x_interp);
    key(
        &mut st.doc,
        e,
        PropKind::TranslationX,
        1.0,
        4.0,
        Interp::Linear,
    );
    key(
        &mut st.doc,
        e,
        PropKind::TranslationY,
        0.0,
        0.0,
        Interp::Linear,
    );
    key(
        &mut st.doc,
        e,
        PropKind::TranslationY,
        0.5,
        2.0,
        Interp::Linear,
    );
    key(
        &mut st.doc,
        e,
        PropKind::TranslationY,
        1.0,
        0.0,
        Interp::Linear,
    );
    (w, st, e)
}

fn at(w: &World, doc: &TimelineDoc, e: u64, t: f64) -> [f32; 2] {
    let p = world_pose_at(w, doc, e, t).expect("pose").translation;
    [p.x, p.y]
}

fn perto(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < TOL && (a[1] - b[1]).abs() < TOL
}

fn alca(v: &[Alca], ta: f64, out: bool) -> Alca {
    *v.iter()
        .find(|a| a.ta == rt(ta) && a.out == out)
        .expect("a alça")
}

fn bez(p: [[f32; 2]; 4], u: f32) -> [f32; 2] {
    let m = 1.0 - u;
    let w = [m * m * m, 3.0 * m * m * u, 3.0 * m * u * u, u * u * u];
    [0, 1].map(|i| (0..4).map(|k| w[k] * p[k][i]).sum())
}

#[test]
fn a_linear_segment_shows_handles_at_a_third_of_the_chord() {
    let (w, st, e) = simples(false);
    let v = alcas(true, true, &w, &st.doc, Some(e));
    assert_eq!(v.len(), 2, "um segmento, duas alças");
    assert!(perto(alca(&v, 0.0, true).at, [1.0, 0.5]), "{v:?}");
    assert!(perto(alca(&v, 0.0, false).at, [2.0, 1.0]), "{v:?}");
    assert!(perto(alca(&v, 0.0, false).key, [3.0, 1.5]));
}

#[test]
fn dragging_the_out_handle_makes_the_path_the_cubic_through_the_pointer() {
    let (w, mut st, e) = simples(false);
    let antes = alcas(true, true, &w, &st.doc, Some(e));
    let c2 = alca(&antes, 0.0, false).at;
    let q = [0.5, 2.5];
    assert!(drag_alca(&mut st, &w, e, rt(0.0), rt(1.0), true, q));
    let k = [[0.0, 0.0], q, c2, [3.0, 1.5]];
    for u in [0.25, 0.5, 0.75] {
        let p = at(&w, &st.doc, e, f64::from(u));
        assert!(
            perto(p, bez(k, u)),
            "u={u}: o caminho {p:?}, a cúbica {:?}",
            bez(k, u)
        );
    }
    let depois = alcas(true, true, &w, &st.doc, Some(e));
    assert!(
        perto(alca(&depois, 0.0, true).at, q),
        "a alça segue o ponteiro"
    );
    assert!(
        perto(alca(&depois, 0.0, false).at, c2),
        "a alça in não se moveu"
    );
}

#[test]
fn a_missing_x_key_is_born_at_the_sampled_value_and_the_rest_does_not_move() {
    let curvo = Interp::bezier_w(1.0 / 3.0, 1.5, 2.0 / 3.0, -0.5);
    for x_interp in [Interp::Linear, curvo] {
        let (w, mut st, e) = cena(x_interp);
        let fora: Vec<_> = [0.6, 0.75, 0.9, 1.0].map(|t| at(&w, &st.doc, e, t)).into();
        let x_meio = at(&w, &st.doc, e, 0.5)[0];
        assert!(drag_alca(
            &mut st,
            &w,
            e,
            rt(0.0),
            rt(0.5),
            true,
            [1.5, 3.0]
        ));
        let b = st
            .doc
            .binding_for(e, PropKind::TranslationX)
            .unwrap()
            .target;
        let k = st.doc.active_clip().track(b).unwrap().keys().to_vec();
        let nova = k
            .iter()
            .find(|k| k.t == rt(0.5))
            .expect("a chave de X em 0,5");
        assert_eq!(nova.value, AnimValue::Float(x_meio), "{x_interp:?}");
        for (t, p) in [0.6, 0.75, 0.9, 1.0].into_iter().zip(fora) {
            let agora = at(&w, &st.doc, e, t);
            assert!(perto(agora, p), "{x_interp:?} t={t}: {p:?} → {agora:?}");
        }
    }
}

#[test]
fn under_a_moved_and_rotated_parent_the_handle_follows_the_pointer_in_world() {
    let (w, mut st, e) = simples(true);
    let antes = alcas(true, true, &w, &st.doc, Some(e));
    let c2 = alca(&antes, 0.0, false).at;
    let q = [4.0, 1.0];
    assert!(drag_alca(&mut st, &w, e, rt(0.0), rt(1.0), true, q));
    let depois = alcas(true, true, &w, &st.doc, Some(e));
    assert!(perto(alca(&depois, 0.0, true).at, q), "{depois:?}");
    assert!(perto(alca(&depois, 0.0, false).at, c2));
    let k0 = alca(&depois, 0.0, true).key;
    let k1 = alca(&depois, 0.0, false).key;
    let p = at(&w, &st.doc, e, 0.5);
    assert!(
        perto(p, bez([k0, q, c2, k1], 0.5)),
        "o mundo é a cúbica: {p:?}"
    );
}

#[test]
fn one_handle_drag_is_one_undo_step_with_the_born_keys() {
    let (w, mut st, e) = cena(Interp::Linear);
    let pre = st.doc.clone();
    let g = MotionPathGrab::Alca {
        entity: e,
        ta: rt(0.0),
        tb: rt(0.5),
        out: true,
    };
    st.history.begin(&st.doc);
    assert!(drag_grab(&mut st, &w, g, [1.5, 3.0]));
    assert!(drag_grab(&mut st, &w, g, [1.0, 3.5]));
    assert!(st.history.commit_if_changed(&st.doc));
    let volta = st.history.undo(&st.doc).expect("um passo");
    assert!(
        volta == pre,
        "1 Ctrl+Z devolve o documento de antes, sem a chave nascida"
    );
    assert!(!st.history.can_undo(), "o gesto inteiro é UM passo");
}

#[test]
fn with_the_checkbox_off_there_are_no_handles_and_no_hit() {
    let (w, st, e) = simples(false);
    let cam = Camera2d::default();
    let a = alca(&alcas(true, true, &w, &st.doc, Some(e)), 0.0, true);
    let (sx, sy) = cam.world_to_screen(a.at, window());
    let hit = |curves| {
        motion_path_hit_with_keys(true, curves, &w, &st.doc, Some(e), &cam, window(), sx, sy)
    };
    assert_eq!(
        hit(true),
        Some(MotionPathGrab::Alca {
            entity: e,
            ta: rt(0.0),
            tb: rt(1.0),
            out: true
        }),
        "controlo positivo"
    );
    assert!(alcas(true, false, &w, &st.doc, Some(e)).is_empty());
    assert!(
        alcas(false, true, &w, &st.doc, Some(e)).is_empty(),
        "fora da aba Keys"
    );
    assert_eq!(hit(false), None);
}

/// Trecho parado (as duas chaves no mesmo sítio): a alça cairia em cima da chave — não se oferece.
/// *Mutação: tirar o filtro `longe_da_chave` do hit ⇒ vermelho.*
#[test]
fn a_handle_collapsed_onto_its_key_is_not_offered() {
    let mut w = World::new();
    let e = w
        .spawn(Transform::from_translation(Vec2::ZERO))
        .id()
        .to_bits();
    let mut st = TimelineState::default();
    for prop in [PropKind::TranslationX, PropKind::TranslationY] {
        for t in [0.0, 1.0] {
            key(&mut st.doc, e, prop, t, 0.5, Interp::Linear);
        }
    }
    let cam = Camera2d::default();
    let a = alca(&alcas(true, true, &w, &st.doc, Some(e)), 0.0, true);
    let (sx, sy) = cam.world_to_screen(a.at, window());
    assert_eq!(
        alca_hit(true, true, &w, &st.doc, Some(e), &cam, window(), sx, sy),
        None
    );
}
