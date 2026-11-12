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

/// O arco de mundo do objeto em `[t0, t1]`, amostrado.
fn arco(w: &World, doc: &TimelineDoc, e: u64, t0: f64, t1: f64) -> Vec<[f32; 2]> {
    const N: usize = 200;
    (0..=N)
        .map(|k| at(w, doc, e, t0 + (t1 - t0) * k as f64 / N as f64))
        .collect()
}

/// Hausdorff entre duas polilinhas: zero = a mesma FORMA, em qualquer ritmo.
fn hausdorff(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    let lado = |p: &[[f32; 2]], q: &[[f32; 2]]| {
        p.iter()
            .map(|&x| {
                q.windows(2)
                    .map(|s| dist_seg(x, s[0], s[1]))
                    .fold(f32::INFINITY, f32::min)
            })
            .fold(0.0, f32::max)
    };
    lado(a, b).max(lado(b, a))
}

fn cubica_amostrada(k: [[f32; 2]; 4]) -> Vec<[f32; 2]> {
    (0..=200).map(|i| bez(k, i as f32 / 200.0)).collect()
}

/// Arrastar a alça: o objeto vira caminho (uma linha «Position») e o arco é a cúbica pelo ponteiro.
#[test]
fn dragging_the_out_handle_makes_the_path_the_cubic_through_the_pointer() {
    let (w, mut st, e) = simples(false);
    let antes = alcas(true, true, &w, &st.doc, Some(e));
    let c2 = alca(&antes, 0.0, false).at;
    let q = [0.5, 2.5];
    assert!(drag_alca(&mut st, &w, e, rt(0.0), rt(1.0), true, q));
    let d = hausdorff(
        &arco(&w, &st.doc, e, 0.0, 1.0),
        &cubica_amostrada([[0.0, 0.0], q, c2, [3.0, 1.5]]),
    );
    assert!(d < 1e-2, "o arco afastou-se {d} da cúbica pelo ponteiro");
    assert!(
        st.doc.binding_for(e, PropKind::TranslationX).is_none(),
        "X e Y viraram Position"
    );
    let pos = st
        .doc
        .binding_for(e, PropKind::Position)
        .expect("uma linha Position")
        .target;
    let a0 = st.doc.path_anchor(pos, 0).expect("a âncora");
    assert!(perto(
        [
            a0.anchor[0] + a0.out_handle[0],
            a0.anchor[1] + a0.out_handle[1]
        ],
        q
    ));
    assert!(
        alcas(true, true, &w, &st.doc, Some(e)).is_empty(),
        "agora quem desenha é o caminho"
    );
}

/// Entortar um segmento não mexe na FORMA dos outros (o eixo Y tem uma chave a mais em 0,5).
#[test]
fn bending_one_segment_keeps_the_shape_of_the_rest() {
    let curvo = Interp::bezier_w(1.0 / 3.0, 1.5, 2.0 / 3.0, -0.5);
    for x_interp in [Interp::Linear, curvo] {
        let (w, mut st, e) = cena(x_interp);
        let fora = arco(&w, &st.doc, e, 0.5, 1.0);
        assert!(drag_alca(
            &mut st,
            &w,
            e,
            rt(0.0),
            rt(0.5),
            true,
            [1.5, 3.0]
        ));
        let d = hausdorff(&fora, &arco(&w, &st.doc, e, 0.5, 1.0));
        assert!(d < 1e-2, "{x_interp:?}: o resto do arco afastou-se {d}");
    }
}

