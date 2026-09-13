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
            scene.translate_path_world(xforms, id, dx, dy);
        }
    }
}

/// Evenly space the selected paths' bbox CENTERS along `axis`, keeping the two
/// extremes fixed (needs ≥3 selected). One undo step iff anything moved.
pub(crate) fn apply_vec_distribute(
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
            scene.translate_path(id, dx, dy);
        }
    }
}

/// Rotate the SELECTED path by `degrees` (panel Transform Angle field — a
/// relative scrub) about its bbox center, recording ONE undo step iff it turned.
/// A zero delta is a no-op (no undo). Free fn (mirror of [`apply_vec_rotate`]).
pub(crate) fn apply_vec_rotate_by(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    degrees: f64,
) {
    if degrees.abs() < 1e-9 {
        return;
    }
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] rotate-by: nenhum path selecionado");
        return;
    };
    let Some((lo, hi)) = scene.path_bbox(sel) else {
        return;
    };
    let pivot = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    scene.rotate_path_by(sel, degrees.to_radians(), pivot);
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
/// buttons), recording ONE undo step iff it changed. Free fn (mirror of
/// [`apply_vec_flip`]).
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

/// Which numeric-transform field a Vector-panel edit targets on the selected
/// path's anchor bbox (X/Y = top-left, W/H = size).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecTransformField {
    X,
    Y,
    W,
    H,
}

/// Map a Vector-panel Transform field `NodeId` to its [`VecTransformField`]
/// (`None` for any other id). Pure — unit-tested; called from the render_loop drain.
pub(crate) fn vec_transform_field_for_id(
    id: ph2d_editor_core::NodeId,
) -> Option<VecTransformField> {
    if id == ph2d_tool_vector::ids::VECTOR_TRANSFORM_X {
        Some(VecTransformField::X)
    } else if id == ph2d_tool_vector::ids::VECTOR_TRANSFORM_Y {
        Some(VecTransformField::Y)
    } else if id == ph2d_tool_vector::ids::VECTOR_TRANSFORM_W {
        Some(VecTransformField::W)
    } else if id == ph2d_tool_vector::ids::VECTOR_TRANSFORM_H {
        Some(VecTransformField::H)
    } else {
        None
    }
}

/// Apply a numeric transform edit to the SELECTED path: X/Y translate the anchor
/// bbox top-left to `target`; W/H scale (about the bbox min) so that dimension
/// becomes `target` (clamped > 0; a degenerate dimension can't be resized).
/// Records ONE undo step iff it changed. Free fn (mirror of [`apply_vec_boolean`]).
///
/// ⚠️ **O `sim`/`map` estão aqui pela RECEITA, e não por conveniência.** Um `W`/`H` escala a
/// GEOMETRIA (`scale_path`), e a geometria de uma forma VIVA é derivada do `w`/`h` guardado no
/// `VecShape::Param`. Escrever só os `verts` deixa a receita a descrever a caixa antiga, e a
/// primeira edição de qualquer parâmetro re-cozinha a partir dela — medido, `100 → 50 → 100`:
/// **o redimensionamento evapora em silêncio**. Este defeito é ANTERIOR à alça do gizmo (este
/// caminho era o único que escalava geometria de forma viva), e a alça só o tornou fácil de
/// alcançar. A porta é a MESMA que ela usa ([`crate::vec_shape_live::resize_recipe`]).
#[allow(clippy::too_many_arguments)] // a receita mora no ECS; a geometria, na cena
pub(crate) fn apply_vec_transform(
    sim: &mut ph2d_ecs::SimWorld,
    map: &ph2d_vec_entities::entities::VecEntityMap,
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    xforms: &ph2d_vec_scene::VecXforms,
    field: VecTransformField,
    target: f64,
) {
    let Some(sel) = pen.selected() else {
        return;
    };
    // Os campos do painel são números de MUNDO (ADR-0112).
    let Some((lo, hi)) = scene.path_world_curve_bbox(xforms, sel) else {
        return;
    };
    // O escalonamento acontece na geometria local, em torno do canto local que
    // corresponde ao `lo` de mundo — a razão de escala é a mesma nos dois espaços.
    let local_lo = scene.path_curve_bbox(sel).map_or([0.0, 0.0], |(l, _)| l);
    match field {
        VecTransformField::X => {
            let dx = target - lo[0];
            dx.abs() > 1e-9 && scene.translate_path_world(xforms, sel, dx, 0.0)
        }
        VecTransformField::Y => {
            let dy = target - lo[1];
            dy.abs() > 1e-9 && scene.translate_path_world(xforms, sel, 0.0, dy)
        }
        VecTransformField::W => {
            let w = hi[0] - lo[0];
            if w <= 1e-6 {
                false
            } else {
                let sx = target.max(1e-4) / w;
                (sx - 1.0).abs() > 1e-9
                    && scene.scale_path(sel, sx, 1.0, local_lo)
                    && keep_recipe_in_step(sim, map, sel, sx, 1.0)
            }
        }
        VecTransformField::H => {
            let h = hi[1] - lo[1];
            if h <= 1e-6 {
                false
            } else {
                let sy = target.max(1e-4) / h;
                (sy - 1.0).abs() > 1e-9
                    && scene.scale_path(sel, 1.0, sy, local_lo)
                    && keep_recipe_in_step(sim, map, sel, 1.0, sy)
            }
        }
    };
}

/// A receita da forma VIVA de `id` passa a medir a caixa escalada por `(sx, sy)`. Devolve
/// **sempre `true`** — a geometria já mudou, e um path CRU (sem receita) não é falha nenhuma.
///
/// ⚠️ Ela vive dentro do `&&` do braço de propósito: pendurada ali, um braço de escala novo que
/// esqueça de a chamar **não conta como escala** — o `changed` fica `false` e o undo grita.
/// Escrita depois do `match`, ela seria uma linha que o próximo braço não sabe que existe.
fn keep_recipe_in_step(
    sim: &mut ph2d_ecs::SimWorld,
    map: &ph2d_vec_entities::entities::VecEntityMap,
    id: ph2d_vec_scene::VecPathId,
    sx: f64,
    sy: f64,
) -> bool {
    if let Some(&bits) = map.get(&id) {
        let e = ph2d_ecs::Entity::from_bits(bits);
        if let Some(ph2d_ecs::VecShape::Param { w, h, .. }) =
            sim.world().get::<ph2d_ecs::VecShape>(e).cloned()
        {
            crate::vec_shape_live::resize_recipe(sim, e, w * sx, h * sy);
        }
    }
    true
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
    if let Some(s) = sim.world().get::<ph2d_render::Sprite>(entity) {
        return (s.anchor, [s.size[0] * 0.5, s.size[1] * 0.5]);
    }
    // `sim`/`vec_scene`/`flip` chegam separados (e não via `AppGfx`) porque
    // `hero_screen` está emprestado mutável no Down — campos irmãos, borrows
    // disjuntos. Vetor OU objeto Flip (ADR-0111): a bbox local + `Transform`.
    if let Some(ah) = crate::vec_gizmo_view::anchor_half(sim, vec_scene, entity) {
        return ah;
    }
    ph2d_app_flip::gizmo_view::anchor_half(sim, flip, entity).unwrap_or(([0.0, 0.0], [0.0, 0.0]))
}
