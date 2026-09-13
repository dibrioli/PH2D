//! **Os testes do despacho** (`input_dispatch::tests`) — o corpo do módulo mudou-se VERBATIM (`line/input-dispatch`,
//! 2026-09-13) para um ficheiro, declarado por `#[path]` no índice com o MESMO nome: nenhum teste muda de nome, e o
//! `use super::{..}` continua a ler o índice (as funções livres que saíram dele voltam por re-exportação).

use super::{
    VecPathShapeOp, VecTransformField, apply_vec_path_shape, apply_vec_transform,
    shape_kind_for_mode, shape_up_consumes, vec_bool_op_for_id, vec_flip_for_id,
    vec_path_shape_for_id, vec_reorder_for_id, vec_rotate_for_id, vec_transform_field_for_id,
    vec_vertex_kind_for_id,
};
use ph2d_tool_vector::DrawMode;
use ph2d_vec_scene::ShapeKind;
use ph2d_vec_scene::{FlipAxis, Rotate90, VertexKind, ZOrder};

#[test]
fn vertex_button_ids_map_to_their_kinds() {
    assert_eq!(
        vec_vertex_kind_for_id(ph2d_tool_vector::ids::VECTOR_VERT_CORNER),
        Some(VertexKind::Corner)
    );
    assert_eq!(
        vec_vertex_kind_for_id(ph2d_tool_vector::ids::VECTOR_VERT_SMOOTH),
        Some(VertexKind::Smooth)
    );
    assert_eq!(
        vec_vertex_kind_for_id(ph2d_tool_vector::ids::VECTOR_VERT_SYMMETRIC),
        Some(VertexKind::Symmetric)
    );
    assert_eq!(
        vec_vertex_kind_for_id(ph2d_panel_vector::ids::VECTOR_BOOL_UNION),
        None
    );
}

#[test]
fn arrange_button_ids_map_to_their_zorder() {
    assert_eq!(
        vec_reorder_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_TO_BACK),
        Some(ZOrder::ToBack)
    );
    assert_eq!(
        vec_reorder_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_BACKWARD),
        Some(ZOrder::Lower)
    );
    assert_eq!(
        vec_reorder_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FORWARD),
        Some(ZOrder::Raise)
    );
    assert_eq!(
        vec_reorder_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_TO_FRONT),
        Some(ZOrder::ToFront)
    );
    // Duplicate is NOT a reorder (handled separately), nor any non-Arrange id.
    assert_eq!(
        vec_reorder_for_id(ph2d_panel_vector::ids::VECTOR_ARRANGE_DUPLICATE),
        None
    );
    assert_eq!(
        vec_reorder_for_id(ph2d_panel_vector::ids::VECTOR_BOOL_UNION),
        None
    );
}

#[test]
fn flip_button_ids_map_to_their_axis() {
    assert_eq!(
        vec_flip_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_H),
        Some(FlipAxis::Horizontal)
    );
    assert_eq!(
        vec_flip_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_V),
        Some(FlipAxis::Vertical)
    );
    // Flip is NOT a reorder and vice-versa.
    assert_eq!(
        vec_flip_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_TO_BACK),
        None
    );
    assert_eq!(
        vec_reorder_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_H),
        None
    );
}

#[test]
fn rotate_button_ids_map_to_their_direction() {
    assert_eq!(
        vec_rotate_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_ROTATE_CW),
        Some(Rotate90::Cw)
    );
    assert_eq!(
        vec_rotate_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_ROTATE_CCW),
        Some(Rotate90::Ccw)
    );
    assert_eq!(
        vec_rotate_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_H),
        None
    );
    assert_eq!(
        vec_flip_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_ROTATE_CW),
        None
    );
}

#[test]
fn transform_fields_map_and_apply_translates_and_scales() {
    use ph2d_vec_scene::{VecScene, rectangle};
    assert_eq!(
        vec_transform_field_for_id(ph2d_tool_vector::ids::VECTOR_TRANSFORM_X),
        Some(VecTransformField::X)
    );
    assert_eq!(
        vec_transform_field_for_id(ph2d_tool_vector::ids::VECTOR_TRANSFORM_H),
        Some(VecTransformField::H)
    );
    assert_eq!(
        vec_transform_field_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_H),
        None
    );

    let mut scene = VecScene::new();
    let id = scene.push_path(rectangle([0.0, 0.0], [10.0, 4.0]));
    // Um path CRU: sem receita a manter em passo (o `keep_recipe_in_step` sai calado).
    let mut sim = ph2d_ecs::SimWorld::default();
    let map = ph2d_vec_entities::entities::VecEntityMap::new();
    let mut pen = ph2d_vec_edit::PenTool::new();
    pen.select(Some(id));

    // X → 5 moves the bbox min; W → 20 doubles the width.
    apply_vec_transform(
        &mut sim,
        &map,
        &mut scene,
        &pen,
        &ph2d_vec_scene::VecXforms::new(),
        VecTransformField::X,
        5.0,
    );
    assert!((scene.path_bbox(id).unwrap().0[0] - 5.0).abs() < 1e-9);
    apply_vec_transform(
        &mut sim,
        &map,
        &mut scene,
        &pen,
        &ph2d_vec_scene::VecXforms::new(),
        VecTransformField::W,
        20.0,
    );
    let (lo, hi) = scene.path_bbox(id).unwrap();
    assert!((hi[0] - lo[0] - 20.0).abs() < 1e-9, "W set to 20");
    assert!((lo[0] - 5.0).abs() < 1e-9, "min x pinned during scale");
}

