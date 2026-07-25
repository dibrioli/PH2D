//! Unit tests da MATH PURA do gizmo de field — as funções que traduzem params↔gizmo — e
//! os seam gates da modalidade (a resposta ao *"não vai atrapalhar os sprites?"*). O
//! `view_from_params` (que precisa de uma `Camera2d`) e a costura de dispatch são
//! exercitados pelo seam gate; aqui o que pode estar aritmeticamente errado + a
//! generalização `FieldSize` (retângulo da box × disco do radial sweep).

use super::*;
use crate::motion_state::MotionState;
use ph2d_editor::screens::layout::CenterSplit;
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

// ── A tabela de specs: qual forma cada field espacial tem ──

#[test]
fn spec_for_covers_the_spatial_fields() {
    // O field ESPACIAL tem spec; os não-espaciais (rank / composição) não. A box é um
    // RETÂNGULO, o radial sweep é um DISCO — a forma decide o mapeamento de tamanho.
    assert_eq!(
        spec_for(NodeTypeId::of("field.box")).map(|s| s.size),
        Some(FieldSize::Rect {
            width: "width",
            height: "height"
        })
    );
    assert_eq!(
        spec_for(NodeTypeId::of("field.radial_sweep")).map(|s| s.size),
        Some(FieldSize::Disk { radius: "radius" })
    );
    assert!(spec_for(NodeTypeId::of("field.index_range")).is_none());
    assert!(spec_for(NodeTypeId::of("field.combine")).is_none());
    assert!(spec_for(NodeTypeId::of("motion.integrate")).is_none());
}

#[test]
fn spec_names_match_the_nodes() {
    // Os nomes TÊM de bater com os params reais dos nós (ph2d-node-field-box /
    // ph2d-node-field-radial-sweep). Um typo aqui = gizmo que escreve num param fantasma.
    let bx = spec_for(NodeTypeId::of("field.box")).unwrap();
    assert_eq!(
        (bx.center_x, bx.center_y, bx.rotation),
        ("center_x", "center_y", "rotation")
    );
    assert_eq!(
        bx.size,
        FieldSize::Rect {
            width: "width",
            height: "height"
        }
    );
    let rs = spec_for(NodeTypeId::of("field.radial_sweep")).unwrap();
    assert_eq!(
        (rs.center_x, rs.center_y, rs.rotation),
        ("center_x", "center_y", "rotation")
    );
    assert_eq!(rs.size, FieldSize::Disk { radius: "radius" });
}

// ── FieldSize::half — params → meia-extensão do gizmo ──

#[test]
fn rect_size_reads_full_extents_as_half() {
    // O retângulo divide cada extensão CHEIA ⇒ meia = /2.
    let p = |name: &str| match name {
        "width" => 8.0,
        "height" => 4.0,
        _ => 0.0,
    };
    assert_eq!(
        FieldSize::Rect {
            width: "width",
            height: "height"
        }
        .half(p),
        [4.0, 2.0]
    );
}

#[test]
fn disk_size_reads_radius_as_a_square_half() {
    // O disco é a caixa QUADRADA que o circunscreve: meia = [radius, radius].
    let p = |name: &str| if name == "radius" { 5.0 } else { 0.0 };
    assert_eq!(FieldSize::Disk { radius: "radius" }.half(p), [5.0, 5.0]);
}

// ── FieldSize::write — (meia intrínseca × escala) → params ──

fn radius_of(motion: &MotionState, nid: NodeId, name: &str) -> f32 {
    crate::render_loop::motion_bridge::params::param_value(motion, nid, name)
}

/// O inverso exato de [`GizmoCamera::screen_to_world`] — para o teste posicionar o cursor
/// num ponto de MUNDO conhecido (a `GizmoCamera` só expõe a projeção reversa).
fn w2s(cam: &GizmoCamera, world: [f32; 2]) -> (f32, f32) {
    let aspect = cam.window_w / cam.window_h.max(1.0);
    let half_h = cam.height_world * 0.5;
    let half_w = half_h * aspect;
    let nx = (world[0] - cam.center[0]) / half_w;
    let ny = -(world[1] - cam.center[1]) / half_h;
    (
        (nx + 1.0) * 0.5 * cam.window_w,
        (ny + 1.0) * 0.5 * cam.window_h,
    )
}

