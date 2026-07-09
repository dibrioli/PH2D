use super::*;

const PTW: f64 = 0.01; // world-units por pixel (câmera fictícia)

fn draw_triangle(pen: &mut PenTool, scene: &mut VecScene) {
    pen.on_press(scene, [0.0, 0.0], PTW, false);
    pen.on_release();
    pen.on_press(scene, [4.0, 0.0], PTW, false);
    pen.on_release();
    pen.on_press(scene, [4.0, 4.0], PTW, false);
    pen.on_release();
    pen.on_press(scene, [0.02, 0.0], PTW, false); // fecha (perto do início)
}

#[test]
fn press_builds_then_closes_a_path() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    draw_triangle(&mut pen, &mut scene);
    assert!(!pen.is_drawing());
    assert!(scene.paths()[0].closed);
    assert_eq!(scene.paths()[0].verts.len(), 3);
}

#[test]
fn drag_makes_a_symmetric_vertex_with_mirrored_handles() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
    assert!(pen.on_drag(&mut scene, [1.0, 0.5]));
    let v = scene.paths()[0].verts[0];
    // The Pen creates classic symmetric handles (mirrored) on drag-out.
    assert_eq!(v.kind, VertexKind::Symmetric);
    assert_eq!(v.out_handle, [1.0, 0.5]);
    assert_eq!(v.in_handle, [-1.0, -0.5]);
    pen.on_release();
}

#[test]
fn grab_and_move_an_existing_anchor_translates_its_handles() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    draw_triangle(&mut pen, &mut scene); // fecha → active None, selected = path
    // pressão sobre a âncora [4,0] → agarra
    assert_eq!(
        pen.on_press(&mut scene, [4.0, 0.0], PTW, false),
        PenClick::Grabbed
    );
    assert!(pen.is_dragging());
    assert!(pen.on_drag(&mut scene, [5.0, 1.0]));
    pen.on_release();
    assert!(!pen.is_dragging());
    assert_eq!(scene.paths()[0].verts[1].anchor, [5.0, 1.0]);
}

#[test]
fn grab_a_handle_reshapes_and_mirrors_when_symmetric() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    // desenha 1 ponto (Symmetric) e fecha via finish (fica selecionado)
    pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
    pen.on_drag(&mut scene, [1.0, 0.0]); // out=[1,0], in=[-1,0], Symmetric
    pen.on_release();
    pen.finish();
    // agarra o out-handle em [1,0] e move → in espelha
    assert_eq!(
        pen.on_press(&mut scene, [1.0, 0.0], PTW, false),
        PenClick::Grabbed
    );
    pen.on_drag(&mut scene, [0.0, 2.0]);
    pen.on_release();
    let v = scene.paths()[0].verts[0];
    assert_eq!(v.out_handle, [0.0, 2.0]);
    assert_eq!(v.in_handle, [0.0, -2.0]); // espelho pela âncora (0,0)
}

/// Draw a vertex, retype it Smooth, then dragging one handle keeps the other
/// COLINEAR but preserves its length (unlike Symmetric which mirrors).
#[test]
fn smooth_handle_drag_keeps_opposite_colinear_and_length() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
    pen.on_drag(&mut scene, [2.0, 0.0]); // out=[2,0], in=[-2,0], Symmetric
    pen.on_release();
    pen.finish();
    assert!(pen.set_selected_vertex_kind(&mut scene, VertexKind::Smooth));
    // Grab the out handle ([2,0]) and swing it up-left.
    assert_eq!(
        pen.on_press(&mut scene, [2.0, 0.0], PTW, false),
        PenClick::Grabbed
    );
    pen.on_drag(&mut scene, [0.0, 3.0]); // out now points +Y, len 3
    pen.on_release();
    let v = scene.paths()[0].verts[0];
    assert_eq!(v.out_handle, [0.0, 3.0]);
    // Opposite stays colinear (opposite dir, −Y) but keeps ITS length (2).
    assert!((v.in_handle[0] - 0.0).abs() < 1e-9);
    assert!(
        (v.in_handle[1] - (-2.0)).abs() < 1e-9,
        "kept length 2, flipped"
    );
}

