//! **As funções livres do vetor: alinhar, distribuir, forma e transformação** — movidas VERBATIM do índice
//! ([`super`], `line/input-dispatch`, 2026-09-13), com os caminhos `crate::input_dispatch::..` preservados por
//! re-exportação. As quatro privadas que os ramos e os testes leem passam a `pub(super)`.

/// An alignment edge/center the Align buttons snap the selected paths' bboxes to
/// (within the selection's union bbox).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecAlign {
    Left,
    HCenter,
    Right,
    Top,
    VCenter,
    Bottom,
}

/// Map an Align button `NodeId` to its [`VecAlign`] (`None` otherwise).
pub(crate) fn vec_align_for_id(id: ph2d_editor_core::NodeId) -> Option<VecAlign> {
    Some(match id {
        x if x == ph2d_tool_vector::ids::VECTOR_ALIGN_LEFT => VecAlign::Left,
        x if x == ph2d_tool_vector::ids::VECTOR_ALIGN_HCENTER => VecAlign::HCenter,
        x if x == ph2d_tool_vector::ids::VECTOR_ALIGN_RIGHT => VecAlign::Right,
        x if x == ph2d_tool_vector::ids::VECTOR_ALIGN_TOP => VecAlign::Top,
        x if x == ph2d_tool_vector::ids::VECTOR_ALIGN_VCENTER => VecAlign::VCenter,
        x if x == ph2d_tool_vector::ids::VECTOR_ALIGN_BOTTOM => VecAlign::Bottom,
        _ => return None,
    })
}

/// Distribute axis: even the selected paths' center spacing horizontally / vertically.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecDistribute {
    Horizontal,
    Vertical,
}

/// Map a Distribute button `NodeId` to its [`VecDistribute`] (`None` otherwise).
pub(crate) fn vec_distribute_for_id(id: ph2d_editor_core::NodeId) -> Option<VecDistribute> {
    if id == ph2d_tool_vector::ids::VECTOR_DISTRIBUTE_H {
        Some(VecDistribute::Horizontal)
    } else if id == ph2d_tool_vector::ids::VECTOR_DISTRIBUTE_V {
        Some(VecDistribute::Vertical)
    } else {
        None
    }
}

/// Align every selected path's bbox to the selection's union bbox per [`VecAlign`]
/// (needs ≥2 selected). One undo step iff anything moved.
pub(crate) fn apply_vec_align(
    sim: &mut ph2d_ecs::SimWorld,
    map: &ph2d_vec_entities::entities::VecEntityMap,
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    xforms: &ph2d_vec_scene::VecXforms,
    kind: VecAlign,
) {
    // Bbox de CURVA (a caixa que o gizmo desenha), não a de âncoras: alinhar tem de
    // casar com o que o usuário vê — uma curva que abaula para fora das âncoras
    // encostaria errado.
    // Bboxes de MUNDO: alinhar compara formas ENTRE SI, e a bbox local de toda forma
    // assentada está centrada na origem (ADR-0112).
    let boxes: Vec<(u64, [f64; 2], [f64; 2])> = pen
        .selected_paths()
        .iter()
        .filter_map(|&id| {
            scene
                .path_world_curve_bbox(xforms, id)
                .map(|(lo, hi)| (id, lo, hi))
        })
        .collect();
    if boxes.len() < 2 {
        return;
    }
    // Union bbox of the selection.
    let mut ulo = [f64::INFINITY; 2];
    let mut uhi = [f64::NEG_INFINITY; 2];
    for &(_, lo, hi) in &boxes {
        ulo[0] = ulo[0].min(lo[0]);
        ulo[1] = ulo[1].min(lo[1]);
        uhi[0] = uhi[0].max(hi[0]);
        uhi[1] = uhi[1].max(hi[1]);
    }
    for (id, lo, hi) in boxes {
        let (mut dx, mut dy) = (0.0, 0.0);
        match kind {
            VecAlign::Left => dx = ulo[0] - lo[0],
            VecAlign::Right => dx = uhi[0] - hi[0],
            VecAlign::HCenter => dx = (ulo[0] + uhi[0]) * 0.5 - (lo[0] + hi[0]) * 0.5,
            // World Y is UP here, so "Top" = the selection's MAX y and "Bottom" =
            // its MIN y (matches what the user sees on-canvas).
            VecAlign::Top => dy = uhi[1] - hi[1],
            VecAlign::Bottom => dy = ulo[1] - lo[1],
            VecAlign::VCenter => dy = (ulo[1] + uhi[1]) * 0.5 - (lo[1] + hi[1]) * 0.5,
        }
        if dx.abs() > 1e-9 || dy.abs() > 1e-9 {
            ph2d_vec_entities::campos_transform::mover_no_mundo(
                sim,
                scene,
                map,
                xforms,
                id,
                [dx, dy],
            );
        }
    }
}