#[test]
fn path_shape_ids_map_and_apply_smooths_then_sharpens() {
    use ph2d_vec_scene::{VertexKind, regular_polygon};
    assert_eq!(
        vec_path_shape_for_id(ph2d_tool_vector::ids::VECTOR_PATH_SMOOTH),
        Some(VecPathShapeOp::Smooth)
    );
    assert_eq!(
        vec_path_shape_for_id(ph2d_tool_vector::ids::VECTOR_PATH_SHARPEN),
        Some(VecPathShapeOp::Sharpen)
    );
    assert_eq!(
        vec_path_shape_for_id(ph2d_tool_vector::ids::VECTOR_PATH_SIMPLIFY),
        Some(VecPathShapeOp::Simplify)
    );
    assert_eq!(
        vec_path_shape_for_id(ph2d_tool_vector::ids::VECTOR_PATH_SUBDIVIDE),
        Some(VecPathShapeOp::Subdivide)
    );
    assert_eq!(
        vec_path_shape_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_H),
        None
    );

    let mut scene = ph2d_vec_scene::VecScene::new();
    let id = scene.push_path(regular_polygon([0.0, 0.0], 5.0, 5.0, 5));
    let mut pen = ph2d_vec_edit::PenTool::new();
    pen.select(Some(id));

    apply_vec_path_shape(&mut scene, &pen, VecPathShapeOp::Smooth);
    assert!(
        scene.paths()[0]
            .verts
            .iter()
            .all(|v| v.kind == VertexKind::Smooth),
        "smooth button curves every vertex"
    );
    apply_vec_path_shape(&mut scene, &pen, VecPathShapeOp::Sharpen);
    assert!(
        scene.paths()[0]
            .verts
            .iter()
            .all(|v| v.kind == VertexKind::Corner && v.in_handle == v.anchor),
        "sharpen button flattens every vertex"
    );

    // Simplify: a closed square with a redundant midpoint on one edge drops it.
    let sq = scene.push_path(ph2d_vec_scene::VecPath {
        verts: vec![
            ph2d_vec_scene::VecVertex::corner([0.0, 0.0]),
            ph2d_vec_scene::VecVertex::corner([5.0, 0.0]), // redundant midpoint
            ph2d_vec_scene::VecVertex::corner([10.0, 0.0]),
            ph2d_vec_scene::VecVertex::corner([10.0, 10.0]),
            ph2d_vec_scene::VecVertex::corner([0.0, 10.0]),
        ],
        closed: true,
        ..ph2d_vec_scene::VecPath::default()
    });
    pen.select(Some(sq));
    let before = scene
        .paths()
        .iter()
        .find(|p| p.id == sq)
        .unwrap()
        .verts
        .len();
    apply_vec_path_shape(&mut scene, &pen, VecPathShapeOp::Simplify);
    let after = scene
        .paths()
        .iter()
        .find(|p| p.id == sq)
        .unwrap()
        .verts
        .len();
    assert_eq!(after, before - 1, "simplify drops the one redundant point");

    // Subdivide: one midpoint per segment (closed ⇒ doubles the vertex count).
    let n = scene
        .paths()
        .iter()
        .find(|p| p.id == sq)
        .unwrap()
        .verts
        .len();
    apply_vec_path_shape(&mut scene, &pen, VecPathShapeOp::Subdivide);
    let n2 = scene
        .paths()
        .iter()
        .find(|p| p.id == sq)
        .unwrap()
        .verts
        .len();
    assert_eq!(n2, n * 2, "subdivide doubles a closed path's vertices");

    // Close/Open toggle flips the selected path's `closed` flag each click.
    let was = scene.paths().iter().find(|p| p.id == sq).unwrap().closed;
    super::apply_vec_toggle_closed(&mut scene, &mut pen);
    assert_eq!(
        scene.paths().iter().find(|p| p.id == sq).unwrap().closed,
        !was,
        "toggle flips closed"
    );
    super::apply_vec_toggle_closed(&mut scene, &mut pen);
    assert_eq!(
        scene.paths().iter().find(|p| p.id == sq).unwrap().closed,
        was,
        "toggle flips back"
    );
    // Closing a never-filled path seeds a fill so it paints immediately.
    assert!(
        scene
            .paths()
            .iter()
            .find(|p| p.id == sq)
            .unwrap()
            .fill
            .is_some(),
        "closing seeds the Style fill (immediate paint)"
    );
}