#[test]
fn rect_write_scales_the_full_extents() {
    // A escala do gizmo multiplica a meia-extensão congelada; o retângulo escreve
    // `2·meia·escala` em cada eixo, independentes.
    let mut motion = MotionState::new();
    let bx = motion.doc.graph.add_node("field.box");
    FieldSize::Rect {
        width: "width",
        height: "height",
    }
    .write(&mut motion.doc.graph, bx, [4.0, 2.0], [1.5, 2.0]);
    assert!(approx(radius_of(&motion, bx, "width"), 12.0));
    assert!(approx(radius_of(&motion, bx, "height"), 8.0));
}

#[test]
fn disk_write_scales_the_radius_uniformly() {
    // O disco é isotrópico: um arrasto de CANTO (escala igual) escreve `meia·escala` exato;
    // um arrasto de BORDA (um eixo só) redimensiona a MÉDIA das duas escalas.
    let mut motion = MotionState::new();
    let sw = motion.doc.graph.add_node("field.radial_sweep");
    let size = FieldSize::Disk { radius: "radius" };
    // Canto: escala uniforme 1.5 ⇒ radius = 5·1.5 = 7.5.
    size.write(&mut motion.doc.graph, sw, [5.0, 5.0], [1.5, 1.5]);
    assert!(approx(radius_of(&motion, sw, "radius"), 7.5), "corner");
    // Borda: escala (1.5, 1.0) ⇒ média 1.25 ⇒ radius = 5·1.25 = 6.25 (sem salto-de-volta).
    size.write(&mut motion.doc.graph, sw, [5.0, 5.0], [1.5, 1.0]);
    assert!(approx(radius_of(&motion, sw, "radius"), 6.25), "edge");
}

#[test]
fn write_under_identity_scale_round_trips() {
    // seed = sample: a meia lida dos params, escrita de volta com escala IDENTIDADE, devolve
    // os mesmos params — o inverso exato (a lição derived-seed-must-match-sample) para as
    // DUAS formas.
    let mut motion = MotionState::new();
    let bx = motion.doc.graph.add_node("field.box");
    motion.doc.graph.set_param(bx, "width", 8.0);
    motion.doc.graph.set_param(bx, "height", 4.0);
    let bsize = spec_for(NodeTypeId::of("field.box")).unwrap().size;
    let bh = bsize.half(|n: &str| radius_of(&motion, bx, n));
    bsize.write(&mut motion.doc.graph, bx, bh, [1.0, 1.0]);
    assert!(approx(radius_of(&motion, bx, "width"), 8.0));
    assert!(approx(radius_of(&motion, bx, "height"), 4.0));

    let sw = motion.doc.graph.add_node("field.radial_sweep");
    motion.doc.graph.set_param(sw, "radius", 7.0);
    let dsize = spec_for(NodeTypeId::of("field.radial_sweep")).unwrap().size;
    let dh = dsize.half(|n: &str| radius_of(&motion, sw, n));
    dsize.write(&mut motion.doc.graph, sw, dh, [1.0, 1.0]);
    assert!(approx(radius_of(&motion, sw, "radius"), 7.0));
}

#[test]
fn wrap180_folds_into_the_slider_range() {
    assert!(approx(wrap180(45.0), 45.0));
    assert!(approx(wrap180(200.0), -160.0));
    assert!(approx(wrap180(-200.0), 160.0));
    assert!(approx(wrap180(370.0), 10.0));
    assert!(approx(wrap180(180.0), -180.0));
}

// ── Seam gates: a modalidade (a resposta ao "não vai atrapalhar os sprites?") ──

