//! Unit tests da MATH PURA do gizmo de field — as funções que traduzem params↔gizmo.
//! O `view_from_params` (que precisa de uma `Camera2d`) e a costura de dispatch são
//! exercitados pelo seam gate; aqui só o que pode estar aritmeticamente errado.

use super::*;
use crate::motion_state::MotionState;
use ph2d_editor::{
    GizmoCamera, GizmoDragKind, GizmoDragState, GizmoModifiers, GizmoSnap, GizmoTarget,
    TransformSnapshot,
};
use ph2d_host::WindowSize;
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_panel_motion_graph::set_graph_selection;
use ph2d_render::Camera2d;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn spec_for_is_box_only() {
    // O field ESPACIAL tem spec; os não-espaciais (rank / composição) não.
    assert_eq!(spec_for(NodeTypeId::of("field.box")), Some(BOX_SPEC));
    assert!(spec_for(NodeTypeId::of("field.index_range")).is_none());
    assert!(spec_for(NodeTypeId::of("field.combine")).is_none());
    assert!(spec_for(NodeTypeId::of("motion.integrate")).is_none());
}

#[test]
fn box_spec_names_match_the_node() {
    // Os nomes TÊM de bater com os params reais do `field.box` (ph2d-node-field-box).
    assert_eq!(BOX_SPEC.center_x, "center_x");
    assert_eq!(BOX_SPEC.center_y, "center_y");
    assert_eq!(BOX_SPEC.width, "width");
    assert_eq!(BOX_SPEC.height, "height");
    assert_eq!(BOX_SPEC.rotation, "rotation");
}

#[test]
fn wrap180_folds_into_the_slider_range() {
    assert!(approx(wrap180(45.0), 45.0));
    assert!(approx(wrap180(200.0), -160.0));
    assert!(approx(wrap180(-200.0), 160.0));
    assert!(approx(wrap180(370.0), 10.0));
    // A borda: 180 e -180 são o MESMO ângulo; a convenção do módulo devolve -180.
    assert!(approx(wrap180(180.0), -180.0));
}

#[test]
fn seed_then_params_round_trips_under_identity() {
    // seed = sample: semear os params e lê-los de volta sob o transform IDENTIDADE (nenhum
    // arrasto) devolve exatamente os params — a lição derived-seed-must-match-sample.
    let (start, half) = seed_start(3.0, -2.0, 8.0, 4.0, 30.0);
    assert_eq!(half, [4.0, 2.0]); // extensões CHEIAS ⇒ meia = /2
    let (cx, cy, w, h, rot) = params_from(start, half);
    assert!(approx(cx, 3.0));
    assert!(approx(cy, -2.0));
    assert!(approx(w, 8.0));
    assert!(approx(h, 4.0));
    assert!(approx(rot, 30.0));
}

#[test]
fn params_from_scale_grows_the_extents() {
    // A escala do gizmo multiplica a meia-extensão CONGELADA (não a viva) ⇒ w = extents·escala.
    let (_, half) = seed_start(0.0, 0.0, 8.0, 4.0, 0.0);
    let scaled = TransformSnapshot {
        translation: [0.0, 0.0],
        rotation: 0.0,
        scale: [1.5, 2.0],
    };
    let (_, _, w, h, _) = params_from(scaled, half);
    assert!(approx(w, 12.0));
    assert!(approx(h, 8.0));
}

#[test]
fn params_from_negative_scale_keeps_positive_extents() {
    // Um field é simétrico: flip (escala negativa) é no-op, e uma extensão nunca é negativa.
    let flipped = TransformSnapshot {
        translation: [0.0, 0.0],
        rotation: 0.0,
        scale: [-1.0, 1.0],
    };
    let (_, _, w, h, _) = params_from(flipped, [4.0, 2.0]);
    assert!(approx(w, 8.0));
    assert!(approx(h, 4.0));
}

#[test]
fn params_from_translate_moves_the_center() {
    let translated = TransformSnapshot {
        translation: [5.0, -7.0],
        rotation: 0.0,
        scale: [1.0, 1.0],
    };
    let (cx, cy, _, _, _) = params_from(translated, [4.0, 2.0]);
    assert!(approx(cx, 5.0));
    assert!(approx(cy, -7.0));
}

#[test]
fn params_from_rotate_writes_degrees() {
    // O gizmo dá radianos; o param `field.box` é em GRAUS.
    let rotated = TransformSnapshot {
        translation: [0.0, 0.0],
        rotation: std::f32::consts::FRAC_PI_2, // 90°
        scale: [1.0, 1.0],
    };
    let (_, _, _, _, rot) = params_from(rotated, [4.0, 2.0]);
    assert!(approx(rot, 90.0));
}

