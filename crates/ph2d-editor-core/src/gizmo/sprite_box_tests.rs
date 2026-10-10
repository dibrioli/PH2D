//! Gates da caixa da sprite contra o MOTOR (`ph2d_ecs::GlobalTransform::from_transform`, a afim
//! que o render desenha): o pai de escala não uniforme não cisalha; o skew sim, e a caixa segue-o;
//! o arrasto de escala com skew fixa o canto oposto PINTADO.

use super::super::camera::world_to_screen_px;
use super::super::{
    GizmoCamera, GizmoDragKind, GizmoDragState, GizmoModifiers, GizmoSnap, GizmoTarget,
    TransformSnapshot, anchor_pivot_world, compose_snapshot, compute_gizmo_transform,
    model_apply, skew_translation_out,
};
use super::*;
use ph2d_ecs::{GlobalTransform, Transform};

fn pose(t: [f32; 2], rot_deg: f32, s: [f32; 2], skew: [f32; 2]) -> Transform {
    let mut x = Transform::IDENTITY;
    x.translation.x = t[0];
    x.translation.y = t[1];
    x.rotation = rot_deg.to_radians();
    x.scale.x = s[0];
    x.scale.y = s[1];
    x.skew_x = skew[0];
    x.skew_y = skew[1];
    x
}

fn desenhado(a: &[f32; 6], q: [f32; 2]) -> [f32; 2] {
    [
        a[0] * q[0] + a[2] * q[1] + a[4],
        a[1] * q[0] + a[3] * q[1] + a[5],
    ]
}

/// Os cantos `(sx, sy) ∈ {±1}²` da caixa, no espaço da caixa e depois por `model`.
fn cantos_da_caixa(b: &SpriteBox) -> [[f32; 2]; 4] {
    let c = [
        (b.bbox_min_world[0] + b.bbox_max_world[0]) * 0.5,
        (b.bbox_min_world[1] + b.bbox_max_world[1]) * 0.5,
    ];
    let h = [
        (b.bbox_max_world[0] - b.bbox_min_world[0]) * 0.5,
        (b.bbox_max_world[1] - b.bbox_min_world[1]) * 0.5,
    ];
    let (s, co) = b.rotation.sin_cos();
    [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)].map(|(sx, sy): (f32, f32)| {
        let (lx, ly) = (sx * h[0], sy * h[1]);
        model_apply(&b.model, [c[0] + lx * co - ly * s, c[1] + lx * s + ly * co])
    })
}