#[test]
fn shape_up_only_consumed_while_a_drag_is_live() {
    // Pen mode never consumes via the shape path (the pen path handles it).
    assert!(!shape_up_consumes(DrawMode::Pen, false));
    assert!(!shape_up_consumes(DrawMode::Pen, true));
    // In a shape mode, a live drag consumes the Up (finalize the shape)...
    assert!(shape_up_consumes(DrawMode::Shape, true));
    // ...but with NO active drag the Up must fall through so a panel-button
    // click (mode switch / boolean / close) is not swallowed. This is the
    // exact regression that made every button dead after entering Rect mode.
    assert!(!shape_up_consumes(DrawMode::Shape, false));
    assert!(
        !shape_up_consumes(DrawMode::Pen, true),
        "a caneta nao e forma"
    );
}

/// **Os OITO ids do Pathfinder mapeiam para as suas ops** (plano 25 §8, W5).
///
/// ⚠️ A lista é enumerada aqui à mão de propósito: derivá-la da tabela do produto tornaria o
/// gate um espelho (encolher a tabela encolheria a lista percorrida, e ele seguiria verde) —
/// o oráculo auto-referente que esta linha já pagou na varredura dos pills.
#[test]
fn pathfinder_button_ids_map_to_their_ops() {
    use ph2d_vec_boolean::PathfinderOp as P;
    for (id, want) in [
        (ph2d_panel_vector::ids::VECTOR_BOOL_UNION, P::Union),
        (ph2d_panel_vector::ids::VECTOR_BOOL_SUBTRACT, P::Subtract),
        (ph2d_panel_vector::ids::VECTOR_BOOL_INTERSECT, P::Intersect),
        (ph2d_panel_vector::ids::VECTOR_BOOL_EXCLUDE, P::Exclude),
        (ph2d_tool_vector::ids::VECTOR_BOOL_MINUS_BACK, P::MinusBack),
        (ph2d_tool_vector::ids::VECTOR_BOOL_TRIM, P::Trim),
        (ph2d_tool_vector::ids::VECTOR_BOOL_CROP, P::Crop),
        (ph2d_tool_vector::ids::VECTOR_BOOL_MERGE, P::Merge),
    ] {
        assert_eq!(vec_bool_op_for_id(id), Some(want), "{want:?}");
    }
    // A non-boolean id (a mode button) is not a boolean op.
    assert_eq!(
        vec_bool_op_for_id(ph2d_tool_vector::ids::VECTOR_MODE_PEN),
        None
    );
}

/// O gesto de canvas só desenha no modo **Shape**, e o que ele desenha é a forma
/// ATIVA do catálogo — não há mais um modo por forma. Com vinte e cinco formas, o
/// `match` antigo (um braço por forma) seria o pior lugar para esquecer uma.
#[test]
fn only_shape_mode_draws_and_it_draws_the_active_shape() {
    use ph2d_tool_vector::VectorDrawConfig;
    let mut cfg = VectorDrawConfig::default();
    for m in [
        DrawMode::Select,
        DrawMode::Node,
        DrawMode::Pen,
        DrawMode::Text,
    ] {
        cfg.mode = m;
        assert_eq!(shape_kind_for_mode(&cfg), None, "{m:?} nao desenha forma");
    }
    cfg.mode = DrawMode::Shape;
    for k in [ShapeKind::Rectangle, ShapeKind::Star, ShapeKind::Arc] {
        cfg.shape = k;
        assert_eq!(shape_kind_for_mode(&cfg), Some(k));
    }
}

/// **A MOLDURA desenha um retângulo ARREDONDÁVEL** (Enio, 2026-08-21: *"o Frame é criado como
/// retângulo de quinas sem a possibilidade de arredondamento"*).
///
/// ⚠️ **E o kind dela NÃO segue o catálogo** — é o par de asserções que importa. A moldura
/// tem de dar `RoundRect` mesmo com a estrela ativa, senão o gesto herdaria a forma do botão
/// aceso e a ferramenta Moldura deixaria de desenhar molduras.
#[test]
fn the_frame_draws_a_roundable_rectangle_whatever_the_catalogue_says() {
    use ph2d_tool_vector::VectorDrawConfig;
    let mut cfg = VectorDrawConfig {
        mode: DrawMode::Frame,
        ..Default::default()
    };
    for catalogue in [ShapeKind::Rectangle, ShapeKind::Star, ShapeKind::Heart] {
        cfg.shape = catalogue;
        assert_eq!(
            shape_kind_for_mode(&cfg),
            Some(ShapeKind::RoundRect),
            "a moldura tem de ser arredondavel, e o catalogo ({catalogue:?}) nao manda nela"
        );
    }
}

