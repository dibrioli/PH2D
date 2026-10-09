//! Gates do campo sob a afim do objecto Motion em edição (onda 4, item 7): a caixa pintada é o
//! paralelogramo exacto e o arrasto mexe o param que a alça diz, com o dono escalado (2, 1) e o
//! campo rodado 30°.

use std::collections::BTreeMap;

use super::super::{apply_field_drag, field_view};
use super::*;
use ph2d_editor_core::{GizmoModifiers, GizmoSnap, HitIndex};
use ph2d_nodegraph::graph::NodeId;
use ph2d_panel_motion_graph::set_graph_selection;
use ph2d_render::Camera2d;

const WIN: (f32, f32) = (800.0, 600.0);
const ALTURA: f32 = 10.0;
const ROT: f32 = 30.0;
const C: [f32; 2] = [1.0, 0.5];
const HALF: [f32; 2] = [2.0, 1.0];

/// O dono do caso: translação `(3, -1)`, rotação 20°, escala `(2, 1)` — `R·S`, um cisalhamento sob
/// o campo rodado.
fn afim_do_dono() -> [f32; 6] {
    let (s, c) = 20f32.to_radians().sin_cos();
    [2.0 * c, 2.0 * s, -s, c, 3.0, -1.0]
}

/// Um `field.box` de centro `C`, extensões `2·HALF`, rotação `ROT`, seleccionado, com dono `afim`.
fn cena(afim: Option<[f32; 6]>) -> (MotionState, NodeId) {
    let mut motion = MotionState::new();
    let bid = motion.doc.graph.add_node("field.box");
    for (n, v) in [
        ("center_x", C[0]),
        ("center_y", C[1]),
        ("width", 2.0 * HALF[0]),
        ("height", 2.0 * HALF[1]),
        ("rotation", ROT),
    ] {
        motion.doc.graph.set_param(bid, n, v);
    }
    if let Some(afim) = afim {
        motion.doc.set_owner(bid, 7);
        let dono = crate::motion_objeto::Dono {
            entity: 1,
            visible: true,
            afim,
        };
        motion.objeto.donos.insert(7, dono);
        motion.objeto.editing = Some(7);
    }
    set_graph_selection(vec![bid.0]);
    (motion, bid)
}

fn cam() -> GizmoCamera {
    GizmoCamera {
        center: [1.0, 0.0],
        height_world: ALTURA,
        window_w: WIN.0,
        window_h: WIN.1,
    }
}

/// Mundo → ecrã pela `cam` (o inverso exacto do `screen_to_world`).
fn ecra(world: [f32; 2]) -> (f32, f32) {
    let c = cam();
    let half_h = ALTURA * 0.5;
    let half_w = half_h * WIN.0 / WIN.1;
    let nx = (world[0] - c.center[0]) / half_w;
    let ny = -(world[1] - c.center[1]) / half_h;
    ((nx + 1.0) * 0.5 * WIN.0, (ny + 1.0) * 0.5 * WIN.1)
}

/// `C + R(rot°)·v`, no espaço LOCAL do objecto.
fn local(rot: f32, v: [f32; 2]) -> [f32; 2] {
    let (s, c) = rot.to_radians().sin_cos();
    [C[0] + v[0] * c - v[1] * s, C[1] + v[0] * s + v[1] * c]
}

fn param(motion: &MotionState, n: NodeId, nome: &str) -> f32 {
    crate::motion_bridge::params::param_value(motion, n, nome)
}

/// Arrasta a alça `kind` de `de` para `para` (pontos LOCAIS, levados ao ecrã pela afim do dono).
fn arrasta(motion: &mut MotionState, kind: GizmoDragKind, de: [f32; 2], para: [f32; 2]) {
    let a = crate::motion_objeto::afim_em_edicao(motion);
    let ponto = |p| ecra(crate::motion_objeto::origem::ponto(&a, p));
    let mut fgd = drag_do_down(motion, kind, &cam(), ponto(de), false).expect("campo seleccionado");
    let (m, s) = (GizmoModifiers::default(), GizmoSnap::default());
    apply_field_drag(motion, &mut fgd, ponto(para), &cam(), m, s);
}

