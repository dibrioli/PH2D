//! Gates da W7.5 Fase 2 — o gizmo da POSE. As mutações que cada um mata estão no
//! doc do teste; todas foram provadas vermelhas na construção (DIRETIVA §3).

use super::*;
use ph2d_ecs::{FlipObjectRef, Name, Transform};
use ph2d_flip::{DupMode, FlipStroke, Hold, KeyKind};

/// Um doc com um objeto (1 camada) cujo traço da chave 0 vai de `a` a `b`, e a chave
/// 12 é uma **INSTÂNCIA** dele. Devolve `(doc, sim, map, oid, lid, entity)`.
fn doc_with_instance(
    a: [f32; 2],
    b: [f32; 2],
) -> (
    FlipDoc,
    SimWorld,
    FlipEntityMap,
    FlipObjectId,
    LayerId,
    ph2d_ecs::Entity,
) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("Obj");
    let obj = doc.object_mut(oid).unwrap();
    let l = obj.add_layer("L");
    let d = obj
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let mut s = FlipStroke::new();
    s.push_default(ph2d_core::Vec2::new(a[0], a[1]));
    s.push_default(ph2d_core::Vec2::new(b[0], b[1]));
    obj.drawing_mut(d).unwrap().strokes.push(s);
    assert!(obj.duplicate_frame(l, 0, 12, DupMode::Instance));
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Obj"), FlipObjectRef(oid.0)))
        .id();
    let mut map = FlipEntityMap::new();
    map.insert(oid, e.to_bits());
    (doc, sim, map, oid, l, e)
}

/// O playhead parado no quadro 12 (a instância), no fps default do objeto.
fn playhead_at_12(doc: &FlipDoc, oid: FlipObjectId) -> Playhead {
    let fps = f64::from(doc.object(oid).unwrap().fps);
    let mut p = Playhead::new(1.0 / 60.0);
    p.pause();
    p.seek_frame(12, fps);
    p
}

/// 🔴 **A reparametrização é EXATA: TRS ↔ Pose fecham ida-e-volta** — para um afim
/// girado, escalado e transladado (na identidade tudo casa por acidente). É o que
/// garante que abrir o gizmo e não mexer não muda a pose.
///
/// Mutação que sangra: `pose_trs` decompor sobre a ORIGEM em vez de `c_local` (a
/// translação devolvida deixa de ser o centro da arte e a volta não fecha).
#[test]
fn the_trs_reparameterization_round_trips_exactly() {
    let c_local = [20.0, 30.0];
    // Um TRS real: rot 30°, escala anisotrópica, transladado.
    let trs = TransformSnapshot {
        translation: [100.0, -40.0],
        rotation: std::f32::consts::FRAC_PI_6,
        scale: [2.0, 1.5],
    };
    let pose = trs_to_pose(trs, c_local);
    let back = pose_trs(pose, c_local);
    assert!(
        (back.translation[0] - trs.translation[0]).abs() < 1e-3
            && (back.translation[1] - trs.translation[1]).abs() < 1e-3,
        "translacao nao fechou: {:?}",
        back.translation
    );
    assert!((back.rotation - trs.rotation).abs() < 1e-5);
    assert!((back.scale[0] - trs.scale[0]).abs() < 1e-5);
    assert!((back.scale[1] - trs.scale[1]).abs() < 1e-5);
    // E a pose reconstruída é a MESMA função afim.
    let p2 = trs_to_pose(back, c_local);
    for (x, y) in [(0.0, 0.0), (10.0, 5.0), (-3.0, 7.0)] {
        let q1 = pose.apply(Vec2::new(x, y));
        let q2 = p2.apply(Vec2::new(x, y));
        assert!((q1.x - q2.x).abs() < 1e-2 && (q1.y - q2.y).abs() < 1e-2);
    }
}