/// Alt + grabbing a handle breaks the tangent: the vertex becomes a Corner
/// (cusp) and only the grabbed handle moves.
#[test]
fn alt_grab_breaks_the_tangent_into_a_corner() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
    pen.on_drag(&mut scene, [2.0, 0.0]); // Symmetric: out=[2,0], in=[-2,0]
    pen.on_release();
    pen.finish();
    // Alt-grab the out handle → break to Corner.
    assert_eq!(
        pen.on_press(&mut scene, [2.0, 0.0], PTW, true),
        PenClick::Grabbed
    );
    pen.on_drag(&mut scene, [2.0, 2.0]);
    pen.on_release();
    let v = scene.paths()[0].verts[0];
    assert_eq!(v.kind, VertexKind::Corner);
    assert_eq!(v.out_handle, [2.0, 2.0]);
    assert_eq!(
        v.in_handle,
        [-2.0, 0.0],
        "in handle untouched (independent)"
    );
}

/// Clicking near a segment of the selected path inserts a vertex (Bézier
/// split) and grabs it — the path gains one Smooth vertex.
#[test]
fn click_on_a_segment_inserts_and_grabs_a_vertex() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    // Two-vertex open path: a straight segment from (0,0) to (4,0).
    pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
    pen.on_release();
    pen.on_press(&mut scene, [4.0, 0.0], PTW, false);
    pen.on_release();
    pen.finish();
    assert_eq!(scene.paths()[0].verts.len(), 2);
    // Click on the middle of the segment (well within hit_r = 10·PTW).
    assert_eq!(
        pen.on_press(&mut scene, [2.0, 0.0], PTW, false),
        PenClick::Inserted
    );
    assert!(
        pen.is_dragging(),
        "new vertex is grabbed for immediate drag"
    );
    assert_eq!(scene.paths()[0].verts.len(), 3);
    assert_eq!(scene.paths()[0].verts[1].kind, VertexKind::Smooth);
    assert_eq!(pen.selected_vert(), Some(1));
    pen.on_release();
}

#[test]
fn far_click_does_not_insert_but_starts_a_new_path() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
    pen.on_release();
    pen.on_press(&mut scene, [4.0, 0.0], PTW, false);
    pen.on_release();
    pen.finish();
    // Far from the segment → a new path starts (not an insert).
    assert_eq!(
        pen.on_press(&mut scene, [2.0, 5.0], PTW, false),
        PenClick::Started
    );
    assert_eq!(scene.paths().len(), 2);
}

#[test]
fn delete_selected_vertex_removes_one_node_and_keeps_the_path() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    draw_triangle(&mut pen, &mut scene); // closed, 3 verts
    pen.on_press(&mut scene, [4.0, 0.0], PTW, false); // select vertex 1
    pen.on_release();
    assert_eq!(pen.selected_vert(), Some(1));
    assert!(pen.delete_selected_vertex(&mut scene));
    assert_eq!(scene.paths()[0].verts.len(), 2, "one node gone, path kept");
}

#[test]
fn deleting_below_two_vertices_removes_the_whole_path() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    // Two-vertex open path.
    pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
    pen.on_release();
    pen.on_press(&mut scene, [4.0, 0.0], PTW, false);
    pen.on_release();
    pen.finish();
    pen.on_press(&mut scene, [0.0, 0.0], PTW, false); // select vertex 0
    pen.on_release();
    assert!(pen.delete_selected_vertex(&mut scene)); // 2 → 1 → remove path
    assert!(scene.is_empty());
    assert_eq!(pen.selected(), None);
    assert_eq!(pen.selected_vert(), None);
}

/// A square path for multi-select tests: 4 corner anchors at the bbox.
fn square_path(scene: &mut VecScene) -> VecPathId {
    scene.push_path(VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([4.0, 0.0]),
            VecVertex::corner([4.0, 4.0]),
            VecVertex::corner([0.0, 4.0]),
        ],
        closed: true,
        ..VecPath::default()
    })
}