// ── Seam gates: a modalidade (a resposta ao "não vai atrapalhar os sprites?") ──

#[test]
fn selected_field_is_some_only_for_a_spatial_field() {
    let mut motion = MotionState::new();
    let bid = motion.doc.graph.add_node("field.box");
    let other = motion.doc.graph.add_node("motion.integrate");
    // Exatamente UM field espacial selecionado ⇒ Some.
    set_graph_selection(vec![bid.0]);
    assert_eq!(selected_field(&motion).map(|(n, _)| n), Some(bid));
    // 🔴 Um nó NÃO-espacial selecionado ⇒ None (sem geometria, sem gizmo). Mata a mutação
    // "spec_for devolve Some para qualquer nó".
    set_graph_selection(vec![other.0]);
    assert!(selected_field(&motion).is_none());
    // Nada selecionado ⇒ None.
    set_graph_selection(vec![]);
    assert!(selected_field(&motion).is_none());
    // Multi-seleção ⇒ None (um gizmo dirige UM field).
    set_graph_selection(vec![bid.0, other.0]);
    assert!(selected_field(&motion).is_none());
    set_graph_selection(vec![]); // higiene do thread-local
}

#[test]
fn field_view_is_published_exactly_when_a_field_is_selected() {
    let mut motion = MotionState::new();
    let bid = motion.doc.graph.add_node("field.box");
    let cam = Camera2d::new([0.0, 0.0], 20.0);
    let win = WindowSize::new(800, 600);
    // 🔴 field selecionado ⇒ a view é publicada (o gizmo aparece). Mata "field_view ignora
    // a seleção".
    set_graph_selection(vec![bid.0]);
    assert!(field_view(&motion, &cam, win, (0.0, 0.0)).is_some());
    // Sem field selecionado ⇒ nada publicado (zero hit-region no canvas dos sprites).
    set_graph_selection(vec![]);
    assert!(field_view(&motion, &cam, win, (0.0, 0.0)).is_none());
}

#[test]
fn a_field_drag_writes_node_params_never_a_transform() {
    // A prova EXECUTÁVEL da preocupação do Enio: o writeback de um arrasto de field escreve
    // os PARAMS do nó, e `apply_field_drag` nem recebe um `SimWorld` — então por construção
    // (e por tipo) NÃO PODE tocar um `Transform` de sprite. O gizmo de field não interfere.
    let mut motion = MotionState::new();
    let bid = motion.doc.graph.add_node("field.box");
    motion.doc.graph.set_param(bid, "center_x", 0.0);
    motion.doc.graph.set_param(bid, "center_y", 0.0);
    motion.doc.graph.set_param(bid, "width", 8.0);
    motion.doc.graph.set_param(bid, "height", 4.0);
    motion.doc.graph.set_param(bid, "rotation", 0.0);

    let (start, half) = seed_start(0.0, 0.0, 8.0, 4.0, 0.0);
    let cam = GizmoCamera {
        center: [0.0, 0.0],
        height_world: 20.0,
        window_w: 800.0,
        window_h: 600.0,
    };
    let start_screen = (400.0, 300.0);
    let mut fgd = FieldGizmoDrag {
        drag: GizmoDragState {
            kind: GizmoDragKind::Translate,
            entity_bits: 0,
            start_screen,
            cursor_screen: start_screen,
            start_transform: start,
            pivot_world: [0.0, 0.0],
            start_cursor_world: cam.screen_to_world(start_screen),
            sprite_half_intrinsic: half,
            anchor_is_center: false,
            target: GizmoTarget::MotionField,
            parent_world: TransformSnapshot::IDENTITY,
            turns: 0,
        },
        node: bid,
        spec: BOX_SPEC,
        intrinsic_half: half,
    };

    // Arrasta o cursor bem longe do centro ⇒ o centro do field anda.
    let (cx, cy, w, h, _rot) = apply_field_drag(
        &mut motion,
        &mut fgd,
        (550.0, 200.0),
        &cam,
        GizmoModifiers::default(),
        GizmoSnap::default(),
    );

    // O writeback caiu nos PARAMS do NÓ — a leitura pela MESMA porta do painel confirma.
    let pv = |name| crate::render_loop::motion_bridge::params::param_value(&motion, bid, name);
    assert_eq!(pv("center_x"), cx);
    assert_eq!(pv("center_y"), cy);
    // 🔴 Um Translate MOVE o centro (mata "apply_field_drag não escreve center") e NÃO mexe
    // nas extensões.
    assert!(cx != 0.0 || cy != 0.0, "o centro do field tinha de andar");
    assert!(approx(w, 8.0));
    assert!(approx(h, 4.0));
}