#[test]
fn under_a_moved_and_rotated_parent_the_handle_follows_the_pointer_in_world() {
    let (w, mut st, e) = simples(true);
    let antes = alcas(true, true, &w, &st.doc, Some(e));
    let (k0, c2, k1) = (
        alca(&antes, 0.0, true).key,
        alca(&antes, 0.0, false).at,
        alca(&antes, 0.0, false).key,
    );
    let q = [4.0, 1.0];
    assert!(drag_alca(&mut st, &w, e, rt(0.0), rt(1.0), true, q));
    let d = hausdorff(
        &arco(&w, &st.doc, e, 0.0, 1.0),
        &cubica_amostrada([k0, q, c2, k1]),
    );
    assert!(
        d < 1e-2,
        "no mundo o arco é a cúbica pelo ponteiro: afastou-se {d}"
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

/// Dono 10/11: o caminho e os losangos das chaves também só existem com a caixa marcada.
/// *Mutação: `key_points(keys_tab, …)` sem `&& curves` no hit ⇒ vermelho.*
#[test]
fn with_the_checkbox_off_the_key_diamonds_are_not_offered_either() {
    let (w, st, e) = simples(false);
    let cam = Camera2d::default();
    let (sx, sy) = cam.world_to_screen(at(&w, &st.doc, e, 1.0), window());
    let hit = |curves| {
        motion_path_hit_with_keys(true, curves, &w, &st.doc, Some(e), &cam, window(), sx, sy)
    };
    assert_eq!(
        hit(true),
        Some(MotionPathGrab::Key {
            entity: e,
            t: rt(1.0)
        }),
        "controlo positivo"
    );
    assert_eq!(hit(false), None);
}

fn dist_seg(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let u = if l2 > 0.0 {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (p[0] - a[0] - u * dx).hypot(p[1] - a[1] - u * dy)
}

/// ⭐ Item 13 (dono 10/11): entortar o arco no canvas e depois escolher um easing na Timeline
/// NÃO pode mudar a forma do arco — só o ritmo ao longo dele (dono 12/11, modelo After Effects).
#[test]
fn choosing_an_easing_after_bending_the_arc_keeps_the_arc() {
    use ph2d_anim::{Easing, EasingFamily, EasingMode};
    use ph2d_timeline::{TimelineIntent, apply_intent};
    let ease = Interp::Eased(Easing {
        family: EasingFamily::Cubic,
        mode: EasingMode::InOut,
    });
    // Os dois gestos de easing do painel: a seleção que o arrasto deixou, e a coluna inteira em 0 s.
    for coluna in [false, true] {
        let (w, mut st, e) = simples(false);
        assert!(drag_alca(
            &mut st,
            &w,
            e,
            rt(0.0),
            rt(1.0),
            true,
            [0.5, 2.5]
        ));
        let torto = st.doc.clone();
        let intents = if coluna {
            ph2d_panel_timeline::presets::column(
                &st,
                ph2d_panel_timeline::presets::Preset::Eased(match ease {
                    Interp::Eased(x) => x,
                    _ => unreachable!(),
                }),
                0.0,
            )
        } else {
            vec![TimelineIntent::SetSelectedInterp { interp: ease }]
        };
        let mut ph = ph2d_core::Playhead::default();
        for i in intents {
            apply_intent(&mut st, &mut ph, i);
        }
        let d = hausdorff(
            &arco(&w, &torto, e, 0.0, 1.0),
            &arco(&w, &st.doc, e, 0.0, 1.0),
        );
        assert!(
            d < 1e-2,
            "coluna={coluna}: o arco afastou-se {d} do que o canvas entortou"
        );
        // E o ritmo mudou mesmo (senão o easing seria um botão morto): no meio do tempo o objeto
        // não está onde estava.
        let (m0, m1) = (at(&w, &torto, e, 0.25), at(&w, &st.doc, e, 0.25));
        assert!(
            !perto(m0, m1),
            "coluna={coluna}: o easing não mudou o ritmo"
        );
    }
}

/// O caminho (âncoras e tangentes) obedece à MESMA caixa «Edit curves on canvas» que as alças; sem
/// cartões (`PH2D_CARDS=0`) não há a caixa e ele fica como antes.
/// *Mutação: `caminho_visivel` devolver `true` ⇒ vermelho.*
#[test]
fn the_path_obeys_the_curves_checkbox_like_the_handles() {
    use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
    let (w, mut st, e) = simples(false);
    assert!(drag_alca(
        &mut st,
        &w,
        e,
        rt(0.0),
        rt(1.0),
        true,
        [0.5, 2.5]
    ));
    let cam = Camera2d::default();
    let (sx, sy) = cam.world_to_screen([3.0, 1.5], window());
    let hit = |curves| {
        motion_path_hit_with_keys(true, curves, &w, &st.doc, Some(e), &cam, window(), sx, sy)
    };
    let ancora = |g: Option<MotionPathGrab>| matches!(g, Some(MotionPathGrab::Anchor { i: 1, .. }));
    force_cards_on_this_thread(Some(true));
    let (com, sem) = (hit(true), hit(false));
    force_cards_on_this_thread(Some(false));
    let classico = hit(false);
    force_cards_on_this_thread(None);
    assert!(ancora(com), "controlo positivo: {com:?}");
    assert_eq!(sem, None, "caixa desmarcada: nada agarra");
    assert!(ancora(classico), "sem cartões, como antes: {classico:?}");
}

/// O caminho Path é LOCAL (vira `Transform.translation`): sob um pai movido, rodado e escalado, o
/// overlay pinta e agarra cada âncora onde o OBJETO está no instante da chave, a ponta da alça onde
/// o ponteiro a deixou, e arrastar uma âncora ou alça a um ponto de mundo põe-na lá (mundo → local).
#[test]
fn under_a_moved_and_rotated_parent_the_path_is_painted_grabbed_and_dragged_in_world() {
    use crate::motion_path_overlay::{anchor_screen, motion_path_hit, tangent_screen};
    let (w, mut st, e) = simples(true);
    let q = [4.0, 1.0];
    assert!(drag_alca(&mut st, &w, e, rt(0.0), rt(1.0), true, q));
    let cam = Camera2d::default();
    let tela = |p: [f32; 2]| {
        let (x, y) = cam.world_to_screen(p, window());
        (f64::from(x), f64::from(y))
    };
    let px =
        |a: ph2d_vector::Point, b: (f64, f64)| (a.x - b.0).abs() < 1e-2 && (a.y - b.1).abs() < 1e-2;
    let ponta = |doc: &TimelineDoc, i: usize, out: bool| {
        tangent_screen(true, &w, doc, Some(e), &cam, window())
            .into_iter()
            .find(|x| x.1 == i && x.2 == out)
            .map(|x| x.4)
    };
    let ancoras = anchor_screen(true, &w, &st.doc, Some(e), &cam, window());
    assert_eq!(ancoras.len(), 2, "duas âncoras: {ancoras:?}");
    for (i, t) in [(0usize, 0.0), (1, 1.0)] {
        let (p, o) = (ancoras[i].2, tela(at(&w, &st.doc, e, t)));
        assert!(
            px(p, o),
            "âncora {i} pintada em {p:?}; o objeto está em {o:?}"
        );
        let g = motion_path_hit(
            true,
            &w,
            &st.doc,
            Some(e),
            &cam,
            window(),
            p.x as f32,
            p.y as f32,
        );
        assert!(
            matches!(g, Some(MotionPathGrab::Anchor { i: j, .. }) if j == i),
            "âncora {i} não se agarra onde está pintada: {g:?}"
        );
    }
    let tip = ponta(&st.doc, 0, true).expect("a alça de saída da âncora 0");
    assert!(
        px(tip, tela(q)),
        "a ponta pintada em {tip:?}; o ponteiro a deixou em {:?}",
        tela(q)
    );

    let target = ancoras[1].0;
    let alvo = [5.0, -2.0];
    assert!(drag_grab(
        &mut st,
        &w,
        MotionPathGrab::Anchor { target, i: 1 },
        alvo
    ));
    let fim = at(&w, &st.doc, e, 1.0);
    assert!(
        perto(fim, alvo),
        "arrastei a âncora a {alvo:?}; o objeto foi a {fim:?}"
    );
    let q2 = [3.0, 3.0];
    let alca_in = MotionPathGrab::Tangent {
        target,
        i: 1,
        out: false,
    };
    assert!(drag_grab(&mut st, &w, alca_in, q2));
    let tip = ponta(&st.doc, 1, false).expect("a alça de entrada da âncora 1");
    assert!(
        px(tip, tela(q2)),
        "a ponta arrastada a {:?} ficou em {tip:?}",
        tela(q2)
    );
}