#[test]
fn box_select_picks_the_anchors_inside_the_box() {
    let mut scene = VecScene::new();
    let id = square_path(&mut scene);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    // Box over the bottom edge → anchors 0 (0,0) and 1 (4,0).
    pen.box_select(&scene, [-1.0, -1.0], [5.0, 1.0]);
    let mut got = pen.selected_verts().to_vec();
    got.sort_unstable();
    assert_eq!(got, vec![0, 1]);
    // With no path pre-selected, box-select auto-picks the path with anchors.
    let mut pen2 = PenTool::new();
    pen2.box_select(&scene, [-1.0, -1.0], [5.0, 5.0]);
    assert_eq!(pen2.selected(), Some(id));
    assert_eq!(pen2.selected_verts().len(), 4);
}

#[test]
fn dragging_a_grouped_anchor_moves_the_whole_selection() {
    let mut scene = VecScene::new();
    let id = square_path(&mut scene);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    pen.box_select(&scene, [-1.0, -1.0], [5.0, 1.0]); // verts 0 + 1
    // Grab anchor 0 (in the group) and drag it +Y by 2.
    assert_eq!(
        pen.on_press(&mut scene, [0.0, 0.0], PTW, false),
        PenClick::Grabbed
    );
    pen.on_drag(&mut scene, [0.0, 2.0]);
    pen.on_release();
    // BOTH grouped anchors moved by (0,+2); the others stayed.
    assert_eq!(scene.paths()[0].verts[0].anchor, [0.0, 2.0]);
    assert_eq!(scene.paths()[0].verts[1].anchor, [4.0, 2.0]);
    assert_eq!(scene.paths()[0].verts[2].anchor, [4.0, 4.0]);
}

#[test]
fn grabbing_an_ungrouped_anchor_collapses_to_single_then_moves_alone() {
    let mut scene = VecScene::new();
    let id = square_path(&mut scene);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    pen.box_select(&scene, [-1.0, -1.0], [5.0, 1.0]); // verts 0 + 1
    // Grab anchor 2 (NOT in the group) → single-select it, move alone.
    pen.on_press(&mut scene, [4.0, 4.0], PTW, false);
    assert_eq!(pen.selected_verts(), &[2]);
    pen.on_drag(&mut scene, [6.0, 6.0]);
    pen.on_release();
    assert_eq!(scene.paths()[0].verts[2].anchor, [6.0, 6.0]);
    assert_eq!(
        scene.paths()[0].verts[0].anchor,
        [0.0, 0.0],
        "vert 0 unmoved"
    );
}

#[test]
fn multi_retype_and_multi_delete_apply_to_all_selected() {
    let mut scene = VecScene::new();
    let id = square_path(&mut scene);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    pen.box_select(&scene, [-1.0, -1.0], [5.0, 1.0]); // verts 0 + 1
    // Retype both to Smooth (auto-smoothed from neighbours).
    assert!(pen.set_selected_vertex_kind(&mut scene, VertexKind::Smooth));
    assert_eq!(scene.paths()[0].verts[0].kind, VertexKind::Smooth);
    assert_eq!(scene.paths()[0].verts[1].kind, VertexKind::Smooth);
    assert_eq!(scene.paths()[0].verts[2].kind, VertexKind::Corner);
    // Delete both → 4 − 2 = 2 verts remain (path kept).
    assert!(pen.delete_selected_vertex(&mut scene));
    assert_eq!(scene.paths()[0].verts.len(), 2);
    assert!(pen.selected_verts().is_empty());
}

/// The Vertex-type buttons target the selected vertex; `selected_vertex_kind`
/// reports it for the panel to highlight.
#[test]
fn selected_vertex_kind_tracks_the_last_touched_vertex() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    draw_triangle(&mut pen, &mut scene); // closes; last vertex selected
    // Grab a specific anchor to select that vertex.
    pen.on_press(&mut scene, [4.0, 0.0], PTW, false);
    pen.on_release();
    assert_eq!(pen.selected_vert(), Some(1));
    assert_eq!(
        pen.selected_vertex_kind(&scene),
        Some(VertexKind::Corner) // straight corners from clicks
    );
    assert!(pen.set_selected_vertex_kind(&mut scene, VertexKind::Symmetric));
    assert_eq!(
        pen.selected_vertex_kind(&scene),
        Some(VertexKind::Symmetric)
    );
    // Selecting a whole path (boolean result) clears the vertex selection.
    pen.select(Some(scene.paths()[0].id));
    assert_eq!(pen.selected_vert(), None);
    assert_eq!(pen.selected_vertex_kind(&scene), None);
}