// ─── boolean/compound: a costura shell ↔ documento ────────────────────────

/// Cena com um quadrado externo e outro DENTRO dele, ambos selecionados
/// (z: externo atrás, interno na frente). Devolve `(scene, pen, ids)`.
fn nested_selection() -> (ph2d_vec_scene::VecScene, ph2d_vec_edit::PenTool, [u64; 2]) {
    let mut scene = ph2d_vec_scene::VecScene::new();
    let outer = scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [10.0, 10.0]));
    let inner = scene.push_path(ph2d_vec_scene::rectangle([3.0, 3.0], [7.0, 7.0]));
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[outer, inner]);
    (scene, pen, [outer, inner])
}

/// A regressão que motivou o bloco: Subtract agia nas duas últimas regiões
/// fechadas do DOCUMENTO, ignorando a seleção — e devolvia dois discos
/// sólidos em vez de uma rosquinha.
#[test]
fn boolean_subtract_uses_the_selection_and_makes_a_real_hole() {
    let (mut scene, mut pen, _) = nested_selection();
    // Um terceiro path, NÃO selecionado, bem longe: a booleana antiga o teria
    // agarrado (é uma das duas últimas fechadas); a nova tem de ignorá-lo.
    let bystander = scene.push_path(ph2d_vec_scene::rectangle([90.0, 90.0], [95.0, 95.0]));

    super::apply_vec_boolean(
        &mut scene,
        &mut pen,
        &ph2d_vec_scene::VecXforms::new(),
        ph2d_vec_boolean::PathfinderOp::Subtract,
    );

    assert_eq!(scene.paths().len(), 2, "resultado + o bystander intacto");
    assert!(scene.paths().iter().any(|p| p.id == bystander));
    let donut = scene.paths().iter().find(|p| p.id != bystander).unwrap();
    assert!(donut.is_compound(), "o furo vive num subpath");
    let id = donut.id;
    assert!(scene.path_contains_point(id, [1.0, 5.0]), "o anel é sólido");
    assert!(
        !scene.path_contains_point(id, [5.0, 5.0]),
        "o centro é vazado"
    );
    // O resultado entra na fatia de z da BASE (não salta pro topo).
    assert_eq!(scene.paths()[0].id, id);
    assert_eq!(pen.selected(), Some(id), "a booleana seleciona o resultado");
}

#[test]
fn boolean_needs_two_selected_closed_regions() {
    let (mut scene, mut pen, ids) = nested_selection();
    pen.select(Some(ids[0])); // só um selecionado
    super::apply_vec_boolean(
        &mut scene,
        &mut pen,
        &ph2d_vec_scene::VecXforms::new(),
        ph2d_vec_boolean::PathfinderOp::Union,
    );
    assert_eq!(scene.paths().len(), 2, "no-op");
}

/// Make Compound é como o usuário desenha um buraco à mão; Release desfaz.
#[test]
fn make_and_release_compound_from_the_selection() {
    let (mut scene, mut pen, ids) = nested_selection();

    super::apply_vec_compound(&mut scene, &mut pen, true);
    assert_eq!(scene.paths().len(), 1);
    assert!(
        !scene.path_contains_point(ids[0], [5.0, 5.0]),
        "virou buraco"
    );
    assert_eq!(pen.selected(), Some(ids[0]));

    super::apply_vec_compound(&mut scene, &mut pen, false);
    assert_eq!(scene.paths().len(), 2);
    assert!(
        scene.path_contains_point(ids[0], [5.0, 5.0]),
        "sólido de novo"
    );
    assert_eq!(pen.selected_paths().len(), 2, "base + liberado");
}

/// A regra de preenchimento troca o buraco por região sólida, sem tocar a geometria.
#[test]
fn fill_rule_toggle_vacates_or_fills_the_hole() {
    let (mut scene, mut pen, ids) = nested_selection();
    super::apply_vec_compound(&mut scene, &mut pen, true);
    assert!(!scene.path_contains_point(ids[0], [5.0, 5.0]));

    super::apply_vec_fill_rule(&mut scene, &pen, false); // Non-Zero
    assert!(
        scene.path_contains_point(ids[0], [5.0, 5.0]),
        "NonZero preenche"
    );
    super::apply_vec_fill_rule(&mut scene, &pen, true); // Even-Odd
    assert!(
        !scene.path_contains_point(ids[0], [5.0, 5.0]),
        "EvenOdd vaza"
    );
}