#[test]
fn selected_field_is_some_only_for_a_spatial_field() {
    let mut motion = MotionState::new();
    let bid = motion.doc.graph.add_node("field.box");
    let sid = motion.doc.graph.add_node("field.radial_sweep");
    let other = motion.doc.graph.add_node("motion.integrate");
    // Exatamente UM field espacial selecionado ⇒ Some (box OU radial sweep).
    set_graph_selection(vec![bid.0]);
    assert_eq!(selected_field(&motion).map(|(n, _)| n), Some(bid));
    set_graph_selection(vec![sid.0]);
    assert_eq!(selected_field(&motion).map(|(n, _)| n), Some(sid));
    // 🔴 Um nó NÃO-espacial selecionado ⇒ None (sem geometria, sem gizmo).
    set_graph_selection(vec![other.0]);
    assert!(selected_field(&motion).is_none());
    // Nada / multi-seleção ⇒ None (um gizmo dirige UM field).
    set_graph_selection(vec![]);
    assert!(selected_field(&motion).is_none());
    set_graph_selection(vec![bid.0, sid.0]);
    assert!(selected_field(&motion).is_none());
    set_graph_selection(vec![]); // higiene do thread-local
}

#[test]
fn field_view_is_published_for_both_spatial_fields() {
    let mut motion = MotionState::new();
    let bid = motion.doc.graph.add_node("field.box");
    let sid = motion.doc.graph.add_node("field.radial_sweep");
    let cam = Camera2d::new([0.0, 0.0], 20.0);
    // 🔴 field selecionado ⇒ a view é publicada (o gizmo aparece), para AMBAS as formas.
    set_graph_selection(vec![bid.0]);
    assert!(field_view(&motion, &cam, 800.0, 600.0, (0.0, 0.0)).is_some());
    set_graph_selection(vec![sid.0]);
    assert!(field_view(&motion, &cam, 800.0, 600.0, (0.0, 0.0)).is_some());
    // Sem field selecionado ⇒ nada publicado (zero hit-region no canvas dos sprites).
    set_graph_selection(vec![]);
    assert!(field_view(&motion, &cam, 800.0, 600.0, (0.0, 0.0)).is_none());
}

#[test]
fn scene_window_wh_is_the_subrect_under_a_split_full_window_without() {
    // O fix do drift crônico do Motion: sob o split a CENA renderiza num sub-retângulo, e o
    // chrome (grade + gizmo + drag) TEM de mapear mundo↔tela com as MESMAS dims.
    let win = WindowSize::new(800, 600);
    assert_eq!(scene_window_wh(CenterSplit::None, win), (800.0, 600.0));
    // 🔴 Horizontal: a cena é a banda de cima (h·t). Mata "o chrome ignora o split".
    assert_eq!(
        scene_window_wh(CenterSplit::Horizontal { t: 0.5 }, win),
        (800.0, 300.0)
    );
    assert_eq!(
        scene_window_wh(CenterSplit::Vertical { t: 0.5 }, win),
        (400.0, 600.0)
    );
}

// Um `FieldGizmoDrag` de teste com um gizmo genérico já semeado — o mínimo para dirigir
// `apply_field_drag` sem uma janela.
fn make_drag(
    node: NodeId,
    spec: FieldGizmoSpec,
    intrinsic_half: [f32; 2],
    kind: GizmoDragKind,
    cam: &GizmoCamera,
    start_screen: (f32, f32),
    start: TransformSnapshot,
    pivot: [f32; 2],
) -> FieldGizmoDrag {
    FieldGizmoDrag {
        drag: GizmoDragState {
            kind,
            entity_bits: 0,
            start_screen,
            cursor_screen: start_screen,
            start_transform: start,
            pivot_world: pivot,
            start_cursor_world: cam.screen_to_world(start_screen),
            sprite_half_intrinsic: intrinsic_half,
            anchor_is_center: false,
            target: GizmoTarget::MotionField,
            parent_world: TransformSnapshot::IDENTITY,
            turns: 0,
        },
        node,
        spec,
        intrinsic_half,
    }
}