fn cantos_desenhados(a: &[f32; 6], anchor: [f32; 2], half: [f32; 2]) -> [[f32; 2]; 4] {
    [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
        .map(|(sx, sy): (f32, f32)| desenhado(a, [anchor[0] + sx * half[0], anchor[1] + sy * half[1]]))
}

fn perto(a: [f32; 2], b: [f32; 2], tol: f32) -> bool {
    (a[0] - b[0]).abs() < tol && (a[1] - b[1]).abs() < tol
}

/// ⭐ A premissa do item (a) MEDIDA: o motor compõe pai×filho somando rotações e multiplicando
/// escalas — um filho rodado 30° sob um pai `(2, 1)` sai um retângulo (colunas perpendiculares),
/// e a caixa de sempre coincide com ele. O controlo: o produto EXACTO das duas afins cisalharia.
#[test]
fn a_non_uniform_parent_does_not_shear_a_rotated_child() {
    let pai = pose([3.0, -1.0], 20.0, [2.0, 1.0], [0.0, 0.0]);
    let filho = pose([1.0, 0.5], 30.0, [1.0, 1.0], [0.0, 0.0]);
    let a = GlobalTransform::from_transform(Transform::compose(pai, filho)).affine();
    let dot = a[0] * a[2] + a[1] * a[3];
    assert!(dot.abs() < 1e-5, "o motor desenha um retângulo: {dot}");
    let b = sprite_box(a, [0.0, 0.0], [0.0, 0.0], [1.0, 0.5]);
    assert_eq!(b.model, MODEL_IDENTITY);
    for (g, d) in cantos_da_caixa(&b).iter().zip(cantos_desenhados(&a, [0.0, 0.0], [1.0, 0.5])) {
        assert!(perto(*g, d, 1e-4), "caixa {g:?} ≠ desenho {d:?}");
    }
    // Controlo: o produto exacto pai·filho NÃO é perpendicular — a lei é do motor, não da álgebra.
    let (p, f) = (
        GlobalTransform::from_transform(pai).affine(),
        GlobalTransform::from_transform(filho).affine(),
    );
    let c0 = [p[0] * f[0] + p[2] * f[1], p[1] * f[0] + p[3] * f[1]];
    let c1 = [p[0] * f[2] + p[2] * f[3], p[1] * f[2] + p[3] * f[3]];
    assert!((c0[0] * c1[0] + c0[1] * c1[1]).abs() > 1e-2);
}

/// ⭐⭐ Com SKEW a sprite é um paralelogramo, e os cantos da caixa (por `model`) são os
/// desenhados. (Mutação: `model` identidade com skew ⇒ RED — o controlo abaixo mede a distância.)
#[test]
fn a_skewed_sprite_box_is_the_drawn_parallelogram() {
    let skew = [0.35, -0.2];
    let a = GlobalTransform::from_transform(pose([2.0, 1.0], 25.0, [1.5, 0.8], skew)).affine();
    let (anchor, half) = ([0.2, -0.1], [1.0, 0.5]);
    let b = sprite_box(a, skew, anchor, half);
    let d = cantos_desenhados(&a, anchor, half);
    for (g, d) in cantos_da_caixa(&b).iter().zip(d) {
        assert!(perto(*g, d, 1e-4), "caixa {g:?} ≠ desenho {d:?}");
    }
    let sem = SpriteBox {
        model: MODEL_IDENTITY,
        ..b
    };
    let longe = cantos_da_caixa(&sem)
        .iter()
        .zip(d)
        .any(|(g, d)| !perto(*g, d, 1e-2));
    assert!(longe, "o caso tem de cisalhar, senão não mede nada");
}

/// Sem skew: `model` identidade e a mesma conta de antes, bit a bit.
#[test]
fn without_skew_the_box_is_the_old_one_bit_for_bit() {
    let a = GlobalTransform::from_transform(pose([2.0, 1.0], 25.0, [1.5, 0.8], [0.0, 0.0])).affine();
    let b = sprite_box(a, [0.0, 0.0], [0.2, -0.1], [1.0, 0.5]);
    let (sx, sy) = (
        (a[0] * a[0] + a[1] * a[1]).sqrt(),
        (a[2] * a[2] + a[3] * a[3]).sqrt(),
    );
    let rot = a[1].atan2(a[0]);
    let (s, c) = libm::sincosf(rot);
    let (ax, ay) = (0.2 * sx, -0.1 * sy);
    let cx = a[4] + ax * c - ay * s;
    assert_eq!(b.rotation.to_bits(), rot.to_bits());
    assert_eq!(b.bbox_min_world[0].to_bits(), (cx - sx).to_bits());
    assert_eq!(b.model, MODEL_IDENTITY);
}

/// ⭐⭐ **O arrasto do canto de uma sprite com skew**: o canto oposto PINTADO fica parado e o canto
/// arrastado vai ao cursor. O cursor e o pen-down entram pela inversa ([`drag_in_skew_frame`]); a
/// translação volta pela afim ([`skew_translation_out`]). (Controlo: sem a volta, o canto anda.)
#[test]
fn a_corner_drag_on_a_skewed_sprite_pins_the_drawn_opposite_corner() {
    let skew = [0.35, 0.0];
    let start = pose([2.0, 1.0], 20.0, [1.5, 1.0], skew);
    let half = [1.0_f32, 0.5];
    let a0 = GlobalTransform::from_transform(start).affine();
    let [o, _, _, canto] = cantos_desenhados(&a0, [0.0, 0.0], half);
    let alvo = [o[0] + 1.3 * (canto[0] - o[0]), o[1] + 1.3 * (canto[1] - o[1])];
    let cam = GizmoCamera {
        center: [2.0, 1.0],
        height_world: 12.0,
        window_w: 800.0,
        window_h: 600.0,
    };
    let ecra = |w: [f32; 2]| {
        let s = world_to_screen_px(cam.center, cam.height_world, cam.window_w, cam.window_h, w);
        (s[0], s[1])
    };
    let snap = TransformSnapshot {
        translation: [2.0, 1.0],
        rotation: start.rotation,
        scale: [1.5, 1.0],
    };
    let kind = GizmoDragKind::ScaleCorner {
        dx_sign: 1.0,
        dy_sign: 1.0,
    };
    let parent = TransformSnapshot::IDENTITY;
    let drag = GizmoDragState {
        kind,
        entity_bits: 1,
        start_screen: ecra(canto),
        cursor_screen: ecra(alvo),
        start_transform: snap,
        pivot_world: anchor_pivot_world(kind, [0.0, 0.0], half, compose_snapshot(parent, snap), false),
        start_cursor_world: canto,
        sprite_half_intrinsic: half,
        anchor_is_center: false,
        target: GizmoTarget::PrimaryIndividual,
        parent_world: parent,
        turns: 0,
    };
    let m = drag_skew_model(&drag, skew);
    let t = compute_gizmo_transform(
        &drag_in_skew_frame(drag, &m, &cam),
        &cam,
        GizmoModifiers::default(),
        GizmoSnap::default(),
        None,
    );
    let desenha = |tr: [f32; 2]| {
        let p = pose(tr, t.rotation.to_degrees(), t.scale, skew);
        let mut p = p;
        p.rotation = t.rotation;
        cantos_desenhados(&GlobalTransform::from_transform(p).affine(), [0.0, 0.0], half)
    };
    let [o1, _, _, c1] = desenha(skew_translation_out(&m, parent, t.translation));
    assert!(perto(o1, o, 1e-3), "o canto oposto pintado andou: {o1:?} ≠ {o:?}");
    assert!(perto(c1, alvo, 1e-3), "o canto arrastado não foi ao cursor: {c1:?} ≠ {alvo:?}");
    let [o_sem, _, _, _] = desenha(t.translation);
    assert!(!perto(o_sem, o, 1e-2), "sem a volta pela afim o canto anda — senão nada é medido");
}

/// Transladar, rodar e o gizmo GLOBAL não pedem `model`: são rígidos sobre o desenho.
#[test]
fn only_scale_and_move_pivot_read_the_cursor_through_the_skew() {
    let mut d = GizmoDragState {
        kind: GizmoDragKind::Translate,
        entity_bits: 1,
        start_screen: (0.0, 0.0),
        cursor_screen: (0.0, 0.0),
        start_transform: TransformSnapshot::IDENTITY,
        pivot_world: [0.0, 0.0],
        start_cursor_world: [0.0, 0.0],
        sprite_half_intrinsic: [1.0, 1.0],
        anchor_is_center: false,
        target: GizmoTarget::PrimaryIndividual,
        parent_world: TransformSnapshot::IDENTITY,
        turns: 0,
    };
    let skew = [0.3, 0.1];
    assert_eq!(drag_skew_model(&d, skew), MODEL_IDENTITY);
    d.kind = GizmoDragKind::Rotate;
    assert_eq!(drag_skew_model(&d, skew), MODEL_IDENTITY);
    d.kind = GizmoDragKind::MovePivot;
    assert_ne!(drag_skew_model(&d, skew), MODEL_IDENTITY);
    d.target = GizmoTarget::Global;
    assert_eq!(drag_skew_model(&d, skew), MODEL_IDENTITY);
    d.target = GizmoTarget::PrimaryIndividual;
    assert_eq!(drag_skew_model(&d, [0.0, 0.0]), MODEL_IDENTITY);
}