#[test]
fn plain_click_stays_a_corner() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
    pen.on_release();
    let v = scene.paths()[0].verts[0];
    assert_eq!(v.kind, VertexKind::Corner);
    assert_eq!(v.out_handle, v.anchor);
}

#[test]
fn history_undo_redo_cycle() {
    let mut h = History::new();
    let mut scene = VecScene::new();
    h.begin(&scene);
    scene = VecScene::demo(); // muta (vazio → 2 paths)
    h.commit_if_changed(&scene);
    assert!(h.can_undo());
    let changed = scene.clone();

    scene = h.undo(&scene).unwrap();
    assert!(scene.is_empty());
    assert!(h.can_redo());

    scene = h.redo(&scene).unwrap();
    assert_eq!(scene, changed);
}

#[test]
fn commit_without_change_is_noop() {
    let mut h = History::new();
    let scene = VecScene::new();
    h.begin(&scene);
    h.commit_if_changed(&scene); // nada mudou entre begin e commit
    assert!(!h.can_undo());
}

#[test]
fn set_style_colors_new_paths_and_survives_clear() {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    let style = PenStyle {
        stroke: Rgba8::new(200, 40, 40, 255),
        stroke_w_px: 5.0,
        fill: Rgba8::new(40, 200, 40, 128),
        ..PenStyle::default()
    };
    pen.set_style(style);
    pen.on_press(&mut scene, [0.0, 0.0], PTW, false);
    let s = scene.paths()[0].stroke.expect("stroke");
    assert_eq!(s.color, style.stroke);
    assert_eq!(s.width, style.stroke_w_px * PTW);
    // Fechar aplica o fill do estilo.
    pen.on_release();
    pen.on_press(&mut scene, [4.0, 0.0], PTW, false);
    pen.on_release();
    pen.on_press(&mut scene, [4.0, 4.0], PTW, false);
    pen.on_release();
    pen.on_press(&mut scene, [0.02, 0.0], PTW, false); // fecha
    assert_eq!(
        scene.paths()[0].fill,
        Some(ph2d_vec_scene::Paint::solid(style.fill))
    );
    // O estilo é config da tool → sobrevive a `clear`.
    pen.clear();
    assert_eq!(pen.style(), style);
}

#[test]
fn nudge_translates_whole_path_or_only_selected_verts() {
    use ph2d_vec_scene::rectangle;
    let mut scene = VecScene::new();
    let id = scene.push_path(rectangle([0.0, 0.0], [4.0, 4.0]));
    let mut pen = PenTool::new();

    // Nothing selected → no-op.
    assert!(!pen.nudge(&mut scene, 1.0, 1.0));

    // Path selected, no verts → the whole path translates.
    pen.select(Some(id));
    let before: Vec<_> = scene.paths()[0].verts.iter().map(|v| v.anchor).collect();
    assert!(pen.nudge(&mut scene, 1.0, 2.0));
    for (b, v) in before.iter().zip(&scene.paths()[0].verts) {
        assert_eq!(v.anchor, [b[0] + 1.0, b[1] + 2.0]);
    }

    // Box-select a single corner → only that vertex moves.
    let now: Vec<_> = scene.paths()[0].verts.iter().map(|v| v.anchor).collect();
    let c = now[0];
    pen.box_select(&scene, [c[0] - 0.1, c[1] - 0.1], [c[0] + 0.1, c[1] + 0.1]);
    assert!(pen.nudge(&mut scene, 10.0, 0.0));
    assert_eq!(scene.paths()[0].verts[0].anchor, [c[0] + 10.0, c[1]]);
    for (i, &expected) in now.iter().enumerate().skip(1) {
        assert_eq!(
            scene.paths()[0].verts[i].anchor,
            expected,
            "outros vértices ficam"
        );
    }
}