#[test]
fn a_box_field_drag_writes_node_params_never_a_transform() {
    // A prova EXECUTÁVEL da preocupação do Enio: o writeback de um arrasto de field escreve
    // os PARAMS do nó, e `apply_field_drag` nem recebe um `SimWorld` — então por construção
    // NÃO PODE tocar um `Transform` de sprite. Um Translate MOVE o centro e deixa as
    // extensões (a box tem width/height) intactas.
    let mut motion = MotionState::new();
    let bid = motion.doc.graph.add_node("field.box");
    for (n, v) in [
        ("center_x", 0.0),
        ("center_y", 0.0),
        ("width", 8.0),
        ("height", 4.0),
        ("rotation", 0.0),
    ] {
        motion.doc.graph.set_param(bid, n, v);
    }
    let spec = spec_for(NodeTypeId::of("field.box")).unwrap();
    let cam = GizmoCamera {
        center: [0.0, 0.0],
        height_world: 20.0,
        window_w: 800.0,
        window_h: 600.0,
    };
    let start_screen = (400.0, 300.0);
    let start = seed_start(0.0, 0.0, 0.0);
    let mut fgd = make_drag(
        bid,
        spec,
        [4.0, 2.0],
        GizmoDragKind::Translate,
        &cam,
        start_screen,
        start,
        [0.0, 0.0],
    );

    let new_t = apply_field_drag(
        &mut motion,
        &mut fgd,
        (550.0, 200.0),
        &cam,
        GizmoModifiers::default(),
        GizmoSnap::default(),
    );

    let pv = |name| radius_of(&motion, bid, name);
    // O writeback caiu nos PARAMS do NÓ — a leitura pela MESMA porta do painel confirma.
    assert_eq!(pv("center_x"), new_t.translation[0]);
    assert_eq!(pv("center_y"), new_t.translation[1]);
    // 🔴 Um Translate MOVE o centro (mata "apply não escreve center") e NÃO mexe nas
    // extensões (escala fica em 1).
    assert!(
        pv("center_x") != 0.0 || pv("center_y") != 0.0,
        "o centro do field tinha de andar"
    );
    assert!(approx(pv("width"), 8.0));
    assert!(approx(pv("height"), 4.0));
}

#[test]
fn a_radial_field_drag_scales_the_radius() {
    // O radar HERDA o gizmo: um arrasto de ESCALA (canto) num `field.radial_sweep` escreve o
    // `radius` (o único param de tamanho do disco), não width/height. Prova que o
    // `FieldSize::Disk` fecha o loop down→move pela mesma porta.
    let mut motion = MotionState::new();
    let sid = motion.doc.graph.add_node("field.radial_sweep");
    motion.doc.graph.set_param(sid, "radius", 5.0);
    motion.doc.graph.set_param(sid, "center_x", 0.0);
    motion.doc.graph.set_param(sid, "center_y", 0.0);
    let spec = spec_for(NodeTypeId::of("field.radial_sweep")).unwrap();
    let cam = GizmoCamera {
        center: [0.0, 0.0],
        height_world: 20.0,
        window_w: 800.0,
        window_h: 600.0,
    };
    // Handle de canto (+x, −y = BottomRight); o pivô fica no canto OPOSTO (a política do
    // sprite).
    let kind = GizmoDragKind::ScaleCorner {
        dx_sign: 1.0,
        dy_sign: -1.0,
    };
    let intrinsic_half = spec.size.half(|n: &str| radius_of(&motion, sid, n)); // [5, 5]
    let start = seed_start(0.0, 0.0, 0.0);
    let start_screen = w2s(&cam, [intrinsic_half[0], -intrinsic_half[1]]);
    let pivot = ph2d_editor::anchor_pivot_world(kind, intrinsic_half, start, false);
    let mut fgd = make_drag(
        sid,
        spec,
        intrinsic_half,
        kind,
        &cam,
        start_screen,
        start,
        pivot,
    );

    // Arrasta o canto para longe do pivô ⇒ a escala cresce ⇒ o raio cresce.
    let far = w2s(&cam, [intrinsic_half[0] * 2.0, -intrinsic_half[1] * 2.0]);
    apply_field_drag(
        &mut motion,
        &mut fgd,
        far,
        &cam,
        GizmoModifiers::default(),
        GizmoSnap::default(),
    );

    // 🔴 O `radius` cresceu (mata "Disk::write não escreve radius"); ele é um PARAM do nó,
    // e nenhum `Transform` foi tocado (sem `SimWorld`).
    assert!(
        radius_of(&motion, sid, "radius") > 5.0,
        "o raio tinha de crescer: {}",
        radius_of(&motion, sid, "radius")
    );
    set_graph_selection(vec![]); // higiene
}