/// ⭐ **Os cantos pintados do campo são `afim do dono × cantos do campo`** (tolerância `1e-4` m).
/// (Mutação: `field_view` sem a `model` ⇒ os cantos caem na caixa local — RED.)
#[test]
fn the_field_box_of_a_sheared_owner_is_painted_as_the_exact_parallelogram() {
    let afim = afim_do_dono();
    let (motion, _) = cena(Some(afim));
    let camera = Camera2d::new([1.0, 0.0], ALTURA);
    let v = field_view(&motion, &camera, WIN.0, WIN.1, (0.0, 0.0)).expect("campo seleccionado");
    set_graph_selection(vec![]);
    let (mut cena_v, mut hits, mut mapa) = (
        ph2d_vector::VectorScene::new(),
        HitIndex::new(),
        BTreeMap::new(),
    );
    let alvo = GizmoTarget::MotionField;
    ph2d_editor_core::paint_sprite_gizmo_keyed(
        &mut cena_v,
        &v,
        ph2d_tokens::Theme::Forge,
        &mut hits,
        &mut mapa,
        alvo,
        1.5,
    );
    let tol_px = 1e-4 * WIN.1 / ALTURA;
    let mut cantos = 0;
    for (id, hit) in &mapa {
        let GizmoDragKind::ScaleCorner { dx_sign, dy_sign } = hit.kind else {
            continue;
        };
        let canto = local(ROT, [dx_sign * HALF[0], dy_sign * HALF[1]]);
        let e = ecra(crate::motion_objeto::origem::ponto(&afim, canto));
        let r = hits.rect_for(*id).expect("registada");
        let p = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(
            (p.0 - e.0).abs() < tol_px && (p.1 - e.1).abs() < tol_px,
            "pintado {p:?} ≠ {e:?}"
        );
        cantos += 1;
    }
    assert_eq!(
        cantos, 4,
        "controlo: as quatro alças de canto foram medidas"
    );
}

/// ⭐ **O arrasto sob a afim mexe o param que a alça diz** — mover, escalar pelo canto e rodar, com o
/// dono escalado (2, 1) e o campo rodado 30°. (Mutação: `drag_do_down`/`apply_field_drag` sem
/// `screen_in_model` ⇒ o cursor fica no mundo e os três reprovam.)
#[test]
fn a_handle_dragged_under_a_sheared_owner_moves_the_param_it_names() {
    let afim = Some(afim_do_dono());
    let perto = |a: f32, b: f32| (a - b).abs() < 1e-3;

    let (mut m, n) = cena(afim);
    arrasta(
        &mut m,
        GizmoDragKind::Translate,
        C,
        [C[0] + 1.0, C[1] - 0.5],
    );
    assert!(perto(param(&m, n, "center_x"), 2.0) && perto(param(&m, n, "center_y"), 0.0));
    assert!(perto(param(&m, n, "width"), 4.0) && perto(param(&m, n, "rotation"), ROT));

    // TR para 1,5× ao longo dos eixos LOCAIS do campo: o canto oposto (BL) fica.
    let (mut m, n) = cena(afim);
    let tr = GizmoDragKind::ScaleCorner {
        dx_sign: 1.0,
        dy_sign: 1.0,
    };
    let (de, para) = (local(ROT, HALF), local(ROT, [HALF[0] * 2.0, HALF[1] * 2.0]));
    arrasta(&mut m, tr, de, para);
    let novo_c = local(ROT, [HALF[0] * 0.5, HALF[1] * 0.5]);
    assert!(perto(param(&m, n, "width"), 6.0) && perto(param(&m, n, "height"), 3.0));
    assert!(
        perto(param(&m, n, "center_x"), novo_c[0]) && perto(param(&m, n, "center_y"), novo_c[1])
    );

    let (mut m, n) = cena(afim);
    arrasta(
        &mut m,
        GizmoDragKind::Rotate,
        local(10.0, [3.0, 0.0]),
        local(55.0, [3.0, 0.0]),
    );
    assert!(
        perto(param(&m, n, "rotation"), ROT + 45.0),
        "{}",
        param(&m, n, "rotation")
    );
    assert!(perto(param(&m, n, "center_x"), C[0]) && perto(param(&m, n, "width"), 4.0));
    set_graph_selection(vec![]);
}

/// ⭐ **Sem dono deslocado, o arrasto é o de antes** — a semente é a que a shell montava (cursor
/// cru, pai identidade) e a caixa é `centro ± meia`, sem afim.
#[test]
fn without_a_moved_owner_the_seed_and_the_box_are_the_old_ones() {
    let (motion, _) = cena(None);
    let px = (431.0, 277.0);
    let fgd = drag_do_down(&motion, GizmoDragKind::Translate, &cam(), px, false).expect("campo");
    let v = field_view(
        &motion,
        &Camera2d::new([1.0, 0.0], ALTURA),
        WIN.0,
        WIN.1,
        px,
    )
    .expect("v");
    set_graph_selection(vec![]);
    assert_eq!(fgd.drag.start_screen, px);
    assert_eq!(fgd.drag.start_cursor_world, cam().screen_to_world(px));
    assert_eq!(fgd.drag.parent_world, TransformSnapshot::IDENTITY);
    assert_eq!(fgd.model, crate::motion_objeto::IDENTIDADE);
    assert_eq!(v.model, ph2d_editor_core::gizmo::MODEL_IDENTITY);
    assert_eq!(v.bbox_min_world, [C[0] - HALF[0], C[1] - HALF[1]]);
    assert_eq!(v.rotation, ROT.to_radians());
}