/// Evenly space the selected paths' bbox CENTERS along `axis`, keeping the two
/// extremes fixed (needs ≥3 selected). One undo step iff anything moved.
pub(crate) fn apply_vec_distribute(
    sim: &mut ph2d_ecs::SimWorld,
    map: &ph2d_vec_entities::entities::VecEntityMap,
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    xforms: &ph2d_vec_scene::VecXforms,
    axis: VecDistribute,
) {
    // (id, center-on-axis) for each selected path.
    let mut items: Vec<(u64, f64)> = pen
        .selected_paths()
        .iter()
        .filter_map(|&id| {
            scene.path_world_curve_bbox(xforms, id).map(|(lo, hi)| {
                let c = match axis {
                    VecDistribute::Horizontal => (lo[0] + hi[0]) * 0.5,
                    VecDistribute::Vertical => (lo[1] + hi[1]) * 0.5,
                };
                (id, c)
            })
        })
        .collect();
    if items.len() < 3 {
        return;
    }
    items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let n = items.len();
    let (first, last) = (items[0].1, items[n - 1].1);
    let step = (last - first) / (n - 1) as f64;
    for (k, &(id, c)) in items.iter().enumerate().take(n - 1).skip(1) {
        let target = first + step * k as f64;
        let d = target - c;
        if d.abs() > 1e-9 {
            let (dx, dy) = match axis {
                VecDistribute::Horizontal => (d, 0.0),
                VecDistribute::Vertical => (0.0, d),
            };
            ph2d_vec_entities::campos_transform::mover_no_mundo(
                sim,
                scene,
                map,
                xforms,
                id,
                [dx, dy],
            );
        }
    }
}

/// Whole-path reshape op (panel "Smooth" / "Sharpen" / "Simplify" buttons).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecPathShapeOp {
    Smooth,
    Sharpen,
    Simplify,
    Subdivide,
}

/// Smooth / sharpen / simplify ALL vertices of the SELECTED path (panel Path
/// buttons), recording ONE undo step iff it changed. Free fn.
pub(crate) fn apply_vec_path_shape(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    op: VecPathShapeOp,
) {
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] path-shape: nenhum path selecionado");
        return;
    };
    match op {
        VecPathShapeOp::Smooth => scene.smooth_path(sel),
        VecPathShapeOp::Sharpen => scene.sharpen_path(sel),
        VecPathShapeOp::Simplify => scene.simplify_path(sel),
        VecPathShapeOp::Subdivide => scene.subdivide_path(sel),
    };
}

/// Map a Vector-panel Path button `NodeId` to its [`VecPathShapeOp`] (`None`
/// otherwise). Pure — unit-tested; called from the render_loop drain.
pub(crate) fn vec_path_shape_for_id(id: ph2d_editor_core::NodeId) -> Option<VecPathShapeOp> {
    if id == ph2d_tool_vector::ids::VECTOR_PATH_SMOOTH {
        Some(VecPathShapeOp::Smooth)
    } else if id == ph2d_tool_vector::ids::VECTOR_PATH_SHARPEN {
        Some(VecPathShapeOp::Sharpen)
    } else if id == ph2d_tool_vector::ids::VECTOR_PATH_SIMPLIFY {
        Some(VecPathShapeOp::Simplify)
    } else if id == ph2d_tool_vector::ids::VECTOR_PATH_SUBDIVIDE {
        Some(VecPathShapeOp::Subdivide)
    } else {
        None
    }
}