/// **Uma pose de translação pura decompõe como o esperado**: rotação 0, escala 1 e a
/// translação do TRS = onde o CENTRO da arte aparece (`c_local + offset`). É o caso
/// de toda instância recém-movida (a Fase 1 só escreve translação).
#[test]
fn a_translation_pose_decomposes_to_the_shifted_art_center() {
    let c_local = [20.0, 30.0];
    let pose = Pose::from_translation(Vec2::new(7.0, -3.0));
    let trs = pose_trs(pose, c_local);
    assert_eq!(trs.rotation, 0.0);
    assert_eq!(trs.scale, [1.0, 1.0]);
    assert_eq!(trs.translation, [27.0, 27.0]);
}

/// 🔴 **Seed = sample: a caixa do gizmo pousa na arte POSADA** — o pivô da view é
/// exatamente `objeto ∘ pose` aplicado ao centro da arte crua, com pose girada/
/// escalada E objeto movido/escalado (o espelho do `the_halo_carries_the_key_pose`).
///
/// Mutação que sangra: a `pose_view` derivar a caixa da bbox CRUA (sem a pose) — o
/// pivô cai longe da arte, que é o bug do halo do W7.2.
#[test]
fn the_pose_gizmo_box_lands_on_the_posed_art() {
    let (mut doc, mut sim, map, oid, lid, e) = doc_with_instance([10.0, 20.0], [30.0, 40.0]);
    // A pose da INSTÂNCIA: girada + escalada + movida (TRS exato via trs_to_pose).
    let c_local = [20.0, 30.0];
    let trs = TransformSnapshot {
        translation: [80.0, -10.0],
        rotation: std::f32::consts::FRAC_PI_4, // 45°
        scale: [2.0, 1.5],
    };
    let pose = trs_to_pose(trs, c_local);
    doc.object_mut(oid).unwrap().set_frame_pose(lid, 12, pose);
    // E o OBJETO também tem pose própria (na identidade o erro se esconde).
    sim.world_mut().entity_mut(e).insert(Transform {
        translation: ph2d_core::Vec2::new(10.0, 5.0),
        scale: ph2d_core::Vec2::new(3.0, 3.0),
        ..Transform::IDENTITY
    });
    let playhead = playhead_at_12(&doc, oid);
    let cam = Camera2d::default();
    let ws = WindowSize {
        width: 800,
        height: 600,
    };
    let v = pose_view(
        &sim,
        &doc,
        &map,
        PoseViewInputs {
            playhead: &playhead,
            active_layer: None,
            last_pointer: (0.0, 0.0),
        },
        &cam,
        ws,
    )
    .expect("quadro instanciado publica a view");
    // O oráculo é a MESMA cadeia do render: objeto ∘ pose, no centro da arte crua.
    let obj_x = crate::flip_transform::object_xform(&sim, e);
    let posed_c = pose.apply(Vec2::new(c_local[0], c_local[1]));
    let want = obj_x.apply([f64::from(posed_c.x), f64::from(posed_c.y)]);
    assert!(
        (f64::from(v.pivot_world[0]) - want[0]).abs() < 1e-3
            && (f64::from(v.pivot_world[1]) - want[1]).abs() < 1e-3,
        "pivo {:?} != arte posada {want:?}",
        v.pivot_world
    );
    // A meia-extensão carrega as DUAS escalas (pose 2×1.5 ⊙ objeto 3): 10·2·3 × 10·1.5·3.
    let half = [
        (v.bbox_max_world[0] - v.bbox_min_world[0]) * 0.5,
        (v.bbox_max_world[1] - v.bbox_min_world[1]) * 0.5,
    ];
    assert!((half[0] - 60.0).abs() < 1e-2 && (half[1] - 45.0).abs() < 1e-2);
    // E a rotação composta (objeto sem rotação: a da pose).
    assert!((v.rotation - std::f32::consts::FRAC_PI_4).abs() < 1e-4);
}

