//! As chaves de posição no canvas (doc 17, Onda 2, item 7).

use super::{drag_key, key_points, local_for_world, motion_path_hit_with_keys, trail};
use crate::motion_path_overlay::MotionPathGrab;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Transform, World};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_timeline::{PropKind, TimelineDoc, TimelineState, world_pose_at};

fn window() -> WindowSize {
    WindowSize {
        width: 800,
        height: 600,
    }
}

fn camera() -> Camera2d {
    Camera2d::default()
}

fn key(doc: &mut TimelineDoc, e: u64, prop: PropKind, t: f64, v: f32) {
    doc.insert_key(
        e,
        prop,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

/// Pai rodado, escalado, com skew e ele próprio animado; o filho com X e Y em 0 s e 1 s.
fn scene(only_y_at_one: bool) -> (World, TimelineDoc, u64) {
    let mut w = World::new();
    let pai = w
        .spawn(Transform {
            translation: Vec2::new(2.0, -1.0),
            rotation: 0.6,
            scale: Vec2::new(1.5, 0.75),
            skew_x: 0.1,
            skew_y: 0.0,
        })
        .id();
    let filho = w
        .spawn((Transform::from_translation(Vec2::ZERO), ChildOf(pai)))
        .id()
        .to_bits();
    let mut doc = TimelineDoc::new();
    key(&mut doc, pai.to_bits(), PropKind::Rotation, 0.0, 0.6);
    key(&mut doc, pai.to_bits(), PropKind::Rotation, 2.0, 1.2);
    key(&mut doc, filho, PropKind::TranslationX, 0.0, 0.0);
    key(&mut doc, filho, PropKind::TranslationY, 0.0, 0.0);
    key(&mut doc, filho, PropKind::TranslationY, 1.0, 1.5);
    if only_y_at_one {
        key(&mut doc, filho, PropKind::TranslationX, 2.0, 3.0);
    } else {
        key(&mut doc, filho, PropKind::TranslationX, 1.0, 2.0);
    }
    (w, doc, filho)
}

fn value_at(doc: &TimelineDoc, e: u64, prop: PropKind, t: RationalTime) -> Option<f32> {
    let b = doc.binding_for(e, prop)?;
    let k = doc
        .active_clip()
        .track(b.target)?
        .keys()
        .iter()
        .find(|k| k.t == t)?
        .value;
    match k {
        AnimValue::Float(v) => Some(v),
        _ => None,
    }
}

#[test]
fn the_value_goes_to_world_and_back_exactly_under_a_rotated_scaled_parent() {
    let (w, doc, e) = scene(false);
    for (t, x, y) in [(0.0, 0.0, 0.0), (1.0, 2.0, 1.5)] {
        let p = world_pose_at(&w, &doc, e, t).expect("pose").translation;
        let l = local_for_world(&w, &doc, e, t, [p.x, p.y]).expect("inversa");
        assert!(
            (l[0] - x).abs() < 1e-4 && (l[1] - y).abs() < 1e-4,
            "t={t}: mundo {p:?} voltou a {l:?}, a chave era ({x}, {y})"
        );
    }
}

#[test]
fn each_painted_point_is_where_the_object_is_at_that_key() {
    let (w, doc, e) = scene(false);
    let pts = key_points(true, &w, &doc, Some(e));
    assert_eq!(pts.len(), 2, "um ponto por instante com chave de X ou Y");
    for k in &pts {
        let p = world_pose_at(&w, &doc, e, k.t.to_seconds())
            .expect("pose")
            .translation;
        assert_eq!(k.at, [p.x, p.y]);
    }
}

#[test]
fn dragging_the_painted_point_writes_both_keys_in_one_undo_step() {
    let (w, doc, e) = scene(false);
    let mut st = TimelineState::default();
    st.doc = doc;
    let t1 = RationalTime::from_seconds(1.0);
    let k = key_points(true, &w, &st.doc, Some(e))[1];
    let (sx, sy) = camera().world_to_screen(k.at, window());
    let grab = motion_path_hit_with_keys(
        true,
        true,
        &w,
        &st.doc,
        Some(e),
        &camera(),
        window(),
        sx,
        sy,
    );
    assert_eq!(grab, Some(MotionPathGrab::Key { entity: e, t: t1 }));
    let alvo = camera().screen_to_world((sx + 40.0, sy - 25.0), window());
    let pre = st.doc.clone();
    st.history.begin(&st.doc);
    assert!(drag_key(&mut st, &w, e, t1, alvo));
    // A chave ficou no MESMO instante e o objeto cai onde o ponteiro largou.
    let p = world_pose_at(&w, &st.doc, e, 1.0)
        .expect("pose")
        .translation;
    assert!(
        (p.x - alvo[0]).abs() < 1e-4 && (p.y - alvo[1]).abs() < 1e-4,
        "o objeto foi a {p:?}, o ponteiro largou em {alvo:?}"
    );
    assert!(value_at(&st.doc, e, PropKind::TranslationX, t1).is_some_and(|v| v != 2.0));
    assert!(value_at(&st.doc, e, PropKind::TranslationY, t1).is_some_and(|v| v != 1.5));
    assert_eq!(
        st.doc
            .active_clip()
            .track(
                st.doc
                    .binding_for(e, PropKind::TranslationX)
                    .unwrap()
                    .target
            )
            .unwrap()
            .keys()
            .len(),
        2
    );
    // O caminho redesenha-se pelo valor novo: a última amostra é o ponto largado.
    let tr = trail(&w, &st.doc, e);
    let fim = tr.last().expect("caminho");
    assert!((fim[0] - alvo[0]).abs() < 1e-4 && (fim[1] - alvo[1]).abs() < 1e-4);
    // Um passo de undo devolve o documento de antes.
    assert!(st.history.commit_if_changed(&st.doc));
    let volta = st.history.undo(&st.doc).expect("um passo");
    assert!(
        volta == pre,
        "1 Ctrl+Z tinha de devolver o documento de antes"
    );
    assert!(!st.history.can_undo(), "o gesto inteiro é UM passo");
}

#[test]
fn pulling_sideways_where_only_y_is_keyed_creates_the_x_key() {
    let (w, doc, e) = scene(true);
    let mut st = TimelineState::default();
    st.doc = doc;
    let t1 = RationalTime::from_seconds(1.0);
    assert!(value_at(&st.doc, e, PropKind::TranslationX, t1).is_none());
    let k = key_points(true, &w, &st.doc, Some(e))[1];
    assert_eq!(k.t, t1);
    let (sx, sy) = camera().world_to_screen(k.at, window());
    let alvo = camera().screen_to_world((sx + 30.0, sy), window());
    assert!(drag_key(&mut st, &w, e, t1, alvo));
    assert!(
        value_at(&st.doc, e, PropKind::TranslationX, t1).is_some(),
        "o eixo sem chave nesse quadro ganha uma (resposta de produto 3)"
    );
    let p = world_pose_at(&w, &st.doc, e, 1.0)
        .expect("pose")
        .translation;
    assert!((p.x - alvo[0]).abs() < 1e-4 && (p.y - alvo[1]).abs() < 1e-4);
}

#[test]
fn nothing_is_offered_without_object_keys_tab_or_keys() {
    let (w, doc, e) = scene(false);
    assert!(
        !key_points(true, &w, &doc, Some(e)).is_empty(),
        "controlo positivo"
    );
    assert!(
        key_points(false, &w, &doc, Some(e)).is_empty(),
        "fora da aba Keys"
    );
    assert!(key_points(true, &w, &doc, None).is_empty(), "sem seleção");
    assert!(
        key_points(true, &w, &TimelineDoc::new(), Some(e)).is_empty(),
        "sem chaves"
    );
    let (sx, sy) = camera().world_to_screen(key_points(true, &w, &doc, Some(e))[0].at, window());
    assert!(
        motion_path_hit_with_keys(false, false, &w, &doc, Some(e), &camera(), window(), sx, sy)
            .is_none()
    );
}

/// **Com a animação a TOCAR, o que se vê é o que se agarra** (doc 17 item 12): a agulha a andar
/// reescreve a pose VIVA do objecto e do pai animado, mas os losangos e as alças são o caminho do
/// documento — pintura e clique leem a mesma porta, e o clique no ponto pintado agarra a chave dele.
/// *Mutação: `key_points` ler a pose viva em vez de `world_pose_at` ⇒ os pontos andam com a agulha.*
#[test]
fn while_playing_the_painted_points_and_handles_are_what_the_click_grabs() {
    let (mut w, doc, e) = scene(false);
    let antes = key_points(true, &w, &doc, Some(e));
    let alcas_antes = crate::motion_key_handles::alcas(true, true, &w, &doc, Some(e));
    assert!(
        antes.len() >= 2 && !alcas_antes.is_empty(),
        "controlo positivo"
    );
    // O que o apply da agulha a tocar escreve: só os canais ANIMADOS (rotação do pai, X/Y do filho).
    let filho = ph2d_ecs::Entity::from_bits(e);
    let pai = w.get::<ChildOf>(filho).expect("pai").0;
    w.get_mut::<Transform>(pai).expect("pai vivo").rotation = 1.0;
    w.get_mut::<Transform>(filho)
        .expect("filho vivo")
        .translation = Vec2::new(7.0, -3.0);
    assert_eq!(
        key_points(true, &w, &doc, Some(e)),
        antes,
        "os losangos não andam com a agulha"
    );
    assert_eq!(
        crate::motion_key_handles::alcas(true, true, &w, &doc, Some(e)),
        alcas_antes,
        "nem as alças"
    );
    for k in &antes {
        let (sx, sy) = camera().world_to_screen(k.at, window());
        let grab =
            motion_path_hit_with_keys(true, true, &w, &doc, Some(e), &camera(), window(), sx, sy);
        assert_eq!(
            grab,
            Some(MotionPathGrab::Key { entity: e, t: k.t }),
            "o clique no losango pintado agarra a chave dele"
        );
    }
}
