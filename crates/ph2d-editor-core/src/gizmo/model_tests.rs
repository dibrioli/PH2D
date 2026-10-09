//! Gates da afim do pai (`GizmoView::model`): o paralelogramo pintado, a identidade bit a bit e o
//! cursor pela inversa.

use super::super::camera::world_to_screen_px;
use super::super::paint::keyed_handle_id;
use super::super::paint_geom::{corners_screen, rotate_rects};
use super::super::{GizmoTarget, GizmoView, ids, paint_sprite_gizmo_keyed};
use super::*;
use crate::interaction::HitIndex;
use crate::zones::Rect;
use std::collections::BTreeMap;

const WIN: (f32, f32) = (800.0, 600.0);
const ALTURA: f32 = 10.0;

/// Uma caixa de centro `(1, 0.5)`, meia-extensão `(2, 1)`, rodada `rot` sob `model`.
fn caixa(rot: f32, model: [f32; 6]) -> GizmoView {
    GizmoView {
        bbox_min_world: [-1.0, -0.5],
        bbox_max_world: [3.0, 1.5],
        rotation: rot,
        camera_center: [1.0, 0.0],
        camera_height_world: ALTURA,
        window_w: WIN.0,
        window_h: WIN.1,
        canvas: Rect::new(0.0, 0.0, WIN.0, WIN.1),
        cursor_screen: None,
        pivot_world: [1.0, 0.5],
        pivot_tool_active: false,
        model,
    }
}

/// O pai do caso: translação `(3, -1)`, rotação 20°, escala NÃO uniforme `(2, 1)` — `R·S`.
fn pai_cisalhante() -> [f32; 6] {
    let (s, c) = 20f32.to_radians().sin_cos();
    [2.0 * c, 2.0 * s, -s, c, 3.0, -1.0]
}

fn cam() -> GizmoCamera {
    GizmoCamera {
        center: [1.0, 0.0],
        height_world: ALTURA,
        window_w: WIN.0,
        window_h: WIN.1,
    }
}

/// ⭐ **Os cantos PINTADOS são `câmara · model · canto`** — o paralelogramo exacto de um campo rodado
/// 30° sob um pai de escala (2, 1). Medido nas alças registadas (o que o clique acerta), com
/// tolerância de `1e-4` m. (Mutação: `world_to_screen` sem a `model` ⇒ RED; e o controlo prova que
/// a figura É um cisalhamento — os lados deixam de ser perpendiculares.)
#[test]
fn a_sheared_parent_paints_the_exact_parallelogram() {
    let rot = 30f32.to_radians();
    let m = pai_cisalhante();
    let v = caixa(rot, m);
    let (mut scene, mut hits, mut mapa) = (
        ph2d_vector::VectorScene::new(),
        HitIndex::new(),
        BTreeMap::new(),
    );
    let alvo = GizmoTarget::MotionField;
    paint_sprite_gizmo_keyed(
        &mut scene,
        &v,
        ph2d_tokens::Theme::Forge,
        &mut hits,
        &mut mapa,
        alvo,
        1.5,
    );
    let (sr, cr) = rot.sin_cos();
    let tol_px = 1e-4 * WIN.1 / ALTURA;
    let mut ecra = Vec::new();
    for (id, sx, sy) in [
        (ids::GIZMO_HANDLE_TL, -1.0, 1.0),
        (ids::GIZMO_HANDLE_TR, 1.0, 1.0),
        (ids::GIZMO_HANDLE_BL, -1.0, -1.0),
        (ids::GIZMO_HANDLE_BR, 1.0, -1.0),
    ] {
        let (lx, ly) = (sx * 2.0, sy * 1.0);
        let local = [1.0 + lx * cr - ly * sr, 0.5 + lx * sr + ly * cr];
        let esperado = world_to_screen_px(
            v.camera_center,
            ALTURA,
            WIN.0,
            WIN.1,
            model_apply(&m, local),
        );
        let r = hits
            .rect_for(keyed_handle_id(alvo, id))
            .expect("a alça foi registada");
        let centro = [r.x + r.w * 0.5, r.y + r.h * 0.5];
        assert!(
            (centro[0] - esperado[0]).abs() < tol_px && (centro[1] - esperado[1]).abs() < tol_px,
            "canto {id:?}: pintado {centro:?} ≠ afim×canto {esperado:?}"
        );
        ecra.push(esperado);
    }
    let (tl, tr, bl) = (ecra[0], ecra[1], ecra[2]);
    let (ex, ey) = (
        [tr[0] - tl[0], tr[1] - tl[1]],
        [tl[0] - bl[0], tl[1] - bl[1]],
    );
    let cos = (ex[0] * ey[0] + ex[1] * ey[1]) / (ex[0].hypot(ex[1]) * ey[0].hypot(ey[1]));
    assert!(
        cos.abs() > 0.1,
        "controlo: a figura tinha de ser cisalhada (cos {cos})"
    );
}