/// 🔴 **Arte EXCLUSIVA não abre gizmo de pose** — na chave 0 (que possui a geometria)
/// o Edit segue no arrasto-geometria; o gizmo é da INSTÂNCIA. Mutação que sangra:
/// tirar o `is_instanced` do `pose_target`.
#[test]
fn an_exclusive_drawing_never_opens_the_pose_gizmo() {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("Obj");
    let obj = doc.object_mut(oid).unwrap();
    let l = obj.add_layer("L");
    let d = obj
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let mut s = FlipStroke::new();
    s.push_default(ph2d_core::Vec2::new(0.0, 0.0));
    s.push_default(ph2d_core::Vec2::new(10.0, 10.0));
    obj.drawing_mut(d).unwrap().strokes.push(s);
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Obj"), FlipObjectRef(oid.0)))
        .id();
    let mut map = FlipEntityMap::new();
    map.insert(oid, e.to_bits());
    let mut p = Playhead::new(1.0 / 60.0);
    p.pause();
    let cam = Camera2d::default();
    let ws = WindowSize {
        width: 800,
        height: 600,
    };
    assert!(
        pose_view(
            &sim,
            &doc,
            &map,
            PoseViewInputs {
                playhead: &p,
                active_layer: None,
                last_pointer: (0.0, 0.0),
            },
            &cam,
            ws,
        )
        .is_none(),
        "chave de arte propria (users=1) nao pode abrir gizmo de pose"
    );
}

/// 🔴 **O arrasto compõe na POSE em torno do CENTRO da arte** — um Rotate deixa o
/// centro posado onde estava e gira o resto em volta dele; e a saída é uma pose (o
/// `Transform` do objeto não entra na conta de escrita).
///
/// O laço espelha o `flip_pose_gizmo_move` com as funções REAIS (`advance_cursor` +
/// `pose_after_drag`); o wiring App→GPU é do smoke (mesmo limite dos outros seams).
///
/// Mutação que sangra: `trs_to_pose` recompor sobre a origem (o centro da arte
/// TRANSLADA ao girar, e a 1ª asserção cai).
#[test]
fn a_rotate_drag_spins_the_pose_about_the_posed_art_center() {
    let c_local = [20.0, 30.0];
    let pose0 = Pose::from_translation(Vec2::new(7.0, -3.0));
    let start = pose_trs(pose0, c_local); // t_c = (27, 27), rot 0, escala 1
    let cam = GizmoCamera {
        center: [0.0, 0.0],
        height_world: 100.0,
        window_w: 800.0,
        window_h: 600.0,
    };
    // MUNDO → tela, o inverso exato do `GizmoCamera::screen_to_world`.
    let to_screen = |w: [f32; 2]| -> (f32, f32) {
        let half_h = cam.height_world * 0.5;
        let half_w = half_h * (cam.window_w / cam.window_h);
        (
            (w[0] + half_w) / (2.0 * half_w) * cam.window_w,
            cam.window_h - (w[1] + half_h) / (2.0 * half_h) * cam.window_h,
        )
    };
    // Objeto na identidade ⇒ pivô de mundo = t_c. O cursor pousa à direita do pivô e
    // sobe 90° (CCW) em dois passos.
    let pivot = start.translation;
    let start_cursor = [pivot[0] + 10.0, pivot[1]];
    let mut drag = ph2d_editor::GizmoDragState {
        kind: ph2d_editor::GizmoDragKind::Rotate,
        entity_bits: 1,
        start_screen: to_screen(start_cursor),
        cursor_screen: to_screen(start_cursor),
        start_transform: start,
        pivot_world: pivot,
        start_cursor_world: start_cursor,
        sprite_half_intrinsic: [10.0, 10.0],
        anchor_is_center: false,
        target: ph2d_editor::GizmoTarget::FlipPose,
        parent_world: TransformSnapshot::IDENTITY,
        turns: 0,
    };
    let mods = GizmoModifiers::default();
    let snap = GizmoSnap::default();
    for w in [
        [pivot[0] + 7.07, pivot[1] + 7.07], // 45°
        [pivot[0], pivot[1] + 10.0],        // 90°
    ] {
        drag.advance_cursor(to_screen(w), &cam);
    }
    let pose = pose_after_drag(&drag, &cam, mods, snap, c_local);
    // (1) O centro da arte NÃO se move: girar é em torno dele.
    let c0 = pose0.apply(Vec2::new(c_local[0], c_local[1]));
    let c1 = pose.apply(Vec2::new(c_local[0], c_local[1]));
    assert!(
        (c1.x - c0.x).abs() < 1e-2 && (c1.y - c0.y).abs() < 1e-2,
        "o centro transladou ao girar: {c0:?} -> {c1:?}"
    );
    // (2) Um ponto fora do centro ORBITOU ~90° CCW: (30,30) está a +10 em x do
    // centro (20,30); depois do giro ele aparece a +10 em y.
    let p1 = pose.apply(Vec2::new(30.0, 30.0));
    assert!(
        (p1.x - c0.x).abs() < 0.1 && (p1.y - (c0.y + 10.0)).abs() < 0.1,
        "o ponto nao orbitou 90 graus: {p1:?} (centro {c0:?})"
    );
}