/// O campo do Transform de um id do painel (`None` para qualquer outro).
pub(crate) fn vec_transform_field_for_id(
    id: ph2d_editor_core::NodeId,
) -> Option<crate::input_dispatch::VecTransformField> {
    use crate::input_dispatch::VecTransformField as F;
    use ph2d_tool_vector::ids as t;
    match id {
        i if i == t::VECTOR_TRANSFORM_X => Some(F::X),
        i if i == t::VECTOR_TRANSFORM_Y => Some(F::Y),
        i if i == t::VECTOR_TRANSFORM_W => Some(F::W),
        i if i == t::VECTOR_TRANSFORM_H => Some(F::H),
        _ => None,
    }
}

/// The shape kind a Vector draw-mode maps to (`None` = Pen, the non-shape
/// gesture). Lets the canvas dispatch route Down/Move/Up to the pen or the
/// A forma que o gesto de canvas desenha: só no modo **Shape**, e é a forma ATIVA do
/// catálogo. Antes cada forma era um modo (e este `match` crescia com o catálogo).
#[must_use]
pub(super) fn shape_kind_for_mode(
    cfg: &ph2d_tool_vector::VectorDrawConfig,
) -> Option<ph2d_vec_scene::ShapeKind> {
    // ⚠️ **A tabela mora na TOOL** (`DrawMode::shape_kind`), e esta função é o adaptador que a
    // lê. Não é cerimónia: o `VectorTool::draw_config` precisa da MESMA resposta para escolher
    // de que slot saem os `values`, e uma segunda tabela aqui divergiria dela em silêncio — que
    // é exactamente o defeito que a moldura arredondável expôs (o gesto cozinhava um kind e lia
    // os parâmetros de outro).
    cfg.mode.shape_kind(cfg.shape)
}

/// As restrições do gesto de forma a partir do teclado — as de todo editor vetorial:
/// **Shift** trava a proporção **1:1** (quadrado / círculo; numa reta, snap de 45°) e
/// **Alt** desenha **a partir do centro**. Combinam.
#[must_use]
pub(super) fn shape_constraint(
    mods: winit::keyboard::ModifiersState,
) -> ph2d_vec_edit::ShapeConstraint {
    ph2d_vec_edit::ShapeConstraint {
        uniform: mods.shift_key(),
        from_center: mods.alt_key(),
    }
}

/// Whether a primary pointer-Up should be **consumed** by the shape tool (and
/// NOT fall through to the chrome dispatch). Critical: in a shape mode the Up is
/// consumed ONLY while a shape drag is actually live — otherwise releasing over
/// a panel button (mode switch / boolean / close) while in a shape mode would
/// silently swallow the click, leaving every button dead.
pub(super) fn shape_up_consumes(mode: ph2d_tool_vector::DrawMode, shape_active: bool) -> bool {
    matches!(
        mode,
        ph2d_tool_vector::DrawMode::Shape | ph2d_tool_vector::DrawMode::Frame
    ) && shape_active
}

/// O `anchor` e o meio-tamanho **intrínsecos** de um objeto do canvas, na linguagem
/// do gizmo de sprite: do `Sprite`, se houver; da bbox local da curva, se for uma
/// forma vetorial (ADR-0111). `([0,0], [0,0])` para o que não é nem um nem outro —
/// um grupo, que não tem geometria própria.
pub(super) fn gizmo_anchor_half(
    sim: &ph2d_ecs::SimWorld,
    vec_scene: &ph2d_vec_scene::VecScene,
    flip: &ph2d_flip::FlipDoc,
    entity: ph2d_ecs::Entity,
) -> ([f32; 2], [f32; 2]) {
    if let Some((s, _)) = ph2d_render::drawn_geometry(sim.world(), entity) {
        return (s.anchor, [s.size[0] * 0.5, s.size[1] * 0.5]);
    }
    // `sim`/`vec_scene`/`flip` chegam separados (e não via `AppGfx`) porque
    // `hero_screen` está emprestado mutável no Down — campos irmãos, borrows
    // disjuntos. Vetor OU objeto Flip (ADR-0111): a bbox local + `Transform`.
    if let Some(ah) = ph2d_app_vec::vec_gizmo_view::anchor_half(sim, vec_scene, entity) {
        return ah;
    }
    ph2d_app_flip::gizmo_view::anchor_half(sim, flip, entity).unwrap_or(([0.0, 0.0], [0.0, 0.0]))
}