/// ⭐ **Identidade = o caminho de antes, BIT a BIT** — os cantos e as zonas de rodar contra a conta
/// antiga escrita à mão, e o `-0.0` que um `1·x + 0` trocaria. (Mutação: tirar o atalho da
/// identidade em `model_apply` ⇒ o `-0.0` vira `0.0` — RED.)
#[test]
fn the_identity_model_paints_byte_for_byte_what_it_painted_before() {
    let rot = 0.7f32;
    let v = caixa(rot, MODEL_IDENTITY);
    let (cos_r, sin_r) = (rot.cos(), rot.sin());
    let antigo = |lx: f32, ly: f32| {
        let w = [1.0 + lx * cos_r - ly * sin_r, 0.5 + lx * sin_r + ly * cos_r];
        world_to_screen_px(v.camera_center, ALTURA, WIN.0, WIN.1, w)
    };
    let esperado = [
        antigo(-2.0, 1.0),
        antigo(2.0, 1.0),
        antigo(-2.0, -1.0),
        antigo(2.0, -1.0),
    ];
    let c = corners_screen(&v);
    assert_eq!(
        c.map(|p| p.map(f32::to_bits)),
        esperado.map(|p| p.map(f32::to_bits))
    );
    let half = super::super::HANDLE_SIZE_PX * 0.5;
    let off = half + super::super::paint::ROTATE_HANDLE_OFFSET * 0.5;
    for ((_, r), (k, (lx, ly))) in rotate_rects(&v, c).into_iter().zip(
        [(-1.0f32, 1.0f32), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
            .into_iter()
            .enumerate(),
    ) {
        let (ox, oy) = (lx * off, ly * off);
        let cx = c[k][0] + cos_r * ox + -sin_r * oy;
        let cy = c[k][1] + -sin_r * ox + -cos_r * oy;
        let h = super::super::paint::ROTATE_HANDLE_OFFSET * 0.5;
        assert_eq!(
            (r.x.to_bits(), r.y.to_bits()),
            ((cx - h).to_bits(), (cy - h).to_bits())
        );
    }
    assert_eq!(
        model_apply(&MODEL_IDENTITY, [-0.0, 1.0])[0].to_bits(),
        (-0.0f32).to_bits()
    );
}

/// ⭐ **O cursor pela inversa**: o pixel que a câmara lê no espaço do modelo é `model⁻¹ · mundo`;
/// identidade e modelo singular devolvem o pixel intacto. (Mutação: `screen_in_model` sem a
/// inversa ⇒ RED.)
#[test]
fn the_cursor_enters_the_model_space_through_the_inverse() {
    let (m, c) = (pai_cisalhante(), cam());
    let px = (512.25, 133.5);
    let local = c.screen_to_world(c.screen_in_model(&m, px));
    let de_volta = model_apply(&m, local);
    let mundo = c.screen_to_world(px);
    assert!((de_volta[0] - mundo[0]).abs() < 1e-4 && (de_volta[1] - mundo[1]).abs() < 1e-4);
    assert_ne!(local, mundo, "controlo: o pai não é a identidade");
    assert_eq!(c.screen_in_model(&MODEL_IDENTITY, px), px);
    assert_eq!(c.screen_in_model(&[0.0, 0.0, 0.0, 1.0, 5.0, 5.0], px), px);
    assert!(model_inverse(&[0.0, 0.0, 0.0, 1.0, 5.0, 5.0]).is_none());
}