/// **Um scale de canto segura o canto OPOSTO** — a política do sprite, reusada. O
/// canto oposto da arte posada fica no lugar; o arrastado dobra a distância.
#[test]
fn a_corner_scale_drag_anchors_the_opposite_corner() {
    let c_local = [20.0, 30.0];
    let pose0 = Pose::from_translation(Vec2::new(0.0, 0.0));
    let start = pose_trs(pose0, c_local); // t_c = (20, 30)
    let h = [10.0, 10.0];
    let cam = GizmoCamera {
        center: [0.0, 0.0],
        height_world: 100.0,
        window_w: 800.0,
        window_h: 600.0,
    };
    let to_screen = |w: [f32; 2]| -> (f32, f32) {
        let half_h = cam.height_world * 0.5;
        let half_w = half_h * (cam.window_w / cam.window_h);
        (
            (w[0] + half_w) / (2.0 * half_w) * cam.window_w,
            cam.window_h - (w[1] + half_h) / (2.0 * half_h) * cam.window_h,
        )
    };
    // Arrasta o canto TR (30,40); o pivô é o canto oposto BL (10,20) — o mesmo
    // `anchor_pivot_world` do down real.
    let kind = ph2d_editor::GizmoDragKind::ScaleCorner {
        dx_sign: 1.0,
        dy_sign: 1.0,
    };
    let pivot = ph2d_editor::anchor_pivot_world(kind, h, start, false);
    assert_eq!(pivot, [10.0, 20.0]);
    let start_cursor = [30.0, 40.0];
    let mut drag = ph2d_editor::GizmoDragState {
        kind,
        entity_bits: 1,
        start_screen: to_screen(start_cursor),
        cursor_screen: to_screen(start_cursor),
        start_transform: start,
        pivot_world: pivot,
        start_cursor_world: start_cursor,
        sprite_half_intrinsic: h,
        anchor_is_center: false,
        target: ph2d_editor::GizmoTarget::FlipPose,
        parent_world: TransformSnapshot::IDENTITY,
        turns: 0,
    };
    // O canto arrastado vai de (30,40) a (50,60): 2× em cada eixo.
    drag.advance_cursor(to_screen([50.0, 60.0]), &cam);
    let pose = pose_after_drag(
        &drag,
        &cam,
        GizmoModifiers::default(),
        GizmoSnap::default(),
        c_local,
    );
    // O canto oposto da ARTE (a bbox crua 10..30 × 20..40) não se move; o arrastado
    // dobra a distância até ele.
    let bl = pose.apply(Vec2::new(10.0, 20.0));
    let tr = pose.apply(Vec2::new(30.0, 40.0));
    assert!(
        (bl.x - 10.0).abs() < 1e-2 && (bl.y - 20.0).abs() < 1e-2,
        "o canto oposto se moveu: {bl:?}"
    );
    assert!(
        (tr.x - 50.0).abs() < 1e-2 && (tr.y - 60.0).abs() < 1e-2,
        "o canto arrastado nao pousou no cursor: {tr:?}"
    );
}
